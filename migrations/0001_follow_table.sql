CREATE TABLE follow (
    id INTEGER NOT NULL,
    first_seen DATETIME NOT NULL,
    last_seen DATETIME NOT NULL,

    PRIMARY KEY (id, first_seen)
);
