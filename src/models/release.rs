use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseSource {
    AniList,
    TMDB,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseDate {
    pub year: Option<u32>,
    pub month: Option<u32>,
    pub day: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub id: Option<u64>,
    pub external_id: u32,
    pub source: ReleaseSource,
    pub title: String,
    pub description: Option<String>,
    pub genres: Vec<String>,
    pub image_url: Option<String>,
    pub release_date: Option<ReleaseDate>,
    pub status: String,
    pub episodes: Option<u32>,
    pub duration: Option<u32>,
    pub last_provider_update: Option<u64>,
    pub season_id: Option<i64>,
}

impl ReleaseDate {
    pub fn to_formatted_string(&self) -> String {
        match (self.year, self.month, self.day) {
            (Some(y), Some(m), Some(d)) => format!("{}-{:02}-{:02}", y, m, d),

            (Some(y), Some(m), None) => format!("{}-{:02}", y, m),

            (Some(y), None, None) => format!("{}", y),
            _ => format!("No date available"),
        }
    }
}
