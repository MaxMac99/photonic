-- Your SQL goes here
CREATE TABLE users (
    id uuid DEFAULT gen_random_uuid(),
    username VARCHAR(255),
    email VARCHAR(255),
    given_name VARCHAR(255),
    quota BIGINT NOT NULL,
    quota_used BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE TABLE albums (
    id uuid DEFAULT gen_random_uuid(),
    owner_id uuid NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    title_medium uuid,
    PRIMARY KEY (id),
    CONSTRAINT fk_user FOREIGN KEY (owner_id) REFERENCES users(id)
);

CREATE TYPE medium_type_enum AS ENUM ('photo', 'video', 'live_photo', 'vector', 'sequence', 'gif', 'other');
CREATE TABLE media (
    id uuid DEFAULT gen_random_uuid(),
    owner_id uuid NOT NULL,
    medium_type medium_type_enum NOT NULL,
    album_id uuid,
    deleted_at TIMESTAMP,
    PRIMARY KEY (id),
    CONSTRAINT fk_user FOREIGN KEY (owner_id) REFERENCES users(id),
    CONSTRAINT fk_album FOREIGN KEY (album_id) REFERENCES albums(id)
);

ALTER TABLE albums ADD CONSTRAINT fk_title FOREIGN KEY (title_medium) REFERENCES media(id);

CREATE TABLE tags (
    id uuid DEFAULT gen_random_uuid(),
    title VARCHAR(100) NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE media_tags (
    medium_id uuid,
    tag_id uuid,
    PRIMARY KEY (medium_id, tag_id),
    CONSTRAINT fk_medium FOREIGN KEY (medium_id) REFERENCES media(id),
    CONSTRAINT fk_tag FOREIGN KEY (tag_id) REFERENCES tags(id)
);

CREATE TYPE store_location_enum AS ENUM ('Originals', 'cache');
CREATE TYPE medium_item_type_enum AS ENUM ('original', 'preview', 'edit');
CREATE TABLE medium_items (
    id uuid DEFAULT gen_random_uuid(),
    medium_id uuid NOT NULL,
    medium_item_type medium_item_type_enum NOT NULL,
    mime VARCHAR(100) NOT NULL,
    filename VARCHAR(255) NOT NULL,
    path VARCHAR(1024) NOT NULL,
    size BIGINT NOT NULL DEFAULT 0,
    location store_location_enum NOT NULL,
    priority INTEGER NOT NULL DEFAULT 1,
    timezone INTEGER NOT NULL DEFAULT 0,
    taken_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_saved TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    PRIMARY KEY (id),
    CONSTRAINT fk_medium FOREIGN KEY (medium_id) REFERENCES media(id)
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
    CONSTRAINT fk_medium FOREIGN KEY (medium_id) REFERENCES media(id)
);

CREATE FUNCTION update_user_quota_used()
    RETURNS TRIGGER
    LANGUAGE PLPGSQL
    AS
$$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        UPDATE users u
        SET quota_used = quota_used - OLD.size
        FROM media m
        WHERE u.id = m.owner_id AND m.id = OLD.medium_id;
    ELSIF (TG_OP = 'UPDATE') THEN
        UPDATE users u
        SET quota_used = quota_used + (NEW.size - OLD.size)
        FROM media m
        WHERE u.id = m.owner_id AND m.id = OLD.medium_id;
    ELSIF (TG_OP = 'INSERT') THEN
        UPDATE users u
        SET quota_used = quota_used + NEW.size
        FROM media m
        WHERE u.id = m.owner_id AND m.id = NEW.medium_id;
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
