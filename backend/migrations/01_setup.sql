-- Active: 1738002585894@@127.0.0.1@5432@user
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS
$$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION trigger_updated_at(table_name TEXT)
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
    ', table_name);
end;
$$ language plpgsql;

CREATE OR REPLACE FUNCTION rename_duplicate()
RETURNS TRIGGER AS $$
DECLARE
    table_name TEXT := TG_ARGV[0];
    pk_column TEXT := TG_ARGV[1];
    scope_column TEXT := TG_ARGV[2];
    new_name TEXT;
    counter INT := 1;
    exists_check BOOLEAN;
BEGIN
    new_name := NEW.name;

    -- RAISE WARNING 'Starting loop with new_name: %', new_name;

    LOOP
        EXECUTE format(
            'SELECT EXISTS (
                SELECT 1 FROM %1$s
                WHERE %2$s = $1.%2$s
                    AND name = $2
                    AND %3$s != $1.%3$s
            )',
            table_name, scope_column, pk_column
        )
        INTO exists_check
        USING NEW, new_name;

        -- RAISE WARNING 'exists_check: %', exists_check;

        EXIT WHEN NOT exists_check;

        new_name := NEW.name || ' (' || counter || ')';
        counter := counter + 1;

        -- RAISE WARNING 'Updated new_name: %', new_name;
    END LOOP;

    NEW.name = new_name;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION trigger_rename_duplicate(
    table_name TEXT,
    pk_column TEXT,
    scope_column TEXT
) RETURNS VOID AS
$$
BEGIN
    EXECUTE format('
        CREATE TRIGGER rename_duplicate
        BEFORE INSERT OR UPDATE
        ON %1$s
        FOR EACH ROW
        EXECUTE FUNCTION rename_duplicate(''%1$s'', ''%2$s'', ''%3$s'');
    ', table_name, pk_column, scope_column);
end;
$$ language plpgsql;

CREATE COLLATION case_insensitive (
    PROVIDER = icu,
    LOCALE = 'und-u-ks-level2',
    DETERMINISTIC = FALSE
);

CREATE DOMAIN numeric_money AS NUMERIC(15, 4);