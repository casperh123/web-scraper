use database::models::website;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct WebsiteDto {
    pub url: String,
    pub average_ttfb: i32,
    pub links_crawled: i32
}

impl From<website::Model> for WebsiteDto {
    fn from(model: website::Model) -> Self {
        Self {
            url: model.url,
            average_ttfb: model.average_ttfb_ms,
            links_crawled: model.links_crawled
        }
    }
}
