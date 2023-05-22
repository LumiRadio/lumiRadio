CREATE TABLE songs (
    file_path VARCHAR(255) NOT NULL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    artist VARCHAR(255) NOT NULL,
    album VARCHAR(255) NOT NULL,

    played INT NOT NULL DEFAULT 0,
    requested INT NOT NULL DEFAULT 0,

    -- Full text search
    tsvector TSVECTOR GENERATED ALWAYS AS (
        to_tsvector('english', title) ||
        to_tsvector('english', artist) ||
        to_tsvector('english', album)
    ) STORED
);