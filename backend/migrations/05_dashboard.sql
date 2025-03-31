/*
A dashboard containing many charts.
*/
CREATE TABLE dashboard (
    dashboard_id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES app_user(user_id),
    name TEXT COLLATE case_insensitive NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ,
    UNIQUE (user_id, name)
);

SELECT trigger_updated_at('dashboard');

SELECT trigger_rename_duplicate('dashboard', 'dashboard_id', 'user_id');

/*
All dynamic views are put under this schema.
*/
CREATE SCHEMA data_view;