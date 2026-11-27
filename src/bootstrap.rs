use std::sync::Arc;

use crate::{
    application::runner::ApplicationRunner,
    clients::{
        anilist_client::AniListClient, http_client::HttpClient, telegram_client::TelegramClient,
    },
    config::{Config, ConfigError},
    database::connection::create_pool,
    repositories::{
        notification_repository::NotificationRepository, release_repository::ReleaseRepository,
        season_repository::SeasonRepository,
    },
    services::{
        anilist_service::AniListService, notification_worker::TelegramWorker,
        release_sync_service::ReleaseSyncService, telegram_service::TelegramService,
    },
};

pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::load()?;
    let animelist_thread_id =
        config.telegram.topics.animelist.ok_or_else(|| {
            ConfigError::MissingVariable("TELEGRAM_ANIMELIST_THREAD_ID".to_string())
        })?;

    let pool = create_pool(&config.database_url).await?;

    let http_client = Arc::new(HttpClient::new());

    let anilist_client = AniListClient::new(Arc::clone(&http_client));
    let telegram_client = TelegramClient::new(Arc::clone(&http_client), config.telegram.bot_token);

    let anilist_service = AniListService::new(anilist_client);

    let telegram_service = TelegramService::new(
        telegram_client,
        config.telegram.chat_id,
        animelist_thread_id,
    );

    let season_repository = SeasonRepository::new();
    let notification_repository = NotificationRepository::new(pool.clone());

    let release_repository = ReleaseRepository::new(
        pool.clone(),
        season_repository,
        notification_repository.clone(),
    );

    let release_sync_service =
        ReleaseSyncService::new(Box::new(anilist_service), release_repository);

    let telegram_worker = TelegramWorker::new(notification_repository, telegram_service);

    let runner = ApplicationRunner::new(release_sync_service, telegram_worker);

    runner.run().await;
    Ok(())
}
