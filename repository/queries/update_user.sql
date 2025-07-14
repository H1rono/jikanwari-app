UPDATE ONLY "users"
SET "name" = $2,
    "updated_at" = NOW()
WHERE "id" = $1
RETURNING "id", "name", "created_at", "updated_at"
