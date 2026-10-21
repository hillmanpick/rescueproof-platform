# Ethereum Sepolia public-network validation

Validated on 2026-10-21 with chain ID **11155111**. These are synthetic, explicitly labelled demo events; the records do not represent real food rescue.

- Contract: [`0xf79c830b25d18eeE339d91551b1e3f2668d583AB`](https://sepolia.etherscan.io/address/0xf79c830b25d18eeE339d91551b1e3f2668d583AB)
- Deployment: [`0x08f6a4b6584dfa4ff56f0359b198fa70198dd4ffc6a084f8d28e6c61a5f49418`](https://sepolia.etherscan.io/tx/0x08f6a4b6584dfa4ff56f0359b198fa70198dd4ffc6a084f8d28e6c61a5f49418), block 11961456.
- Anchor: [`0x0ee297d8bedcb2638edfe04352cd4a2ea1946a5f0012f73f0bce8272b2781c4b`](https://sepolia.etherscan.io/tx/0x0ee297d8bedcb2638edfe04352cd4a2ea1946a5f0012f73f0bce8272b2781c4b), block 11961467.
- Root: `0xb40545752f5723f034ed775573963907d1b8ae798748171f831ba0178dbc89d3`.

The application API returned 28 confirmed event proofs in one anchor. Every canonical payload, leaf and sorted-pair Merkle path was recomputed independently with ethers. Modifying each payload invalidated its proof (28 of 28 rejected). Both PublicNode and dRPC returned the successful receipt, matching contract, matching block, `RootAnchored` event and nonzero `roots(root)` value at that block. There were 15 confirmations at verification time. The current local batch detail page displayed the confirmed status and the correct Sepolia explorer link.

Deployment cost: 0.00000037176360628 SepoliaETH. Anchor cost: 0.00000002595611436 SepoliaETH. These are historical testnet fees, not production estimates. The application remains locally hosted; contract deployment does not deploy the web application to a public server. Explorer source-code verification is separate from checking deployed code and successful transactions.

To inspect a local demo, sign in with an involved demo organisation, open `/batches/de000000-0000-4000-8000-00000000006b` or `/batches/de000000-0000-4000-8000-00000000006d`, and use **Download full verification proof**. Recompute the Merkle proof as documented in [Relayer](relayer.md), then query `roots(root)` on the contract using an independent Ethereum Sepolia RPC. No signing key or test ETH is needed for these read-only queries.
