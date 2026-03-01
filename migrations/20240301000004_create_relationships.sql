CREATE TABLE IF NOT EXISTS entity_relationships (
    user_id INTEGER NOT NULL,
    plugin_id TEXT NOT NULL,
    source_ns TEXT NOT NULL,
    source_type TEXT NOT NULL,
    source_id TEXT NOT NULL,
    predicate TEXT NOT NULL,
    target_ns TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(user_id, plugin_id, source_ns, source_type, source_id, predicate, target_ns, target_type, target_id)
);
CREATE INDEX idx_rel_source ON entity_relationships(user_id, source_ns, source_type, source_id);
CREATE INDEX idx_rel_target ON entity_relationships(user_id, target_ns, target_type, target_id);
