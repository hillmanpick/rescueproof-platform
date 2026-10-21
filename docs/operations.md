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

Run `sh scripts/backup.sh` from a trusted host with Docker Compose and a POSIX shell. The script uses private file permissions, writes a temporary PostgreSQL custom-format archive under ignored `work/backups/`, validates its archive index, then publishes the complete file and a SHA-256 checksum. `BACKUP_DIR` can select another protected backup directory. Incomplete dumps are removed on failure. These archives contain account/session data and must stay protected.

Run `sh scripts/verify-backup.sh path/to/backup.dump` to perform a complete restore in an isolated temporary PostgreSQL 18 container. The official image must already exist locally. The test container has no network or production volumes; its database lives in a size-limited temporary filesystem. The script checks restore errors and prints migration status, core data counts, constraint status and all public-table counts; its exit trap removes the test container and data. Compare the results with the expected backup snapshot. This test uses a 256MB memory ceiling and 0.25 CPU.

The `/home/rescueproof` deployment includes `deploy/rescueproof-backup.service` and `.timer`: daily at 19:00 UTC (03:00 Asia/Shanghai) with up to ten minutes of randomized delay. `Persistent=true` catches a missed run after startup. Install the units under `/etc/systemd/system`, reload systemd, enable the timer and perform an immediate service run. Inspect with `systemctl list-timers rescueproof-backup.timer` and `journalctl -u rescueproof-backup.service`. The unit assumes the root project path above and an installed Docker Compose plugin; adjust its working directory for another installation.

Copy backups to separate protected storage. On 2026-10-21 a complete restore succeeded for all 16 public tables, including 21 batches and 58 event proofs, and a checksum-matching copy was saved to protected operator storage outside the server. Ongoing automatic transfer off the server and backup-retention policy are not configured. Keep encryption keys and `.env` out of the repository and backups accessible to untrusted users.

## First operator

Register the operator account through the UI, then from the server run `docker compose exec api rescueproof promote-admin operator@example.org`. Confirm the organisation matches a real representative through a separate process before approving it. The command only promotes an existing active account and reveals no credentials.

## Current prototype boundaries

Email verification, password reset and external email delivery are not implemented. Notifications appear in the in-app dashboard; a newly published batch is discoverable on the public list but automatic nearby-recipient targeting needs organisation service-area data. Chain anchors remain pending until a compatible relayer and RPC are configured. No live blockchain write is performed by default.
