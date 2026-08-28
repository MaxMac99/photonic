-- Add down migration script here
DROP TRIGGER IF EXISTS update_user_quota_used ON medium_items;
DROP FUNCTION IF EXISTS update_user_quota_used;

DROP TABLE IF EXISTS metadata;
DROP TABLE IF EXISTS tasks;
DROP TABLE IF EXISTS locations;
DROP TABLE IF EXISTS media_tags;
DROP TABLE IF EXISTS medium_items;
DROP TABLE IF EXISTS media;
DROP TABLE IF EXISTS albums;
DROP TABLE IF EXISTS users;

DROP TYPE IF EXISTS orientation_enum;
DROP TYPE IF EXISTS task_status_enum;
DROP TYPE IF EXISTS store_location_enum;
DROP TYPE IF EXISTS medium_item_type_enum;
DROP TYPE IF EXISTS medium_type_enum;