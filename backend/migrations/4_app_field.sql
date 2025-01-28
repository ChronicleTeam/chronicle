CREATE TYPE app_field_enum AS ENUM(
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
    'AppTable',
);

CREATE TABLE app_field (
    app_field_id SERIAL PRIMARY KEY,
    app_table_id INT NOT NULL REFERENCES app_table(app_table_id),
);

CREATE TABLE app_field_text (
    app_field_id INT REFERENCES app_field(app_field_id),
    is_required BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (app_field_id)
);

CREATE TABLE app_field_number (
    app_field_id INT REFERENCES app_field(app_field_id),
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
    PRIMARY KEY (app_field_id)
);

CREATE TABLE app_field_progress (
    app_field_id INT REFERENCES app_field(app_field_id),
    total_steps INT NOT NULL,
    PRIMARY KEY (app_field_id)
);

CREATE TABLE app_field_date_time (
    app_field_id INT REFERENCES app_field(app_field_id),
    is_required BOOLEAN NOT NULL DEFAULT FALSE,
    value_range_start TIMESTAMP,
    value_range_end TIMESTAMP,
    date_time_format TEXT,
    CONSTRAINT check_value_range CHECK (
        value_range_start IS NULL
        OR value_range_end IS NULL
        OR value_range_start <= value_range_end
    ),
    PRIMARY KEY (app_field_id)
);

CREATE TABLE app_field_interval (
    app_field_id INT REFERENCES app_field(app_field_id),
    is_required BOOLEAN NOT NULL DEFAULT FALSE,
    value_range_start INTERVAL,
    value_range_end INTERVAL,
    interval_format TEXT,
    CONSTRAINT check_value_range CHECK (
        value_range_start IS NULL
        OR value_range_end IS NULL
        OR value_range_start <= value_range_end
    ),
    PRIMARY KEY (app_field_id)
);

CREATE TABLE app_field_web_link (
    app_field_id INT REFERENCES app_field(app_field_id),
    is_required BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (app_field_id)
);

CREATE TABLE app_field_email (
    app_field_id INT REFERENCES app_field(app_field_id),
    is_required BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (app_field_id)
);

CREATE TABLE app_field_checkbox (
    app_field_id INT REFERENCES app_field(app_field_id),
    default_value BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (app_field_id)
);