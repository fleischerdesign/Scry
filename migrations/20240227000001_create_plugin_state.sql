-- Migration: Create plugin_state table
CREATE TABLE IF NOT EXISTS plugin_state (
    plugin_name TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (plugin_name, key)
);
