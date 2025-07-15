-- Add up migration script here

CREATE TABLE IF NOT EXISTS users (
    "id" uuid PRIMARY KEY,
    "name" VARCHAR NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS groups (
    "id" uuid PRIMARY KEY,
    "name" VARCHAR NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS group_members (
    "group_id" uuid REFERENCES groups(id) ON DELETE CASCADE,
    "user_id" uuid REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY ("group_id", "user_id")
);
