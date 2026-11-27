mod application;
mod bootstrap;
mod clients;
mod config;
mod database;
mod models;
mod provider;
mod repositories;
mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bootstrap::run().await
}
