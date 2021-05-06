CREATE TABLE IF NOT EXISTS responses (
  id                INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  discord_id        INTEGER NOT NULL,
  candidate_id      INTEGER NOT NULL,
  response          BOOLEAN NOT NULL,
  UNIQUE(discord_id, candidate_id)
);
