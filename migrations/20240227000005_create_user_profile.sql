-- Migration: Create User Profile table for shared settings
CREATE TABLE IF NOT EXISTS user_profile (
    user_id INTEGER NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (user_id, key),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Initiale Standardwerte für User 1 (Philipp) zum Testen
INSERT OR IGNORE INTO user_profile (user_id, key, value) VALUES (1, 'location.city', 'Berlin');
INSERT OR IGNORE INTO user_profile (user_id, key, value) VALUES (1, 'identity.name', 'Philipp');
