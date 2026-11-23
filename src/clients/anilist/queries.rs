pub const UPCOMING_ANIME: &str = r#"
    query ($page: Int, $perPage: Int) {
    Page(page: $page, perPage: $perPage) {
        media(
            type: ANIME
            status: NOT_YET_RELEASED
            sort: START_DATE
        ) {
            id
            title {
                romaji
                english
                native
            }
            startDate {
                year
                month
                day
            }
            season
            seasonYear
            status
            genres
            description(asHtml: false)
episodes
coverImage {
    large
    extraLarge
}
bannerImage
updatedAt
duration
        }
pageInfo {
    currentPage
    hasNextPage
}
    }
}
"#;
