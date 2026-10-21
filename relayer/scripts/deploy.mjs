import { readFile } from "node:fs/promises";
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
if (network.chainId !== 84532n) throw new Error(`Refusing deployment to chain ${network.chainId}; this script is for Base Sepolia (84532)`);
const factory = new ContractFactory(artifact.abi, `0x${artifact.evm.bytecode.object}`, signer);
const deployed = await factory.deploy(signer.address);
await deployed.waitForDeployment();
console.log(JSON.stringify({ chain_id: Number((await provider.getNetwork()).chainId), deployer: signer.address, contract_address: await deployed.getAddress(), transaction_hash: deployed.deploymentTransaction().hash }, null, 2));
