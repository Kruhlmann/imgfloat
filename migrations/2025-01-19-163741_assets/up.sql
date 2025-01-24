CREATE TABLE assets (
    local_filename VARCHAR NOT NULL,
    original_filename VARCHAR NOT NULL,
    checksum VARCHAR NOT NULL,
    content_type VARCHAR NOT NULL,
    username VARCHAR NOT NULL,
    PRIMARY KEY(local_filename),
    FOREIGN KEY(username) REFERENCES users(username),
    UNIQUE(local_filename),
    UNIQUE(checksum)
)
