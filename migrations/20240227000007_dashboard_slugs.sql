-- Migration: Add Slugs to Dashboards
ALTER TABLE dashboards ADD COLUMN slug TEXT;

-- Bestehende Dashboards mit Slugs versorgen
UPDATE dashboards SET slug = 'default' WHERE id = 'default-1';
