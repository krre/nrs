CREATE TABLE IF NOT EXISTS user_groups (
    id serial PRIMARY KEY,
    name TEXT NOT NULL,
    role smallint NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

INSERT INTO user_groups (name, role) VALUES('Admins', 0);
INSERT INTO user_groups (name, role) VALUES('Users', 1);

CREATE TABLE IF NOT EXISTS users (
    id serial PRIMARY KEY,
    group_id INT NOT NULL DEFAULT 1 REFERENCES user_groups(id) ON DELETE CASCADE ON UPDATE CASCADE,
    sign TEXT NOT NULL,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL UNIQUE,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX users_email_idx ON users (email);
