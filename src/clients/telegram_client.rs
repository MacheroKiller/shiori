use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::clients::http_client::HttpClient;

pub struct TelegramClient {
    http_client: Arc<HttpClient>,
    bot_token: String,
}

impl TelegramClient {
    pub fn new(http_client: Arc<HttpClient>, bot_token: String) -> Self {
        TelegramClient {
            http_client,
            bot_token,
        }
    }
    pub async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<TelegramResponse<TelegramMessage>, reqwest::Error> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
        self.http_client
            .post::<TelegramResponse<TelegramMessage>, _>(&url, &request)
            .await
    }

    pub async fn send_photo(
        &self,
        request: SendPhotoRequest,
    ) -> Result<TelegramResponse<TelegramMessage>, reqwest::Error> {
        let url = format!("https://api.telegram.org/bot{}/sendPhoto", self.bot_token);

        self.http_client
            .post::<TelegramResponse<TelegramMessage>, _>(&url, &request)
            .await
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
    TelegramApi(String),
}

impl From<reqwest::Error> for TelegramError {
    fn from(error: reqwest::Error) -> Self {
        TelegramError::Http(error)
    }
}

impl std::fmt::Display for TelegramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TelegramError::Http(error) => write!(f, "HTTP error: {}", error),
            TelegramError::TelegramApi(error) => {
                write!(f, "Telegram couldn't send the message.\n Reason: {}", error)
            }
        }
    }
}

impl std::error::Error for TelegramError {}
