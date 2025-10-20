/*
A dashboard containing many charts.
*/
CREATE TABLE IF NOT EXISTS dashboard (
    dashboard_id SERIAL PRIMARY KEY,
    name TEXT COLLATE case_insensitive NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

SELECT trigger_updated_at('dashboard');

-- SELECT trigger_rename_duplicate('dashboard', 'dashboard_id', 'user_id');

CREATE TABLE IF NOT EXISTS dashboard_access (
    user_id INT NOT NULL REFERENCES app_user(user_id) ON DELETE CASCADE,
    resource_id INT NOT NULL REFERENCES dashboard(dashboard_id) ON DELETE CASCADE,
    access_role access_role NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    PRIMARY KEY (user_id, resource_id)
);

SELECT trigger_updated_at('dashboard_access');


/*
All dynamic views are put under this schema.
*/
CREATE SCHEMA IF NOT EXISTS data_view;