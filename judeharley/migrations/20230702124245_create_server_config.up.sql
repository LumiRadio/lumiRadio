CREATE TABLE server_channel_config (
    id BIGINT NOT NULL PRIMARY KEY,
    server_id BIGINT NOT NULL,
    allow_watch_time_accumulation BOOLEAN NOT NULL DEFAULT 'false',
    allow_point_accumulation BOOLEAN NOT NULL DEFAULT 'false'
);