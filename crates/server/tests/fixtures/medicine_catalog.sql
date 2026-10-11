-- Stand-in for meditrino_new's medicine_catalog and medicine_catalog_changes
-- (migrations 20261003, 20261005, 20261008). Only the columns the pharmacy
-- server reads are reproduced, with the same types and constraints.
CREATE TABLE medicine_catalog (
    id UUID PRIMARY KEY,
    brand_name TEXT NOT NULL CHECK (length(btrim(brand_name)) > 0),
    generic_name TEXT,
    strength TEXT,
    dosage_form TEXT,
    manufacturer TEXT,
    package_description TEXT,
    reference_price_paisa BIGINT CHECK (reference_price_paisa >= 0),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE medicine_catalog_changes (
    revision BIGSERIAL PRIMARY KEY,
    medicine_id UUID NOT NULL REFERENCES medicine_catalog(id)
);
