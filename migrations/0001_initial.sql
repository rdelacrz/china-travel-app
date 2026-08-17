CREATE TABLE trips (
    id          INTEGER PRIMARY KEY,
    name        TEXT NOT NULL CHECK (length(trim(name)) > 0),
    created_at  INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at  INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE checklist_items (
    id          INTEGER PRIMARY KEY,
    trip_id     INTEGER NOT NULL REFERENCES trips(id) ON DELETE CASCADE,
    text        TEXT NOT NULL CHECK (length(trim(text)) > 0),
    is_checked  INTEGER NOT NULL DEFAULT 0 CHECK (is_checked IN (0, 1)),
    sort_order  INTEGER NOT NULL CHECK (sort_order >= 0),
    created_at  INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at  INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE (trip_id, sort_order)
);

CREATE INDEX idx_checklist_items_trip
    ON checklist_items(trip_id, sort_order, id);

CREATE TABLE travel_documents (
    id                       INTEGER PRIMARY KEY,
    trip_id                  INTEGER NOT NULL REFERENCES trips(id) ON DELETE CASCADE,
    name                     TEXT NOT NULL CHECK (length(trim(name)) > 0),
    description              TEXT NOT NULL DEFAULT '',
    attachment_uri           TEXT,
    attachment_display_name  TEXT,
    attachment_mime_type     TEXT,
    created_at               INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at               INTEGER NOT NULL DEFAULT (unixepoch()),
    CHECK (attachment_uri IS NULL OR length(trim(attachment_uri)) > 0)
);

CREATE INDEX idx_travel_documents_trip
    ON travel_documents(trip_id, updated_at DESC, id DESC);
