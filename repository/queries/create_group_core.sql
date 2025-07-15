INSERT INTO "groups" ("id", "name", "created_at", "updated_at")
VALUES
    ($1, $2, NOW(), NOW())
RETURNING "id", "name", "created_at", "updated_at"
