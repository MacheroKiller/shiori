use sqlx::PgPool;

use crate::{
    models::{
        release::Release,
        release_event::{ReleaseChange, ReleaseEvent},
        season::Season,
        sync_result::SyncResult,
    },
    repositories::{
        notification_repository::{self, NotificationRepository},
        season_repository::SeasonRepository,
    },
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
    pub season_id: Option<i64>,
    pub season_name: Option<String>,
    pub season_year: Option<i32>,
}

pub struct ReleaseRepository {
    pool: PgPool,
    season_repository: SeasonRepository,
    notification_repository: NotificationRepository,
}

impl ReleaseRepository {
    pub fn new(
        pool: PgPool,
        season_repository: SeasonRepository,
        notification_repository: NotificationRepository,
    ) -> Self {
        Self {
            pool,
            season_repository,
            notification_repository,
        }
    }

    pub fn detect_changes(
        stored: &StoredRelease,
        incoming: &Release,
        season: Option<&Season>,
    ) -> Vec<ReleaseChange> {
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

        // Season
        let old_season = match (&stored.season_name, stored.season_year) {
            (Some(name), Some(year)) => Some(format!("{} {}", name, year)),
            _ => None,
        };

        let new_season = season
            .map(|season| format!("{:?} {}", season.name, season.year))
            .or_else(|| old_season.clone());

        if old_season != new_season {
            changes.push(ReleaseChange {
                field_changed: "season".to_string(),
                old_value: old_season,
                new_value: new_season,
            });
        }

        changes
    }

    pub async fn sync_many(
        &self,
        releases: Vec<Release>,
    ) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut sync_result = SyncResult::default();
        let mut transaction = self.pool.begin().await?;

        for mut release in releases {
            sync_result.summary.total_received += 1;

            let source_name = format!("{:?}", release.source);

            let release_date = release.release_date.as_ref().and_then(|date| {
                chrono::NaiveDate::from_ymd_opt(date.year? as i32, date.month?, date.day?)
            });

            // Find existing release
            let existing_release = sqlx::query_as::<_, StoredRelease>(
                r#"
       SELECT
    r.id,
    r.title,
    r.description,
    r.genres,
    r.image_url,
    r.release_date,
    r.status,
    r.episodes,
    r.duration,
    r.season_id,
    s.name AS season_name,
    s.year AS season_year
FROM releases r
LEFT JOIN seasons s ON s.id = r.season_id
WHERE r.external_id = $1
  AND r.source = $2
FOR UPDATE OF r
            "#,
            )
            .bind(release.external_id as i64)
            .bind(&source_name)
            .fetch_optional(&mut *transaction)
            .await?;

            // Determine the season based on the release date
            let mut assigned_season: Option<Season> = None;

            if let Some(release_date) = release_date {
                let season = Season::from_date(release_date);

                let persisted_season = self
                    .season_repository
                    .get_or_create(&mut transaction, season)
                    .await?;

                release.season_id = persisted_season.id;
                assigned_season = Some(persisted_season);
            } else if let Some(existing_release) = &existing_release {
                // Preserve the previous season when the release date is incomplete
                release.season_id = existing_release.season_id;
            }

            match existing_release {
                Some(existing_release) => {
                    // Detect changes
                    release.id = Some(existing_release.id as u64);

                    let release_changes =
                        Self::detect_changes(&existing_release, &release, assigned_season.as_ref());

                    // Save changes to history
                    for change in &release_changes {
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
                        .bind(existing_release.id)
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
                        last_provider_update = $9,
                        season_id = $10
                    WHERE id = $11
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
                    .bind(release.season_id)
                    .bind(existing_release.id)
                    .execute(&mut *transaction)
                    .await?;

                    if release_changes.is_empty() {
                        sync_result.summary.unchanged += 1;
                    } else {
                        let event = ReleaseEvent::ReleaseUpdated {
                            release,
                            changes: release_changes,
                        };

                        self.notification_repository
                            .enqueue_in_transaction(&mut transaction, &event)
                            .await?;

                        sync_result.events.push(event);
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
                        last_provider_update,
                        season_id
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                    RETURNING id
                    "#,
                    )
                    .bind(release.external_id as i64)
                    .bind(&source_name)
                    .bind(&release.title)
                    .bind(&release.description)
                    .bind(&release.genres)
                    .bind(&release.image_url)
                    .bind(release_date)
                    .bind(&release.status)
                    .bind(release.episodes.map(|value| value as i32))
                    .bind(release.duration.map(|value| value as i32))
                    .bind(release.last_provider_update.map(|value| value as i64))
                    .bind(release.season_id)
                    .fetch_one(&mut *transaction)
                    .await?;

                    release.id = Some(id as u64);

                    sync_result.summary.inserted += 1;
                    let event = ReleaseEvent::NewRelease(release);

                    self.notification_repository
                        .enqueue_in_transaction(&mut transaction, &event)
                        .await?;

                    sync_result.events.push(event);
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
        r.id,
        r.title,
        r.description,
        r.genres,
        r.image_url,
        r.release_date,
        r.status,
        r.episodes,
        r.duration,
        r.season_id,
        s.name AS season_name,
        s.year AS season_year
    FROM releases r
    LEFT JOIN seasons s ON s.id = r.season_id
    WHERE r.external_id = $1
      AND r.source = $2
    "#,
        )
        .bind(external_id)
        .bind(source)
        .fetch_optional(&self.pool)
        .await?;

        Ok(release)
    }
}
