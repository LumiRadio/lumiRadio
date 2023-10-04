CREATE TABLE connected_youtube_accounts (
    id SERIAL PRIMARY KEY NOT NULL,
    user_id BIGINT NOT NULL,
    youtube_channel_id TEXT NOT NULL,
    youtube_channel_name TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id)
);