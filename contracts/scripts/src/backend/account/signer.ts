import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { calculateFee as _calculateFee } from "@cosmjs/stargate";
import { stringToPath, HdPath } from "@cosmjs/crypto";

export interface SignerData {
  signer: DirectSecp256k1HdWallet;
  owner: string;
}

export async function getSigner(
  prefix: string,
  seed: string,
  hdPath?: HdPath
): Promise<SignerData> {
  const signer = await DirectSecp256k1HdWallet.fromMnemonic(seed, {
    prefix,
    hdPaths: hdPath ? [hdPath] : undefined,
  });
  const [{ address: owner }] = await signer.getAccounts();

  return { signer, owner };
}

export async function getMultipleSigners(
  prefix: string,
  seed: string,
  numAccounts: number
): Promise<SignerData[]> {
  const signers = [];

  for (let i = 0; i < numAccounts; i++) {
    // https://www.ledger.com/blog/understanding-crypto-addresses-and-derivation-paths
    const hdPath = stringToPath(`m/44'/118'/0'/0/${i}`);
    const { signer, owner } = await getSigner(prefix, seed, hdPath);
    signers.push({ signer, owner });
  }

  return signers;
}
