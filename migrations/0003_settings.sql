-- Migration 0003: App settings for safe mode
CREATE TABLE IF NOT EXISTS app_settings (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

INSERT OR IGNORE INTO app_settings (key, value)
VALUES ('safe_mode', 'false');