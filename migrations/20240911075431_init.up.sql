-- Add up migration script here

CREATE TABLE schema_def_view
(
    view_id text                        NOT NULL,
    version bigint CHECK (version >= 0) NOT NULL,
    payload json                        NOT NULL,
    PRIMARY KEY (view_id)
);
CREATE TABLE events
(
    aggregate_type text                         NOT NULL,
    aggregate_id   text                         NOT NULL,
    sequence       bigint CHECK (sequence >= 0) NOT NULL,
    event_type     text                         NOT NULL,
    event_version  text                         NOT NULL,
    payload        json                         NOT NULL,
    metadata       json                         NOT NULL,
    timestamp      timestamp with time zone DEFAULT (CURRENT_TIMESTAMP),
    PRIMARY KEY (aggregate_type, aggregate_id, sequence)
);

CREATE TABLE snapshots
(
    aggregate_type   text                                 NOT NULL,
    aggregate_id     text                                 NOT NULL,
    last_sequence    bigint CHECK (last_sequence >= 0)    NOT NULL,
    current_snapshot bigint CHECK (current_snapshot >= 0) NOT NULL,
    payload          json                                 NOT NULL,
    timestamp        timestamp with time zone DEFAULT (CURRENT_TIMESTAMP),
    PRIMARY KEY (aggregate_type, aggregate_id, last_sequence)
);


CREATE TABLE pekko_projection_offset_store
(
    projection_name VARCHAR(255) NOT NULL, -- This column stores the name of the projection. Each projection is uniquely identified by this name.
    projection_key  VARCHAR(255) NOT NULL, -- The key for a specific projection instance (often associated with a shard or partition).
    current_offset  VARCHAR(255) NOT NULL, -- This stores the current offset, which could be a sequence number or a time-based offset.
    manifest        VARCHAR(4)   NOT NULL, -- The manifest is a 4-character string that identifies the type of the offset. (e.g., "SEQ" for a sequence-based offset).
    mergeable       BOOLEAN      NOT NULL, -- This flag indicates whether the offset is mergeable with other offsets.
    last_updated    BIGINT       NOT NULL, -- A timestamp indicating when the offset was last updated.
    PRIMARY KEY (projection_name, projection_key)
);

-- CREATE INDEX projection_name_index ON pekko_projection_offset_store (projection_name);

CREATE TABLE pekko_projection_management
(
    projection_name VARCHAR(255) NOT NULL, -- This column stores the name of the projection. Each projection is uniquely identified by this name.
    projection_key  VARCHAR(255) NOT NULL, -- The key for a specific projection instance (often associated with a shard or partition).
    paused          BOOLEAN      NOT NULL, -- This flag indicates whether the projection is paused.
    last_updated    BIGINT       NOT NULL, -- A timestamp indicating when the projection was last updated.
    PRIMARY KEY (projection_name, projection_key)
);



CREATE TABLE event_journal
(
    ordering           BIGSERIAL,
    persistence_id     VARCHAR(255)          NOT NULL,
    sequence_number    BIGINT                NOT NULL,
    deleted            BOOLEAN DEFAULT FALSE NOT NULL,
    writer             VARCHAR(255)          NOT NULL,
    write_timestamp    BIGINT,
    adapter_manifest   VARCHAR(255),
    event_ser_id       INTEGER               NOT NULL,
    event_ser_manifest VARCHAR(255)          NOT NULL,
    event_payload      BYTEA                 NOT NULL,
    meta_ser_id        INTEGER,
    meta_ser_manifest  VARCHAR(255),
    meta_payload       BYTEA,

    PRIMARY KEY (persistence_id, sequence_number)
);

CREATE UNIQUE INDEX event_journal_ordering_idx ON event_journal (ordering);

CREATE TABLE event_tag
(
    event_id BIGINT,
    tag      VARCHAR(256),
    PRIMARY KEY (event_id, tag),
    CONSTRAINT fk_event_journal
        FOREIGN KEY (event_id)
            REFERENCES event_journal (ordering)
            ON DELETE CASCADE
);

CREATE TABLE snapshot
(
    persistence_id        VARCHAR(255) NOT NULL,
    sequence_number       BIGINT       NOT NULL,
    created               BIGINT       NOT NULL,
    snapshot_ser_id       INTEGER      NOT NULL,
    snapshot_ser_manifest VARCHAR(255) NOT NULL,
    snapshot_payload      BYTEA        NOT NULL,
    meta_ser_id           INTEGER,
    meta_ser_manifest     VARCHAR(255),
    meta_payload          BYTEA,
    PRIMARY KEY (persistence_id, sequence_number)
);

CREATE TABLE durable_state
(
    global_offset         BIGSERIAL,
    persistence_id        VARCHAR(255) NOT NULL,
    revision              BIGINT       NOT NULL,
    state_payload         BYTEA        NOT NULL,
    state_serial_id       INTEGER      NOT NULL,
    state_serial_manifest VARCHAR(255),
    tag                   VARCHAR,
    state_timestamp       BIGINT       NOT NULL,
    PRIMARY KEY (persistence_id)
);
-- CREATE INDEX CONCURRENTLY state_tag_idx on durable_state (tag);
-- CREATE INDEX CONCURRENTLY state_global_offset_idx on durable_state (global_offset);