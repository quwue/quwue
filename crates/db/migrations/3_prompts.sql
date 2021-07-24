CREATE TABLE IF NOT EXISTS prompt (
  id                   INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  prompt_discriminant  INTEGER NOT NULL,
  prompt_message_id    INTEGER NOT NULL,
  prompt_payload       INTEGER,
  recipient_discord_id INTEGER NOT NULL,
  FOREIGN KEY (recipient_discord_id) references users(discord_id)
);
