CREATE TABLE event_streams (
    global_sequence BIGINT NOT NULL REFERENCES events(global_sequence),
    stream_category VARCHAR(255) NOT NULL,
    stream_id VARCHAR(255) NOT NULL,
    stream_version BIGINT NOT NULL,
    PRIMARY KEY (stream_category, stream_id, stream_version),
    UNIQUE (global_sequence, stream_category, stream_id)
);

CREATE INDEX idx_event_streams_lookup ON event_streams (stream_category, stream_id, stream_version);
