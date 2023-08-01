-- Add migration script here
CREATE TABLE users(
    id INTEGER PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NULL
);

