# Local demo fixtures

Run from the repository root, with `.env` pointing to a loopback development database and a local `PUBLIC_ORIGIN`:

```powershell
cargo run --bin seed_demo -- <existing-local-user-uuid>
```

The command requires an existing active local user and creates 14 food batches: six open, two claimed, one handed over, two completed, one disputed, one expired, and one cancelled. Bakery, produce, packaged food and prepared meals are represented. Every fixture batch and organisation is labelled `【演示】`, and event payloads include `demo: true`.

The existing user becomes a member of three approved demo donor organisations so their workspace immediately displays the fixtures. Their existing password, organisation review status and administrator privileges remain unchanged. Separate demo donor, recipient and administrator accounts have randomly generated passwords saved in the Git-ignored `work/demo-accounts.json`. The recipient can generate pickup codes, confirm receipts and open disputes; the administrator can review the pending demo organisation and inspect the seeded dispute.

Fixed UUIDs and a transaction-level advisory lock make reruns repeatable without duplicates. Already existing fixture batches and their lifecycle progress are preserved. Pickup windows are based on the first run and expire normally; rerunning does not extend them. Credentials must be kept with the database so they still match on reruns.

Timeline events go through the application's Merkle batching code. Proofs remain pending until a real chain transaction is submitted and checked. The seeder itself disables its relay configuration and never broadcasts transactions; an independently running Worker with anchoring configured can subsequently relay these marked demo records to a testnet.

The local overview includes two simulated completed deliveries totalling 20.5 kg, not actual rescued food. `/impact` reports `demo_data: true` when marked fixtures exist, and the homepage displays an explicit demo disclaimer in all four interface languages. Do not load fixtures into production or treat their totals as real impact.
