CREATE TABLE alerts (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id      TEXT        NOT NULL,
    title        TEXT        NOT NULL,
    level        TEXT        NOT NULL,
    event        JSONB       NOT NULL,
    triggered_at TIMESTAMPTZ NOT NULL
);