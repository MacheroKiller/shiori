use crate::models::release::Release;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReleaseEvent {
    NewRelease(Release),

    ReleaseUpdated {
        release: Release,
        changes: Vec<ReleaseChange>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseChange {
    pub field_changed: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}
