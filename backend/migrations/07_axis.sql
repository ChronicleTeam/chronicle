/*
Types of aggregate functions for an axis.
*/
CREATE TYPE aggregate AS ENUM (
    'Sum',
    'Average',
    'Min',
    'Max',
    'Count'
);

/*
Kind of axis. Behavior depends on the chart_kind.
*/
CREATE TYPE axis_kind AS ENUM (
    'X',
    'Y',
    'Color',
    'Size',
    'Tooltip',
    'Label'
);

/*
An axis in a chart. Represents a column in the actual SQL view.
*/
CREATE TABLE axis (
    axis_id SERIAL PRIMARY KEY,
    chart_id INT NOT NULL REFERENCES chart(chart_id) ON DELETE CASCADE,
    field_id INT NOT NULL REFERENCES meta_field(field_id),
    axis_kind axis_kind NOT NULL,
    aggregate aggregate,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

SELECT trigger_updated_at('axis');
