/*
Contains table meta data and is the parent entity to user fields and entries.
*/
CREATE TABLE meta_table (
    table_id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES app_user(user_id),
    parent_id INT REFERENCES meta_table(table_id),
    name TEXT COLLATE case_insensitive NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ,
    UNIQUE (user_id, name)
);

SELECT trigger_updated_at('meta_table');

SELECT trigger_rename_duplicate('meta_table', 'table_id', 'user_id');

-- CREATE OR REPLACE FUNCTION rename_duplicate_table()
-- RETURNS TRIGGER AS $$
-- DECLARE
--     new_name TEXT;
--     counter INT := 1;
-- BEGIN
--     new_name := NEW.name;

--     WHILE EXISTS (
--         SELECT 1 FROM meta_table 
--         WHERE user_id = NEW.user_id 
--           AND name = new_name
--           AND table_id != NEW.table_id
--     ) LOOP
--         new_name := NEW.name || ' (' || counter || ')';
--         counter := counter + 1;
--     END LOOP;

--     NEW.name := new_name;
--     RETURN NEW;
-- END;
-- $$ LANGUAGE plpgsql;

-- CREATE TRIGGER trigger_rename_duplicate_table
-- BEFORE INSERT OR UPDATE ON meta_table
-- FOR EACH ROW EXECUTE FUNCTION rename_duplicate_table();




-- All user tables will be organized under this schema.
CREATE SCHEMA data_table;


