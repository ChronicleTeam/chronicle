-- Active: 1738094970568@@127.0.0.1@5432@user@public
INSERT INTO
    app_user (username, password_hash)
VALUES ('admin', 'password');


-- INSERT INTO meta_field (table_id, name, options)
-- VALUES (1, 'Test Field', '{"type": "text", "is_required": false}'::json)
-- RETURNING field_id, data_field_name;

