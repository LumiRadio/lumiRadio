-- Add up migration script here
ALTER TABLE server_config ADD COLUMN dice_roll INTEGER NOT NULL DEFAULT 0;
