use sqlx::PgPool;

use crate::models::{
    release::Release,
    release_event::{ReleaseChange, ReleaseEvent},
    sync_result::SyncResult,
    sync_summary::SyncSummary,
};

use chrono::NaiveDate;

#[derive(Debug, sqlx::FromRow)]
pub struct StoredRelease {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub genres: Vec<String>,
    pub image_url: Option<String>,
    pub release_date: Option<NaiveDate>,
    pub status: String,
    pub episodes: Option<i32>,
    pub duration: Option<i32>,
}

pub struct ReleaseRepository {
    pool: PgPool,
}

impl ReleaseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn detect_changes(stored: &StoredRelease, incoming: &Release) -> Vec<ReleaseChange> {
        let mut changes = Vec::new();

        // Title
        if stored.title != incoming.title {
            changes.push(ReleaseChange {
                field_changed: "title".to_string(),
                old_value: Some(stored.title.clone()),
                new_value: Some(incoming.title.clone()),
            });
        }

        // Description
        if stored.description.as_deref() != incoming.description.as_deref() {
            changes.push(ReleaseChange {
                field_changed: "description".to_string(),
                old_value: stored.description.clone(),
                new_value: incoming.description.clone(),
            });
        }

        // Genres
        if stored.genres != incoming.genres {
            changes.push(ReleaseChange {
                field_changed: "genres".to_string(),
                old_value: Some(stored.genres.join(", ")),
                new_value: Some(incoming.genres.join(", ")),
            });
        }

        // Image
        if stored.image_url.as_deref() != incoming.image_url.as_deref() {
            changes.push(ReleaseChange {
                field_changed: "image_url".to_string(),
                old_value: stored.image_url.clone(),
                new_value: incoming.image_url.clone(),
            });
        }

        // Release Date
        let new_date = incoming.release_date.as_ref().and_then(|date| {
            match (date.year, date.month, date.day) {
                (Some(year), Some(month), Some(day)) => {
                    chrono::NaiveDate::from_ymd_opt(year as i32, month, day)
                }
                _ => None,
            }
        });

        if stored.release_date != new_date {
            changes.push(ReleaseChange {
                field_changed: "release_date".to_string(),
                old_value: stored.release_date.map(|date| date.to_string()),
                new_value: new_date.map(|date| date.to_string()),
            });
        }

        // Status
        if stored.status != incoming.status {
            changes.push(ReleaseChange {
                field_changed: "status".to_string(),
                old_value: Some(stored.status.clone()),
                new_value: Some(incoming.status.clone()),
            });
        }

        // Episodes
        let new_episodes = incoming.episodes.map(|value| value as i32);

        if stored.episodes != new_episodes {
            changes.push(ReleaseChange {
                field_changed: "episodes".to_string(),
                old_value: stored.episodes.map(|value| value.to_string()),
                new_value: new_episodes.map(|value| value.to_string()),
            });
        }

        // Duration
        let new_duration = incoming.duration.map(|value| value as i32);

        if stored.duration != new_duration {
            changes.push(ReleaseChange {
                field_changed: "duration".to_string(),
                old_value: stored.duration.map(|value| value.to_string()),
                new_value: new_duration.map(|value| value.to_string()),
            });
        }

        changes
    }
    pub async fn sync_many(&self, releases: Vec<Release>) -> Result<SyncResult, sqlx::Error> {
        let mut sync_result = SyncResult::default();
        let mut transaction = self.pool.begin().await?;

        for release in releases {
            sync_result.summary.total_received += 1;
            let source = format!("{:?}", release.source);

            let release_date = release.release_date.as_ref().and_then(|date| {
                match (date.year, date.month, date.day) {
                    (Some(year), Some(month), Some(day)) => {
                        chrono::NaiveDate::from_ymd_opt(year as i32, month, day)
                    }
                    _ => None,
                }
            });

            // Find existing release
            let stored = sqlx::query_as::<_, StoredRelease>(
                r#"
            SELECT
                id,
                title,
                description,
                genres,
                image_url,
                release_date,
                status,
                episodes,
                duration
            FROM releases
            WHERE external_id = $1
              AND source = $2
            FOR UPDATE
            "#,
            )
            .bind(release.external_id as i64)
            .bind(&source)
            .fetch_optional(&mut *transaction)
            .await?;

            match stored {
                Some(stored) => {
                    // Detect changes
                    let mut release = release;
                    release.id = Some(stored.id as u64);
                    let changes = Self::detect_changes(&stored, &release);

                    // Save changes to history
                    for change in &changes {
                        sqlx::query(
                            r#"
                        INSERT INTO release_history (
                            release_id,
                            field_changed,
                            old_value,
                            new_value
                        )
                        VALUES ($1, $2, $3, $4)
                        "#,
                        )
                        .bind(stored.id)
                        .bind(&change.field_changed)
                        .bind(&change.old_value)
                        .bind(&change.new_value)
                        .execute(&mut *transaction)
                        .await?;
                    }

                    // Update existing release
                    sqlx::query(
                        r#"
                    UPDATE releases
                    SET
                        title = $1,
                        description = $2,
                        genres = $3,
                        image_url = $4,
                        release_date = $5,
                        status = $6,
                        episodes = $7,
                        duration = $8,
                        last_provider_update = $9
                    WHERE id = $10
                    "#,
                    )
                    .bind(&release.title)
                    .bind(&release.description)
                    .bind(&release.genres)
                    .bind(&release.image_url)
                    .bind(release_date)
                    .bind(&release.status)
                    .bind(release.episodes.map(|value| value as i32))
                    .bind(release.duration.map(|value| value as i32))
                    .bind(release.last_provider_update.map(|value| value as i64))
                    .bind(stored.id)
                    .execute(&mut *transaction)
                    .await?;

                    if changes.is_empty() {
                        sync_result.summary.unchanged += 1;
                    } else {
                        sync_result.summary.updated += 1;
                        sync_result
                            .events
                            .push(ReleaseEvent::ReleaseUpdated { release, changes });
                    }
                }

                None => {
                    // Insert new release
                    let id: i64 = sqlx::query_scalar(
                        r#"
    INSERT INTO releases (
        external_id,
        source,
        title,
        description,
        genres,
        image_url,
        release_date,
        status,
        episodes,
        duration,
        last_provider_update
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
    RETURNING id
    "#,
                    )
                    .bind(release.external_id as i64)
                    .bind(&source)
                    .bind(&release.title)
                    .bind(&release.description)
                    .bind(&release.genres)
                    .bind(&release.image_url)
                    .bind(release_date)
                    .bind(&release.status)
                    .bind(release.episodes.map(|value| value as i32))
                    .bind(release.duration.map(|value| value as i32))
                    .bind(release.last_provider_update.map(|value| value as i64))
                    .fetch_one(&mut *transaction)
                    .await?;
                    let mut release = release;
                    release.id = Some(id as u64);

                    sync_result.summary.inserted += 1;
                    sync_result.events.push(ReleaseEvent::NewRelease(release));
                }
            }
        }

        transaction.commit().await?;

        Ok(sync_result)
    }
    pub async fn find_by_external_id(
        &self,
        external_id: i64,
        source: &str,
    ) -> Result<Option<StoredRelease>, sqlx::Error> {
        let release = sqlx::query_as::<_, StoredRelease>(
            r#"
        SELECT
            id,
            title,
            description,
            genres,
            image_url,
            release_date,
            status,
            episodes,
            duration
        FROM releases
        WHERE external_id = $1
          AND source = $2
        "#,
        )
        .bind(external_id)
        .bind(source)
        .fetch_optional(&self.pool)
        .await?;

        Ok(release)
    }
}
