-- Add migration script here
ALTER TABLE users
ALTER COLUMN id TYPE INT8 USING (id::INT8);