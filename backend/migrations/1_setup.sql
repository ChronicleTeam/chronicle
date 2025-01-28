-- Active: 1738002585894@@127.0.0.1@5432@postgres
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS
$$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION trigger_updated_at(tablename REGCLASS)
RETURNS VOID AS
$$
BEGIN
    EXECUTE format('
        CREATE TRIGGER set_updated_at
        BEFORE UPDATE
        ON %s
        FOR EACH ROW
        WHEN (OLD IS DISTINCT FROM NEW)
        EXECUTE FUNCTION set_updated_at();
    ', tablename);
end;
$$ language plpgsql;

CREATE COLLATION case_insensitive (PROVIDER = icu, LOCALE = 'und-u-ks-level2', DETERMINISTIC = FALSE);