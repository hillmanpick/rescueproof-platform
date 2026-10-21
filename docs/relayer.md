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

Use a dedicated signer with only the right to call `anchor`, keep it outside the application server and set transaction limits. Configure `ANCHOR_CHAIN_ID`, `ANCHOR_CONTRACT_ADDRESS`, `ANCHOR_RPC_URL`, `ANCHOR_RELAYER_URL` and `ANCHOR_RELAYER_TOKEN` together. Prefer Base Sepolia for a demo; do not write mainnet transactions until the user has configured and reviewed the target contract and relayer. When any values are missing, evidence remains pending.

## Merkle format

The canonical leaf payload is a compact JSON tuple in fixed order:

```text
["rescueproof:v1", event UUID, batch UUID, event kind, sanitized JSON payload, UTC timestamp with microseconds]
```

Leaf hash is `keccak256(keccak256(UTF8(canonical payload)))`. Internal nodes are `keccak256(min(left,right) || max(left,right))`; an odd leaf is paired with itself. This matches OpenZeppelin's `StandardMerkleTree` double-hashed, sorted-pair conventions. The `/proof` response contains the canonical leaf payload, leaf hash, siblings, root, contract, chain, transaction and confirmation status. A verifier recomputes the leaf and folds the proof through the sorted-pair hash until it matches the root, then checks `roots(root)` on the stated chain.

Payloads intentionally omit user IDs and private data. Public batch and organisation IDs are not sufficient to identify a person. The contract only timestamps roots; it does not assert the truth of off-chain statements.
