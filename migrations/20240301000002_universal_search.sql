-- Universal Search Index via FTS5
-- Wir nutzen 'porter' für intelligentes Stemming und 'unicode61' für Sonderzeichen
CREATE VIRTUAL TABLE IF NOT EXISTS universal_search USING fts5(
    item_id UNINDEXED, 
    type UNINDEXED,     -- 'event', 'entity'
    content,            -- Der durchsuchbare Text (Payload + Metadaten)
    subtext UNINDEXED,  -- Zusatzinfo für die Liste (z.B. Kategorie)
    link UNINDEXED,     -- Navigationspfad
    user_id UNINDEXED,
    tokenize='porter unicode61'
);

-- Trigger für neue Events: Nutzt display_title und display_subtitle falls vorhanden, sonst payload
CREATE TRIGGER IF NOT EXISTS trg_events_search_insert AFTER INSERT ON events BEGIN
    INSERT INTO universal_search(item_id, type, content, subtext, link, user_id)
    VALUES (
        new.id, 
        'event',
        COALESCE(new.display_title, '') || ' ' || COALESCE(new.display_subtitle, '') || ' ' || new.category || ' ' || CAST(new.payload AS TEXT),
        new.category,
        '/event/' || new.id,
        new.user_id
    );
END;

-- Trigger für neue Entitäten
CREATE TRIGGER IF NOT EXISTS trg_entities_search_insert AFTER INSERT ON entities BEGIN
    INSERT INTO universal_search(item_id, type, content, subtext, link, user_id)
    VALUES (
        new.id, 
        'entity',
        new.id || ' ' || new.typ || ' ' || new.namespace,
        new.namespace || ' / ' || new.typ,
        '/entity/' || new.namespace || '/' || new.typ || '/' || new.id,
        new.user_id
    );
END;

-- Trigger für neue Traits (Biografien, Beschreibungen etc.)
CREATE TRIGGER IF NOT EXISTS trg_traits_search_insert AFTER INSERT ON entity_traits 
BEGIN
    -- Wir aktualisieren den Index für die Entität, wenn ein neuer Trait hinzukommt
    INSERT INTO universal_search(item_id, type, content, subtext, link, user_id)
    VALUES (
        new.entity_id, 
        'entity',
        new.trait_id || ' ' || CAST(new.value_json AS TEXT),
        'Trait Update: ' || new.trait_id,
        '/entity/' || new.namespace || '/' || new.entity_type || '/' || new.entity_id,
        new.user_id
    );
END;
