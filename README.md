# RescueProof

RescueProof helps food businesses share surplus food with approved community organisations. Both sides confirm a handover, and the system produces verifiable integrity evidence without publishing personal details or claiming to certify food safety.

The first release is a modular Rust/Axum API with PostgreSQL as the source of truth, Redis for rate limits, a Vue 3 mobile-friendly web app, an Outbox worker and an optional EVM root-anchor relayer.

## Repository

```text
frontend/      Vue 3 + TypeScript application
backend/       Axum API, domain rules, worker, SQL migrations
contracts/     Minimal EVM root-anchor contract
deploy/        Caddy and container build definitions
docs/          API, security, operations and relayer notes
scripts/       Backup helpers
compose.yaml   Production-shaped single-host deployment
```

## Local development

Prerequisites: Rust 1.96+, Node.js 22+, Docker Desktop and a Redis service. From PowerShell:

```powershell
Copy-Item .env.example .env
docker compose -f compose.dev.yaml up -d
```

The development database listens only on `127.0.0.1:54329`; Redis listens on `127.0.0.1:63799`. If you already run Redis locally, use `redis://127.0.0.1:6379` in `.env` instead. Set `DATABASE_URL` to the development port and use a local-only password. Start separate terminals:

```powershell
cargo run -- serve
cargo run -- worker
cd frontend
npm ci --registry=https://registry.npmjs.org
npm run dev
```

Open <http://127.0.0.1:5173>. Migrations apply automatically at service start. Readiness checks both PostgreSQL and Redis.

For a first local admin, register a normal user and promote it from the same trusted machine:

```powershell
cargo run -- promote-admin admin@example.org
```

The admin account can approve donor and recipient organisations in the UI. Registration intentionally leaves organisations pending until a human operator verifies them.

## Checks

For local UI and workflow testing, populate the development database using the [demo data command](docs/demo-data.md). It preserves existing users and supplies labelled food batches, timeline events, proofs and separate role accounts.

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
$env:TEST_DATABASE_URL = 'postgres://rescueproof:password@127.0.0.1:54329/rescueproof_test'
$env:TEST_REDIS_URL = 'redis://127.0.0.1:63799'
cargo test --test workflow -- --ignored
cd frontend
npm run build
npm audit --registry=https://registry.npmjs.org
```

The ignored integration test requires a fresh, dedicated database with `test` in its database name. It exercises real PostgreSQL transactions, concurrent claims, access controls, session revocation, token rotation, dispute metrics and Merkle proofs. Never point it at a development or production database.

## Production deployment

For a domain deployment, copy `.env.example` to an untracked `.env`, create unique random database and relayer credentials, set `PUBLIC_ORIGIN=https://your-domain`, and leave `COOKIE_SECURE=true`. Permit the selected public web ports through the firewall. PostgreSQL and Redis live on an internal container network and have no host port published. For a public IP without a domain, use the [IP HTTPS deployment guide](docs/deployment-ip.md), including the small-server Compose override and automatic IP certificate renewal. Keep secure cookies enabled.

```powershell
docker compose up -d --build
docker compose ps
docker compose logs --tail 100 api worker web
```

Promote the first operator through an authenticated server shell after registration. Back up with `sh scripts/backup.sh`; copy verified archives to storage outside this server and test restores. Container volumes alone are not backups.

See [Operations](docs/operations.md), [API](docs/api.md), [Security](docs/security.md), and [Relayer](docs/relayer.md).

For an Ethereum Sepolia anchoring demo (chain ID 11155111), deploy the `RescueProofAnchor` contract with a dedicated test wallet and configure the isolated Relayer using [Relayer setup](docs/relayer.md). The chain integration is optional; without the complete configuration, records remain pending and are never presented as on-chain verified. Run `cd relayer; npm ci; npm test` to exercise contract deployment, relayer authentication/idempotency, a mined local EVM transaction, and on-chain root lookup.

## Important limits

- Organisation review and any food-safety decisions require accountable human operators.
- Impact figures use donor/recipient supplied data and are not independently audited.
- An anchored root proves that an event payload has not changed relative to the root. It does not prove that food existed, was safe, or was physically delivered.
- Without a configured relayer, evidence stays `pending`; no chain transaction is fabricated.
