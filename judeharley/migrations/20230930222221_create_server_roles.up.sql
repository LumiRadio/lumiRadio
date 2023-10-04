CREATE TABLE server_role_config (
    id SERIAL PRIMARY KEY NOT NULL,
    guild_id BIGINT NOT NULL,
    role_id BIGINT NOT NULL,
    minimum_hours INTEGER NOT NULL
)