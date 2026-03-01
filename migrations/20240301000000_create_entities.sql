-- Semantic Entities & Traits Table
CREATE TABLE IF NOT EXISTS entities (
    user_id INTEGER NOT NULL,
    namespace TEXT NOT NULL,
    typ TEXT NOT NULL,
    id TEXT NOT NULL,
    canonical_id TEXT, -- Verweis auf die "Haupt-ID" falls Alias
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(user_id, namespace, typ, id),
    FOREIGN KEY(user_id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS entity_traits (
    user_id INTEGER NOT NULL,
    plugin_id TEXT NOT NULL, -- Welches Plugin hat diesen Wert geliefert?
    namespace TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    trait_id TEXT NOT NULL,
    value_json TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(user_id, plugin_id, namespace, entity_type, entity_id, trait_id),
    FOREIGN KEY(user_id, namespace, entity_type, entity_id) REFERENCES entities(user_id, namespace, typ, id)
);

CREATE INDEX IF NOT EXISTS idx_entity_traits_lookup ON entity_traits(user_id, namespace, entity_type, entity_id);
