ALTER TABLE song_requests
ADD CONSTRAINT fk_song_requests_song_id FOREIGN KEY (song_id) REFERENCES songs (file_path) ON DELETE NO ACTION;
ALTER TABLE played_songs
ADD CONSTRAINT fk_played_songs_song_id FOREIGN KEY (song_id) REFERENCES songs (file_path) ON DELETE NO ACTION;