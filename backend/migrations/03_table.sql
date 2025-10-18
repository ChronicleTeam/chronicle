/*
Contains table meta data and is the parent entity to user fields and entries.
Can have a parent table or children tables. Represents an actual SQL table.
*/
CREATE TABLE meta_table (
    table_id SERIAL PRIMARY KEY,
    parent_id INT REFERENCES meta_table(table_id),
    name TEXT COLLATE case_insensitive NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

SELECT trigger_updated_at('meta_table');

-- SELECT trigger_rename_duplicate('meta_table', 'table_id', 'user_id');

CREATE TABLE meta_table_access (
    user_id INT NOT NULL REFERENCES app_user(user_id) ON DELETE CASCADE,
    resource_id INT NOT NULL REFERENCES meta_table(table_id) ON DELETE CASCADE,
    access_role access_role NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    PRIMARY KEY (user_id, resource_id)
);

SELECT trigger_updated_at('meta_table_access');


/*
All dynamic tables are put under this schema.
*/
CREATE SCHEMA data_table;


