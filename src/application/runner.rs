use std::time::Duration;

use tokio::time::{MissedTickBehavior, interval};

use crate::services::{
    notification_worker::TelegramWorker, release_sync_service::ReleaseSyncService,
};

pub struct ApplicationRunner {
    release_sync_service: ReleaseSyncService,
    telegram_worker: TelegramWorker,
}

impl ApplicationRunner {
    pub fn new(release_sync_service: ReleaseSyncService, telegram_worker: TelegramWorker) -> Self {
        Self {
            release_sync_service,
            telegram_worker,
        }
    }

    pub async fn run(&self) {
        tokio::join!(self.run_sync_loop(), self.run_notification_loop(),);
    }

    async fn run_sync_loop(&self) {
        let mut sync_interval = interval(Duration::from_secs(6 * 60 * 60));
        sync_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            sync_interval.tick().await;

            match self.release_sync_service.sync().await {
                Ok(result) => {
                    println!("Release synchronization completed!");

                    println!("Total received: {}", result.summary.total_received);
                    println!("Inserted: {}", result.summary.inserted);
                    println!("Updated: {}", result.summary.updated);
                    println!("Unchanged: {}", result.summary.unchanged);
                    println!("Events generated: {}", result.events.len());
                }

                Err(error) => {
                    eprintln!("AniList synchronization failed: {}", error);
                }
            }
        }
    }

    async fn run_notification_loop(&self) {
        let mut notification_interval = interval(Duration::from_secs(30));

        notification_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            notification_interval.tick().await;

            if let Err(error) = self.telegram_worker.process_pending().await {
                eprintln!("Notification processing failed: {}", error);
            }
        }
    }
}
