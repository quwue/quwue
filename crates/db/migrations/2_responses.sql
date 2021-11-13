CREATE TABLE IF NOT EXISTS responses (
  id BIGSERIAL NOT NULL PRIMARY KEY,
  discord_id INTEGER NOT NULL,
  candidate_id INTEGER NOT NULL,
  response BOOLEAN NOT NULL,
  dismissed BOOLEAN NOT NULL,
  UNIQUE(discord_id, candidate_id),
  FOREIGN KEY(discord_id) REFERENCES users(discord_id),
  FOREIGN KEY(candidate_id) REFERENCES users(discord_id)
);
