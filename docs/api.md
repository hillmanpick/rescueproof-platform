# API

All application routes use `/api/v1`, JSON request/response bodies, and same-origin session cookies. Cookie-authenticated `POST` requests must include `X-Requested-With: RescueProof`, the configured `Origin`, and an `Idempotency-Key` for state-changing operations. Error shape:

```json
{"error":{"code":"BATCH_UNAVAILABLE","message":"This batch is no longer available"}}
```

## Routes

| Method | Path | Access | Purpose |
| --- | --- | --- | --- |
| POST | `/auth/register` | Public, rate limited | Create user and pending organisation |
| POST | `/auth/login` | Public, rate limited | Create revocable HttpOnly session |
| POST | `/auth/logout` | Session | Revoke session |
| GET | `/auth/me` | Session | Current user and organisations |
| GET | `/batches` | Public | List open batches; public fields only |
| GET | `/batches/{id}` | Public fields / private after membership | Read a batch |
| POST | `/batches` | Approved donor | Publish a batch; idempotent |
| POST | `/batches/{id}/cancel` | Owning donor | Cancel an open batch |
| POST | `/batches/{id}/claims` | Approved recipient | Atomically claim an open batch |
| POST | `/claims/{id}/pickup-token` | Claiming recipient | Generate a 30-minute one-time handover token |
| POST | `/claims/{id}/handover` | Owning donor | Validate token and confirm donor handover |
| POST | `/claims/{id}/receipt` | Claiming recipient | Confirm receipt and complete claim |
| POST | `/claims/{id}/cancel` | Claiming recipient | Cancel before handover |
| POST | `/claims/{id}/disputes` | Either party | Open a dispute |
| GET | `/dashboard` | Session | Member batches and notifications |
| GET | `/impact` | Public | Aggregate, participant-reported metrics |
| GET | `/batches/{id}/events` | Batch party | Sanitized audit events |
| GET | `/batches/{id}/proof` | Batch party | Merkle proofs and chain status |
| GET | `/admin` | Admin | Organisations, disputes and dead tasks |
| POST | `/admin/organizations/{id}/review` | Admin | Approve or suspend an organisation |
| POST | `/admin/disputes/{id}/resolve` | Admin | Resolve without fabricating handover |

All timestamps are ISO 8601 UTC. Batch category is `bakery`, `produce`, `packaged` or `prepared`; quantity unit is `portions`, `items` or `boxes`.

## Request examples

After registration, the session cookie is returned as `rp_session`. A frontend request also sends `X-Requested-With: RescueProof` and a fresh UUID as `Idempotency-Key`:

```json
{
  "organization_id":"00000000-0000-4000-8000-000000000001",
  "title":"Surplus bakery box",
  "category":"bakery",
  "quantity":12,
  "unit":"portions",
  "weight_kg":3.5,
  "area":"Pilot district",
  "pickup_address":"Full address (only party members can read)",
  "contact":"Coordinator contact (only party members can read)",
  "storage_notes":"Keep sealed and cool",
  "allergens":"Contains wheat and milk",
  "pickup_start":"2026-10-22T09:00:00Z",
  "pickup_end":"2026-10-22T11:00:00Z",
  "expires_at":"2026-10-22T13:00:00Z",
  "safety_acknowledged":true
}
```

Claim with `{"organization_id":"<approved-recipient-org>"}`. The response returns a claim ID. The recipient then requests a pickup token and shows its QR code to the donor. The donor submits the scanned token to `/handover`; the recipient confirms the package and optionally records actual weight at `/receipt`.

See `backend/src/lib.rs` for the canonical route table and `backend/migrations/0001_initial.sql` for the schema.
