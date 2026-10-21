# Operations

## Environment

`PUBLIC_ORIGIN` must be one HTTP(S) origin with no path. Production uses HTTPS and `COOKIE_SECURE=true`. Compose exposes only the Caddy web ports; its PostgreSQL and Redis network is marked internal. Do not publish database or Redis ports.

The API applies SQL migrations on startup. The Worker uses PostgreSQL `SKIP LOCKED` and an advisory lock, so a second Worker process does not duplicate a Merkle batch. Restart policy is `unless-stopped`; inspect dead outbox jobs through the admin view and structured logs.

## Health and logs

`GET /health/live` checks process liveness. `GET /health/ready` checks PostgreSQL and Redis. Useful commands:

```powershell
docker compose ps
docker compose logs --since 15m api worker web
docker compose exec postgres pg_isready -U rescueproof
docker compose exec redis redis-cli ping
```

## Backups

Run `sh scripts/backup.sh` from a trusted host with Docker Compose and a POSIX shell. The script creates a PostgreSQL custom-format archive under ignored `work/backups/`, checks that `pg_restore` can read its archive index, and keeps it local. Copy backups to a separate protected storage location and rehearse restores into a fresh database. Keep encryption keys and `.env` out of the repository and backups that are accessible to untrusted users.

## First operator

Register the operator account through the UI, then from the server run `docker compose exec api rescueproof promote-admin operator@example.org`. Confirm the organisation matches a real representative through a separate process before approving it. The command only promotes an existing active account and reveals no credentials.

## Current prototype boundaries

Email verification, password reset and external email delivery are not implemented. Notifications appear in the in-app dashboard; a newly published batch is discoverable on the public list but automatic nearby-recipient targeting needs organisation service-area data. Chain anchors remain pending until a compatible relayer and RPC are configured. No live blockchain write is performed by default.
