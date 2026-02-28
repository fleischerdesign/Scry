-- Migration: Dashboard and Widget system (Grid-based)
CREATE TABLE IF NOT EXISTS dashboards (
    id TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    is_default BOOLEAN DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS dashboard_widgets (
    id TEXT PRIMARY KEY,
    dashboard_id TEXT NOT NULL,
    type TEXT NOT NULL, -- 'semantic_top', 'semantic_series', 'stat'
    title TEXT,
    config TEXT NOT NULL, -- JSON config
    width_span INTEGER DEFAULT 1, -- 1 to 4 columns
    sort_order INTEGER DEFAULT 0,
    FOREIGN KEY (dashboard_id) REFERENCES dashboards(id) ON DELETE CASCADE
);
