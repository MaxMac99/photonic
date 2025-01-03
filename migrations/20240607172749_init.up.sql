CREATE TYPE medium_type_enum AS ENUM ('photo', 'video', 'live_photo', 'vector', 'sequence', 'gif', 'other');
CREATE TYPE medium_item_type_enum AS ENUM ('original', 'preview', 'edit', 'sidecar');
CREATE TYPE store_location_enum AS ENUM ('originals', 'cache', 'temp');

CREATE TABLE users (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(255),
    email VARCHAR(255),
    quota BIGINT NOT NULL,
    quota_used BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE media (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    medium_type medium_type_enum NOT NULL,
    leading_item_id uuid NOT NULL,
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
    last_saved TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP
);

CREATE TABLE medium_item_info (
    id uuid PRIMARY KEY REFERENCES medium_items(id),
    taken_at TIMESTAMP WITH TIME ZONE,
    taken_at_timezone INTEGER,
    width INTEGER,
    height INTEGER
);

ALTER TABLE media ADD CONSTRAINT leading_item_id_fk FOREIGN KEY (leading_item_id) REFERENCES medium_items(id) DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE media ADD CONSTRAINT leading_item_id_info_fk FOREIGN KEY (leading_item_id) REFERENCES medium_item_info(id) DEFERRABLE INITIALLY DEFERRED;

CREATE TABLE tags (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(100) NOT NULL
);

CREATE TABLE media_tags (
    medium_id uuid NOT NULL REFERENCES media(id) ON DELETE CASCADE,
    tag_id uuid NOT NULL REFERENCES tags(id),
    PRIMARY KEY (medium_id, tag_id)
);

CREATE TABLE locations (
    item_id uuid NOT NULL REFERENCES medium_items(id) DEFERRABLE INITIALLY DEFERRED,
    path VARCHAR(1024) NOT NULL,
    variant store_location_enum NOT NULL,
    PRIMARY KEY (item_id, variant)
);

CREATE FUNCTION update_user_quota_used()
    RETURNS TRIGGER
    LANGUAGE PLPGSQL
    AS
$$
DECLARE
    v_quota integer;
    v_quota_used integer;
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
