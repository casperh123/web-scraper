use reqwest::Url;

pub(super) fn has_crawlable_query(url: &Url) -> bool {
    let Some(query) = url.query() else { return true };
    
    if query.is_empty() { return true; }
    
    const NAVIGATION_PARAMS: [&str; 18] = [
        "page",
        "p",
        "id",
        "category",
        "cat",
        "tag",
        "type",
        "section",
        "topic",
        "subject",
        "year",
        "month",
        "lang",
        "language",
        "view",
        "tab",
        "step",
        "chapter",
    ];
    
    url.query_pairs().all(|(key, _)| {
        NAVIGATION_PARAMS.iter().any(|&p| key.as_ref() == p)
    })
}

