SELECT "id", "name", "created_at", "updated_at"
FROM "users"
WHERE "id" = $1
LIMIT 1
