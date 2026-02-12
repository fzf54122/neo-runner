CREATE TABLE IF NOT EXISTS users (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL
);

INSERT INTO users (name) VALUES ('alice');
INSERT INTO users (name) VALUES ('bob');
