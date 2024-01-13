CREATE TABLE IF NOT EXISTS user_groups (
    id bigserial PRIMARY KEY,
    name text NOT NULL,
    role smallint NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

INSERT INTO user_groups (name, role) VALUES('Admins', 0);
INSERT INTO user_groups (name, role) VALUES('Users', 1);

CREATE TABLE IF NOT EXISTS users (
    id bigserial PRIMARY KEY,
    group_id int8 NOT NULL DEFAULT 1 REFERENCES user_groups(id) ON DELETE CASCADE ON UPDATE CASCADE,
    login text NOT NULL UNIQUE,
    full_name text NOT NULL,
    email text NOT NULL UNIQUE,
    password text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS users_email_idx ON users (email);
