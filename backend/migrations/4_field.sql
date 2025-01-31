CREATE TYPE field_kind AS ENUM(
    'Text',
    'Integer',
    'Decimal',
    'Money',
    'Progress',
    'DateTime',
    'Interval',
    'WebLink',
    'Email',
    'Checkbox',
    'Enumeration',
    'CreationDate',
    'ModificationDate',
    'Image',
    'File',
    'Table'
);

CREATE TABLE table_field (
    field_id SERIAL PRIMARY KEY,
    table_id INT NOT NULL REFERENCES table_metadata(table_id),
    field_kind_value field_kind NOT NULL,
    created_at TIMESTAMPTZTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZTZ
);

SELECT trigger_updated_at('table_field');

CREATE SCHEMA field_option

CREATE TABLE field_option.is_required (
    field_id INT NOT NULL REFERENCES table_field(field_id),
    is_required BOOLEAN NOT NULL,
    PRIMARY KEY (field_id)
)

CREATE TABLE field_option.range_bigint (
    field_id INT NOT NULL REFERENCES table_field(field_id),
    range_start BIGINT,
    range_end BIGINT,
    CONSTRAINT check_range CHECK (
        range_start IS NULL
        OR range_end IS NULL
        OR range_start <= range_end
    ),
    PRIMARY KEY (field_id)
);

CREATE TABLE field_option.range_double (
    field_id INT NOT NULL REFERENCES table_field(field_id),
    range_start DOUBLE,
    range_end DOUBLE,
    CONSTRAINT check_range CHECK (
        range_start IS NULL
        OR range_end IS NULL
        OR range_start <= range_end
    ),
    PRIMARY KEY (field_id)
);


CREATE TABLE field_option.range_numeric_money (
    field_id INT NOT NULL REFERENCES table_field(field_id),
    range_start numeric_money,
    range_end numeric_money,
    CONSTRAINT check_range CHECK (
        range_start IS NULL
        OR range_end IS NULL
        OR range_start <= range_end
    ),
    PRIMARY KEY (field_id)
);

CREATE TABLE field_option.range_timestamptz (
    field_id INT NOT NULL REFERENCES table_field(field_id),
    range_start TIMESTAMPTZ,
    range_end TIMESTAMPTZ,
    date_time_format TEXT NOT NULL,
    CONSTRAINT check_range CHECK (
        range_start IS NULL
        OR range_end IS NULL
        OR range_start <= range_end
    ),
    PRIMARY KEY (field_id)
);

CREATE TABLE field_option.range_interval (
    field_id INT NOT NULL REFERENCES table_field(field_id),
    range_start INTERVAL,
    range_end INTERVAL,
    interval_format TEXT NOT NULL,
    CONSTRAINT check_range CHECK (
        range_start IS NULL
        OR range_end IS NULL
        OR range_start <= range_end
    ),
    PRIMARY KEY (field_id)
);

CREATE TABLE field_option.number_format (
    field_id INT NOT NULL REFERENCES table_field(field_id),
    scientifc_notation BOOLEAN NOT NULL,
    number_precision INT,
    number_scale INT,
    PRIMARY KEY (field_id)
);


CREATE TABLE field_option.date_time_format (
    field_id INT NOT NULL REFERENCES table_field(field_id),
    date_time_format TEXT NOT NULL,
    CONSTRAINT check_range CHECK (
        range_start IS NULL
        OR range_end IS NULL
        OR range_start <= range_end
    ),
    PRIMARY KEY (field_id)
);


CREATE TABLE field_option.interval_format (
    field_id INT NOT NULL REFERENCES table_field(field_id),
    interval_format TEXT NOT NULL,
    PRIMARY KEY (field_id)
);


CREATE TABLE field_option.total_steps (
    field_id INT NOT NULL REFERENCES table_field(field_id),
    total_steps INT NOT NULL,
    PRIMARY KEY (field_id)
);

CREATE TABLE field_option.checkbox_default (
    field_id INT NOT NULL REFERENCES table_field(field_id),
    default_value BOOLEAN NOT NULL,
    PRIMARY KEY (field_id)
);

CREATE TABLE field_option.enumeration_value (
    field_id INT NOT NULL REFERENCES table_field(field_id),
    value TEXT NOT NULL,
    ordering INT NOT NULL,
    is_default BOOLEAN NOT NULL,
    UNIQUE (field_id, value)
);
