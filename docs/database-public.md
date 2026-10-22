# PostgreSQL public client access

The authorized public endpoint is `124.156.199.59:5678`, mapped to container port
5432. Database and username are both `rescueproof`; obtain the password from
`POSTGRES_PASSWORD` in the protected `/home/rescueproof/.env`. Never put it in
documentation or a command-line connection URL.

In Navicat or DBeaver, select PostgreSQL, enter the host and port above, then
enable SSL/TLS. A direct public connection does not require an SSH tunnel.
Prefer `verify-full` with a current trusted CA bundle. Some libpq installations
do not resolve `sslrootcert=system` to a usable CA store; supplying a current CA
bundle worked in the acceptance test. SSL mode `require` encrypts the connection
but does not provide the same server identity verification as `verify-full`.
Plaintext TCP connections are rejected by `deploy/pg_hba.public.conf`; encrypted
connections use SCRAM-SHA-256 password authentication. Local administrative
commands continue to use the container Unix socket.

The server `.env` selects all deployment overlays:

```dotenv
COMPOSE_FILE=compose.yaml:compose.ip.yaml:compose.database-public.yaml
POSTGRES_PUBLIC_PORT=5678
```

From `/home/rescueproof`, use `sudo docker compose up -d --no-build` for updates.
If passing `-f` explicitly, include **all three** files or the public database
configuration will be omitted. Only PostgreSQL needs recreation to apply the
public overlay. Its existing named data volume is retained. PostgreSQL joins
both the data and edge networks because Docker does not publish a service on
an exclusively internal network.

Run `sudo env POSTGRES_TLS_BOOTSTRAP=true sh scripts/sync-postgres-tls.sh` before
first enabling this overlay. The script copies the internal certificate manager's IP certificate
and key to protected `work/postgres-tls`, checks expiry and matching public keys,
then gives the files to UID/GID 70 used by `postgres:18-alpine`. Only PostgreSQL
receives this directory; unrelated services are unchanged. The
`rescueproof-postgres-tls.timer` runs certificate synchronization every 15 minutes;
changed files trigger a PostgreSQL configuration reload. It also validates and
gracefully reloads Nginx to load renewed certificates. Keep public port 80 open;
Nginx forwards HTTP-01 challenges to the internal certificate manager. If changing the PostgreSQL base image,
verify its user ID before reusing this synchronization script.

The original internal-only topology remains available by omitting the public
overlay deliberately. Direct public access was requested for desktop database
clients; SSH tunneling offers a smaller public surface but adds a tunnel step.
The chosen public configuration requires TLS and password authentication rather
than allowing plaintext database traffic.

On 2026-10-22 the operator machine connected through public port 5678 with
`sslmode=verify-full` and a current CA bundle. PostgreSQL reported TLS 1.3 with
`TLS_AES_256_GCM_SHA384`; queries returned 21 food batches and 58 proofs. A second
connection with TLS disabled was rejected. Existing API/Worker connections were
also observed using TLS 1.3, and public website readiness returned HTTP 200.
