-- Event store: append-only log of all domain events
CREATE TABLE events (
    global_sequence BIGSERIAL PRIMARY KEY,
    stream_id       VARCHAR(255) NOT NULL,      -- e.g., "Medium-{uuid}"
    version         BIGINT NOT NULL,            -- per-stream sequence (1-based)
    event_type      VARCHAR(255) NOT NULL,      -- e.g., "MediumCreated"
    payload         JSONB NOT NULL,
    event_id        UUID NOT NULL UNIQUE,
    occurred_at     TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE (stream_id, version)
);

CREATE INDEX idx_events_stream ON events (stream_id, version);
CREATE INDEX idx_events_type ON events (event_type, global_sequence);

-- Snapshots: periodic aggregate state snapshots for fast reconstitution
CREATE TABLE snapshots (
    stream_id    VARCHAR(255) PRIMARY KEY,
    version      BIGINT NOT NULL,
    payload      JSONB NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Projection checkpoints: tracks each projection's position in the event stream
CREATE TABLE projection_checkpoints (
    projection_name VARCHAR(255) PRIMARY KEY,
    last_global_sequence BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Idempotency: tracks which events have been processed by each consumer
CREATE TABLE processed_events (
    consumer_name VARCHAR(200) NOT NULL,
    event_id UUID NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (consumer_name, event_id)
);
