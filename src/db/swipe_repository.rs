use crate::app::types::IncomingLikeTargetKind;
use crate::db::profile_repository::ProfileRow;
use sqlx::FromRow;
use sqlx::PgPool;

pub async fn save_swipe(
    pool: &PgPool,
    from_user_id: i64,
    to_user_id: i64,
    is_like: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "
        INSERT INTO swipes (
            from_user_id,
            to_user_id,
            is_like,
            swiped_at,
            available_again_at
        )
        VALUES ($1, $2, $3, NOW(), NOW() + INTERVAL '7 days')
        ON CONFLICT (from_user_id, to_user_id)
        DO UPDATE SET
            is_like = EXCLUDED.is_like,
            swiped_at = EXCLUDED.swiped_at,
            available_again_at = EXCLUDED.available_again_at
        ",
        from_user_id,
        to_user_id,
        is_like
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn did_user_like_me(
    pool: &PgPool,
    my_user_id: i64,
    other_user_id: i64,
) -> Result<bool, sqlx::Error> {
    let liked = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM swipes
            WHERE from_user_id = $1
              AND to_user_id = $2
              AND is_like = TRUE
              AND available_again_at > NOW()
        ) AS "exists!"
        "#,
        other_user_id,
        my_user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(liked)
}

pub async fn get_incoming_like_for_user(
    pool: &PgPool,
    telegram_user_id: i64,
) -> Result<Option<ProfileRow>, sqlx::Error> {
    sqlx::query_as!(
        ProfileRow,
        r#"
        SELECT
            p.telegram_user_id,
            p.chat_id,
            p.username,
            p.language_code,
            p.name,
            p.gender,
            p.looking_for,
            p.age,
            p.location,
            p.description,
            p.photo_file_id,
            p.latitude,
            p.longitude
        FROM swipes s
        JOIN profiles p
          ON p.telegram_user_id = s.from_user_id
        WHERE s.to_user_id = $1
          AND s.is_like = TRUE
          AND s.available_again_at > NOW()
          AND NOT EXISTS (
              SELECT 1
              FROM swipes my_swipe
              WHERE my_swipe.from_user_id = $1
                AND my_swipe.to_user_id = s.from_user_id
                AND my_swipe.available_again_at > NOW()
          )
        ORDER BY s.swiped_at DESC
        LIMIT 1
        "#,
        telegram_user_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn count_incoming_like_targets_for_user(
    pool: &PgPool,
    user_id: i64,
) -> Result<i64, sqlx::Error> {
    let count = sqlx::query_scalar!(
        r#"
        SELECT COALESCE((
            (
                SELECT COUNT(*)
                FROM swipes s
                WHERE s.to_user_id = $1
                  AND s.is_like = TRUE
                  AND s.available_again_at > NOW()
                  AND NOT EXISTS (
                      SELECT 1
                      FROM swipes my_swipe
                      WHERE my_swipe.from_user_id = $1
                        AND my_swipe.to_user_id = s.from_user_id
                        AND my_swipe.available_again_at > NOW()
                  )
            )
            +
            (
                SELECT COUNT(*)
                FROM swipes s1
                JOIN swipes s2
                  ON s1.from_user_id = s2.to_user_id
                 AND s1.to_user_id = s2.from_user_id
                WHERE s1.from_user_id = $1
                  AND s1.is_like = TRUE
                  AND s1.available_again_at > NOW()
                  AND s2.is_like = TRUE
                  AND s2.available_again_at > NOW()
                  AND NOT EXISTS (
                      SELECT 1
                      FROM match_notifications mn
                      WHERE mn.user_id = $1
                        AND mn.other_user_id = s1.to_user_id
                  )
            )
        ), 0)::BIGINT AS "count!"
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(count)
}

#[derive(Debug, FromRow)]
struct PendingIncomingLikeTargetRow {
    telegram_user_id: i64,
    chat_id: i64,
    username: Option<String>,
    language_code: String,
    name: String,
    gender: String,
    looking_for: String,
    age: i16,
    location: String,
    description: String,
    photo_file_id: String,
    latitude: f64,
    longitude: f64,
    target_kind: String,
    pending_like_count: i64,
}

#[derive(Debug)]
pub struct PendingIncomingLikeTarget {
    pub profile_row: ProfileRow,
    pub target_kind: IncomingLikeTargetKind,
    pub pending_like_count: i64,
}
// take trosku nestastne no..
impl PendingIncomingLikeTargetRow {
    fn into_pending_target(self) -> PendingIncomingLikeTarget {
        let target_kind = match self.target_kind.as_str() {
            "incoming_like" => IncomingLikeTargetKind::IncomingLike,
            "pending_mutual_match" => IncomingLikeTargetKind::PendingMutualMatch,
            other => {
                log::warn!("Unexpected pending target kind from database: {other}");
                IncomingLikeTargetKind::IncomingLike
            }
        };

        PendingIncomingLikeTarget {
            profile_row: ProfileRow {
                telegram_user_id: self.telegram_user_id,
                chat_id: self.chat_id,
                username: self.username,
                language_code: self.language_code,
                name: self.name,
                gender: self.gender,
                looking_for: self.looking_for,
                age: self.age,
                location: self.location,
                description: self.description,
                photo_file_id: self.photo_file_id,
                latitude: self.latitude,
                longitude: self.longitude,
            },
            target_kind,
            pending_like_count: self.pending_like_count,
        }
    }
}

pub async fn get_pending_incoming_like_target_for_user(
    pool: &PgPool,
    user_id: i64,
) -> Result<Option<PendingIncomingLikeTarget>, sqlx::Error> {
    let row = sqlx::query_as!(
        PendingIncomingLikeTargetRow,
        r#"
        WITH pending_targets AS (
            SELECT
                p.telegram_user_id,
                p.chat_id,
                p.username,
                p.language_code,
                p.name,
                p.gender,
                p.looking_for,
                p.age,
                p.location,
                p.description,
                p.photo_file_id,
                p.latitude,
                p.longitude,
                'incoming_like' AS target_kind,
                s.swiped_at AS sort_time,
                0 AS priority
            FROM swipes s
            JOIN profiles p
              ON p.telegram_user_id = s.from_user_id
            WHERE s.to_user_id = $1
              AND s.is_like = TRUE
              AND s.available_again_at > NOW()
              AND NOT EXISTS (
                  SELECT 1
                  FROM swipes my_swipe
                  WHERE my_swipe.from_user_id = $1
                    AND my_swipe.to_user_id = s.from_user_id
                    AND my_swipe.available_again_at > NOW()
              )

            UNION ALL

            SELECT
                p.telegram_user_id,
                p.chat_id,
                p.username,
                p.language_code,
                p.name,
                p.gender,
                p.looking_for,
                p.age,
                p.location,
                p.description,
                p.photo_file_id,
                p.latitude,
                p.longitude,
                'pending_mutual_match' AS target_kind,
                s2.swiped_at AS sort_time,
                1 AS priority
            FROM swipes s1
            JOIN swipes s2
              ON s1.from_user_id = s2.to_user_id
             AND s1.to_user_id = s2.from_user_id
            JOIN profiles p
              ON p.telegram_user_id = s1.to_user_id
            WHERE s1.from_user_id = $1
              AND s1.is_like = TRUE
              AND s1.available_again_at > NOW()
              AND s2.is_like = TRUE
              AND s2.available_again_at > NOW()
              AND NOT EXISTS (
                  SELECT 1
                  FROM match_notifications mn
                  WHERE mn.user_id = $1
                    AND mn.other_user_id = s1.to_user_id
              )
        )
        SELECT
            telegram_user_id AS "telegram_user_id!",
            chat_id AS "chat_id!",
            username,
            language_code AS "language_code!",
            name AS "name!",
            gender AS "gender!",
            looking_for AS "looking_for!",
            age AS "age!",
            location AS "location!",
            description AS "description!",
            photo_file_id AS "photo_file_id!",
            latitude AS "latitude!",
            longitude AS "longitude!",
            target_kind AS "target_kind!",
            COUNT(*) OVER() AS "pending_like_count!"
        FROM pending_targets
        ORDER BY priority, sort_time DESC
        LIMIT 1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(PendingIncomingLikeTargetRow::into_pending_target))
}

pub async fn was_match_shown_to_user(
    pool: &PgPool,
    user_id: i64,
    other_user_id: i64,
) -> Result<bool, sqlx::Error> {
    let exists = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM match_notifications
            WHERE user_id = $1
              AND other_user_id = $2
        ) AS "exists!"
        "#,
        user_id,
        other_user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(exists)
}

pub async fn mark_match_shown_to_user(
    pool: &PgPool,
    user_id: i64,
    other_user_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO match_notifications (user_id, other_user_id)
        VALUES ($1, $2)
        ON CONFLICT (user_id, other_user_id) DO NOTHING
        "#, user_id, other_user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}
