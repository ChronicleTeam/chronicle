/*
Types of aggregate functions for an axis.
*/
DO $$ BEGIN
    CREATE TYPE aggregate AS ENUM (
        'Sum',
        'Average',
        'Min',
        'Max',
        'Count'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;


/*
Kind of axis. Behavior depends on the chart_kind.
*/
DO $$ BEGIN
    CREATE TYPE axis_kind AS ENUM (
        'X',
        'Y',
        'Color',
        'Size',
        'Tooltip',
        'Label'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

/*
An axis in a chart. Represents a column in the actual SQL view.
*/
CREATE TABLE IF NOT EXISTS axis (
    axis_id SERIAL PRIMARY KEY,
    chart_id INT NOT NULL REFERENCES chart(chart_id) ON DELETE CASCADE,
    field_id INT NOT NULL REFERENCES meta_field(field_id),
    axis_kind axis_kind NOT NULL,
    aggregate aggregate,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

SELECT trigger_updated_at('axis');
