-- Issue #1: denormalized thumbhash on the media row for fast list queries.
-- Computed server-side from the tiny thumbnail when it is uploaded.
ALTER TABLE media ADD COLUMN thumbhash BYTEA;
