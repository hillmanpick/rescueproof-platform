# Security model

- PostgreSQL is the sole authority for membership, claim ownership and workflow state. Row locks, partial unique indexes and transactions stop duplicate claims even if Redis is unavailable.
- Passwords use Argon2id. Session cookies are opaque 256-bit random values; only SHA-256 token digests are stored. Sessions can be revoked server-side.
- Organization state starts pending. A local operator explicitly promotes the first admin, who reviews each organization before it can publish or claim.
- Private pickup addresses and contacts are omitted from list/search results and only returned to the donor or active claim recipient.
- One-time pickup tokens expire within 30 minutes, rotate on regeneration, are stored as SHA-256 digests, and are cleared on handover, cancellation or expiry.
- State-changing requests require the app-specific request header, reject cross-site fetch metadata, check `Origin` when present and use SameSite cookies. The API does not enable permissive CORS.
- Rate limits use atomic Redis Lua increments with expiry and a configurable namespace. In Compose, Caddy overwrites the forwarded client IP and is the only public route to the API. `TRUST_PROXY=true` is safe only behind this trusted proxy; keep it false for direct local use.
- Idempotency records commit in the same transaction as the action, are bound to a request fingerprint, and expire after seven days.
- Outbox retries and chain anchoring never hold database transactions open across external HTTP calls. Chain confirmation is recorded only after transaction receipt, target contract and stored root are checked against the configured RPC and chain ID.
- Logs avoid request bodies, authentication headers, passwords, tokens and database URLs. Caddy and containers rotate logs.
- No food participant identity, address, contact, or detailed food description is written to chain.

Before production, use HTTPS, unique secrets, a firewall, external off-server backups, tested restore, dependency updates, contract review and an operator identity-verification procedure. The prototype has not had an independent penetration test or smart-contract audit.
