CREATE TABLE channel_admins (
  id VARCHAR NOT NULL,
  admin_username VARCHAR NOT NULL,
  broadcaster_username VARCHAR NOT NULL,
  PRIMARY KEY(id),
  FOREIGN KEY(broadcaster_username) REFERENCES users(twitch_username),
  FOREIGN KEY(admin_username) REFERENCES users(twitch_username)
)
