import { ChainConfig, LocalInterchainLogs } from "../../common/interfaces";
import LOGS from "../../common/config/logs.json";
import { writeFile, readFile } from "fs/promises";
import { ENCODING, PATH_TO_CONFIG_JSON } from "./utils";

async function main() {
  const logs = LOGS as LocalInterchainLogs;
  const configJsonStr: string = await readFile(PATH_TO_CONFIG_JSON, {
    encoding: ENCODING,
  });
  let configJson: ChainConfig = JSON.parse(configJsonStr);

  // update CONFIG_JSON with rpc_address from LOGS
  for (const { chain_id, rpc_address } of logs.chains) {
    configJson = {
      ...configJson,
      CHAINS: configJson.CHAINS.map((chain) => {
        return {
          ...chain,
          OPTIONS: chain.OPTIONS.map((option) => {
            if (option.CHAIN_ID !== chain_id) return option;

            return {
              ...option,
              RPC_LIST: [rpc_address],
            };
          }),
        };
      }),
    };
  }

  // update CONFIG_JSON with IBC parameters from LOGS
  for (const {
    chain_id,
    channel: { channel_id, port_id, counterparty },
  } of logs.ibc_channels) {
    configJson = {
      ...configJson,
      CHAINS: configJson.CHAINS.map((chain) => {
        return {
          ...chain,
          OPTIONS: chain.OPTIONS.map((option) => {
            if (option.CHAIN_ID !== chain_id) return option;

            // find counterparty
            const [counterpartyConfig] = logs.ibc_channels.filter(
              (x) =>
                x.channel.channel_id === counterparty.channel_id &&
                x.channel.port_id === counterparty.port_id &&
                x.chain_id !== chain_id
            );
            if (!counterpartyConfig) {
              throw new Error("Counterparty config is not found!");
            }

            return {
              ...option,
              IBC: [
                {
                  CHANNEL_ID: channel_id,
                  PORT: port_id,
                  COUNTERPARTY_CHAIN_ID: counterpartyConfig.chain_id,
                },
              ],
            };
          }),
        };
      }),
    };
  }

  await writeFile(PATH_TO_CONFIG_JSON, JSON.stringify(configJson, null, 2), {
    encoding: ENCODING,
  });
}

main();
