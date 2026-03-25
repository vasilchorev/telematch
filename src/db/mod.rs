pub mod profile_repository;

use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

pub async fn connect_db(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await?;

    sqlx::migrate!("src/migrations").run(&pool).await?;

    Ok(pool)
}