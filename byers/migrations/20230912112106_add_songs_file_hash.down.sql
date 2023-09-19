ALTER TABLE songs DROP CONSTRAINT songs_file_hash_unique;
ALTER TABLE songs DROP COLUMN file_hash;