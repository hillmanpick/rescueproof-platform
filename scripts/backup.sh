#!/bin/sh
set -eu
# Run from repository root. Avoid piping PostgreSQL custom format through PowerShell text redirection.
mkdir -p work/backups
backup="work/backups/rescueproof-$(date -u +%Y%m%dT%H%M%SZ).dump"
docker compose exec -T postgres sh -c 'pg_dump -U "$POSTGRES_USER" -d "$POSTGRES_DB" -Fc' > "$backup"
docker compose exec -T postgres pg_restore --list < "$backup" > /dev/null
printf 'Backup written and archive index checked: %s\n' "$backup"
