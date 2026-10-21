# Anchor relayer contract

The API Worker sends a batched Merkle root to a separately operated, trusted relayer. The API never stores the relayer signing key. A relayer must provide:

```http
POST /anchors
Authorization: Bearer <shared-service-secret>
Idempotency-Key: <anchor UUID>
Content-Type: application/json
```

```json
{
  "anchor_id":"00000000-0000-4000-8000-000000000001",
  "root":"0x<32-byte-keccak-root>",
  "chain_id":84532,
  "contract_address":"0x<deployed-RescueProofAnchor-address>"
}
```

Success must return `{"transaction_hash":"0x<32-byte transaction hash>"}` and must be idempotent for the anchor ID. Returning a hash means only “submitted”, not “confirmed”. The Worker independently queries the configured RPC, requires a successful receipt addressed to the configured contract, checks that `roots(bytes32)` at that block contains the exact submitted root, and verifies the RPC chain ID. Only then does the API mark it confirmed.

The repository includes the isolated Node.js Relayer in `relayer/`. It accepts only authenticated, bounded requests with matching idempotency IDs; checks the configured chain, deployed contract code and owner at startup; serializes writes to a contract; waits for a mined transaction; and stores idempotency results in a persistent file. Its signing key is not passed to the API container. In Compose, enable it with `docker compose --profile anchor up -d --build relayer`; the API reaches it at `http://relayer:8080` over the non-published Docker network. HTTP is allowed only for this fixed private service hostname. Keep the signer funded only with a small testnet balance.

For the Base Sepolia demo, set `ANCHOR_CHAIN_ID=84532`, `ANCHOR_RPC_URL`, `ANCHOR_RELAYER_URL=http://relayer:8080`, a random `ANCHOR_RELAYER_TOKEN`, and a freshly deployed `ANCHOR_CONTRACT_ADDRESS`. Put the dedicated key in `RELAYER_PRIVATE_KEY`; do not use a personal wallet key or commit `.env`. Deploy from a secure local terminal after funding a dedicated wallet with Base Sepolia test ETH:

```powershell
cd relayer
npm ci
$env:ANCHOR_RPC_URL = 'https://your-base-sepolia-rpc'
$env:RELAYER_PRIVATE_KEY = '0x<dedicated-testnet-key>'
npm run deploy
```

Copy the reported contract address and the same RPC, chain ID, relayer URL, private key and shared token into the untracked root `.env`. Start the app with `docker compose --profile anchor up -d --build`; verify `docker compose ps` and worker logs. The public proof response includes the receipt status, contract, chain, block and transaction hash. The Detail page links confirmed Base Sepolia and Ethereum Sepolia transactions to their explorers. Do not use a mainnet key or mainnet RPC for this demo.

Run the local-chain end-to-end test with `cd relayer; npm ci; npm test`. It compiles and deploys the Solidity contract on an ephemeral Ganache EVM, submits a signed transaction through the HTTP Relayer, repeats the same idempotency key, and independently checks the receipt and stored root over JSON-RPC. This proves the software path, not a public-network transaction.

## Merkle format

The canonical leaf payload is a compact JSON tuple in fixed order:

```text
["rescueproof:v1", event UUID, batch UUID, event kind, sanitized JSON payload, UTC timestamp with microseconds]
```

Leaf hash is `keccak256(keccak256(UTF8(canonical payload)))`. Internal nodes are `keccak256(min(left,right) || max(left,right))`; an odd leaf is paired with itself. This matches OpenZeppelin's `StandardMerkleTree` double-hashed, sorted-pair conventions. The `/proof` response contains the canonical leaf payload, leaf hash, siblings, root, contract, chain, transaction and confirmation status. A verifier recomputes the leaf and folds the proof through the sorted-pair hash until it matches the root, then checks `roots(root)` on the stated chain.

Payloads intentionally omit user IDs and private data. Public batch and organisation IDs are not sufficient to identify a person. The contract only timestamps roots; it does not assert the truth of off-chain statements.
