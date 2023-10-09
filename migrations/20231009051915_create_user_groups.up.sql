CREATE TABLE IF NOT EXISTS user_groups (
    id serial PRIMARY KEY,
    name TEXT NOT NULL,
    role smallint NOT NULL,
    created_ts timestamptz NOT NULL DEFAULT now()
);

INSERT INTO user_groups(name, role) VALUES('Admins', 0);
INSERT INTO user_groups(name, role) VALUES('Users', 1);
