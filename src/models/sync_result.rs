use crate::models::{release_event::ReleaseEvent, sync_summary::SyncSummary};

#[derive(Debug, Default)]
pub struct SyncResult {
    pub summary: SyncSummary,
    pub events: Vec<ReleaseEvent>,
}
