ALTER TABLE songs
ADD COLUMN file_hash VARCHAR(64) NOT NULL DEFAULT '';
ALTER TABLE songs
ADD CONSTRAINT songs_file_hash_unique UNIQUE (file_hash);