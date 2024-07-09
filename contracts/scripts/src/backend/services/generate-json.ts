import { CHAIN_CONFIG } from "../../common/config";
import { writeFile } from "fs/promises";
import { ENCODING, PATH_TO_CONFIG_JSON } from "./utils";

async function main() {
  await writeFile(PATH_TO_CONFIG_JSON, JSON.stringify(CHAIN_CONFIG, null, 2), {
    encoding: ENCODING,
  });
}

main();
