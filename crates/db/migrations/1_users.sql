CREATE TABLE IF NOT EXISTS users (
  id                INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  discord_id        INTEGER NOT NULL UNIQUE,
  prompt            TEXT             DEFAULT NULL,
  prompt_message_id INTEGER          DEFAULT NULL,
  welcomed          BOOLEAN NOT NULL DEFAULT FALSE,
  bio               TEXT             DEFAULT NULL,
  profile_image_url TEXT             DEFAULT NULL
);
