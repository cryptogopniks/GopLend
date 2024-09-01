import { l } from "../../common/utils";
import { readFile } from "fs/promises";
import { ChainConfig } from "../../common/interfaces";
import { ENCODING, PATH_TO_CONFIG_JSON, parseStoreArgs } from "./utils";
import { getChainOptionById } from "../../common/config/config-utils";
import { getCwQueryHelpers } from "../../common/account/cw-helpers";
import { PlatformType } from "../../common/codegen/AdapterMarketplaceStargaze.types";
import {
  ArrayOfPriceItemForFundsForToken,
  PriceItemForFundsForToken,
} from "../../common/codegen/Oracle.types";
import {
  calcConditionalLtv,
  calcLiquidationSet,
  calcLiquidationValue,
  calcLtvMax,
  choosePrice,
  matchCollateralsAndBidsLegacy,
} from "./math";
import {
  Addr,
  ArrayOfQueryLiquidationBidsByCollectionAddressListResponseItem,
  Borrower,
  Collateral,
  CurrencyForToken,
  LiquidationBid,
  LiquidationItem,
} from "../../common/codegen/LendingPlatform.types";

const PAGINATION_AMOUNT = 100;

// TODO: can multiple users share same collection offer?
export async function queryTargets(
  platformType: PlatformType,
  borrowers: string[] // TODO: remove
): Promise<LiquidationItem[]> {
  // configs
  const configJsonStr = await readFile(PATH_TO_CONFIG_JSON, {
    encoding: ENCODING,
  });
  const CHAIN_CONFIG: ChainConfig = JSON.parse(configJsonStr);
  const { chainId } = parseStoreArgs();
  const {
    OPTION: {
      RPC_LIST: [RPC],
    },
  } = getChainOptionById(CHAIN_CONFIG, chainId);

  const { lending, oracle, utils } = await getCwQueryHelpers(chainId, RPC);

  // helpers
  const getBorrowers = async (
    borrowers: string[]
  ): Promise<[Addr, Borrower][]> => {
    return (
      await lending.pQueryBorrowerList(platformType, PAGINATION_AMOUNT, 0, true)
    )
      .filter(({ address }) => borrowers.includes(address))
      .map(({ address, borrower }) => [address, borrower]);
  };

  const getCollectionPrices = async (
    borrowers: [Addr, Borrower][],
    collectionAndCollateralList: [Addr, Collateral[]][]
  ): Promise<ArrayOfPriceItemForFundsForToken> => {
    // collect collection addresses over current page borrowers
    const borrowerAddresses: Addr[] = borrowers.map(([address, _]) => address);

    let collectionAddressesByBorrowers: string[] = collectionAndCollateralList
      .filter(([_, collaterals]) =>
        collaterals.some(({ owner }) => borrowerAddresses.includes(owner))
      )
      .map(([collectionAddress]) => collectionAddress);

    // dedup
    collectionAddressesByBorrowers = [
      ...new Set(collectionAddressesByBorrowers),
    ];

    return await oracle.cwQueryPrices(
      0, // as collectionAddressesByBorrowers is used
      undefined, // as collectionAddressesByBorrowers is used
      collectionAddressesByBorrowers,
      true
    );
  };

  const getCollateralsByBorrower = (
    collectionAndCollateralList: [Addr, Collateral[]][],
    borrowerAddress: Addr
  ): [Addr, Collateral][] => {
    // get collaterals owned by borrower
    let collectionAndCollateralListByBorrower: [Addr, Collateral][] = [];

    for (const [collection, collaterals] of collectionAndCollateralList) {
      // single collateral per collection
      const collateral = collaterals.find(
        (collateral) => collateral.owner === borrowerAddress
      );

      if (collateral) {
        collectionAndCollateralListByBorrower.push([collection, collateral]);
      }
    }

    return collectionAndCollateralListByBorrower;
  };

  const getCollateralValue = (
    mainCurrency: CurrencyForToken,
    collectionAndCollateralListByBorrower: [Addr, Collateral][],
    prices: PriceItemForFundsForToken[]
  ) => {
    let isPriceOutdated = false;

    const collateralValue = collectionAndCollateralListByBorrower.reduce(
      (acc, [collectionAddress, collateral]) => {
        const priceItem = prices.find((x) => x.address === collectionAddress);
        if (!priceItem) return acc;

        if (priceItem.is_outdated) {
          isPriceOutdated = true;
        }

        const price = choosePrice(mainCurrency, priceItem);

        return acc + Number(price) * collateral.token_item_list.length;
      },
      0
    );

    return !isPriceOutdated ? collateralValue : undefined;
  };

  const queryLiquidationBids = async (
    collectionAddressList: string[],
    isMarketplaceInterfaceEnabled: boolean
  ): Promise<ArrayOfQueryLiquidationBidsByCollectionAddressListResponseItem> => {
    // query liquidation bids
    let liquidationBids =
      await lending.pQueryLiquidationBidsByCollectionAddressList(
        platformType,
        PAGINATION_AMOUNT,
        0,
        true
      );

    // query marketplace offers
    const offers = await utils.cwGetMarketplaceOffers(
      platformType,
      collectionAddressList,
      isMarketplaceInterfaceEnabled
    );
    liquidationBids = [...liquidationBids, ...offers];

    return liquidationBids;
  };

  // find bids containing at least 1 borrower's collateral token
  const getCollectionAndBidList = (
    liquidationBids: ArrayOfQueryLiquidationBidsByCollectionAddressListResponseItem,
    collectionAndCollateralListByBorrower: [Addr, Collateral][]
  ): [Addr, LiquidationBid][] => {
    let collectionAndBidList: [Addr, LiquidationBid][] = [];

    for (const { collection_address, liquidation_bids } of liquidationBids) {
      for (const bid of liquidation_bids) {
        const matchingCollateral = collectionAndCollateralListByBorrower.some(
          ([collection, collateral]) =>
            collection === collection_address &&
            bid.token_id_list.some((bidTokenId) =>
              collateral.token_item_list.some(({ id }) => id === bidTokenId)
            )
        );

        if (matchingCollateral) {
          collectionAndBidList.push([collection_address, bid]);
        }
      }
    }

    return collectionAndBidList;
  };

  // core logic
  const rateConfig = await lending.cwQueryRateConfig(platformType, true);
  const commonConfig = await lending.cwQueryCommonConfig(platformType, true);
  const borrowerList = await getBorrowers(borrowers);

  // get all collaterals
  const collectionAndCollateralList: [Addr, Collateral[]][] = (
    await lending.pQueryCollateralList(platformType, PAGINATION_AMOUNT, 0, true)
  ).map(({ address, collateral }) => [address, collateral]);

  // get collection prices
  const prices = await getCollectionPrices(
    borrowerList,
    collectionAndCollateralList
  );
  const blockTime = await oracle.cwQueryBlockTime(true);

  // accumulators
  let targets: LiquidationItem[] = [];

  // iterate over borrowers
  for (const [borrowerAddress, { accumulated_loan, loan }] of borrowerList) {
    // get collaterals owned by borrower
    const collectionAndCollateralListByBorrower = getCollateralsByBorrower(
      collectionAndCollateralList,
      borrowerAddress
    );

    const collateralValueOption = getCollateralValue(
      commonConfig.main_currency,
      collectionAndCollateralListByBorrower,
      prices
    );

    // go to next borrower if any price is outdated
    if (!collateralValueOption) {
      continue;
    }

    const collateralValue = collateralValueOption;

    // check ltv
    const ltvMax = calcLtvMax(
      Number(rateConfig.bid_min_rate),
      Number(rateConfig.discount_max_rate),
      Number(rateConfig.discount_min_rate)
    );

    const ltv = calcConditionalLtv(
      Number(rateConfig.borrow_apr),
      0,
      Number(accumulated_loan),
      Number(loan.amount),
      0,
      collateralValue,
      blockTime,
      loan.creation_date
    );

    // if ltv is low check next borrower, otherwise liquidate required amount
    if (ltv <= ltvMax) {
      continue;
    }

    // find bids containing at least 1 borrower's collateral token
    const collectionAddressList = collectionAndCollateralListByBorrower.map(
      ([address, _]) => address
    );
    const liquidationBids = await queryLiquidationBids(
      collectionAddressList,
      commonConfig.is_marketplace_interface_enabled
    );
    const collectionAndBidList = getCollectionAndBidList(
      liquidationBids,
      collectionAndCollateralListByBorrower
    );

    const biddedCollateralList = matchCollateralsAndBidsLegacy(
      commonConfig.main_currency,
      collectionAndCollateralListByBorrower,
      collectionAndBidList,
      prices
    );

    // calculate how much tokens and what collection must be liquidated to get proper ltv
    // for worst case (max discount)
    const maxLiquidationValue = calcLiquidationValue(
      collateralValue,
      Number(rateConfig.discount_max_rate),
      ltv,
      ltvMax,
      1
    );

    const liquidation_set = calcLiquidationSet(
      biddedCollateralList,
      maxLiquidationValue,
      ltvMax,
      Number(rateConfig.borrow_apr),
      Number(accumulated_loan),
      Number(loan.amount),
      collateralValue,
      blockTime,
      loan.creation_date
    );

    // check liquidation set
    if (!liquidation_set.length) {
      l(
        `⚠️⚠️⚠️ Warning: No (proper) bids/offers to liquidate ${borrowerAddress}`
      );
    } else {
      targets.push({
        borrower: borrowerAddress,
        liquidation_set,
      });
    }
  }

  return targets;
}
