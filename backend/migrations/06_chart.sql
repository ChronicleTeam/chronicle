CREATE TABLE chart (
    chart_id SERIAL PRIMARY KEY,
    dashboard_id INT NOT NULL REFERENCES dashboard(dashboard_id),
    title TEXT COLLATE case_insensitive NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ,
    UNIQUE (user_id, name)
);