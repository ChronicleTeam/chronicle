CREATE TABLE app_table (
    app_table_id SERIAL PRIMARY KEY,
    app_user_id INT NOT NULL REFERENCES app_user(app_user_id),
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    real_table_name TEXT,
    field_table_name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ,
    UNIQUE (app_user_id, name)
);

SELECT trigger_updated_at('app_table');

CREATE OR REPLACE FUNCTION set_real_table_name()
RETURNS TRIGGER AS
$$
BEGIN
    NEW.real_table_name := '_app_table_' || NEW.app_table_id;
    NEW.field_table_name := '_app_table_field_' || NEW.app_table_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


CREATE TRIGGER trigger_real_table_name
BEFORE INSERT OR UPDATE ON app_table
FOR EACH ROW
WHEN (NEW.real_table_name IS NULL OR NEW.field_table_name IS NULL)
EXECUTE FUNCTION set_real_table_name();
