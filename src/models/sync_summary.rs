#[derive(Default, Debug)]
pub struct SyncSummary {
    pub total_received: u64,
    pub inserted: u64,
    pub updated: u64,
    pub unchanged: u64,
}
