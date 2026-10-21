#!/bin/sh
set -eu
# Run from repository root. Avoid piping PostgreSQL custom format through PowerShell text redirection.
umask 077
backup_dir="${BACKUP_DIR:-work/backups}"
mkdir -p "$backup_dir"
partial=$(mktemp "$backup_dir/.rescueproof-$(date -u +%Y%m%dT%H%M%SZ)-XXXXXX.dump.partial")
trap 'rm -f "$partial"' EXIT
trap 'exit 1' HUP INT TERM
backup="${partial%.partial}"
backup="${backup_dir}/$(basename "$backup" | cut -c 2-)"
docker compose exec -T postgres sh -c 'exec pg_dump -U "$POSTGRES_USER" -d "$POSTGRES_DB" -Fc' < /dev/null > "$partial"
docker compose exec -T postgres pg_restore --list < "$partial" > /dev/null
mv "$partial" "$backup"
sha256sum "$backup" > "$backup.sha256"
printf 'Backup written, archive index checked, SHA-256 recorded: %s\n' "$backup"
