-- check group and member existence
WITH g AS (
    SELECT "id" FROM "groups" WHERE "id" = $1
),
m AS (
    SELECT $1::uuid AS "group_id", "id" AS "user_id"
    FROM "users"
    WHERE "id" = ANY ($2::uuid[])
)
SELECT COUNT(g."id") AS "group_count!", COUNT(m."user_id") AS "member_count!"
FROM g JOIN m ON g."id" = m."group_id"
GROUP BY g."id"
