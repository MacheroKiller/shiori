use sqlx::{PgPool, Postgres, Transaction};

use crate::models::{
    notification::{Notification, NotificationStatus},
    release_event::ReleaseEvent,
};

#[derive(Debug, sqlx::FromRow)]
struct StoredNotification {
    id: i64,
    event_type: String,
    payload: serde_json::Value,
    status: String,
    attempts: i32,
    next_attempt_at: chrono::DateTime<chrono::Utc>,
    last_error: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    sent_at: Option<chrono::DateTime<chrono::Utc>>,
    locked_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Clone)]
pub struct NotificationRepository {
    pool: PgPool,
}

impl NotificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn enqueue_in_transaction(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        event: &ReleaseEvent,
    ) -> Result<(), NotificationRepositoryError> {
        let event_type = match event {
            ReleaseEvent::NewRelease(_) => "NewRelease",
            ReleaseEvent::ReleaseUpdated { .. } => "ReleaseUpdated",
        };

        let payload = serde_json::to_value(event)?;

        sqlx::query(
            r#"
        INSERT INTO notifications (
            event_type,
            payload,
            status
        )
        VALUES ($1, $2, $3)
        "#,
        )
        .bind(event_type)
        .bind(payload)
        .bind(NotificationStatus::Pending.as_str())
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
    pub async fn claim_pending(
        &self,
        limit: i64,
    ) -> Result<Vec<Notification>, NotificationRepositoryError> {
        let mut transaction = self.pool.begin().await?;

        let notifications = sqlx::query_as::<_, StoredNotification>(
            r#"
        UPDATE notifications
        SET status = 'Processing',
         locked_at = NOW()
        WHERE id IN (
            SELECT id
            FROM notifications
            WHERE status = 'Pending'
              AND next_attempt_at <= NOW()
            ORDER BY next_attempt_at, created_at
            LIMIT $1
            FOR UPDATE SKIP LOCKED
        )
        RETURNING
            id,
            event_type,
            payload,
            status,
            attempts,
            next_attempt_at,
            locked_at,
            last_error,
            created_at,
            sent_at
        "#,
        )
        .bind(limit)
        .fetch_all(&mut *transaction)
        .await?;

        transaction.commit().await?;

        let notifications = notifications
            .into_iter()
            .map(Notification::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(notifications)
    }
    pub async fn mark_as_sent(
        &self,
        notification_id: i64,
    ) -> Result<(), NotificationRepositoryError> {
        sqlx::query(
            r#"
        UPDATE notifications
        SET
            status = 'Sent',
            sent_at = NOW(),
            last_error = NULL,
            locked_at = NULL
        WHERE id = $1
        "#,
        )
        .bind(notification_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn schedule_retry(
        &self,
        notification_id: i64,
        retry_after: i64,
        error: &str,
    ) -> Result<(), NotificationRepositoryError> {
        sqlx::query(
            r#"
        UPDATE notifications
        SET
            status = 'Pending',
            attempts = attempts + 1,
            next_attempt_at = NOW() + ($1 * INTERVAL '1 second'),
            locked_at = NULL,
            last_error = $2
        WHERE id = $3
        "#,
        )
        .bind(retry_after)
        .bind(error)
        .bind(notification_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn mark_as_failed(
        &self,
        notification_id: i64,
        error: &str,
    ) -> Result<(), NotificationRepositoryError> {
        sqlx::query(
            r#"
        UPDATE notifications
        SET
            status = 'Failed',
            attempts = attempts + 1,
            locked_at = NULL,
            last_error = $1
        WHERE id = $2
        "#,
        )
        .bind(error)
        .bind(notification_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn recover_abandoned(
        &self,
        timeout_minutes: i64,
    ) -> Result<u64, NotificationRepositoryError> {
        let result = sqlx::query(
            r#"
        UPDATE notifications
        SET
            status = 'Pending',
            locked_at = NULL,
            next_attempt_at = NOW(),
            last_error = 'Recovered after processing timeout'
        WHERE status = 'Processing'
          AND (
      locked_at IS NULL
      OR locked_at < NOW() - ($1 * INTERVAL '1 minute')
  )        "#,
        )
        .bind(timeout_minutes)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

impl TryFrom<StoredNotification> for Notification {
    type Error = NotificationRepositoryError;

    fn try_from(stored: StoredNotification) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Some(stored.id),
            event_type: stored.event_type,
            payload: stored.payload,
            status: NotificationStatus::try_from(stored.status.as_str())
                .map_err(NotificationRepositoryError::InvalidStatus)?,
            attempts: stored.attempts,
            next_attempt_at: stored.next_attempt_at,
            last_error: stored.last_error,
            created_at: stored.created_at,
            sent_at: stored.sent_at,
            locked_at: stored.locked_at,
        })
    }
}

#[derive(Debug)]
pub enum NotificationRepositoryError {
    Serialization(serde_json::Error),
    Database(sqlx::Error),
    InvalidStatus(String),
}

impl From<serde_json::Error> for NotificationRepositoryError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}

impl From<sqlx::Error> for NotificationRepositoryError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

impl std::fmt::Display for NotificationRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialization(error) => {
                write!(f, "Notification serialization error: {}", error)
            }
            Self::Database(error) => {
                write!(f, "Notification database error: {}", error)
            }
            Self::InvalidStatus(status) => {
                write!(f, "Invalid notification status: {}", status)
            }
        }
    }
}

impl std::error::Error for NotificationRepositoryError {}
