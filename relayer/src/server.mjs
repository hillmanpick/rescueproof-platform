import { createServer } from "node:http";
import { readFile, rename, writeFile } from "node:fs/promises";
import { timingSafeEqual } from "node:crypto";
import { JsonRpcProvider, Wallet, Contract } from "ethers";

const required = ["RELAYER_TOKEN", "RELAYER_PRIVATE_KEY", "CHAIN_ID", "CONTRACT_ADDRESS", "RPC_URL"];
for (const key of required) if (!process.env[key]) throw new Error(`${key} is required`);
const token = Buffer.from(process.env.RELAYER_TOKEN);
const chainId = Number(process.env.CHAIN_ID);
if (!Number.isSafeInteger(chainId) || chainId < 1) throw new Error("CHAIN_ID must be a positive integer");
const provider = new JsonRpcProvider(process.env.RPC_URL, chainId, { staticNetwork: true });
const signer = new Wallet(process.env.RELAYER_PRIVATE_KEY, provider);
const contractAddress = process.env.CONTRACT_ADDRESS;
const contract = new Contract(contractAddress, ["function owner() view returns (address)", "function anchor(bytes32 root) external", "function roots(bytes32) view returns (uint256)", "event RootAnchored(bytes32 indexed root, uint256 timestamp)"], signer);
const network = await provider.getNetwork();
if (network.chainId !== BigInt(chainId)) throw new Error("RPC chain ID does not match CHAIN_ID");
if (await provider.getCode(contractAddress) === "0x") throw new Error("CONTRACT_ADDRESS has no deployed code");
if ((await contract.owner()).toLowerCase() !== signer.address.toLowerCase()) throw new Error("Relayer signer is not the contract owner");
const stateFile = process.env.STATE_FILE || "./relayer-state.json";
let state = {};
try { state = JSON.parse(await readFile(stateFile, "utf8")); } catch (error) { if (error.code !== "ENOENT") throw error; }
let queue = Promise.resolve();

function equalSecret(candidate) {
  const value = Buffer.from(candidate || "");
  return value.length === token.length && timingSafeEqual(value, token);
}
async function save() {
  const next = `${stateFile}.tmp`;
  await writeFile(next, JSON.stringify(state), { mode: 0o600 });
  await rename(next, stateFile);
}
async function body(req) {
  let data = "";
  for await (const chunk of req) {
    data += chunk;
    if (data.length > 4096) throw Object.assign(new Error("Request too large"), { status: 413 });
  }
  return JSON.parse(data);
}
function send(res, status, value) {
  res.writeHead(status, { "content-type": "application/json", "cache-control": "no-store" });
  res.end(JSON.stringify(value));
}

const server = createServer(async (req, res) => {
  if (req.method === "GET" && req.url === "/health") return send(res, 200, { ok: true, chain_id: chainId });
  if (req.method !== "POST" || req.url !== "/anchors") return send(res, 404, { error: "not_found" });
  if (!equalSecret(req.headers.authorization?.replace(/^Bearer\s+/i, ""))) return send(res, 401, { error: "unauthorized" });
  let input;
  try { input = await body(req); } catch (error) { return send(res, error.status || 400, { error: "invalid_request" }); }
  const key = req.headers["idempotency-key"];
  if (typeof key !== "string" || key !== input.anchor_id || !/^0x[0-9a-fA-F]{64}$/.test(input.root || "") || input.chain_id !== chainId || input.contract_address?.toLowerCase() !== contractAddress.toLowerCase()) {
    return send(res, 400, { error: "invalid_anchor" });
  }
  try {
    const pending = queue.then(async () => {
      if (state[key]?.root !== input.root.toLowerCase()) {
        const stored = await contract.roots(input.root);
        let hash = state[key]?.transaction_hash;
        if (stored === 0n) {
          const tx = await contract.anchor(input.root);
          hash = tx.hash;
          state[key] = { root: input.root.toLowerCase(), transaction_hash: hash };
          await save();
          await tx.wait(1);
        } else if (!hash) {
          const filter = contract.filters.RootAnchored(input.root);
          const logs = await contract.queryFilter(filter, Math.max(0, await provider.getBlockNumber() - 10000));
          hash = logs.at(-1)?.transactionHash;
          if (!hash) throw new Error("Root exists but source transaction is unavailable");
          state[key] = { root: input.root.toLowerCase(), transaction_hash: hash };
          await save();
        }
      }
      return state[key].transaction_hash;
    });
    queue = pending.catch(() => {});
    const result = await pending;
    return send(res, 200, { transaction_hash: result });
  } catch {
    return send(res, 502, { error: "anchor_submission_failed" });
  }
});
server.listen(Number(process.env.PORT || 8080), "0.0.0.0");
