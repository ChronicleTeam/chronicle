/*
A field in a user table. Contains JSON options under field_kind. See src/model/data/field.rs
*/
CREATE TABLE meta_field (
    field_id SERIAL PRIMARY KEY,
    table_id INT NOT NULL REFERENCES meta_table (table_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    ordering INT,
    field_kind JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ,
    UNIQUE (table_id, name)
);

SELECT trigger_updated_at('meta_field');

SELECT trigger_rename_duplicate('meta_field', 'field_id', 'table_id');

CREATE OR REPLACE FUNCTION set_default_ordering()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.ordering IS NULL THEN
        SELECT COALESCE(MAX(ordering) + 1, 0)
        INTO NEW.ordering
        FROM meta_field
        WHERE table_id = NEW.table_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_set_default_ordering
BEFORE INSERT ON meta_field
FOR EACH ROW
EXECUTE FUNCTION set_default_ordering();

CREATE OR REPLACE FUNCTION decrement_orderings()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE meta_field
    SET ordering = ordering - 1
    WHERE table_id = OLD.table_id
        AND ordering > OLD.ordering;

    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_decrement_orderings
AFTER DELETE ON meta_field
FOR EACH ROW
EXECUTE FUNCTION decrement_orderings();

