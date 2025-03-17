
CREATE TYPE chart_kind AS ENUM (
    'Table',
    'Bar',
    'Line'
);

CREATE TABLE chart (
    chart_id SERIAL PRIMARY KEY,
    dashboard_id INT NOT NULL REFERENCES dashboard(dashboard_id),
    table_id INT NOT NULL REFERENCES meta_table(table_id),
    name TEXT COLLATE case_insensitive NOT NULL,
    chart_kind chart_kind NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

SELECT trigger_updated_at('chart');

SELECT trigger_rename_duplicate('chart', 'chart_id', 'dashboard_id');
