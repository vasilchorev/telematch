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
    sqlx::query(
        r#"
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
        "#,
    )
    .bind(from_user_id)
    .bind(to_user_id)
    .bind(is_like)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn did_user_like_me(
    pool: &PgPool,
    my_user_id: i64,
    other_user_id: i64,
) -> Result<bool, sqlx::Error> {
    let liked = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM swipes
            WHERE from_user_id = $1
              AND to_user_id = $2
              AND is_like = TRUE
              AND available_again_at > NOW()
        )
        "#,
    )
    .bind(other_user_id)
    .bind(my_user_id)
    .fetch_one(pool)
    .await?;

    Ok(liked)
}

pub async fn get_incoming_like_for_user(
    pool: &PgPool,
    telegram_user_id: i64,
) -> Result<Option<ProfileRow>, sqlx::Error> {
    sqlx::query_as::<_, ProfileRow>(
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
    )
    .bind(telegram_user_id)
    .fetch_optional(pool)
    .await
}

pub async fn count_incoming_like_targets_for_user(
    pool: &PgPool,
    user_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT
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
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

#[derive(Debug, FromRow)]
struct PendingIncomingLikeTargetRow {
    telegram_user_id: i64,
    chat_id: Option<i64>,
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
    let row = sqlx::query_as::<_, PendingIncomingLikeTargetRow>(
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
            telegram_user_id,
            chat_id,
            username,
            language_code,
            name,
            gender,
            looking_for,
            age,
            location,
            description,
            photo_file_id,
            latitude,
            longitude,
            target_kind,
            COUNT(*) OVER() AS pending_like_count
        FROM pending_targets
        ORDER BY priority, sort_time DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(PendingIncomingLikeTargetRow::into_pending_target))
}

pub async fn was_match_shown_to_user(
    pool: &PgPool,
    user_id: i64,
    other_user_id: i64,
) -> Result<bool, sqlx::Error> {
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM match_notifications
            WHERE user_id = $1
              AND other_user_id = $2
        )
        "#,
    )
    .bind(user_id)
    .bind(other_user_id)
    .fetch_one(pool)
    .await?;

    Ok(exists)
}

pub async fn mark_match_shown_to_user(
    pool: &PgPool,
    user_id: i64,
    other_user_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO match_notifications (user_id, other_user_id)
        VALUES ($1, $2)
        ON CONFLICT (user_id, other_user_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(other_user_id)
    .execute(pool)
    .await?;

    Ok(())
}
