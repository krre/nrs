CREATE TABLE IF NOT EXISTS modules (
    id bigserial PRIMARY KEY,
    project_id int8 NOT NULL REFERENCES projects(id) ON DELETE CASCADE ON UPDATE CASCADE,
    module_id int8 REFERENCES modules(id) ON DELETE CASCADE ON UPDATE CASCADE,
    name text NOT NULL,
    visibility smallint NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
