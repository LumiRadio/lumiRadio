BEGIN;
UPDATE users
SET watched_time = 0
WHERE watched_time IS NULL;
ALTER TABLE users
ALTER COLUMN watched_time
SET DEFAULT 0,
    ALTER COLUMN watched_time
SET NOT NULL;
COMMIT;