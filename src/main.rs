mod clients;
mod database;
mod models;
mod provider;
mod repositories;
mod services;

use crate::{
    clients::{
        anilist_client::AniListClient,
        http_client::HttpClient,
        telegram_client::{SendMessageRequest, SendPhotoRequest, TelegramClient},
    },
    database::connection::create_pool,
    repositories::release_repository::ReleaseRepository,
    services::{
        anilist_service::AniListService, release_sync_service::ReleaseSyncService,
        telegram_service::TelegramService,
    },
};
use std::{error::Error, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenvy::dotenv().ok();

    let database_url = dotenvy::var("DATABASE_URL")?;

    let telegram_bot_token = dotenvy::var("TELEGRAM_BOT_TOKEN")?;
    let telegram_chat_id = dotenvy::var("TELEGRAM_CHAT_ID")?;
    let telegram_animelist_thread_id =
        dotenvy::var("TELEGRAM_ANIMELIST_THREAD_ID")?.parse::<u64>()?;

    let pool = create_pool(&database_url).await?;

    let http_client = Arc::new(HttpClient::new());

    let anilist_client = AniListClient::new(Arc::clone(&http_client));

    let telegram_client = TelegramClient::new(Arc::clone(&http_client), telegram_bot_token);

    let anilist_service = AniListService::new(anilist_client);

    let release_repository = ReleaseRepository::new(pool);

    let release_sync_service =
        ReleaseSyncService::new(Box::new(anilist_service), release_repository);

    let telegram_service = TelegramService::new(
        telegram_client,
        telegram_chat_id,
        telegram_animelist_thread_id,
    );

    let result = release_sync_service.sync().await?;

    telegram_service.notify(&result.events).await?;

    println!("Telegram notifications processed successfully!");

    println!("Total recibidos: {}", result.summary.total_received);
    println!("Nuevos: {}", result.summary.inserted);
    println!("Actualizados: {}", result.summary.updated);
    println!("Sin cambios: {}", result.summary.unchanged);

    println!("Eventos generados: {}", result.events.len());
    println!("{:#?}", result.events);

    Ok(())
}
