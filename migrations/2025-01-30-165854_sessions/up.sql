CREATE TABLE sessions (
  username VARCHAR NOT NULL,
  token VARCHAR NOT NULL,
  expires DATETIME NOT NULL,
  PRIMARY KEY(username),
  FOREIGN KEY(username) REFERENCES users(username)
)
