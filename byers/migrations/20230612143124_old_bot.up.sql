CREATE TABLE slcb_currency (
    id SERIAL NOT NULL PRIMARY KEY,
    username TEXT NOT NULL,
    points INT NOT NULL DEFAULT 0,
    hours INT NOT NULL DEFAULT 0
);
CREATE TABLE slcb_rank (
    id SERIAL NOT NULL PRIMARY KEY,
    rank_name TEXT NOT NULL,
    hour_requirement INT NOT NULL,
    channel_id TEXT
);
CREATE TABLE bp_counters (
    constant TEXT PRIMARY KEY NOT NULL,
    value INT NOT NULL
);