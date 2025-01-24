CREATE TABLE user_settings (
    username VARCHAR NOT NULL,
    background_opacity INTEGER NOT NULL,
    fps_target INTEGER NOT NULL,
    PRIMARY KEY(username),
    FOREIGN KEY(username) REFERENCES users(username)
)
