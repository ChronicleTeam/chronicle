/*
A field in a user table. Contains JSON options under field_kind. See src/model/data/field.rs
*/
CREATE TABLE meta_field (
    field_id SERIAL PRIMARY KEY,
    table_id INT NOT NULL REFERENCES meta_table(table_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    field_kind JSONB NOT NULL,
    data_field_name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ,
    UNIQUE (table_id, name)
);

SELECT trigger_updated_at('meta_field');

/*
data_field_name is the name of the actual column name in postgres 
and it is set dynamically to prevent injection.
This should never be sent to the front-end.
*/
CREATE OR REPLACE FUNCTION set_data_field_name()
RETURNS TRIGGER AS
$$
DECLARE
    field_count INTEGER;
BEGIN
    NEW.data_field_name := 'f' || NEW.field_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


CREATE TRIGGER trigger_data_field_name
BEFORE INSERT ON meta_field
FOR EACH ROW
WHEN (NEW.data_field_name IS NULL)
EXECUTE FUNCTION set_data_field_name();