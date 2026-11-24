use chrono::{Datelike, Duration, NaiveDate};

#[derive(Debug)]
pub struct Season {
    pub id: Option<i64>,
    pub name: SeasonName,
    pub year: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub activation_date: NaiveDate,
    pub tracking_status: SeasonTrackingStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeasonName {
    Winter,
    Spring,
    Summer,
    Autumn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeasonTrackingStatus {
    Scheduled,
    Active,
    Completed,
}

impl Season {
    pub fn from_date(date: NaiveDate) -> Self {
        let year = date.year();
        let season_name = match date.month() {
            1 | 2 | 3 => SeasonName::Winter,
            4 | 5 | 6 => SeasonName::Spring,
            7 | 8 | 9 => SeasonName::Summer,
            10 | 11 | 12 => SeasonName::Autumn,
            _ => unreachable!("Invalid month"),
        };

        let start_date = match season_name {
            SeasonName::Winter => NaiveDate::from_ymd_opt(year, 1, 1),
            SeasonName::Spring => NaiveDate::from_ymd_opt(year, 4, 1),
            SeasonName::Summer => NaiveDate::from_ymd_opt(year, 7, 1),
            SeasonName::Autumn => NaiveDate::from_ymd_opt(year, 10, 1),
        }
        .expect("Invalid season start date");

        let end_date = match season_name {
            SeasonName::Winter => NaiveDate::from_ymd_opt(year, 3, 31),
            SeasonName::Spring => NaiveDate::from_ymd_opt(year, 6, 30),
            SeasonName::Summer => NaiveDate::from_ymd_opt(year, 9, 30),
            SeasonName::Autumn => NaiveDate::from_ymd_opt(year, 12, 31),
        }
        .expect("Invalid season end date");

        let activation_date = start_date - Duration::days(15);

        Season {
            id: None,
            name: season_name,
            year,
            start_date,
            end_date,
            activation_date,
            tracking_status: SeasonTrackingStatus::Scheduled,
        }
    }
}
