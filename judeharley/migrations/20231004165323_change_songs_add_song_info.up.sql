ALTER TABLE songs
ADD COLUMN bitrate INTEGER NOT NULL DEFAULT 0;
CREATE TABLE song_tags (
    id SERIAL PRIMARY KEY NOT NULL,
    song_id VARCHAR(255) NOT NULL,
    tag VARCHAR(255) NOT NULL,
    value VARCHAR(255) NOT NULL
)