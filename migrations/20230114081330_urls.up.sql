-- Add up migration script here
CREATE TABLE IF NOT EXISTS urls(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    channel_id TEXT NOT NULL,
    src TEXT NOT NULL UNIQUE,
    num TEXT NOT NULL,
    active BOOLEAN NOT NULL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);
