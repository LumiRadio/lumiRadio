-- Up migration
ALTER TABLE users
ADD COLUMN last_message_sent TIMESTAMP;