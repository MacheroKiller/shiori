use sqlx::{PgPool, Postgres, Transaction};

use crate::models::season::{Season, SeasonName, SeasonTrackingStatus};

pub struct SeasonRepository;

impl SeasonRepository {
    pub fn new() -> Self {
        Self
    }
    pub async fn get_or_create(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        mut season: Season,
    ) -> Result<Season, sqlx::Error> {
        let season_name = match season.name {
            SeasonName::Winter => "Winter",
            SeasonName::Spring => "Spring",
            SeasonName::Summer => "Summer",
            SeasonName::Autumn => "Autumn",
        };

        let tracking_status = match season.tracking_status {
            SeasonTrackingStatus::Scheduled => "Scheduled",
            SeasonTrackingStatus::Active => "Active",
            SeasonTrackingStatus::Completed => "Completed",
        };

        let id = sqlx::query_scalar::<_, i64>(
            r#"
    INSERT INTO seasons (
        name,
        year,
        start_date,
        end_date,
        activation_date,
        tracking_status
    )
    VALUES ($1, $2, $3, $4, $5, $6)
    ON CONFLICT (name, year)
    DO UPDATE SET name = EXCLUDED.name
    RETURNING id
    "#,
        )
        .bind(season_name)
        .bind(season.year)
        .bind(season.start_date)
        .bind(season.end_date)
        .bind(season.activation_date)
        .bind(tracking_status)
        .fetch_one(&mut **transaction)
        .await?;

        season.id = Some(id);
        Ok(season)
    }
}
