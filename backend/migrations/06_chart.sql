/*
Kind of chart to display.
*/
DO $$ BEGIN
    CREATE TYPE chart_kind AS ENUM (
        'Table',
        'Bar',
        'Line'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

/*
A chart containing many axes depending on the chart_kind.
Represents an actual SQL view on the reference table.
*/
CREATE TABLE IF NOT EXISTS chart (
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
