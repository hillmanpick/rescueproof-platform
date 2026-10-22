#!/bin/sh
set -eu
# Run from the project root as root; never print certificate private keys.
umask 077
mkdir -p work/postgres-tls
staging=$(mktemp -d work/postgres-tls/.incoming-XXXXXX)
cleanup() {
    rm -f "$staging/server.crt" "$staging/server.key"
    rmdir "$staging"
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM
docker compose exec -T cert-manager sh -ec 'cat "/data/caddy/certificates/acme-v02.api.letsencrypt.org-directory/$SITE_ADDRESS/$SITE_ADDRESS.crt"' < /dev/null > "$staging/server.crt"
docker compose exec -T cert-manager sh -ec 'cat "/data/caddy/certificates/acme-v02.api.letsencrypt.org-directory/$SITE_ADDRESS/$SITE_ADDRESS.key"' < /dev/null > "$staging/server.key"
openssl x509 -in "$staging/server.crt" -noout -checkend 3600 > /dev/null
openssl pkey -in "$staging/server.key" -check -noout > /dev/null 2>&1
cert_pub=$(openssl x509 -in "$staging/server.crt" -pubkey -noout | openssl pkey -pubin -outform DER | sha256sum)
key_pub=$(openssl pkey -in "$staging/server.key" -pubout -outform DER | sha256sum)
test "$cert_pub" = "$key_pub"
# Nginx holds certificates in memory; a graceful reload picks up renewed files.
docker compose exec -T web nginx -t < /dev/null
docker compose exec -T web nginx -s reload < /dev/null
if cmp -s "$staging/server.crt" work/postgres-tls/server.crt && cmp -s "$staging/server.key" work/postgres-tls/server.key; then
    printf '%s\n' 'PostgreSQL TLS certificate is current.'
    exit 0
fi
# postgres:18-alpine uses UID/GID 70. Bind the directory so replacements are visible.
chown 70:70 "$staging/server.crt" "$staging/server.key"
chmod 600 "$staging/server.crt" "$staging/server.key"
mv "$staging/server.key" work/postgres-tls/server.key
mv "$staging/server.crt" work/postgres-tls/server.crt
chown 70:70 work/postgres-tls
chmod 700 work/postgres-tls
if [ "${POSTGRES_TLS_BOOTSTRAP:-false}" != true ]; then
    docker compose exec -T postgres sh -ec 'exec psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 -c "SELECT pg_reload_conf();"' < /dev/null
fi
printf '%s\n' 'PostgreSQL TLS certificate synchronized.'
