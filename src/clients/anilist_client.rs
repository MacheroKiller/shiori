use std::sync::Arc;

use crate::{
    clients::http_client::HttpClient,
    models::anilist::{AniListRequest, AniListResponse, GraphQLError},
};

#[derive(Debug)]
pub enum AniListError {
    Http(reqwest::Error),
    GraphQL(Vec<GraphQLError>),
    MissingData,
}

pub struct AniListClient {
    http_client: Arc<HttpClient>,
}

impl AniListClient {
    pub fn new(http_client: Arc<HttpClient>) -> Self {
        AniListClient { http_client }
    }

    pub async fn execute<T, V>(&self, query: &str, variables: V) -> Result<T, AniListError>
    where
        T: serde::de::DeserializeOwned,
        V: serde::Serialize,
    {
        let body = AniListRequest {
            query: query.to_string(),
            variables,
        };
        let response = self
            .http_client
            .post::<AniListResponse<T>, _>("https://graphql.anilist.co", &body)
            .await?;

        if let Some(errors) = response.errors {
            Err(AniListError::GraphQL(errors))
        } else if let Some(data) = response.data {
            Ok(data)
        } else {
            Err(AniListError::MissingData)
        }
    }
}

impl From<reqwest::Error> for AniListError {
    fn from(error: reqwest::Error) -> Self {
        AniListError::Http(error)
    }
}

impl std::fmt::Display for AniListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AniListError::Http(error) => write!(f, "HTTP error: {}", error),
            AniListError::GraphQL(errors) => {
                write!(f, "GraphQL errors: {:?}", errors)
            }
            AniListError::MissingData => write!(f, "AniList returned no data"),
        }
    }
}

impl std::error::Error for AniListError {}
