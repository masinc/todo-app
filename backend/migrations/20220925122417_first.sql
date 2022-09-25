-- Add migration script here

CREATE TABLE IF NOT EXISTS tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title STRING NOT NULL,
    done INTEGER NOT NULL DEFAULT 0
);

INSERT INTO tasks (title)
VAlUES
    ("First Task"),
    ("Second Task");
