use crate::models::{CompleteProfile, Gender, Profile};
use sqlx::{FromRow, PgPool};

#[derive(Debug, Clone, FromRow)]
pub struct ProfileRow {
    pub telegram_user_id: i64,
    pub chat_id: Option<i64>,
    pub username: Option<String>,
    pub language_code: String,
    pub name: String,
    pub gender: String,
    pub looking_for: String,
    pub age: i16,
    pub location: String,
    pub description: String,
    pub photo_file_id: String,
    pub latitude: f64,
    pub longitude: f64,
}

impl ProfileRow {
    pub fn to_profile(&self) -> Profile {
        Profile {
            telegram_user_id: Some(self.telegram_user_id),
            chat_id: self.chat_id,
            username: self.username.clone(),
            language_code: Some(self.language_code.clone()),
            name: Some(self.name.clone()),
            gender: Gender::from_db_code(&self.gender),
            looking_for: Gender::from_db_code(&self.looking_for),
            age: Some(self.age as u8),
            location: Some(self.location.clone()),
            description: Some(self.description.clone()),
            photo: Some(self.photo_file_id.clone()),
            latitude: Some(self.latitude.clone()),
            longitude: Some(self.longitude.clone()),
        }
    }
}

pub async fn save_profile(pool: &PgPool, profile: &CompleteProfile) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO profiles (
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
            longitude
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,$12,$13)
        ON CONFLICT (telegram_user_id)
        DO UPDATE SET
            chat_id = EXCLUDED.chat_id,
            username = EXCLUDED.username,
            language_code = EXCLUDED.language_code,
            name = EXCLUDED.name,
            gender = EXCLUDED.gender,
            looking_for = EXCLUDED.looking_for,
            age = EXCLUDED.age,
            location = EXCLUDED.location,
            description = EXCLUDED.description,
            photo_file_id = EXCLUDED.photo_file_id,
            latitude = EXCLUDED.latitude,
            longitude = EXCLUDED.longitude
        "#,
    )
    .bind(profile.telegram_user_id)
    .bind(profile.chat_id)
    .bind(profile.username.as_deref())
    .bind(&profile.language_code)
    .bind(&profile.name)
    .bind(profile.gender.as_db_code())
    .bind(profile.looking_for.as_db_code())
    .bind(profile.age as i16)
    .bind(&profile.location)
    .bind(&profile.description)
    .bind(&profile.photo)
    .bind(profile.latitude)
    .bind(profile.longitude)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_next_profile_for_user(
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
        FROM profiles p
        JOIN profiles me
            ON me.telegram_user_id = $1
        WHERE p.telegram_user_id <> me.telegram_user_id
          AND p.gender = me.looking_for
          AND p.looking_for = me.gender
          AND p.is_active = TRUE
          AND me.is_active = TRUE
          AND p.latitude IS NOT NULL
          AND p.longitude IS NOT NULL
          AND NOT EXISTS (
              SELECT 1
              FROM swipes s
              WHERE s.from_user_id = me.telegram_user_id
                AND s.to_user_id = p.telegram_user_id
                AND s.available_again_at > NOW()
          )
        ORDER BY
          CASE
              WHEN me.latitude IS NULL OR me.longitude IS NULL THEN NULL
              ELSE 6371.0 * ACOS(
                  LEAST(
                      1.0,
                      GREATEST(
                          -1.0,
                          COS(RADIANS(me.latitude)) * COS(RADIANS(p.latitude)) *
                          COS(RADIANS(p.longitude) - RADIANS(me.longitude)) +
                          SIN(RADIANS(me.latitude)) * SIN(RADIANS(p.latitude))
                      )
                  )
              )
          END ASC NULLS LAST,
          p.id DESC
        LIMIT 1
        "#,
    )
    .bind(telegram_user_id)
    .fetch_optional(pool)
    .await
}

pub async fn get_profile_by_user_id(
    pool: &PgPool,
    telegram_user_id: i64,
) -> Result<Option<ProfileRow>, sqlx::Error> {
    sqlx::query_as::<_, ProfileRow>(
        r#"
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
            longitude
        FROM profiles
        WHERE telegram_user_id = $1
        LIMIT 1
        "#,
    )
    .bind(telegram_user_id)
    .fetch_optional(pool)
    .await
}

pub async fn deactivate_profile(pool: &PgPool, telegram_user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE profiles
        SET is_active = FALSE
        WHERE telegram_user_id = $1
        "#,
    )
    .bind(telegram_user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn activate_profile(pool: &PgPool, telegram_user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE profiles
        SET is_active = TRUE
        WHERE telegram_user_id = $1
        "#,
    )
    .bind(telegram_user_id)
    .execute(pool)
    .await?;

    Ok(())
}
