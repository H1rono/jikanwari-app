WITH g AS (
    UPDATE ONLY "groups"
    SET "name" = $2,
        "updated_at" = NOW()
    WHERE
        "id" = $1
    RETURNING "id", "name", "created_at", "updated_at"
)
SELECT
    g."id", g."name", g."created_at", g."updated_at",
    (SELECT array_agg("user_id") FROM group_members WHERE "group_id" = g.id) AS "members!"
FROM g
