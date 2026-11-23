use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]

pub struct AniListRequest<T> {
    pub query: String,
    pub variables: T,
}

#[derive(Debug, Deserialize)]
pub struct AniListResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]

pub struct GraphQLError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct MediaResponse {
    #[serde(rename = "Media")]
    pub media: Media,
}

#[derive(Debug, Deserialize)]
pub struct Media {
    pub id: u32,
    pub title: MediaTitle,
    #[serde(rename = "startDate")]
    pub start_date: Option<MediaDate>,
    #[serde(rename = "endDate")]
    pub end_date: Option<MediaDate>,
    pub season: Option<String>,
    #[serde(rename = "seasonYear")]
    pub season_year: Option<u32>,
    pub status: String,
    pub genres: Vec<String>,
    pub description: Option<String>,
    pub episodes: Option<u32>,

    #[serde(rename = "coverImage")]
    pub cover_image: Option<MediaImage>,

    #[serde(rename = "bannerImage")]
    pub banner_image: Option<String>,

    #[serde(rename = "updatedAt")]
    pub updated_at: u64,
    pub duration: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct MediaImage {
    pub large: Option<String>,
    pub extra_large: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MediaTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MediaDate {
    pub year: Option<u32>,
    pub month: Option<u32>,
    pub day: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct UpcomingAnimeResponse {
    #[serde(rename = "Page")]
    pub page: AnimePage,
}

#[derive(Debug, Deserialize)]
pub struct AnimePage {
    pub media: Vec<Media>,

    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}

#[derive(Debug, Deserialize)]
pub struct PageInfo {
    #[serde(rename = "currentPage")]
    pub current_page: u32,

    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
}
