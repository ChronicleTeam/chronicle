-- Active: 1738002585894@@127.0.0.1@5432@user
CREATE TABLE meta_field (
    field_id SERIAL PRIMARY KEY,
    table_id INT NOT NULL REFERENCES meta_table(table_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    options JSONB NOT NULL,
    data_field_name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ,
    UNIQUE (table_id, name)
);

SELECT trigger_updated_at('meta_field');

CREATE OR REPLACE FUNCTION set_data_field_name()
RETURNS TRIGGER AS
$$
DECLARE
    field_count INTEGER;
BEGIN
    SELECT COUNT(*) + 1 INTO field_count
    FROM meta_field
    WHERE table_id = NEW.table_id;

    NEW.data_field_name := '_' || field_count;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


CREATE TRIGGER trigger_data_field_name
BEFORE INSERT ON meta_field
FOR EACH ROW
WHEN (NEW.data_field_name IS NULL)
EXECUTE FUNCTION set_data_field_name();

CREATE TABLE field_enumeration (
    enumeration_id SERIAL PRIMARY KEY,
    field_id INT NOT NULL REFERENCES meta_field(field_id),
    enumeration_value TEXT NOT NULL,
    UNIQUE (field_id, enumeration_value)
);