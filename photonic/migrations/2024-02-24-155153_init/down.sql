-- This file should undo anything in `up.sql`
DROP TRIGGER IF EXISTS update_user_quota_used ON medium_items;
DROP TRIGGER IF EXISTS update_user_quota_used ON sidecars;
DROP FUNCTION IF EXISTS update_user_quota_used;

DROP TABLE IF EXISTS sidecars;
DROP TABLE IF EXISTS medium_items;
DROP TYPE IF EXISTS medium_item_type_enum;
DROP TABLE IF EXISTS media_tags;
DROP TABLE IF EXISTS tags;
ALTER TABLE IF EXISTS albums DROP CONSTRAINT fk_title;
DROP TABLE IF EXISTS media;
DROP TYPE IF EXISTS medium_type_enum;
DROP TYPE IF EXISTS store_location_enum;
DROP TABLE IF EXISTS albums;
DROP TABLE IF EXISTS users;
