ALTER TABLE trips ADD COLUMN start_date TEXT;
ALTER TABLE trips ADD COLUMN end_date TEXT;

CREATE TABLE calendar_events (
    id          INTEGER PRIMARY KEY,
    trip_id     INTEGER NOT NULL REFERENCES trips(id) ON DELETE CASCADE,
    name        TEXT NOT NULL CHECK (length(trim(name)) > 0),
    start_date  TEXT NOT NULL,
    end_date    TEXT NOT NULL,
    created_at  INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at  INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE INDEX idx_calendar_events_trip_dates
    ON calendar_events(trip_id, start_date, end_date, id);
