CREATE TABLE IF NOT EXISTS prompts (
  id BIGSERIAL NOT NULL PRIMARY KEY,
  discriminant BIGINT NOT NULL,
  message_id BIGINT NOT NULL,
  payload BIGINT,
  recipient_discord_id BIGINT NOT NULL UNIQUE,
  FOREIGN KEY (recipient_discord_id) references users(discord_id)
);
