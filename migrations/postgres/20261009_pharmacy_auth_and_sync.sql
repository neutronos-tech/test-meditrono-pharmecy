BEGIN;

CREATE TABLE pharmacies (
    id UUID PRIMARY KEY,
    username TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX pharmacies_username_lower_unique
ON pharmacies (lower(username));

CREATE TABLE pharmacy_sessions (
    id UUID PRIMARY KEY,
    pharmacy_id UUID NOT NULL REFERENCES pharmacies(id) ON DELETE CASCADE,
    token_hash BYTEA NOT NULL UNIQUE,
    csrf_hash BYTEA,
    kind TEXT NOT NULL CHECK(kind IN ('browser', 'desktop')),
    created_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX pharmacy_sessions_pharmacy_idx ON pharmacy_sessions(pharmacy_id);

CREATE TABLE pharmacy_suppliers (
    id UUID PRIMARY KEY,
    pharmacy_id UUID NOT NULL REFERENCES pharmacies(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    UNIQUE(pharmacy_id, name)
);

CREATE TABLE pharmacy_medicines (
    id UUID PRIMARY KEY,
    pharmacy_id UUID NOT NULL REFERENCES pharmacies(id) ON DELETE CASCADE,
    central_medicine_id UUID REFERENCES medicine_catalog(id),
    name TEXT NOT NULL,
    generic_name TEXT,
    strength TEXT,
    dosage_form TEXT,
    manufacturer TEXT,
    package_description TEXT,
    reference_price NUMERIC,
    selling_price NUMERIC,
    origin TEXT NOT NULL DEFAULT 'local',
    active BOOLEAN NOT NULL DEFAULT TRUE
);
CREATE INDEX pharmacy_medicines_owner_idx ON pharmacy_medicines(pharmacy_id);

CREATE TABLE pharmacy_operations (
    id UUID PRIMARY KEY,
    pharmacy_id UUID NOT NULL REFERENCES pharmacies(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    actor_id UUID NOT NULL,
    actor_name TEXT NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    reversal_of UUID REFERENCES pharmacy_operations(id),
    UNIQUE(pharmacy_id, id)
);
CREATE UNIQUE INDEX pharmacy_operations_one_reversal
ON pharmacy_operations(pharmacy_id, reversal_of)
WHERE reversal_of IS NOT NULL;

CREATE TABLE pharmacy_purchases (
    id UUID PRIMARY KEY,
    pharmacy_id UUID NOT NULL REFERENCES pharmacies(id) ON DELETE CASCADE,
    operation_id UUID NOT NULL UNIQUE REFERENCES pharmacy_operations(id),
    supplier_id UUID NOT NULL REFERENCES pharmacy_suppliers(id),
    invoice_number TEXT NOT NULL,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    UNIQUE(pharmacy_id, supplier_id, invoice_number)
);
CREATE TABLE pharmacy_purchase_items (
    id UUID PRIMARY KEY,
    pharmacy_id UUID NOT NULL REFERENCES pharmacies(id) ON DELETE CASCADE,
    purchase_id UUID NOT NULL REFERENCES pharmacy_purchases(id),
    medicine_id UUID NOT NULL REFERENCES pharmacy_medicines(id),
    quantity INTEGER NOT NULL CHECK(quantity > 0),
    unit_cost NUMERIC NOT NULL,
    received_quantity INTEGER NOT NULL DEFAULT 0 CHECK(received_quantity >= 0 AND received_quantity <= quantity)
);

CREATE TABLE pharmacy_inventory_batches (
    id UUID PRIMARY KEY,
    pharmacy_id UUID NOT NULL REFERENCES pharmacies(id) ON DELETE CASCADE,
    medicine_id UUID NOT NULL REFERENCES pharmacy_medicines(id),
    batch_number TEXT NOT NULL,
    expiry_date DATE NOT NULL,
    quantity_on_hand BIGINT NOT NULL DEFAULT 0 CHECK(quantity_on_hand >= 0),
    UNIQUE(pharmacy_id, medicine_id, batch_number)
);
CREATE TABLE pharmacy_stock_inputs (
    id UUID PRIMARY KEY,
    pharmacy_id UUID NOT NULL REFERENCES pharmacies(id) ON DELETE CASCADE,
    operation_id UUID NOT NULL REFERENCES pharmacy_operations(id),
    purchase_item_id UUID NOT NULL REFERENCES pharmacy_purchase_items(id),
    batch_id UUID NOT NULL REFERENCES pharmacy_inventory_batches(id),
    quantity INTEGER NOT NULL CHECK(quantity > 0),
    created_at TIMESTAMPTZ NOT NULL
);
CREATE TABLE pharmacy_inventory_movements (
    id UUID PRIMARY KEY,
    pharmacy_id UUID NOT NULL REFERENCES pharmacies(id) ON DELETE CASCADE,
    operation_id UUID NOT NULL REFERENCES pharmacy_operations(id),
    batch_id UUID NOT NULL REFERENCES pharmacy_inventory_batches(id),
    quantity_delta BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE pharmacy_sales (
    id UUID PRIMARY KEY,
    pharmacy_id UUID NOT NULL REFERENCES pharmacies(id) ON DELETE CASCADE,
    operation_id UUID NOT NULL UNIQUE REFERENCES pharmacy_operations(id),
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);
CREATE TABLE pharmacy_sale_items (
    id UUID PRIMARY KEY,
    pharmacy_id UUID NOT NULL REFERENCES pharmacies(id) ON DELETE CASCADE,
    sale_id UUID NOT NULL REFERENCES pharmacy_sales(id),
    medicine_id UUID NOT NULL REFERENCES pharmacy_medicines(id),
    quantity INTEGER NOT NULL CHECK(quantity > 0),
    unit_price NUMERIC NOT NULL
);
CREATE TABLE pharmacy_sale_item_batches (
    pharmacy_id UUID NOT NULL REFERENCES pharmacies(id) ON DELETE CASCADE,
    sale_item_id UUID NOT NULL REFERENCES pharmacy_sale_items(id),
    batch_id UUID NOT NULL REFERENCES pharmacy_inventory_batches(id),
    quantity INTEGER NOT NULL CHECK(quantity > 0),
    PRIMARY KEY(sale_item_id, batch_id)
);

CREATE TABLE pharmacy_processed_changes (
    pharmacy_id UUID NOT NULL REFERENCES pharmacies(id) ON DELETE CASCADE,
    change_id UUID NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY(pharmacy_id, change_id)
);

CREATE TABLE pharmacy_changes (
    revision BIGSERIAL PRIMARY KEY,
    pharmacy_id UUID NOT NULL REFERENCES pharmacies(id) ON DELETE CASCADE,
    entity_type TEXT NOT NULL,
    entity_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX pharmacy_changes_lookup_idx ON pharmacy_changes(pharmacy_id, revision);

COMMIT;
