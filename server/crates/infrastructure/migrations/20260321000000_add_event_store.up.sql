-- Event store: append-only log of all domain events
CREATE TABLE events (
    global_sequence BIGSERIAL PRIMARY KEY,
    event_type      VARCHAR(255) NOT NULL,
    payload         JSONB NOT NULL,
    event_id        UUID NOT NULL UNIQUE,
    occurred_at     TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_events_type ON events (event_type, global_sequence);

-- Stream membership: maps global events to per-aggregate streams
CREATE TABLE event_streams (
    global_sequence BIGINT NOT NULL REFERENCES events(global_sequence),
    stream_category VARCHAR(255) NOT NULL,
    stream_id       VARCHAR(255) NOT NULL,
    stream_version  BIGINT NOT NULL,
    PRIMARY KEY (stream_category, stream_id, stream_version),
    UNIQUE (global_sequence, stream_category, stream_id)
);

CREATE INDEX idx_event_streams_lookup ON event_streams (stream_category, stream_id, stream_version);

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
