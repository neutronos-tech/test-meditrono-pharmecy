# Agent notes

## Identifiers

- Default to incrementing integer ids (`BIGINT GENERATED ALWAYS AS IDENTITY` or `INTEGER`).
- Use a UUID only when it gives a concrete advantage: ids minted offline on a client and synced later, ids that must stay stable across independent re-imports (for example deterministic UUIDv5 from a source key), or ids exposed publicly that must not be enumerable. Name the advantage in the migration or PR.
- Clients page server data by cursor on an ordered unique key (keyset paging), never by `OFFSET`.
