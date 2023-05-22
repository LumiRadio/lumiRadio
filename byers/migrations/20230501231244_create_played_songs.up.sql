CREATE TABLE played_songs (
    id SERIAL PRIMARY KEY,
    song_id VARCHAR(255) NOT NULL,
    played_at TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_played_songs_song_id FOREIGN KEY (song_id) REFERENCES songs (file_path)
);