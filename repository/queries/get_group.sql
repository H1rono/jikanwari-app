SELECT
    "id", "name", "created_at", "updated_at",
    (SELECT array_agg("user_id") FROM group_members WHERE "group_id" = groups.id) AS "members!"
FROM "groups"
WHERE
    "id" = $1
