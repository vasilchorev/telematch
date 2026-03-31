mod app;
mod db;
mod domain;
mod services;
mod telegram;

pub use app::{AppResult, Language};

#[tokio::main]
async fn main() -> AppResult<()> {
    app::run().await
}
