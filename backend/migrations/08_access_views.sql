CREATE VIEW meta_table_access_v AS
SELECT user_id, resource_id, access_role
FROM meta_table_access;

CREATE VIEW dashboard_access_v AS
SELECT user_id, resource_id, access_role
FROM dashboard_access;
