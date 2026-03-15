pub struct CrawlResult {
    pub url: String,
    pub average_ttfb_ms: i32,
    pub links_crawled: i32
}

pub struct PageResult {
    pub ttfb_ms: i32,
    pub request_duration_ms: i32,
}
