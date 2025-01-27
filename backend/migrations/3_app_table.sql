CREATE TABLE app_table (
    app_table_id SERIAL PRIMARY KEY,
    app_user_id INT NOT NULL REFERENCES app_user(app_user_id),
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ,
    UNIQUE (app_user_id, name)
);

SELECT trigger_updated_at('app_table');