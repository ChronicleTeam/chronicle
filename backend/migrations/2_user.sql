CREATE TABLE user(
    user_id SERIAL PRIMARY KEY,
    username TEXT COLLATE case_insensitive UNIQUE NOT NULL,
    -- email TEXT COLLATE case_insensitive UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

SELECT trigger_updated_at('"user"');