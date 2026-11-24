ALTER TABLE releases
ADD COLUMN season_id BIGINT REFERENCES seasons(id);
COMMENT ON COLUMN releases.season_id IS
    'Reference to the release original season of origin. This relationship remains unchanged even if the release continues into subsequent calendar seasons. NULL when no release date is available.';
