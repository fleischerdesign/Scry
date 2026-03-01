-- Migration: Fix plugin_config multi-tenancy
DROP TABLE IF EXISTS plugin_config;
CREATE TABLE plugin_config (
    user_id INTEGER NOT NULL,
    plugin_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    is_secret BOOLEAN DEFAULT 0,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(user_id, plugin_id, key),
    FOREIGN KEY(user_id) REFERENCES users(id)
);
