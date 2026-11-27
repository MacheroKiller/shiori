/// Application configuration.
pub struct Config {
    /// PostgreSQL connection URL.
    pub database_url: String,
    /// Telegram configuration.
    pub telegram: TelegramConfig,
}

/// Telegram bot configuration.
pub struct TelegramConfig {
    /// Telegram bot authentication token.
    pub bot_token: String,
    /// Telegram destination chat ID.
    pub chat_id: String,
    /// Available Telegram notification topics.
    pub topics: TelegramTopics,
}

/// Telegram notification topics.
pub struct TelegramTopics {
    /// AniList notification topic ID.
    pub animelist: Option<u64>,
    /// TMDB notification topic ID.
    pub tmdb: Option<u64>,
}

#[derive(Debug)]
pub enum ConfigError {
    MissingVariable(String),
    InvalidValue(String),
}

impl Config {
    /// Loads application configuration from environment variables.
    pub fn load() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        let database_url = Self::get_data("DATABASE_URL")?;
        let telegram_bot_token = Self::get_data("TELEGRAM_BOT_TOKEN")?;
        let telegram_chat_id = Self::get_data("TELEGRAM_CHAT_ID")?;

        let telegram_animelist_thread_id = dotenvy::var("TELEGRAM_ANIMELIST_THREAD_ID")
            .ok()
            .map(|value| {
                value.parse::<u64>().map_err(|_| {
                    ConfigError::InvalidValue("TELEGRAM_ANIMELIST_THREAD_ID".to_string())
                })
            })
            .transpose()?;

        Ok(Config {
            database_url,
            telegram: TelegramConfig {
                bot_token: telegram_bot_token,
                chat_id: telegram_chat_id,
                topics: TelegramTopics {
                    animelist: telegram_animelist_thread_id,
                    tmdb: None,
                },
            },
        })
    }

    fn get_data(key: &str) -> Result<String, ConfigError> {
        dotenvy::var(key).map_err(|_| ConfigError::MissingVariable(key.to_string()))
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingVariable(error) => {
                write!(f, "Missing Variable: {}", error)
            }
            ConfigError::InvalidValue(error) => {
                write!(f, "Invalid Value: {}", error)
            }
        }
    }
}

impl std::error::Error for ConfigError {}
