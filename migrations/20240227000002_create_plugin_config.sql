-- Migration: Create plugin_config table
CREATE TABLE IF NOT EXISTS plugin_config (
    plugin_name TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    is_secret BOOLEAN DEFAULT 0,
    PRIMARY KEY (plugin_name, key)
);

-- Beispiel-Konfiguration für das Musik-Plugin (optional)
INSERT OR IGNORE INTO plugin_config (plugin_name, key, value, is_secret) 
VALUES ('scry_music_plugin', 'poll_interval_seconds', '60', 0);
