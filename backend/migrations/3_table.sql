CREATE TABLE table_metadata (
    table_id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES app_user(user_id),
    name COLLATE case_insensitive TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    data_table_name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ,
    UNIQUE (user_id, name)
);

SELECT trigger_updated_at('table_metadata');

CREATE SCHEMA user_table;

CREATE OR REPLACE FUNCTION set_data_table_name()
RETURNS TRIGGER AS
$$
BEGIN
    NEW.data_table_name := 'user_table._' || NEW.table_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


CREATE TRIGGER trigger_data_table_name
BEFORE INSERT OR UPDATE ON table_metadata
FOR EACH ROW
WHEN (NEW.data_table_name IS NULL)
EXECUTE FUNCTION set_data_table_name();
