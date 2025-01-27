CREATE TABLE dynamic_table (
    dynamic_table_id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES user(user_id),
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ UNIQUE,
    UNIQUE (user_id, name)
);

SELECT trigger_updated_at ('"dynamic_table"');