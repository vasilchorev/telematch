pub mod profile_repository;
pub mod swipe_repository;

use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

pub async fn connect_db(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await?;

    sqlx::migrate!("src/db/migrations").run(&pool).await?;

    Ok(pool)
}
