ALTER TABLE operations ADD COLUMN actor_id TEXT NOT NULL DEFAULT '';
ALTER TABLE operations ADD COLUMN actor_name TEXT NOT NULL DEFAULT '';
UPDATE operations
SET actor_id = actor,
    actor_name = COALESCE((SELECT username FROM users WHERE users.id = operations.actor), 'Unknown');

CREATE TABLE remote_auth (
    singleton INTEGER PRIMARY KEY CHECK(singleton = 1),
    pharmacy_id TEXT NOT NULL,
    username TEXT NOT NULL,
    session_token TEXT NOT NULL
);
ALTER TABLE sync_state ADD COLUMN pharmacy_revision INTEGER NOT NULL DEFAULT 0;

DROP TABLE user_roles;
DROP TABLE role_permissions;
DROP TABLE roles;
DROP TABLE users;
