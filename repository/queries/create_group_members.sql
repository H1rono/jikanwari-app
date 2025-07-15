INSERT INTO "group_members" ("group_id", "user_id")
(
    SELECT $1 AS "group_id", "user_id"
    FROM unnest($2::uuid[]) AS "user_id"
)
RETURNING "group_id", "user_id"
