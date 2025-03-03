CREATE TYPE aggregate AS ENUM (
    'Sum',
    'Average',
    'Count'
);


CREATE TYPE mark_kind AS ENUM (
    'Color',
    'Size',
    'Tooltip',
    'Label'
)

CREATE TYPE chart_kind AS ENUM (
    'Bar',
    'Line'
)

CREATE TYPE axis AS (
    field_id INT NOT NULL REFERENCES meta_field(field_id),
    aggregate aggregate
);

CREATE TABLE chart (
    chart_id SERIAL PRIMARY KEY,
    dashboard_id INT NOT NULL REFERENCES dashboard(dashboard_id),
    title TEXT COLLATE case_insensitive NOT NULL,
    chart_kind chart_kind NOT NULL,
    x_axis axis NOT NULL,
    y_axis axis NOT NULL, 
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);



CREATE TABLE mark (
    mark_id SERIAL PRIMARY KEY,
    chart_id INT NOT NULL REFERENCES chart(chart_id),
    field_id INT NOT NULL REFERENCES meta_field(field_id),
    mark_kind mark_kind NOT NULL,
);