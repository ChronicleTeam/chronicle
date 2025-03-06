CREATE TYPE aggregate AS ENUM (
    'Sum',
    'Average',
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

CREATE TYPE chart_kind AS ENUM (
    'Bar',
    'Line'
);

CREATE TABLE chart (
    chart_id SERIAL PRIMARY KEY,
    dashboard_id INT NOT NULL REFERENCES dashboard(dashboard_id),
    table_id INT NOT NULL REFERENCES meta_Table(table_id),
    title TEXT COLLATE case_insensitive NOT NULL,
    chart_kind chart_kind NOT NULL,
    data_view_name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

SELECT trigger_updated_at('chart');

CREATE OR REPLACE FUNCTION set_data_view_name()
RETURNS TRIGGER AS
$$
BEGIN
    NEW.data_view_name := 'data_view.c' || NEW.chart_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_data_view_name
BEFORE INSERT OR UPDATE ON chart
FOR EACH ROW
WHEN (NEW.data_view_name IS NULL)
EXECUTE FUNCTION set_data_view_name();

CREATE TABLE axis (
    axis_id SERIAL PRIMARY KEY,
    chart_id INT NOT NULL REFERENCES chart(chart_id),
    field_id INT NOT NULL REFERENCES meta_field(field_id),
    axis_kind axis_kind NOT NULL,
    aggregate aggregate,
    data_item_name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

SELECT trigger_updated_at('axis');

CREATE OR REPLACE FUNCTION set_data_item_name()
RETURNS TRIGGER AS
$$
BEGIN
    NEW.data_item_name := 'i' || NEW.axis_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_data_item_name
BEFORE INSERT OR UPDATE ON axis
FOR EACH ROW
WHEN (NEW.data_item_name IS NULL)
EXECUTE FUNCTION set_data_item_name();