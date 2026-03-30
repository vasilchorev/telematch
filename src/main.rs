mod app;
mod bot;
mod db;
mod models;
mod geocoding;

pub use app::Lang;

#[tokio::main]
async fn main() {
    app::run().await;
}
