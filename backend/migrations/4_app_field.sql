CREATE TYPE field_enum AS ENUM(
    'Text',
    'Number',
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
    'AppTable'
);

CREATE TABLE app_field (
    field_id SERIAL PRIMARY KEY,
    table_id INT NOT NULL REFERENCES app_table(table_id),
    field_kind field_enum NOT NULL
);

CREATE TABLE field_text (
    field_id INT NOT NULL REFERENCES app_field(field_id),
    is_required BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (field_id)
);

CREATE TABLE field_number (
    field_id INT NOT NULL REFERENCES app_field(field_id),
    is_required BOOLEAN NOT NULL DEFAULT FALSE,
    value_range_start NUMERIC,
    value_range_end NUMERIC,
    scientifc_notation BOOLEAN,
    number_precision INT,
    number_scale INT,
    is_money BOOLEAN,
    CONSTRAINT check_value_range CHECK (
        value_range_start IS NULL
        OR value_range_end IS NULL
        OR value_range_start <= value_range_end
    ),
    PRIMARY KEY (field_id)
);

CREATE TABLE field_progress (
    field_id INT NOT NULL REFERENCES app_field(field_id),
    total_steps INT NOT NULL,
    PRIMARY KEY (field_id)
);

CREATE TABLE field_date_time (
    field_id INT NOT NULL REFERENCES app_field(field_id),
    is_required BOOLEAN NOT NULL DEFAULT FALSE,
    value_range_start TIMESTAMP,
    value_range_end TIMESTAMP,
    date_time_format TEXT,
    CONSTRAINT check_value_range CHECK (
        value_range_start IS NULL
        OR value_range_end IS NULL
        OR value_range_start <= value_range_end
    ),
    PRIMARY KEY (field_id)
);

CREATE TABLE field_interval (
    field_id INT NOT NULL REFERENCES app_field(field_id),
    is_required BOOLEAN NOT NULL DEFAULT FALSE,
    value_range_start INTERVAL,
    value_range_end INTERVAL,
    interval_format TEXT,
    CONSTRAINT check_value_range CHECK (
        value_range_start IS NULL
        OR value_range_end IS NULL
        OR value_range_start <= value_range_end
    ),
    PRIMARY KEY (field_id)
);

CREATE TABLE field_web_link (
    field_id INT NOT NULL REFERENCES app_field(field_id),
    is_required BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (field_id)
);

CREATE TABLE field_email (
    field_id INT NOT NULL REFERENCES app_field(field_id),
    is_required BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (field_id)
);

CREATE TABLE field_checkbox (
    field_id INT NOT NULL REFERENCES app_field(field_id),
    default_value BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (field_id)
);

CREATE TABLE enumeration_value (
    field_id INT NOT NULL REFERENCES app_field(field_id),
    value TEXT NOT NULL,
    ordering INT NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE (field_id, value)
);

CREATE TABLE field_enumeration (
    field_id INT NOT NULL REFERENCES app_field(field_id),
    is_required BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (field_id)
);

CREATE TABLE field_creation_date (
    field_id INT NOT NULL REFERENCES app_field(field_id),
    date_time_format TEXT,
    PRIMARY KEY (field_id)
);


CREATE TABLE field_modification_date (
    field_id INT NOT NULL REFERENCES app_field(field_id),
    date_time_format TEXT,
    PRIMARY KEY (field_id)
);

CREATE TABLE field_image (
    field_id INT NOT NULL REFERENCES app_field(field_id),
    is_required BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (field_id)
);

CREATE TABLE field_file (
    field_id INT NOT NULL REFERENCES app_field(field_id),
    is_required BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (field_id)
);

