/*
Application user. Passwords are stored as hashes.
*/
CREATE TABLE IF NOT EXISTS app_user (
    user_id SERIAL PRIMARY KEY,
    username TEXT COLLATE case_insensitive UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

SELECT trigger_updated_at('app_user');

DO $$ BEGIN
    CREATE TYPE access_role AS ENUM (
        'Owner',
        'Editor',
        'Viewer'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;