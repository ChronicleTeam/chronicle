/*
A field in a user table. Contains JSON options under field_kind. See src/model/data/field.rs
*/
CREATE TABLE meta_field (
    field_id SERIAL PRIMARY KEY,
    table_id INT NOT NULL REFERENCES meta_table(table_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    ordering INT NOT NULL,
    field_kind JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ,
    UNIQUE (table_id, name)
);

SELECT trigger_updated_at('meta_field');
