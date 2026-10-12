#!/usr/bin/env bash
# Check out the Meditrino Pharmacy branch and run the server on this machine.
#
# Development only: it loads a stand-in for MEDITRINO's medicine_catalog when that
# table is missing, and creates a demo account when no pharmacy exists.
#
# Requirements: git (with GitHub access to neutronos-tech/meditrino_pharmacy),
# a Rust toolchain (cargo), and PostgreSQL with psql.
#
# Usage:
#   ./pharmacy-local.sh                       # checks out to ~/pharmacy-local/meditrino_pharmacy
#   PORT=8080 ./pharmacy-local.sh             # different port
#   PG_URL=postgresql://user:pass@host:5432 ./pharmacy-local.sh
#
# Then open http://localhost:3001 and sign in as demo-pharmacy / demo-pass-123.
set -euo pipefail

REPO_URL="${REPO_URL:-https://github.com/neutronos-tech/meditrino_pharmacy.git}"
BRANCH="${BRANCH:-claude/adoring-brown-k7aa6w}"
CHECKOUT_DIR="${CHECKOUT_DIR:-$HOME/pharmacy-local/meditrino_pharmacy}"
PG_URL="${PG_URL:-postgresql://postgres:postgres@localhost:5432}"
DB_NAME="${DB_NAME:-pharmacy_local}"
PORT="${PORT:-3001}"
BOOTSTRAP_USERNAME="${PHARMACY_BOOTSTRAP_USERNAME:-demo-pharmacy}"
BOOTSTRAP_PASSWORD="${PHARMACY_BOOTSTRAP_PASSWORD:-demo-pass-123}"

for tool in git cargo psql; do
  command -v "$tool" >/dev/null || { echo "error: '$tool' is required" >&2; exit 1; }
done

echo "==> Checking out ${BRANCH} into ${CHECKOUT_DIR}"
if [[ ! -d "$CHECKOUT_DIR/.git" ]]; then
  mkdir -p "$(dirname "$CHECKOUT_DIR")"
  git clone --branch "$BRANCH" "$REPO_URL" "$CHECKOUT_DIR"
else
  cd "$CHECKOUT_DIR"
  if [[ -n "$(git status --porcelain)" ]]; then
    echo "error: $CHECKOUT_DIR has local changes; commit or stash them, or set CHECKOUT_DIR" >&2
    exit 1
  fi
  git fetch origin "$BRANCH"
  git checkout "$BRANCH"
  git pull --ff-only origin "$BRANCH"
fi
cd "$CHECKOUT_DIR"
echo "    at $(git rev-parse --short HEAD)"

db_url="${PG_URL%/}/${DB_NAME}"
admin_url="${PG_URL%/}/postgres"

echo "==> Database: ${DB_NAME}"
if [[ "$(psql "$admin_url" -tAc "SELECT 1 FROM pg_database WHERE datname = '${DB_NAME}'")" != "1" ]]; then
  psql "$admin_url" -v ON_ERROR_STOP=1 -qc "CREATE DATABASE \"${DB_NAME}\""
  echo "    created"
fi

# The pharmacy migration depends on medicine_catalog, so the stand-in goes first.
if [[ "$(psql "$db_url" -tAc "SELECT to_regclass('medicine_catalog') IS NOT NULL")" != "t" ]]; then
  echo "==> Loading medicine_catalog stand-in (development only)"
  psql "$db_url" -v ON_ERROR_STOP=1 -q -f crates/server/tests/fixtures/medicine_catalog.sql
fi

if [[ "$(psql "$db_url" -tAc "SELECT to_regclass('pharmacies') IS NOT NULL")" != "t" ]]; then
  echo "==> Applying pharmacy migration"
  psql "$db_url" -v ON_ERROR_STOP=1 -q -f migrations/postgres/20261009_pharmacy_auth_and_sync.sql
fi

echo "==> Building server (first build can take a few minutes)"
cargo build -q -p server

if [[ "$(psql "$db_url" -tAc "SELECT count(*) FROM pharmacies")" == "0" ]]; then
  echo "==> Creating demo pharmacy account '${BOOTSTRAP_USERNAME}'"
  DATABASE_URL="$db_url" \
    PHARMACY_BOOTSTRAP_USERNAME="$BOOTSTRAP_USERNAME" \
    PHARMACY_BOOTSTRAP_PASSWORD="$BOOTSTRAP_PASSWORD" \
    ./target/debug/server create-pharmacy
fi

echo "==> Server starting on http://localhost:${PORT}"
echo "    sign in as '${BOOTSTRAP_USERNAME}' / '${BOOTSTRAP_PASSWORD}'"
exec env DATABASE_URL="$db_url" \
  ADDR="0.0.0.0:${PORT}" \
  PHARMACY_SECURE_COOKIES=false \
  ./target/debug/server
