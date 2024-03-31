import { getSigner } from "../account/signer";
import { l } from "../../common/utils";
import { readFile } from "fs/promises";
import { ChainConfig } from "../../common/interfaces";
import {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  getWallets,
  parseStoreArgs,
} from "./utils";
import {
  getChainOptionById,
  getContractByWasm,
} from "../../common/config/config-utils";
import {
  getSgQueryHelpers,
  getSgExecHelpers,
} from "../../common/account/sg-helpers";
import {
  getCwExecHelpers,
  getCwQueryHelpers,
} from "../../common/account/cw-helpers";

import { LiquidationItem } from "../../common/codegen/LendingPlatform.types";

// : Promise<LiquidationItem[]>
export async function queryTargets(borrowers: string[]) {
  // let block_time = env.block.time.seconds();
  // let address_config = ADDRESS_CONFIG.load(deps.storage)?;
  // let rate_config = RATE_CONFIG.load(deps.storage)?;
  // let common_config = COMMON_CONFIG.load(deps.storage)?;
  // let oracle = &unwrap_field(address_config.oracle, "oracle")?;
  // let borrowers = get_borrowers(deps.storage, deps.api, &borrowers)?;
  // // get all collaterals
  // let collection_and_collateral_list: Vec<(Addr, Vec<Collateral>)> = COLLATERALS
  //     .range(deps.storage, None, None, Order::Ascending)
  //     .flatten()
  //     .collect();
  // // get collection prices
  // let prices = get_collection_prices(deps, &borrowers, &collection_and_collateral_list, oracle)?;
  // // accumulators
  // let mut targets: Vec<LiquidationItem> = vec![];
  // // iterate over borrowers
  // for (
  //     borrower_address,
  //     Borrower {
  //         accumulated_loan,
  //         loan,
  //     },
  // ) in borrowers
  // {
  //     // get collaterals owned by borrower
  //     let collection_and_collateral_list_by_borrower =
  //         get_collaterals_by_borrower(&collection_and_collateral_list, &borrower_address)?;
  //     let collateral_value_option = get_collateral_value(
  //         &collection_and_collateral_list_by_borrower,
  //         &prices,
  //         block_time,
  //         common_config.price_update_period,
  //     )?;
  //     // go to next borrower if any price is outdated
  //     if collateral_value_option.is_none() {
  //         continue;
  //     }
  //     let collateral_value = collateral_value_option.unwrap();
  //     // check ltv
  //     let ltv_max = math::calc_ltv_max(
  //         rate_config.bid_min_rate,
  //         rate_config.discount_max_rate,
  //         rate_config.discount_min_rate,
  //     );
  //     let ltv = math::calc_conditional_ltv(
  //         rate_config.borrow_apr,
  //         Uint128::zero(),
  //         accumulated_loan,
  //         loan.amount,
  //         Uint128::zero(),
  //         collateral_value,
  //         block_time,
  //         loan.creation_date,
  //     );
  //     // if ltv is low check next borrower, otherwise liquidate required amount
  //     if ltv <= ltv_max {
  //         continue;
  //     }
  //     // find bids containing at least 1 borrower's collateral token
  //     let collection_and_bid_list =
  //         get_collection_and_bid_list(deps.storage, &collection_and_collateral_list_by_borrower)?;
  //     let bidded_collateral_list = math::match_collaterals_and_bids(
  //         &collection_and_collateral_list_by_borrower,
  //         &collection_and_bid_list,
  //         &prices,
  //     )?;
  //     // calculate how much tokens and what collection must be liquidated to get proper ltv
  //     // for worst case (max discount)
  //     let max_liquidation_value = math::calc_liquidation_value(
  //         collateral_value,
  //         rate_config.discount_max_rate,
  //         ltv,
  //         ltv_max,
  //     );
  //     let liquidation_set = math::calc_liquidation_set(
  //         &bidded_collateral_list,
  //         max_liquidation_value,
  //         ltv_max,
  //         rate_config.borrow_apr,
  //         accumulated_loan,
  //         loan.amount,
  //         collateral_value,
  //         block_time,
  //         loan.creation_date,
  //     );
  //     targets.push(LiquidationItem {
  //         borrower: borrower_address.to_string(),
  //         liquidation_set,
  //     });
  // }
  // Ok(targets)
}

async function main() {
  // const configJsonStr = await readFile(PATH_TO_CONFIG_JSON, {
  //   encoding: ENCODING,
  // });
  // const CHAIN_CONFIG: ChainConfig = JSON.parse(configJsonStr);
  // const { chainId } = parseStoreArgs();
  // const {
  //   PREFIX: PREFIX,
  //   OPTION: {
  //     RPC_LIST: [RPC],
  //     DENOM: DENOM,
  //     CONTRACTS: CONTRACTS,
  //     GAS_PRICE_AMOUNT: GAS_PRICE_AMOUNT,
  //     TYPE: TYPE,
  //   },
  // } = getChainOptionById(CHAIN_CONFIG, chainId);
  // const LENDING_PLATFORM_CONTRACT = getContractByWasm(
  //   CONTRACTS,
  //   "lending_platform.wasm"
  // );
  // const ORACLE_CONTRACT = getContractByWasm(CONTRACTS, "oracle.wasm");
  // const gasPrice = `${GAS_PRICE_AMOUNT}${DENOM}`;
  // const testWallets = await getWallets(TYPE);
  // const { signer: signer, owner: owner } = await getSigner(
  //   PREFIX,
  //   testWallets.SEED_ADMIN
  // );
  // const sgQueryHelpers = await getSgQueryHelpers(RPC);
  // const sgExecHelpers = await getSgExecHelpers(RPC, owner, signer);
  // const {
  //   cwQueryLendingPlatformConfig,
  //   cwQueryOracleConfig,
  //   cwQueryPrices,
  //   cwQueryBlockCounter,
  // } = await getCwQueryHelpers(chainId, RPC);
  // const { cwUpdateConfig, cwUpdatePrices } = await getCwExecHelpers(
  //   chainId,
  //   RPC,
  //   owner,
  //   signer
  // );
}
