CREATE TABLE IF NOT EXISTS responses (
  id                INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  discord_id        INTEGER NOT NULL UNIQUE,
  candidate_id      INTEGER NOT NULL UNIQUE,
  response          BOOLEAN NOT NULL
);
