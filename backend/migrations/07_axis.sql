CREATE TYPE aggregate AS ENUM (
    'Sum',
    'Average',
    'Min',
    'Max',
    'Count'
);

CREATE TYPE axis_kind AS ENUM (
    'X',
    'Y',
    'Color',
    'Size',
    'Tooltip',
    'Label'
);


CREATE TABLE axis (
    axis_id SERIAL PRIMARY KEY,
    chart_id INT NOT NULL REFERENCES chart(chart_id) ON DELETE CASCADE,
    field_id INT NOT NULL REFERENCES meta_field(field_id),
    axis_kind axis_kind NOT NULL,
    aggregate aggregate,
    data_item_name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

SELECT trigger_updated_at('axis');
