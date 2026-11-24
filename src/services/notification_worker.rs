use crate::clients::telegram_client::TelegramError;
use crate::models::notification::Notification;
use crate::models::release_event::ReleaseEvent;
use crate::repositories::notification_repository::{
    NotificationRepository, NotificationRepositoryError,
};
use crate::services::telegram_service::TelegramService;

pub struct TelegramWorker {
    notification_repository: NotificationRepository,
    telegram_service: TelegramService,
}

impl TelegramWorker {
    pub fn new(
        notification_repository: NotificationRepository,
        telegram_service: TelegramService,
    ) -> Self {
        Self {
            notification_repository,
            telegram_service,
        }
    }

    pub async fn process_pending(&self) -> Result<(), TelegramWorkerError> {
        // Recover notifications abandoned by previous workers
        self.notification_repository.recover_abandoned(5).await?;

        // Claim pending notifications
        let notifications = self.notification_repository.claim_pending(10).await?;

        // Process notifications sequentially
        for notification in notifications {
            self.process_notification(notification).await?;
        }

        Ok(())
    }

    async fn process_notification(
        &self,
        notification: Notification,
    ) -> Result<(), TelegramWorkerError> {
        let notification_id = notification
            .id
            .ok_or(TelegramWorkerError::MissingNotificationId)?;

        // Deserialize the stored event
        let event: ReleaseEvent = serde_json::from_value(notification.payload)?;

        // Send the notification through Telegram
        match self.telegram_service.notify(&[event]).await {
            Ok(()) => {
                self.notification_repository
                    .mark_as_sent(notification_id)
                    .await?;
            }

            Err(error) => {
                let error_message = error.to_string();

                match &error {
                    TelegramError::TelegramApi {
                        error_code: Some(429),
                        retry_after,
                        ..
                    } => {
                        let retry_seconds = retry_after.unwrap_or(30);

                        self.notification_repository
                            .schedule_retry(notification_id, retry_seconds as i64, &error_message)
                            .await?;
                    }

                    TelegramError::Http(_) => {
                        self.notification_repository
                            .schedule_retry(notification_id, 60, &error_message)
                            .await?;
                    }

                    TelegramError::HttpStatus { status, .. } if *status >= 500 => {
                        self.notification_repository
                            .schedule_retry(notification_id, 60, &error_message)
                            .await?;
                    }

                    _ => {
                        self.notification_repository
                            .mark_as_failed(notification_id, &error_message)
                            .await?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum TelegramWorkerError {
    Repository(NotificationRepositoryError),
    Serialization(serde_json::Error),
    MissingNotificationId,
}

impl From<NotificationRepositoryError> for TelegramWorkerError {
    fn from(error: NotificationRepositoryError) -> Self {
        Self::Repository(error)
    }
}

impl From<serde_json::Error> for TelegramWorkerError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}

impl std::fmt::Display for TelegramWorkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Repository(error) => {
                write!(f, "Notification repository error: {:?}", error)
            }
            Self::Serialization(error) => {
                write!(f, "Notification serialization error: {}", error)
            }
            Self::MissingNotificationId => {
                write!(f, "Notification is missing its database ID")
            }
        }
    }
}

impl std::error::Error for TelegramWorkerError {}
