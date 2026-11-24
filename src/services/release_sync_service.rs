use crate::{
    models::sync_result::SyncResult, provider::release_provider::ReleaseProvider,
    repositories::release_repository::ReleaseRepository,
};

pub struct ReleaseSyncService {
    provider: Box<dyn ReleaseProvider + Send + Sync>,
    release_repository: ReleaseRepository,
}

impl ReleaseSyncService {
    pub fn new(
        provider: Box<dyn ReleaseProvider + Send + Sync>,
        release_repository: ReleaseRepository,
    ) -> Self {
        Self {
            provider,
            release_repository,
        }
    }

    pub async fn sync(&self) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let releases = self.provider.fetch_releases().await?;

        let summary = self.release_repository.sync_many(releases).await?;

        Ok(summary)
    }
}
