# Local closed-loop smoke test

Run from the repository root after starting the local API, PostgreSQL, Redis, Worker and Sepolia Relayer, and after explicitly creating the demo fixtures:

```powershell
node scripts/smoke-local.mjs
```

The script uses only `http://127.0.0.1:3000`, reads ignored `work/demo-accounts.json`, and uses separate donor, recipient and admin sessions. It does not read a wallet private key. It requires the ethers dependency installed in `relayer/`. Reports and exported proofs default to `work/e2e/`; override with `RESCUEPROOF_TEST_OUTPUT_DIR`. Demo credentials and session cookies are never included in the report.

Each run creates three explicitly labelled demo batches. It checks publishing, concurrent claims, pickup-code rotation, handover, receipt, notification processing, impact totals, cancellation and re-claiming, dispute resolution, permission boundaries, idempotency and logout. It waits up to 180 seconds for every newly generated event to be anchored, then independently verifies receipts, contract events, stored roots, Merkle paths and tamper rejection over dRPC on Ethereum Sepolia. Newly created data is retained as an audit trail. The script fails if the configured application does not produce confirmed Sepolia proofs; it does not substitute a mock chain or edit statuses in the database.

The enabled Worker may broadcast transactions as a consequence of these API operations. Use this against the local demo deployment with a dedicated Sepolia signer. These are synthetic food records, not real food delivery.

## Recorded run on 2026-10-21

All 62 API and chain checks passed using newly created batches. The successful handover batch was `129a57a0-fa5b-4578-9201-2f11219afe36`; both parties saw it completed, exactly four business events were recorded, and received weight increased the demonstration impact total by exactly 2.4 kg once.

Additional browser tests published and cancelled batch `dd9582e5-10df-495d-a6c0-fee34ed20eb7`, observed its proofs automatically become confirmed without a reload, and downloaded the evidence from the real UI. Both downloaded proofs passed independent on-chain and Merkle verification. Four-language catalog validation, TypeScript checks and production build passed. In total, 68 recorded checks passed; 16 new event proofs were confirmed in four transactions. These are separate new proofs from the initial 28 seeded-event proofs described in [Sepolia validation](sepolia-validation.md).

The test exposed stale confirmation status on the detail page. It now polls incomplete evidence every five seconds while visible, retains existing evidence during transient failures, stops polling after all event proofs are confirmed, and clears its timer on unmount. It also shows the pending notice when a later event has not yet been included in the displayed proofs.

The main business flow was exercised through actual HTTP requests with independent account sessions. Browser interactions covered the publishing form, cancellation, automatic status updates and proof download. This does not constitute an automated browser test of every button or a production deployment/load test.
