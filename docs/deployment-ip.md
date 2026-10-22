# Public IP deployment

The deployed demonstration lives in `/home/rescueproof` and is reachable at
`https://124.156.199.59:10880`. It uses the existing Ethereum Sepolia contract
`0xf79c830b25d18eeE339d91551b1e3f2668d583AB` (chain ID 11155111).

## Entry point and certificates

Update on 2026-10-22: desktop PostgreSQL access was explicitly enabled on TCP
5678 with mandatory TLS. The deployed `.env` now selects
`compose.yaml:compose.ip.yaml:compose.database-public.yaml`; use `sudo docker compose`
without `-f`, or include all three files in explicit commands below. The
internal-only description below records the initial topology. See
[public database access](database-public.md) for client settings and certificate renewal.

Use all three Compose files above. Nginx serves the prebuilt Vue bundle and
proxies same-origin `/api/*` and `/health/*` requests to Rust. Host TCP 80 maps
to Nginx HTTP; 10880 maps to Nginx HTTPS on container 443. Existing host services
on 443 and 8443 are preserved. `deploy/nginx.conf.template` is the active entry
configuration; old Caddy entry files are not mounted.

The internal-only `cert-manager` uses `deploy/Caddyfile.certificates` to renew
the trusted IP certificate using ACME `shortlived` and HTTP-01. It has no public
port mapping and does not serve the application. Nginx forwards only HTTP-01
challenge paths to it. **Keep TCP 80 open for renewal.** Existing `caddy_data`
and `caddy_config` volumes are retained; never delete certificate storage. The
certificate lasts approximately six days. Nginx mounts certificates read-only.

The existing `rescueproof-postgres-tls.timer` validates certificate/key pairs,
reloads Nginx gracefully and updates PostgreSQL TLS every 15 minutes. Run
`sudo systemctl start rescueproof-postgres-tls.service` after manual renewal.
Nginx enables HTTP/1.1 and HTTP/2, a default TLS virtual host for IP/no-SNI
clients, asset caching, gzip and SPA fallback. Port 80 redirects to HTTPS.
Access logs include request IDs and upstream status, without query strings.

On a fresh server, obtain a certificate before starting Nginx. Start
`cert-manager`, temporarily route public port 80 to its port 80 for HTTP-01,
and wait for the certificate files before transferring that listener to Nginx.
This migration reused existing valid certificate storage.

Required non-secret settings in the protected server `.env`:

```dotenv
SITE_ADDRESS=124.156.199.59
PUBLIC_ORIGIN=https://124.156.199.59:10880
WEB_HTTP_PORT=80
WEB_HTTPS_PORT=10880
COOKIE_SECURE=true
ANCHOR_CHAIN_ID=11155111
ANCHOR_CONTRACT_ADDRESS=0xf79c830b25d18eeE339d91551b1e3f2668d583AB
ANCHOR_RPC_URL=https://ethereum-sepolia-rpc.publicnode.com
ANCHOR_RELAYER_URL=http://relayer:8080
WEB_IMAGE=rescueproof-web:20261022-nginx
API_IMAGE=rescueproof-api:20261021-ip10880
RELAYER_IMAGE=rescueproof-relayer:20261021-ip10880
```

Generate separate random `POSTGRES_PASSWORD` and `ANCHOR_RELAYER_TOKEN`; configure
the dedicated test wallet in `RELAYER_PRIVATE_KEY`. Never commit these values or
print expanded Compose configuration. Keep `.env` mode 600 and the project
directory mode 700. Only the Relayer container receives the signing key.

## Build and operate

The IP override expects `frontend/dist` to be built beforehand. On a development
machine, run `npm ci` and `npm run build` from `frontend`, then transfer the source
and bundle without `.env`, `node_modules`, `target`, `.git`, or `work`.
The Dockerfile-specific ignore file permits this bundle into the web image while
excluding other local files. On the server:

```sh
cd /home/rescueproof
sudo docker compose --profile anchor config --quiet
sudo docker compose build --build-arg CARGO_BUILD_JOBS=1 api web relayer
sudo docker compose --profile anchor up -d --no-build
sudo docker compose exec -T web nginx -t
sudo docker compose --profile anchor ps
sudo docker compose logs --tail 100 web api worker relayer
curl --fail https://124.156.199.59:10880/health/ready
```

For a shared 2GB machine, constrain a dedicated BuildKit builder rather than
letting Rust compilation compete with existing services. The initial build used
one compilation job, one CPU, a 1200MB builder memory limit and 2200MB memory plus
swap allowance. Runtime service memory limits total 1024MB; these are ceilings,
not reserved allocations. PostgreSQL uses 64MB shared buffers and 50 connections;
Redis caps its cache at 64MB. No database or Relayer port is published.

Release-specific image tags allow a deliberate rollback to retained images.
Keep previous images, matching source and a database backup; database migrations
may require a compatible restore rather than merely changing an image tag.
Do not run `docker compose down -v` during routine updates.

## Data and signing ownership

The initial migration copied 18 labelled food batches and 44 event proofs;
password hashes were preserved but sessions and idempotency-key data were not.
Users must log in again. Subsequent public testing added three synthetic batches.
No real food or verified social impact is claimed by these records.

The production Relayer persists idempotency state in `relayer_data`, writable by
its non-root Node user. The corresponding local Relayer and configured Worker
were stopped and local default anchoring configuration removed, so the server
is the sole active signer. Do not start another Relayer with the same wallet
without coordinating nonce management and state. Keep the dedicated wallet
funded with Sepolia test ETH only.

## Backups and recovery

Run a PostgreSQL custom-format dump to a protected file. On Linux, from the
project directory (not through a PowerShell text pipeline):

```sh
umask 077
mkdir -p work/backups
backup="work/backups/rescueproof-$(date -u +%Y%m%dT%H%M%SZ).dump"
sudo docker compose exec -T postgres pg_dump -U rescueproof -d rescueproof -Fc > "$backup"
sudo docker compose exec -T postgres pg_restore --list < "$backup" > /dev/null
chmod 600 "$backup"
```

Archive-index validation checks readability, not a complete restore rehearsal.
For a complete isolated rehearsal, run `sudo sh scripts/verify-backup.sh path/to/backup.dump`.
It uses a temporary, network-isolated PostgreSQL 18 container and never attaches
production data volumes. Daily systemd backups are installed for 03:00 Asia/Shanghai,
with up to ten minutes of randomized delay; see [Operations](operations.md).
Transfer backups to protected storage outside the server. Back up `.env`, the
Relayer state and certificate storage separately with appropriate access control.
Restore with `pg_restore --no-owner --no-acl --exit-on-error` into a **fresh**
database, then verify counts and application behavior before switching traffic.

## Public validation on 2026-10-21

Public HTTPS tests exercised separate donor, recipient and administrator sessions,
Secure/HttpOnly cookies, concurrent claims, role boundaries, pickup-code rotation,
handover, receipt, cancellation, dispute resolution, idempotency and logout.
The three new batches produced 14 event proofs in four real Sepolia transactions.
An independent RPC (`https://sepolia.drpc.org`) verified successful receipts,
contract address, block number, `RootAnchored` logs and stored roots. All 14 Merkle
paths passed; all 14 modified payloads failed verification.

An initial public query timed out; the same saved batches were subsequently
queried and independently verified successfully, without recreating the workflow.
The resulting report contains 65 passed checkpoints, including repeated logins
used for recovery, and retains the transient timeout. This validates this test
run, not a guarantee of uninterrupted availability or a full security audit.

API, Worker and Relayer were then restarted. Readiness, static assets, SPA routes,
HTTP-to-HTTPS redirect, login and the same 14 confirmed proofs remained available.
The database contained 21 batches, 58 proofs, nine confirmed anchor batches and
no unprocessed outbox jobs. A protected 52KB database archive was created and its
archive index checked. The in-app browser reported `ERR_CONNECTION_CLOSED`, so
browser visual validation was not completed; strict certificate validation,
HTTP/1.1, HTTP/2 and the public API checks succeeded from the operator machine.

Database acceptance was then repeated: PostgreSQL 18.6 has migration 1 applied,
`fsync` and `synchronous_commit` enabled, and the named `rescueproof_postgres_data`
volume mounted at `/var/lib/postgresql`. A new automated backup was restored in
full into an isolated temporary database: all 16 public tables were present,
21 batches and 58 proofs survived, nine anchors remained confirmed, and no
unvalidated constraints were found. The SHA-256-matching backup was also copied
to protected operator storage outside the server. PostgreSQL itself was restarted;
counts, public readiness, login and the same 14 confirmed proofs still passed.
The daily timer is enabled and active. Automatic off-server replication and
retention cleanup remain separate operational work.
