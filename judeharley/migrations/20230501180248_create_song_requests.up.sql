CREATE TABLE song_requests (
    id SERIAL PRIMARY KEY,
    song_id VARCHAR(255) NOT NULL,
    user_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_song_requests_song_id FOREIGN KEY (song_id) REFERENCES songs (file_path),
    CONSTRAINT fk_song_requests_user_id FOREIGN KEY (user_id) REFERENCES users (id)
);