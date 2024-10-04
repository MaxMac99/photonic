CREATE TABLE users (
    id uuid DEFAULT gen_random_uuid(),
    username VARCHAR(255),
    email VARCHAR(255),
    quota BIGINT NOT NULL,
    quota_used BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE TYPE medium_type_enum AS ENUM ('photo', 'video', 'live_photo', 'vector', 'sequence', 'gif', 'other');
CREATE TABLE media (
    id uuid DEFAULT gen_random_uuid(),
    owner_id uuid NOT NULL,
    medium_type medium_type_enum NOT NULL,
    deleted_at TIMESTAMP,
    PRIMARY KEY (id),
    CONSTRAINT fk_user FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE tags (
    id uuid DEFAULT gen_random_uuid(),
    title VARCHAR(100) NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE media_tags (
    medium_id uuid,
    tag_id uuid,
    PRIMARY KEY (medium_id, tag_id),
    CONSTRAINT fk_medium FOREIGN KEY (medium_id) REFERENCES media(id) ON DELETE CASCADE,
    CONSTRAINT fk_tag FOREIGN KEY (tag_id) REFERENCES tags(id)
);

CREATE TYPE store_location_enum AS ENUM ('originals', 'cache', 'temp');
CREATE TYPE medium_item_type_enum AS ENUM ('original', 'preview', 'edit');
CREATE TABLE medium_items (
    id uuid DEFAULT gen_random_uuid(),
    medium_id uuid NOT NULL,
    medium_item_type medium_item_type_enum NOT NULL,
    mime VARCHAR(100) NOT NULL,
    filename VARCHAR(255) NOT NULL,
    path VARCHAR(1024) NOT NULL,
    size BIGINT NOT NULL DEFAULT 0,
    variant store_location_enum NOT NULL,
    priority INTEGER NOT NULL DEFAULT 1,
    taken_at TIMESTAMP WITH TIME ZONE,
    taken_at_timezone INTEGER,
    last_saved TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP,
    width INTEGER,
    height INTEGER,
    PRIMARY KEY (id),
    CONSTRAINT fk_medium FOREIGN KEY (medium_id) REFERENCES media(id) ON DELETE CASCADE
);

CREATE TABLE sidecars (
    id uuid DEFAULT gen_random_uuid(),
    medium_id uuid NOT NULL,
    mime VARCHAR(100) NOT NULL,
    filename VARCHAR(255) NOT NULL,
    path VARCHAR(1024) NOT NULL,
    size BIGINT NOT NULL DEFAULT 0,
    location store_location_enum NOT NULL,
    priority INTEGER NOT NULL DEFAULT 1,
    last_saved TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP,
    PRIMARY KEY (id),
    CONSTRAINT fk_medium FOREIGN KEY (medium_id) REFERENCES media(id) ON DELETE CASCADE
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

CREATE TRIGGER update_user_quota_used
    AFTER INSERT OR UPDATE OR DELETE
    ON sidecars
    FOR EACH ROW
    EXECUTE PROCEDURE update_user_quota_used();
