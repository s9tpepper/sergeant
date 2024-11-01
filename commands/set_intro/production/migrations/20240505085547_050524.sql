-- Add migration script here
CREATE TABLE IF NOT EXISTS intros (
  id INTEGER PRIMARY KEY NOT NULL,
  name VARCHAR(255) UNIQUE NOT NULL,
  file_path VARCHAR(255) NOT NULL,
  message VARCHAR(255) NOT NULL,
  approved BOOLEAN DEFAULT FALSE,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  last_played_on DATETIME
);
