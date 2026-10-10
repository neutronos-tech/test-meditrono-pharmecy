CREATE TABLE users (id TEXT PRIMARY KEY, username TEXT NOT NULL UNIQUE, password_hash TEXT NOT NULL, disabled INTEGER NOT NULL DEFAULT 0);
CREATE TABLE roles (id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE);
CREATE TABLE role_permissions (role_id TEXT NOT NULL REFERENCES roles(id), permission TEXT NOT NULL, PRIMARY KEY(role_id, permission));
CREATE TABLE user_roles (user_id TEXT NOT NULL REFERENCES users(id), role_id TEXT NOT NULL REFERENCES roles(id), PRIMARY KEY(user_id, role_id));
CREATE TABLE suppliers (id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE);
CREATE TABLE medicines (
 id TEXT PRIMARY KEY, name TEXT NOT NULL, generic_name TEXT, strength TEXT, dosage_form TEXT,
 manufacturer TEXT, package_description TEXT, reference_price TEXT, selling_price TEXT,
 origin TEXT, source TEXT, source_record_id TEXT, sync_revision INTEGER NOT NULL DEFAULT 0,
 active INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE operations (
 id TEXT PRIMARY KEY, kind TEXT NOT NULL CHECK(kind IN ('purchase','stock_receipt','stock_receipt_reversal','sale','sale_reversal')),
 actor TEXT NOT NULL, occurred_at TEXT NOT NULL, reversal_of TEXT REFERENCES operations(id)
);
CREATE UNIQUE INDEX operations_one_reversal ON operations(reversal_of) WHERE reversal_of IS NOT NULL;
CREATE TABLE purchases (
 id TEXT PRIMARY KEY, operation_id TEXT NOT NULL UNIQUE REFERENCES operations(id),
 supplier_id TEXT NOT NULL REFERENCES suppliers(id), invoice_number TEXT NOT NULL,
 created_by TEXT NOT NULL, created_at TEXT NOT NULL,
 UNIQUE(supplier_id, invoice_number)
);
CREATE TABLE purchase_items (
 id TEXT PRIMARY KEY, purchase_id TEXT NOT NULL REFERENCES purchases(id), medicine_id TEXT NOT NULL REFERENCES medicines(id),
 quantity INTEGER NOT NULL CHECK(quantity > 0), unit_cost TEXT NOT NULL,
 received_quantity INTEGER NOT NULL DEFAULT 0 CHECK(received_quantity >= 0 AND received_quantity <= quantity)
);
CREATE TABLE inventory_batches (
 id TEXT PRIMARY KEY, medicine_id TEXT NOT NULL REFERENCES medicines(id), batch_number TEXT NOT NULL,
 expiry_date TEXT NOT NULL, quantity_on_hand INTEGER NOT NULL DEFAULT 0 CHECK(quantity_on_hand >= 0),
 UNIQUE(medicine_id, batch_number)
);
CREATE TABLE stock_inputs (
 id TEXT PRIMARY KEY, operation_id TEXT NOT NULL REFERENCES operations(id),
 purchase_item_id TEXT NOT NULL REFERENCES purchase_items(id), batch_id TEXT NOT NULL REFERENCES inventory_batches(id),
 quantity INTEGER NOT NULL CHECK(quantity > 0), created_at TEXT NOT NULL
);
CREATE TABLE inventory_movements (
 id TEXT PRIMARY KEY, operation_id TEXT NOT NULL REFERENCES operations(id),
 batch_id TEXT NOT NULL REFERENCES inventory_batches(id), quantity_delta INTEGER NOT NULL, created_at TEXT NOT NULL
);
CREATE TABLE sales (
 id TEXT PRIMARY KEY, operation_id TEXT NOT NULL UNIQUE REFERENCES operations(id),
 created_by TEXT NOT NULL, created_at TEXT NOT NULL
);
CREATE TABLE sale_items (
 id TEXT PRIMARY KEY, sale_id TEXT NOT NULL REFERENCES sales(id), medicine_id TEXT NOT NULL REFERENCES medicines(id),
 quantity INTEGER NOT NULL CHECK(quantity > 0), unit_price TEXT NOT NULL
);
CREATE TABLE sale_item_batches (
 sale_item_id TEXT NOT NULL REFERENCES sale_items(id), batch_id TEXT NOT NULL REFERENCES inventory_batches(id),
 quantity INTEGER NOT NULL CHECK(quantity > 0), PRIMARY KEY(sale_item_id, batch_id)
);
CREATE TABLE sync_outbox (
 seq INTEGER PRIMARY KEY AUTOINCREMENT, change_id TEXT NOT NULL UNIQUE,
 entity_type TEXT NOT NULL, entity_id TEXT NOT NULL,
 operation TEXT NOT NULL CHECK(operation IN ('upsert', 'delete')), created_at TEXT NOT NULL
);
CREATE TABLE sync_state (
 singleton INTEGER PRIMARY KEY CHECK(singleton = 1), last_server_revision INTEGER NOT NULL DEFAULT 0
);
INSERT INTO sync_state(singleton, last_server_revision) VALUES(1, 0);
