/*
Contains table meta data and is the parent entity to user fields and entries.
*/
CREATE TABLE meta_table (
    table_id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES app_user(user_id),
    name TEXT COLLATE case_insensitive NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    data_table_name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ,
    UNIQUE (user_id, name)
);

SELECT trigger_updated_at('meta_table');


-- All user tables will be organized under this schema.
CREATE SCHEMA data_table;
