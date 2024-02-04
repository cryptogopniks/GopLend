import { l, wait, Request } from "../../common/utils";
import { getChainOptionById } from "../../common/config/config-utils";
import { readFile } from "fs/promises";
import { getCwClient } from "../../common/account/clients";
import { getSigner } from "../account/signer";
import { ChainConfig, QueryProposalsResponse } from "../../common/interfaces";
import { VoteOption } from "cosmjs-types/cosmos/gov/v1beta1/gov";
import { chains } from "../../common/config/logs.json";
import {
  parseStoreArgs,
  ENCODING,
  PATH_TO_CONFIG_JSON,
  getWallets,
} from "./utils";
import {
  getSgQueryHelpers,
  getSgExecHelpers,
} from "../../common/account/sg-helpers";

const req = new Request();

async function main() {
  try {
    const configJsonStr: string = await readFile(PATH_TO_CONFIG_JSON, {
      encoding: ENCODING,
    });
    let configJson: ChainConfig = JSON.parse(configJsonStr);

    const { chainId } = parseStoreArgs();
    const {
      PREFIX,
      OPTION: {
        DENOM,
        RPC_LIST: [RPC],
        GAS_PRICE_AMOUNT,
        TYPE,
      },
    } = getChainOptionById(configJson, chainId);

    // admin as proposer
    // alice as voter
    const testWallets = await getWallets(TYPE);
    const { signer: signerVoter, owner: ownerVoter } = await getSigner(
      PREFIX,
      testWallets.SEED_ALICE
    );
    const cwClientVoter = await getCwClient(RPC, ownerVoter, signerVoter);
    if (!cwClientVoter) throw new Error("cwClient is not found!");

    // bob as admin
    const { signer: signerAdmin, owner: ownerAdmin } = await getSigner(
      PREFIX,
      testWallets.SEED_BOB
    );
    const cwClientAdmin = await getCwClient(RPC, ownerAdmin, signerAdmin);
    if (!cwClientAdmin) throw new Error("cwClient is not found!");

    const gasPrice = `${GAS_PRICE_AMOUNT}${DENOM}`;

    const { getValidators } = await getSgQueryHelpers(RPC);

    const { sgDelegate, sgVote } = await getSgExecHelpers(
      RPC,
      ownerVoter,
      signerVoter
    );

    const queryProposals = async () => {
      const endpoint = `${rest_address}/cosmos/gov/v1beta1/proposals?pagination.reverse=true&pagination.limit=3`;
      const {
        proposals: [prop],
      } = await req.get<QueryProposalsResponse>(endpoint);
      return prop;
    };

    // 1) delegate to be able to vote
    const delegationAmount = 10_000_000_000_000; // defaultStake is 5_000_000_000_000;
    const [{ operatorAddress, tokens }] = await getValidators();

    if (+tokens < delegationAmount) {
      await sgDelegate(operatorAddress, delegationAmount, DENOM, gasPrice);
    }

    // 2) query proposal
    const [{ rest_address }] = chains;
    const prop = await queryProposals();
    const proposalId = +prop.proposal_id;
    l({ proposalId }, "\n");

    // 3) vote
    await sgVote(proposalId, VoteOption.VOTE_OPTION_YES, gasPrice);

    // 4) check results
    await wait(9_000);

    const propRes = await queryProposals();
    l(propRes);
  } catch (error) {
    l(error);
  }
}

main();
