
-- Create seasons table.
-- Stores the calendar seasons managed by Release Tracker,
-- independently of the season classifications provided by external providers.

CREATE TABLE seasons (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name VARCHAR(10) NOT NULL,
    year INTEGER NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    activation_date DATE NOT NULL,
    tracking_status VARCHAR(20) NOT NULL DEFAULT 'Scheduled',

    CONSTRAINT uq_seasons_name_year UNIQUE (name, year),

    CONSTRAINT chk_seasons_name
        CHECK (name IN ('Winter', 'Spring', 'Summer', 'Autumn')),

    CONSTRAINT chk_seasons_tracking_status
        CHECK (tracking_status IN ('Scheduled', 'Active', 'Completed')),

    CONSTRAINT chk_seasons_dates
        CHECK (
            start_date <= end_date
            AND activation_date < start_date
        )
);

-- Table documentation.
COMMENT ON TABLE seasons IS
    'Stores the calendar seasons managed by Release Tracker.';

-- Column documentation.
COMMENT ON COLUMN seasons.id IS
    'Unique internal identifier of the season.';

COMMENT ON COLUMN seasons.name IS
    'Calendar season name: Winter, Spring, Summer or Autumn.';

COMMENT ON COLUMN seasons.year IS
    'Calendar year to which the season belongs.';

COMMENT ON COLUMN seasons.start_date IS
    'First calendar day of the season and the date when seasonal tracking begins.';

COMMENT ON COLUMN seasons.end_date IS
    'Last calendar day of the season.';

COMMENT ON COLUMN seasons.activation_date IS
    'Date when the season becomes eligible for preparation, 15 days before its start date.';

COMMENT ON COLUMN seasons.tracking_status IS
    'Season tracking lifecycle status: Scheduled, Active or Completed.';
