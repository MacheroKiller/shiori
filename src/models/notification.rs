use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationStatus {
    Pending,
    Processing,
    Sent,
    Failed,
}

use chrono::{DateTime, Utc};
use serde_json::Value;

pub struct Notification {
    pub id: Option<i64>,
    pub event_type: String,
    pub payload: Value,
    pub status: NotificationStatus,
    pub attempts: i32,
    pub next_attempt_at: DateTime<Utc>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub locked_at: Option<DateTime<Utc>>,
}

impl NotificationStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "Pending",
            Self::Processing => "Processing",
            Self::Sent => "Sent",
            Self::Failed => "Failed",
        }
    }
}

impl TryFrom<&str> for NotificationStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Pending" => Ok(Self::Pending),
            "Processing" => Ok(Self::Processing),
            "Sent" => Ok(Self::Sent),
            "Failed" => Ok(Self::Failed),
            _ => Err(format!("Invalid notification status: {}", value)),
        }
    }
}
