-- Devices, editable plans and an audit log for per-device licensing.
-- Server-only tables, so they use integer ids (AGENTS.md).
BEGIN;

-- The owner edits prices here (admin pricing page later); nothing is hard-coded.
CREATE TABLE pharmacy_plans (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    monthly_price_paisa BIGINT NOT NULL CHECK(monthly_price_paisa >= 0),
    yearly_price_paisa BIGINT NOT NULL CHECK(yearly_price_paisa >= 0),
    trial_days INTEGER NOT NULL CHECK(trial_days >= 0),
    ai_scan_cap INTEGER NOT NULL CHECK(ai_scan_cap >= 0),
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX pharmacy_plans_one_default ON pharmacy_plans(is_default) WHERE is_default;
INSERT INTO pharmacy_plans(code, name, monthly_price_paisa, yearly_price_paisa, trial_days, ai_scan_cap, is_default)
VALUES ('standard', 'Standard', 0, 0, 30, 50, TRUE);

-- One row per licensed computer. paid_until is extended by payments or by
-- manual activation; a new device starts with the plan's trial.
CREATE TABLE pharmacy_devices (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    pharmacy_id UUID NOT NULL REFERENCES pharmacies(id) ON DELETE CASCADE,
    fingerprint_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    plan_id BIGINT NOT NULL REFERENCES pharmacy_plans(id),
    paid_until TIMESTAMPTZ NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(pharmacy_id, fingerprint_hash)
);

CREATE TABLE audit_log (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    pharmacy_id UUID REFERENCES pharmacies(id) ON DELETE CASCADE,
    actor TEXT NOT NULL,
    action TEXT NOT NULL,
    detail JSONB NOT NULL DEFAULT '{}',
    source TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX audit_log_pharmacy_idx ON audit_log(pharmacy_id, id);

COMMIT;
