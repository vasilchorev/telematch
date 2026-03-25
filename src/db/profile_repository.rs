use crate::models::{Gender, Profile};
use sqlx::{FromRow, PgPool};

#[derive(Debug, Clone, FromRow)]
pub struct ProfileRow {
    pub telegram_user_id: i64,
    pub name: String,
    pub gender: String,
    pub looking_for: String,
    pub age: i16,
    pub location: String,
    pub description: String,
    pub photo_file_id: String,
}

impl ProfileRow {
    pub fn into_profile(self) -> Profile {
        Profile {
            name: Some(self.name),
            gender: Gender::from_db_code(&self.gender),
            looking_for: Gender::from_db_code(&self.looking_for),
            age: Some(self.age as u8),
            location: Some(self.location),
            description: Some(self.description),
            photo: Some(self.photo_file_id),
        }
    }
}
pub async fn save_profile(
    pool: &PgPool,
    telegram_user_id: i64,
    profile: &Profile,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO profiles (
            telegram_user_id,
            name,
            gender,
            looking_for,
            age,
            location,
            description,
            photo_file_id
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        ON CONFLICT (telegram_user_id)
        DO UPDATE SET
            name = EXCLUDED.name,
            gender = EXCLUDED.gender,
            looking_for = EXCLUDED.looking_for,
            age = EXCLUDED.age,
            location = EXCLUDED.location,
            description = EXCLUDED.description,
            photo_file_id = EXCLUDED.photo_file_id
        "#,
    )
        .bind(telegram_user_id)
        .bind(profile.name.as_deref().unwrap_or_default())
        .bind(
            profile
                .gender
                .as_ref()
                .map(|g| g.as_db_code())
                .unwrap_or_default(),
        )
        .bind(
            profile
                .looking_for
                .as_ref()
                .map(|g| g.as_db_code())
                .unwrap_or_default(),
        )
        .bind(profile.age.map(|a| a as i16).unwrap_or_default())
        .bind(profile.location.as_deref().unwrap_or_default())
        .bind(profile.description.as_deref().unwrap_or_default())
        .bind(profile.photo.as_deref().unwrap_or_default())
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
            p.name,
            p.gender,
            p.looking_for,
            p.age,
            p.location,
            p.description,
            p.photo_file_id
        FROM profiles p
        JOIN profiles me
            ON me.telegram_user_id = $1
        WHERE p.telegram_user_id <> me.telegram_user_id
          AND p.gender = me.looking_for
          AND p.looking_for = me.gender
        ORDER BY p.id DESC
        LIMIT 1
        "#
    )
        .bind(telegram_user_id)
        .fetch_optional(pool)
        .await
}