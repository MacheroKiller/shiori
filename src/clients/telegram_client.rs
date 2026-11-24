use std::sync::Arc;

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::clients::http_client::HttpClient;

pub struct TelegramClient {
    http_client: Arc<HttpClient>,
    bot_token: String,
}

impl TelegramClient {
    pub fn new(http_client: Arc<HttpClient>, bot_token: String) -> Self {
        Self {
            http_client,
            bot_token,
        }
    }

    pub async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<TelegramResponse<TelegramMessage>, TelegramError> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);

        self.send_request(&url, &request).await
    }

    pub async fn send_photo(
        &self,
        request: SendPhotoRequest,
    ) -> Result<TelegramResponse<TelegramMessage>, TelegramError> {
        let url = format!("https://api.telegram.org/bot{}/sendPhoto", self.bot_token);

        self.send_request(&url, &request).await
    }

    async fn send_request<T, B>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<TelegramResponse<T>, TelegramError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let response = self.http_client.post_raw(url, body).await?;
        let status = response.status();
        let body = response.text().await?;

        let telegram_response: TelegramResponse<T> =
            serde_json::from_str(&body).map_err(|_| TelegramError::InvalidResponse {
                status: status.as_u16(),
                body: body.clone(),
            })?;

        if !telegram_response.ok {
            return Err(TelegramError::TelegramApi {
                error_code: telegram_response.error_code,
                description: telegram_response
                    .description
                    .unwrap_or_else(|| "Unknown Telegram API error".to_string()),
                retry_after: telegram_response
                    .parameters
                    .and_then(|parameters| parameters.retry_after),
            });
        }

        if !status.is_success() {
            return Err(TelegramError::HttpStatus {
                status: status.as_u16(),
                body,
            });
        }

        Ok(telegram_response)
    }
}

#[derive(Debug, Serialize)]
pub struct SendMessageRequest {
    pub chat_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<u64>,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SendPhotoRequest {
    pub chat_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<u64>,
    pub photo: String,
    pub caption: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramResponse<T> {
    pub ok: bool,
    pub result: Option<T>,
    pub description: Option<String>,
    pub error_code: Option<u16>,
    pub parameters: Option<TelegramResponseParameters>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramResponseParameters {
    pub retry_after: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramMessage {
    message_id: u64,
    text: Option<String>,
    chat: TelegramChat,
    caption: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramChat {
    id: i64,
}

#[derive(Debug)]
pub enum TelegramError {
    Http(reqwest::Error),
    TelegramApi {
        error_code: Option<u16>,
        description: String,
        retry_after: Option<u64>,
    },
    HttpStatus {
        status: u16,
        body: String,
    },
    InvalidResponse {
        status: u16,
        body: String,
    },
}

impl TelegramError {
    pub fn retry_after(&self) -> Option<u64> {
        match self {
            Self::TelegramApi { retry_after, .. } => *retry_after,
            _ => None,
        }
    }
}

impl From<reqwest::Error> for TelegramError {
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error)
    }
}

impl std::fmt::Display for TelegramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(error) => write!(f, "HTTP request error: {}", error),

            Self::TelegramApi {
                error_code,
                description,
                retry_after,
            } => {
                write!(
                    f,
                    "Telegram API error (code: {:?}): {}. Retry after: {:?}",
                    error_code, description, retry_after
                )
            }

            Self::HttpStatus { status, body } => {
                write!(f, "HTTP status {}: {}", status, body)
            }

            Self::InvalidResponse { status, body } => {
                write!(f, "Invalid Telegram response (HTTP {}): {}", status, body)
            }
        }
    }
}

impl std::error::Error for TelegramError {}
