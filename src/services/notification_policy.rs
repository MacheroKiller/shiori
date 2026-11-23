use crate::models::release_event::ReleaseEvent;

pub struct NotificationPolicy;

impl NotificationPolicy {
    pub fn should_notify(event: &ReleaseEvent) -> bool {
        match event {
            ReleaseEvent::NewRelease(_) => true,
            ReleaseEvent::ReleaseUpdated { changes, .. } => changes.iter().any(|change| {
                change.field_changed == "title"
                    || change.field_changed == "release_date"
                    || change.field_changed == "status"
                    || change.field_changed == "episodes"
            }),
        }
    }
}
