-- Migration: Multi-Tenancy Support
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS api_keys (
    key TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL,
    label TEXT NOT NULL,
    scopes TEXT NOT NULL, -- Kommagetrennte Liste: "data:read,data:write"
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(user_id) REFERENCES users(id)
);

-- Bestehende Tabellen migrieren
-- Wir löschen sie für den Prototyp einfach und legen sie neu an mit user_id
DROP TABLE IF EXISTS events;
CREATE TABLE events (
    id TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL,
    timestamp TEXT NOT NULL,
    category TEXT NOT NULL,
    source TEXT NOT NULL,
    payload BLOB NOT NULL, -- JSONB
    metadata BLOB,
    FOREIGN KEY(user_id) REFERENCES users(id)
);

DROP TABLE IF EXISTS plugin_state;
CREATE TABLE plugin_state (
    user_id INTEGER NOT NULL,
    plugin_name TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(user_id, plugin_name, key),
    FOREIGN KEY(user_id) REFERENCES users(id)
);

CREATE INDEX idx_events_user_time ON events(user_id, timestamp);
CREATE INDEX idx_events_user_cat ON events(user_id, category);
