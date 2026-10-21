import { after, before, test } from "node:test";
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { createServer } from "node:net";
import { fileURLToPath } from "node:url";
import ganache from "ganache";
import solc from "solc";
import { ContractFactory, JsonRpcProvider, Wallet } from "ethers";
import { readFile } from "node:fs/promises";

const token = "test-relayer-token";
const relayerKey = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
let chain, provider, wallet, contract, relayer, relayerPort, chainId;

before(async () => {
  chain = ganache.server({ logging: { quiet: true }, wallet: { accounts: [{ secretKey: relayerKey, balance: "0x3635c9adc5dea00000" }] } });
  await new Promise((resolve) => chain.listen(0, "127.0.0.1", resolve));
  const rpc = `http://127.0.0.1:${chain.address().port}`;
  provider = new JsonRpcProvider(rpc);
  chainId = Number((await provider.getNetwork()).chainId);
  const source = await readFile(new URL("../../contracts/RescueProofAnchor.sol", import.meta.url), "utf8");
  const input = { language: "Solidity", sources: { "RescueProofAnchor.sol": { content: source } }, settings: { optimizer: { enabled: true, runs: 200 }, outputSelection: { "*": { "*": ["abi", "evm.bytecode.object"] } } } };
  const output = JSON.parse(solc.compile(JSON.stringify(input)));
  const artifact = output.contracts["RescueProofAnchor.sol"].RescueProofAnchor;
  wallet = Wallet.fromPhrase("test test test test test test test test test test test junk", provider);
  contract = await new ContractFactory(artifact.abi, `0x${artifact.evm.bytecode.object}`, wallet).deploy(wallet.address).then((value) => value.waitForDeployment());
  const tcp = createServer();
  tcp.listen(0, "127.0.0.1");
  await once(tcp, "listening");
  relayerPort = tcp.address().port;
  tcp.close();
  relayer = spawn(process.execPath, [fileURLToPath(new URL("../src/server.mjs", import.meta.url))], {
    env: { ...process.env, PORT: String(relayerPort), RELAYER_TOKEN: token, RELAYER_PRIVATE_KEY: relayerKey, CHAIN_ID: String(chainId), CONTRACT_ADDRESS: await contract.getAddress(), RPC_URL: rpc, STATE_FILE: `${process.env.TEMP || "/tmp"}/rescueproof-relayer-${process.pid}.json` },
    stdio: "ignore",
  });
  await waitForHealth();
});

after(async () => {
  relayer?.kill();
  await provider?.destroy();
  if (chain) await chain.close();
});

async function waitForHealth() {
  for (let i = 0; i < 50; i++) {
    try { if ((await fetch(`http://127.0.0.1:${relayerPort}/health`)).ok) return; } catch {}
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error("Relayer did not start");
}

test("authenticated idempotent anchor is mined and queryable on chain", async () => {
  const anchorId = "test-anchor-0001";
  const root = `0x${"42".repeat(32)}`;
  const url = `http://127.0.0.1:${relayerPort}/anchors`;
  const request = (authorization = `Bearer ${token}`) => fetch(url, { method: "POST", headers: { authorization, "content-type": "application/json", "idempotency-key": anchorId }, body: JSON.stringify({ anchor_id: anchorId, root, chain_id: chainId, contract_address: awaitAddress }) });
  const awaitAddress = await contract.getAddress();
  assert.equal((await request("Bearer wrong")).status, 401);
  const first = await request();
  assert.equal(first.status, 200);
  const firstBody = await first.json();
  assert.match(firstBody.transaction_hash, /^0x[0-9a-f]{64}$/i);
  const second = await request();
  assert.deepEqual(await second.json(), firstBody);
  await provider.waitForTransaction(firstBody.transaction_hash, 1);
  assert.ok((await contract.roots(root)) > 0n);
  const receipt = await provider.getTransactionReceipt(firstBody.transaction_hash);
  assert.equal(receipt.status, 1);
  assert.equal(receipt.to.toLowerCase(), awaitAddress.toLowerCase());
  const changed = await fetch(url, { method: "POST", headers: { authorization: `Bearer ${token}`, "content-type": "application/json", "idempotency-key": anchorId }, body: JSON.stringify({ anchor_id: anchorId, root: `0x${"43".repeat(32)}`, chain_id: chainId, contract_address: awaitAddress }) });
  assert.equal(changed.status, 409);
  assert.equal(await contract.roots(`0x${"43".repeat(32)}`), 0n);
});

test("malformed authenticated requests do not crash the relayer", async () => {
  for (const input of [null, {}, { anchor_id: "test-invalid-001", root: `0x${"44".repeat(32)}`, chain_id: chainId, contract_address: 123 }]) {
    const res = await fetch(`http://127.0.0.1:${relayerPort}/anchors`, { method: "POST", headers: { authorization: `Bearer ${token}`, "content-type": "application/json", "idempotency-key": "test-invalid-001" }, body: JSON.stringify(input) });
    assert.equal(res.status, 400);
  }
  assert.equal((await fetch(`http://127.0.0.1:${relayerPort}/health`)).status, 200);
});
