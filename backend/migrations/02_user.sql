CREATE TYPE user_role AS ENUM (
    'Admin',
    'Normal'
);


CREATE TABLE app_user (
    user_id SERIAL PRIMARY KEY,
    username TEXT COLLATE case_insensitive UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role user_role NOT NULL DEFAULT 'Normal',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

SELECT trigger_updated_at('app_user');