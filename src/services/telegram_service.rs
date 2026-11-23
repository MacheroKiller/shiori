use crate::{
    clients::telegram_client::{
        SendMessageRequest, SendPhotoRequest, TelegramClient, TelegramError,
    },
    models::{release::Release, release_event::ReleaseEvent},
    services::notification_policy::{self, NotificationPolicy},
};

pub struct TelegramService {
    telegram_client: TelegramClient,
    chat_id: String,
    message_thread_id: u64,
}

impl TelegramService {
    pub fn new(telegram_client: TelegramClient, chat_id: String, message_thread_id: u64) -> Self {
        TelegramService {
            telegram_client,
            chat_id,
            message_thread_id,
        }
    }

    pub async fn notify(&self, events: &[ReleaseEvent]) -> Result<(), TelegramError> {
        for event in events {
            if !NotificationPolicy::should_notify(event) {
                continue;
            }

            match event {
                ReleaseEvent::NewRelease(release) => {
                    let caption = Self::format_new_release_caption(release);

                    if let Some(image_url) = &release.image_url {
                        let request = SendPhotoRequest {
                            chat_id: self.chat_id.clone(),
                            message_thread_id: Some(self.message_thread_id),
                            photo: image_url.clone(),
                            caption,
                            parse_mode: Some("HTML".to_string()),
                        };

                        let response = self.telegram_client.send_photo(request).await?;

                        if !response.ok {
                            return Err(TelegramError::TelegramApi(
                                response
                                    .description
                                    .as_deref()
                                    .unwrap_or("Unknown")
                                    .to_string(),
                            ));
                        }
                    } else {
                        let request = SendMessageRequest {
                            chat_id: self.chat_id.clone(),
                            message_thread_id: Some(self.message_thread_id),
                            text: caption,
                            parse_mode: Some("HTML".to_string()),
                        };

                        let response = self.telegram_client.send_message(request).await?;

                        if !response.ok {
                            return Err(TelegramError::TelegramApi(
                                response
                                    .description
                                    .as_deref()
                                    .unwrap_or("Unknown")
                                    .to_string(),
                            ));
                        }
                    }
                }

                ReleaseEvent::ReleaseUpdated { .. } => {
                    let text = Self::format_event(event);

                    let request = SendMessageRequest {
                        chat_id: self.chat_id.clone(),
                        message_thread_id: Some(self.message_thread_id),
                        text,
                        parse_mode: Some("HTML".to_string()),
                    };

                    let response = self.telegram_client.send_message(request).await?;

                    if !response.ok {
                        return Err(TelegramError::TelegramApi(
                            response
                                .description
                                .as_deref()
                                .unwrap_or("Unknown")
                                .to_string(),
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    fn format_event(event: &ReleaseEvent) -> String {
        match event {
            ReleaseEvent::NewRelease(data) => {
                let title = Self::escape_html(&data.title);

                let description = data
                    .description
                    .as_deref()
                    .map(Self::format_description)
                    .unwrap_or_else(|| "No synopsis available".to_string());

                let genres = if data.genres.is_empty() {
                    "Unknown".to_string()
                } else {
                    Self::escape_html(&data.genres.join(" · "))
                };

                let release_date = data
                    .release_date
                    .as_ref()
                    .map(|value| value.to_formatted_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                let status = Self::format_status(&data.status);

                format!(
                    "🆕 <b>NEW ANIME RELEASE</b>\n\n\
                🎬 <b>{}</b>\n\n\
                📅 <b>Release date:</b> {}\n\
                📌 <b>Status:</b> {}\n\
                🏷 <b>Genres:</b> {}\n\n\
                📝 <b>Synopsis</b>\n\
                <i>{}</i>\n\n\
                🤖 <i>Eiko Reporter</i>",
                    title,
                    Self::escape_html(&release_date),
                    Self::escape_html(&status),
                    genres,
                    description
                )
            }

            ReleaseEvent::ReleaseUpdated { release, changes } => {
                let title = Self::escape_html(&release.title);

                let mut content = String::new();

                for change in changes {
                    let field = Self::format_field_name(&change.field_changed);

                    let old_value =
                        Self::escape_html(change.old_value.as_deref().unwrap_or("Unknown"));

                    let new_value =
                        Self::escape_html(change.new_value.as_deref().unwrap_or("Unknown"));

                    content.push_str(&format!(
                        "🔹 <b>{}</b>\n<s>{}</s> → <b>{}</b>\n\n",
                        field, old_value, new_value
                    ));
                }

                format!(
                    "🔄 <b>ANIME UPDATED</b>\n\n\
                🎬 <b>{}</b>\n\n\
                📋 <b>Changes detected:</b>\n\n\
                {}\
                🤖 <i>Eiko Reporter</i>",
                    title, content
                )
            }
        }
    }

    fn escape_html(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    fn format_description(description: &str) -> String {
        let mut cleaned = String::new();
        let mut inside_tag = false;
        let mut tag = String::new();

        for character in description.chars() {
            match character {
                '<' => {
                    inside_tag = true;
                    tag.clear();
                }
                '>' if inside_tag => {
                    let tag_name = tag.trim().to_lowercase();

                    if tag_name.starts_with("br") || tag_name == "p" || tag_name.starts_with("/p") {
                        cleaned.push('\n');
                    }

                    inside_tag = false;
                }
                _ if inside_tag => tag.push(character),
                _ => cleaned.push(character),
            }
        }

        let cleaned = cleaned
            .replace("&nbsp;", " ")
            .replace("&mdash;", "—")
            .replace("&ndash;", "–")
            .replace("&hellip;", "…")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&");

        let cleaned = cleaned.trim();

        let truncated: String = cleaned.chars().take(600).collect();

        let truncated = if cleaned.chars().count() > 600 {
            format!("{}...", truncated)
        } else {
            truncated
        };

        Self::escape_html(&truncated)
    }

    fn format_status(status: &str) -> String {
        match status {
            "NOT_YET_RELEASED" => "Not yet released".to_string(),
            "RELEASING" => "Currently releasing".to_string(),
            "FINISHED" => "Finished".to_string(),
            "CANCELLED" => "Cancelled".to_string(),
            "HIATUS" => "On hiatus".to_string(),
            _ => status.replace('_', " "),
        }
    }

    fn format_field_name(field: &str) -> String {
        match field {
            "title" => "Title".to_string(),
            "release_date" => "Release date".to_string(),
            "status" => "Status".to_string(),
            "episodes" => "Episodes".to_string(),
            "description" => "Description".to_string(),
            "genres" => "Genres".to_string(),
            "image_url" => "Cover image".to_string(),
            "duration" => "Episode duration".to_string(),
            _ => field.to_string(),
        }
    }

    fn format_new_release_caption(release: &Release) -> String {
        let title = Self::escape_html(&release.title);

        let genres = if release.genres.is_empty() {
            "Unknown".to_string()
        } else {
            Self::escape_html(&release.genres.join(" · "))
        };

        let release_date = release
            .release_date
            .as_ref()
            .map(|date| date.to_formatted_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let status = Self::format_status(&release.status);

        format!(
            "🆕 <b>NEW ANIME RELEASE</b>\n\n\
        🎬 <b>{}</b>\n\n\
        📅 <b>Release date:</b> {}\n\
        📌 <b>Status:</b> {}\n\
        🏷 <b>Genres:</b> {}\n\n\
        🤖 <i>Eiko Reporter</i>",
            title,
            Self::escape_html(&release_date),
            Self::escape_html(&status),
            genres
        )
    }
}
