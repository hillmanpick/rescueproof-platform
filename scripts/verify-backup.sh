#!/bin/sh
set -eu
# Rehearse a restore without attaching production networks, volumes or workers.
dump=${1:?Usage: sh scripts/verify-backup.sh path/to/backup.dump}
test -r "$dump"
container_id=""
cleanup() {
    if [ -n "$container_id" ]; then
        docker rm -f "$container_id" > /dev/null
    fi
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM
container_id=$(docker run --rm -d --pull never --network none --memory 256m --cpus 0.25 \
    --tmpfs /var/lib/postgresql:rw,size=256m \
    -e POSTGRES_HOST_AUTH_METHOD=trust -e POSTGRES_DB=rescueproof_restore postgres:18-alpine)
attempt=0
until docker exec "$container_id" pg_isready -U postgres -d rescueproof_restore > /dev/null 2>&1; do
    attempt=$((attempt + 1))
    if [ "$attempt" -ge 30 ]; then
        printf '%s\n' 'Restore database did not become ready' >&2
        exit 1
    fi
    sleep 1
done
docker exec -i "$container_id" pg_restore -U postgres -d rescueproof_restore \
    --no-owner --no-acl --exit-on-error < "$dump"
docker exec "$container_id" psql -U postgres -d rescueproof_restore -v ON_ERROR_STOP=1 -c \
    "SELECT version,success FROM _sqlx_migrations ORDER BY version;
     SELECT count(*) AS batches FROM food_batches;
     SELECT count(*) AS proofs FROM anchor_leaves;
     SELECT status,count(*) FROM anchor_batches GROUP BY status;
     SELECT count(*) AS unvalidated_constraints FROM pg_constraint WHERE NOT convalidated;"
# Print only table names and counts. Never dump account, session or private payload rows.
docker exec -i "$container_id" psql -U postgres -d rescueproof_restore -v ON_ERROR_STOP=1 <<'SQL'
SELECT format('SELECT %L AS table_name, count(*) AS rows FROM %I.%I;', tablename, schemaname, tablename)
FROM pg_tables WHERE schemaname='public' ORDER BY tablename
\gexec
SQL
printf '%s\n' 'Isolated restore completed; temporary container and in-memory data will be removed.'
