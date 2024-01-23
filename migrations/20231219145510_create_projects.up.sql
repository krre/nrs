CREATE TABLE IF NOT EXISTS projects (
    id bigserial PRIMARY KEY,
    user_id int8 NOT NULL REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE,
    name text NOT NULL,
    target smallint NOT NULL,
    description text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS projects_user_id_idx ON projects (user_id);
