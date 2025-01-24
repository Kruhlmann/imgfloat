CREATE TABLE channel_admins (
  username VARCHAR NOT NULL,
  broadcaster_username VARCHAR NOT NULL,
  PRIMARY KEY(username),
  FOREIGN KEY(username) REFERENCES users(username),
  FOREIGN KEY(broadcaster_username) REFERENCES users(username)
)
