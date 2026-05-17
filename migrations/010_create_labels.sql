CREATE TABLE IF NOT EXISTS labels (
    id          UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        VARCHAR(64)  NOT NULL UNIQUE,
    color       VARCHAR(32),
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS task_labels (
    task_id  UUID NOT NULL REFERENCES tasks(id)  ON DELETE CASCADE,
    label_id UUID NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
    PRIMARY KEY (task_id, label_id)
);

CREATE INDEX IF NOT EXISTS idx_task_labels_label_id ON task_labels(label_id);
