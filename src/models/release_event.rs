use crate::models::release::Release;

#[derive(Debug)]
pub enum ReleaseEvent {
    NewRelease(Release),

    ReleaseUpdated {
        release: Release,
        changes: Vec<ReleaseChange>,
    },
}

#[derive(Debug)]
pub struct ReleaseChange {
    pub field_changed: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}
