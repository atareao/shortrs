-- Add up migration script here
CREATE TABLE IF NOT EXISTS urls(
    url_id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    src TEXT NOT NULL UNIQUE,
    num INTEGER NOT NULL,
    active BOOLEAN NOT NULL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);
