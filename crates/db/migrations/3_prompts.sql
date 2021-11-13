CREATE TABLE IF NOT EXISTS prompts (
  id BIGSERIAL NOT NULL PRIMARY KEY,
  discriminant INTEGER NOT NULL,
  message_id INTEGER NOT NULL,
  payload INTEGER,
  recipient_discord_id INTEGER NOT NULL UNIQUE,
  FOREIGN KEY (recipient_discord_id) references users(discord_id)
);
