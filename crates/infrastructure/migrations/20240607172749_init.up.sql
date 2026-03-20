CREATE TYPE medium_type_enum AS ENUM ('photo', 'video', 'live_photo', 'vector', 'sequence', 'gif', 'other');
CREATE TYPE medium_item_type_enum AS ENUM ('original', 'preview', 'edit', 'sidecar');
CREATE TYPE store_location_enum AS ENUM ('originals', 'cache', 'temp');
CREATE TYPE task_type_enum AS ENUM ('metadata_extraction', 'temp_cleanup');
CREATE TYPE task_status_enum AS ENUM ('pending', 'in_progress', 'completed', 'failed');
CREATE TYPE orientation_enum AS ENUM (
    'normal', 'mirror_horizontal', 'rotate180', 'mirror_vertical',
    'mirror_horizontal_rotate270_cw', 'rotate90_cw',
    'mirror_horizontal_rotate90_cw', 'rotate270_cw'
);

CREATE TABLE users (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    version BIGINT NOT NULL DEFAULT 1,
    username VARCHAR(255),
    email VARCHAR(255),
    quota BIGINT NOT NULL,
    quota_used BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE albums (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP
);

CREATE TABLE media (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    medium_type medium_type_enum NOT NULL,
    leading_item_id uuid NOT NULL,
    album_id uuid REFERENCES albums(id) ON DELETE SET NULL,
    taken_at TIMESTAMP WITH TIME ZONE,
    taken_at_timezone INTEGER,
    camera_make VARCHAR(100),
    camera_model VARCHAR(100),
    gps_latitude DOUBLE PRECISION,
    gps_longitude DOUBLE PRECISION,
    gps_altitude DOUBLE PRECISION,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP
);

CREATE TABLE medium_items (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    medium_id uuid NOT NULL REFERENCES media(id) ON DELETE CASCADE,
    medium_item_type medium_item_type_enum NOT NULL,
    mime VARCHAR(100) NOT NULL,
    filename VARCHAR(255) NOT NULL,
    size BIGINT NOT NULL DEFAULT 0,
    priority INTEGER NOT NULL DEFAULT 1,
    width INTEGER,
    height INTEGER,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP
);

ALTER TABLE media ADD CONSTRAINT leading_item_id_fk FOREIGN KEY (leading_item_id) REFERENCES medium_items(id) DEFERRABLE INITIALLY DEFERRED;

CREATE TABLE media_tags (
    medium_id uuid NOT NULL REFERENCES media(id) ON DELETE CASCADE,
    tag_title VARCHAR(100) NOT NULL,
    PRIMARY KEY (medium_id, tag_title)
);

CREATE TABLE locations (
    item_id uuid NOT NULL REFERENCES medium_items(id) DEFERRABLE INITIALLY DEFERRED,
    path VARCHAR(1024) NOT NULL,
    variant store_location_enum NOT NULL,
    PRIMARY KEY (item_id, variant)
);

CREATE TABLE tasks (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    task_type task_type_enum NOT NULL,
    reference_id uuid NOT NULL,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status task_status_enum NOT NULL DEFAULT 'pending',
    error TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP,
    completed_at TIMESTAMP
);

CREATE INDEX idx_tasks_reference ON tasks(reference_id, task_type);

CREATE TABLE metadata (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    medium_id uuid NOT NULL REFERENCES media(id) ON DELETE CASCADE,
    extracted_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- File info
    mime_type VARCHAR(100) NOT NULL,
    file_size BIGINT NOT NULL,
    file_modified_at TIMESTAMP WITH TIME ZONE,
    -- Camera info
    camera_make VARCHAR(100),
    camera_model VARCHAR(100),
    capture_date TIMESTAMP WITH TIME ZONE,
    modified_date TIMESTAMP WITH TIME ZONE,
    lens_make VARCHAR(100),
    lens_model VARCHAR(100),
    exposure_time DOUBLE PRECISION,
    f_number DOUBLE PRECISION,
    iso SMALLINT,
    focal_length DOUBLE PRECISION,
    flash BOOLEAN,
    -- Location
    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,
    altitude DOUBLE PRECISION,
    direction DOUBLE PRECISION,
    horizontal_position_error DOUBLE PRECISION,
    -- Technical
    width INTEGER,
    height INTEGER,
    orientation orientation_enum,
    -- Additional (flexible)
    additional JSONB DEFAULT '{}'
);

CREATE UNIQUE INDEX idx_metadata_medium_id ON metadata(medium_id);

-- Indexes for GPS queries (denormalized for fast map view)
CREATE INDEX idx_media_gps_location ON media (gps_latitude, gps_longitude)
    WHERE gps_latitude IS NOT NULL;

COMMENT ON COLUMN media.gps_latitude IS 'Denormalized from metadata for efficient map view queries';
COMMENT ON COLUMN media.gps_longitude IS 'Denormalized from metadata for efficient map view queries';
COMMENT ON COLUMN media.gps_altitude IS 'Denormalized from metadata - GPS altitude in meters';

CREATE FUNCTION update_user_quota_used()
    RETURNS TRIGGER
    LANGUAGE PLPGSQL
    AS
$$
DECLARE
    v_quota bigint;
    v_quota_used bigint;
BEGIN
    IF (TG_OP = 'DELETE') THEN
        UPDATE users u
        SET quota_used = quota_used - OLD.size
        FROM media m
        WHERE u.id = m.owner_id AND m.id = OLD.medium_id;

        SELECT u.quota, u.quota_used
        INTO STRICT v_quota, v_quota_used
        FROM users u
        JOIN media m
        ON m.owner_id = u.id
        WHERE m.id = OLD.medium_id;
    ELSIF (TG_OP = 'UPDATE') THEN
        UPDATE users u
        SET quota_used = quota_used + (NEW.size - OLD.size)
        FROM media m
        WHERE u.id = m.owner_id AND m.id = OLD.medium_id;

        SELECT u.quota, u.quota_used
        INTO STRICT v_quota, v_quota_used
        FROM users u
        JOIN media m
        ON m.owner_id = u.id
        WHERE m.id = OLD.medium_id;
    ELSIF (TG_OP = 'INSERT') THEN
        UPDATE users u
        SET quota_used = quota_used + NEW.size
        FROM media m
        WHERE u.id = m.owner_id AND m.id = NEW.medium_id;

        SELECT u.quota, u.quota_used
        INTO STRICT v_quota, v_quota_used
        FROM users u
        JOIN media m
        ON m.owner_id = u.id
        WHERE m.id = NEW.medium_id;
    END IF;

    IF v_quota_used > v_quota THEN
        RAISE EXCEPTION 'quota exceeded';
    END IF;

    RETURN NULL;
END;
$$;

CREATE TRIGGER update_user_quota_used
    AFTER INSERT OR UPDATE OR DELETE
    ON medium_items
    FOR EACH ROW
    EXECUTE PROCEDURE update_user_quota_used();
