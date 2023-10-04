CREATE TABLE users (
    -- ID is a BIGINT because it's a Snowflake ID
    id BIGINT PRIMARY KEY NOT NULL,


    -- YouTube channel ID, obtained from linking your account
    youtube_channel_id TEXT,

    -- Imported information from the old bot
    watched_time NUMERIC,
    boonbucks INT NOT NULL DEFAULT 0,

    -- Metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);