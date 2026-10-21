CREATE TABLE users (
    id uuid PRIMARY KEY,
    email text NOT NULL UNIQUE CHECK (email = lower(email)),
    name text NOT NULL,
    password_hash text NOT NULL,
    is_admin boolean NOT NULL DEFAULT false,
    disabled boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE organizations (
    id uuid PRIMARY KEY,
    name text NOT NULL,
    kind text NOT NULL CHECK (kind IN ('donor','recipient')),
    status text NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','approved','suspended')),
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE organization_reviews (
    id uuid PRIMARY KEY,
    organization_id uuid NOT NULL REFERENCES organizations(id),
    reviewer_id uuid NOT NULL REFERENCES users(id),
    decision text NOT NULL CHECK(decision IN ('approved','suspended')),
    note text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE organization_members (
    user_id uuid NOT NULL REFERENCES users(id),
    organization_id uuid NOT NULL REFERENCES organizations(id),
    role text NOT NULL CHECK (role IN ('owner','member')),
    PRIMARY KEY (user_id,organization_id)
);
CREATE TABLE sessions (
    token_hash text PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX sessions_user ON sessions(user_id);
CREATE TABLE food_batches (
    id uuid PRIMARY KEY,
    donor_id uuid NOT NULL REFERENCES organizations(id),
    title text NOT NULL,
    category text NOT NULL CHECK (category IN ('bakery','produce','packaged','prepared')),
    quantity integer NOT NULL CHECK (quantity > 0 AND quantity <= 100000),
    unit text NOT NULL CHECK (unit IN ('portions','items','boxes')),
    weight_kg numeric(12,3) NOT NULL CHECK (weight_kg > 0 AND weight_kg <= 100000),
    area text NOT NULL,
    pickup_address text NOT NULL,
    contact text NOT NULL,
    storage_notes text NOT NULL,
    allergens text NOT NULL,
    pickup_start timestamptz NOT NULL,
    pickup_end timestamptz NOT NULL,
    expires_at timestamptz NOT NULL,
    status text NOT NULL DEFAULT 'open' CHECK (status IN ('open','claimed','handed_over','completed','cancelled','expired','disputed')),
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (pickup_start < pickup_end AND pickup_end <= expires_at)
);
CREATE INDEX food_batches_open ON food_batches(pickup_end,created_at DESC) WHERE status='open';
CREATE INDEX food_batches_donor ON food_batches(donor_id,created_at DESC);
CREATE TABLE claims (
    id uuid PRIMARY KEY,
    batch_id uuid NOT NULL REFERENCES food_batches(id),
    recipient_id uuid NOT NULL REFERENCES organizations(id),
    status text NOT NULL CHECK (status IN ('claimed','handed_over','completed','cancelled','expired','disputed')),
    pickup_token_hash text,
    token_expires_at timestamptz,
    actual_weight_kg numeric(12,3),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX one_active_claim ON claims(batch_id) WHERE status IN ('claimed','handed_over','completed','disputed');
CREATE INDEX claims_recipient ON claims(recipient_id,created_at DESC);
CREATE TABLE handover_events (
    id uuid PRIMARY KEY,
    batch_id uuid NOT NULL REFERENCES food_batches(id),
    claim_id uuid REFERENCES claims(id),
    actor_id uuid NOT NULL REFERENCES users(id),
    kind text NOT NULL,
    payload jsonb NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE disputes (
    id uuid PRIMARY KEY,
    batch_id uuid NOT NULL REFERENCES food_batches(id),
    claim_id uuid NOT NULL REFERENCES claims(id),
    opened_by uuid NOT NULL REFERENCES users(id),
    reason text NOT NULL,
    resolution text,
    resolved_by uuid REFERENCES users(id),
    resolved_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX one_open_dispute ON disputes(claim_id) WHERE resolved_at IS NULL;
CREATE TABLE idempotency_keys (
    user_id uuid NOT NULL REFERENCES users(id),
    key text NOT NULL,
    fingerprint text NOT NULL,
    response jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(user_id,key)
);
CREATE TABLE audit_events (
    id uuid PRIMARY KEY,
    actor_id uuid REFERENCES users(id),
    action text NOT NULL,
    entity_id uuid NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE outbox_events (
    id uuid PRIMARY KEY,
    kind text NOT NULL,
    payload jsonb NOT NULL,
    attempts integer NOT NULL DEFAULT 0,
    available_at timestamptz NOT NULL DEFAULT now(),
    processed_at timestamptz,
    dead_at timestamptz,
    last_error text,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX outbox_pending ON outbox_events(available_at) WHERE processed_at IS NULL AND dead_at IS NULL;
CREATE TABLE notifications (
    id uuid PRIMARY KEY,
    event_id uuid NOT NULL,
    organization_id uuid NOT NULL REFERENCES organizations(id),
    message text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE(event_id,organization_id)
);
CREATE TABLE anchor_batches (
    id uuid PRIMARY KEY,
    root text NOT NULL UNIQUE,
    status text NOT NULL DEFAULT 'pending' CHECK(status IN ('pending','submitted','confirmed')),
    transaction_hash text,
    chain_id bigint,
    contract_address text,
    block_number bigint,
    created_at timestamptz NOT NULL DEFAULT now(),
    confirmed_at timestamptz
);
CREATE TABLE anchor_leaves (
    event_id uuid PRIMARY KEY REFERENCES handover_events(id),
    anchor_id uuid NOT NULL REFERENCES anchor_batches(id),
    leaf_index integer NOT NULL,
    leaf_hash text NOT NULL,
    canonical_payload text NOT NULL,
    proof jsonb NOT NULL,
    UNIQUE(anchor_id,leaf_index)
);
