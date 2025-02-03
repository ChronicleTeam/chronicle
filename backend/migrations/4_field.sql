CREATE TABLE table_field (
    field_id SERIAL PRIMARY KEY,
    table_id INT NOT NULL REFERENCES table_metadata(table_id),
    field_options JSONB NOT NULL,
    data_field_name TEXT,
    created_at TIMESTAMPTZTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZTZ
);

SELECT trigger_updated_at('table_field');

CREATE OR REPLACE FUNCTION set_data_field_name()
RETURNS TRIGGER AS
$$
BEGIN
    NEW.data_field_name := '_' ||
        SELECT COUNT(*) + 1
        FROM table_field
        WHERE table_id = NEW.table_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


CREATE TRIGGER trigger_data_field_name
BEFORE INSERT ON table_field
FOR EACH ROW
WHEN (NEW.data_field_name IS NULL)
EXECUTE FUNCTION set_data_field_name();

CREATE TABLE field_enumeration (
    enumeration_id SERIAL PRIMARY KEY,
    field_id INT NOT NULL REFERENCES table_field(field_id),
    enumeration_value TEXT NOT NULL,
    UNIQUE (field_id, enumeration_value)
);