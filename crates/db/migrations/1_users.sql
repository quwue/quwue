CREATE TABLE IF NOT EXISTS users (
  id                  INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  discord_id          INTEGER NOT NULL UNIQUE,
  welcomed            BOOLEAN NOT NULL DEFAULT FALSE,
  bio                 TEXT             DEFAULT NULL
);
