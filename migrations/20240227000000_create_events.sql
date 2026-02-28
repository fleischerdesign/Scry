-- Migration: Create events table
CREATE TABLE IF NOT EXISTS events (
    id UUID PRIMARY KEY NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    category TEXT NOT NULL,
    source TEXT NOT NULL,
    payload JSONB NOT NULL,
    metadata JSONB
);

-- Index für schnellere Suche nach Kategorie und Zeit
CREATE INDEX IF NOT EXISTS idx_events_category ON events (category);
CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events (timestamp);
