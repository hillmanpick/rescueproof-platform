import { readFile, writeFile, rename } from "node:fs/promises";
import solc from "solc";
import { ContractFactory, JsonRpcProvider, Wallet } from "ethers";

const rpc = process.env.ANCHOR_RPC_URL;
const key = process.env.RELAYER_PRIVATE_KEY;
if (!rpc || !key) throw new Error("Set ANCHOR_RPC_URL and RELAYER_PRIVATE_KEY in the environment");
const source = await readFile(new URL("../../contracts/RescueProofAnchor.sol", import.meta.url), "utf8");
const input = {
  language: "Solidity",
  sources: { "RescueProofAnchor.sol": { content: source } },
  settings: { optimizer: { enabled: true, runs: 200 }, outputSelection: { "*": { "*": ["abi", "evm.bytecode.object"] } } },
};
const output = JSON.parse(solc.compile(JSON.stringify(input)));
const errors = (output.errors || []).filter((error) => error.severity === "error");
if (errors.length) throw new Error(errors.map((error) => error.formattedMessage).join("\n"));
const artifact = output.contracts["RescueProofAnchor.sol"].RescueProofAnchor;
const provider = new JsonRpcProvider(rpc);
const signer = new Wallet(key, provider);
const network = await provider.getNetwork();
if (network.chainId !== 11155111n) throw new Error(`Refusing deployment to chain ${network.chainId}; this script is for Ethereum Sepolia (11155111)`);
const deploymentFile = process.env.DEPLOYMENT_FILE;
async function persist(value) {
  if (!deploymentFile) return;
  await writeFile(`${deploymentFile}.tmp`, JSON.stringify(value, null, 2));
  await rename(`${deploymentFile}.tmp`, deploymentFile);
}
let saved;
if (deploymentFile) {
  try { saved = JSON.parse(await readFile(deploymentFile, "utf8")); }
  catch (error) { if (error.code !== "ENOENT") throw error; }
}
if (saved) {
  if (saved.chain_id !== Number(network.chainId) || saved.deployer !== signer.address) throw new Error("Deployment file belongs to another chain or signer");
  const receipt = await provider.waitForTransaction(saved.transaction_hash, 1);
  if (receipt.status !== 1 || receipt.contractAddress?.toLowerCase() !== saved.contract_address.toLowerCase() || receipt.from.toLowerCase() !== signer.address.toLowerCase() || await provider.getCode(saved.contract_address) === "0x") throw new Error("Saved deployment did not succeed");
  saved.block_number = receipt.blockNumber;
  await persist(saved);
  console.log(JSON.stringify(saved, null, 2));
  provider.destroy();
} else {
  const factory = new ContractFactory(artifact.abi, `0x${artifact.evm.bytecode.object}`, signer);
  const deployed = await factory.deploy(signer.address);
  const result = { chain_id: Number(network.chainId), deployer: signer.address, contract_address: await deployed.getAddress(), transaction_hash: deployed.deploymentTransaction().hash };
  await persist(result);
  await deployed.waitForDeployment();
  const receipt = await deployed.deploymentTransaction().wait(1);
  result.block_number = receipt.blockNumber;
  await persist(result);
  console.log(JSON.stringify(result, null, 2));
  provider.destroy();
}
