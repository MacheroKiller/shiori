use async_trait::async_trait;

use crate::{
    clients::{
        anilist::queries::UPCOMING_ANIME,
        anilist_client::{AniListClient, AniListError},
    },
    models::{
        anilist::{Media, UpcomingAnimeResponse},
        release::{Release, ReleaseDate, ReleaseSource},
    },
    provider::release_provider::ReleaseProvider,
};

pub struct AniListService {
    anilist_client: AniListClient,
}

impl AniListService {
    pub fn new(anilist_client: AniListClient) -> Self {
        Self { anilist_client }
    }

    pub async fn get_upcoming_anime(&self) -> Result<Vec<Release>, AniListError> {
        let mut anime_list: Vec<Media> = Vec::new();
        let mut current_page = 1;
        let query = UPCOMING_ANIME;

        loop {
            let variables = serde_json::json!({
                "page": current_page,
                "perPage": 50
            });
            let response = self
                .anilist_client
                .execute::<UpcomingAnimeResponse, _>(query, &variables)
                .await;

            match response {
                Ok(media) => {
                    let filtered_data = media.page.media.into_iter().filter(|anime| {
                        anime.start_date.as_ref().is_some_and(|date| {
                            date.year.is_some() && date.month.is_some() && date.day.is_some()
                        })
                    });
                    anime_list.extend(filtered_data);

                    if media.page.page_info.has_next_page {
                        current_page = media.page.page_info.current_page + 1;
                    } else {
                        break;
                    }
                }

                Err(error) => {
                    eprintln!("Error consultando AniList: {:?}", error);
                    return Err(error);
                }
            }
        }

        let release: Vec<Release> = anime_list
            .iter()
            .map(|value| {
                let release_date = value.start_date.as_ref().map(|date| ReleaseDate {
                    year: date.year,
                    month: date.month,
                    day: date.day,
                });

                let image_url = value
                    .banner_image
                    .clone()
                    .or_else(|| {
                        value
                            .cover_image
                            .as_ref()
                            .and_then(|image| image.extra_large.clone())
                    })
                    .or_else(|| {
                        value
                            .cover_image
                            .as_ref()
                            .and_then(|image| image.large.clone())
                    });

                let release_return = Release {
                    id: None,
                    external_id: value.id,
                    title: value
                        .title
                        .english
                        .as_deref()
                        .or(value.title.romaji.as_deref())
                        .unwrap_or("No title registered!")
                        .to_string(),
                    description: value.description.clone(),

                    genres: value.genres.clone(),

                    image_url,
                    status: value.status.clone(),
                    episodes: value.episodes,
                    duration: value.duration,
                    last_provider_update: Some(value.updated_at),
                    source: ReleaseSource::AniList,
                    release_date,
                    season_id: None,
                };
                release_return
            })
            .collect();

        Ok(release)
    }
}

#[async_trait]
impl ReleaseProvider for AniListService {
    async fn fetch_releases(
        &self,
    ) -> Result<Vec<Release>, Box<dyn std::error::Error + Send + Sync>> {
        let releases = self.get_upcoming_anime().await?;

        Ok(releases)
    }
}
