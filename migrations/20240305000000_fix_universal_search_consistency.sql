-- Fix Universal Search Consistency: Add DELETE and UPDATE triggers
-- ensure that when data is removed or changed, the search index stays in sync.

-- 1. Events
CREATE TRIGGER IF NOT EXISTS trg_events_search_delete AFTER DELETE ON events BEGIN
    DELETE FROM universal_search WHERE item_id = old.id AND type = 'event';
END;

CREATE TRIGGER IF NOT EXISTS trg_events_search_update AFTER UPDATE ON events BEGIN
    DELETE FROM universal_search WHERE item_id = old.id AND type = 'event';
    INSERT INTO universal_search(item_id, type, content, subtext, link, user_id)
    VALUES (
        new.id, 
        'event',
        COALESCE(new.display_title, '') || ' ' || COALESCE(new.display_subtitle, '') || ' ' || new.category,
        new.category,
        '/event/' || new.id,
        new.user_id
    );
END;

-- 2. Entities
CREATE TRIGGER IF NOT EXISTS trg_entities_search_delete AFTER DELETE ON entities BEGIN
    DELETE FROM universal_search WHERE item_id = old.id AND type = 'entity';
END;

CREATE TRIGGER IF NOT EXISTS trg_entities_search_update AFTER UPDATE ON entities BEGIN
    DELETE FROM universal_search WHERE item_id = old.id AND type = 'entity';
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

-- 3. Traits (Maintenance)
-- If a trait is updated, we just add to the index (FTS5 handles multiple entries for same item_id well)
-- but on delete we might want to keep the entry clean. For now, deleting the entity cleans everything.
-- No trigger needed here for now as entity delete handles cleanup.
