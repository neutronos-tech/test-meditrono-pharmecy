# Pharmacy server

`DATABASE_URL` must point to the shared PostgreSQL database that already
contains MEDITRINO's `medicine_catalog`. Apply both forward migrations from the
central MEDITRINO project before starting the server:

```sh
psql "$DATABASE_URL" -v ON_ERROR_STOP=1 \
  -f ../meditrino_new/database/migrations/20261008_add_medicine_catalog_changes.sql
psql "$DATABASE_URL" -v ON_ERROR_STOP=1 \
  -f ../meditrino_new/database/migrations/20261009_pharmacy_auth_and_sync.sql
```

Create the first account through the controlled bootstrap command. There is no
public registration route:

```sh
DATABASE_URL='postgresql://...' \
PHARMACY_BOOTSTRAP_USERNAME='pharmacy-name' \
PHARMACY_BOOTSTRAP_PASSWORD='choose-a-secret' \
cargo run -p server -- create-pharmacy
```

Then start the server from the workspace root:

```sh
ADDR=127.0.0.1:3001 cargo run -p server
```

`ADDR` defaults to `127.0.0.1:3001`. Set `PHARMACY_SECURE_COOKIES=false` for
local HTTP browser testing. The desktop client defaults to
`https://pharmacy.neutorium.com`; set `PHARMACY_API_BASE_URL` to override it for
development. Protected API routes use the bearer token returned by login.

Future central medicine edits must update `medicine_catalog` and insert a
`medicine_catalog_changes` row in one PostgreSQL transaction. Future Pharmacy
data mutations must write their `pharmacy_changes` marker in the same
transaction.
