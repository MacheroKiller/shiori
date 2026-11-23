use async_trait::async_trait;

use crate::models::release::Release;

#[async_trait]
pub trait ReleaseProvider {
    async fn fetch_releases(
        &self,
    ) -> Result<Vec<Release>, Box<dyn std::error::Error + Send + Sync>>;
}
