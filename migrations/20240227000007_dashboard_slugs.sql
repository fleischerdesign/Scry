-- Migration: Add Slugs to Dashboards
ALTER TABLE dashboards ADD COLUMN slug TEXT;
