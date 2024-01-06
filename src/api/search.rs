use lemmy_api_common::{
    lemmy_db_schema::{SearchType, SortType},
    site::{Search, SearchResponse},
};

pub fn fetch_search(
    page: i64,
    query: String,
    search_type: Option<SearchType>,
) -> std::result::Result<SearchResponse, reqwest::Error> {
    let params = Search {
        q: query,
        sort: Some(SortType::TopMonth),
        page: Some(page),
        type_: search_type,
        ..Default::default()
    };

    super::get("/search", &params)
}
