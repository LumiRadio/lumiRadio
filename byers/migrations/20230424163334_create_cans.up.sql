CREATE TABLE cans (
    id SERIAL PRIMARY KEY,
    added_by BIGINT NOT NULL,
    added_at TIMESTAMP NOT NULL DEFAULT NOW(),

    -- Foreign key to the user who added this can
    FOREIGN KEY (added_by) REFERENCES users (id)
);