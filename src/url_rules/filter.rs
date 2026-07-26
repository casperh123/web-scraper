use reqwest::Url;

use crate::url_rules::{extension::is_image_or_file, query::has_crawlable_query};

pub fn should_crawl(base_url: &str, url: &str) -> Option<Url> {
    let url = Url::parse(url)
        .ok()
        .or_else(|| Url::parse(base_url).ok()?.join(url).ok())?;

    if url.fragment().is_some() {
        return None;
    }
    if !has_crawlable_query(&url) {
        return None;
    }
    if is_image_or_file(&url) {
        return None;
    }
    if !url.host_str().map(|h| h.ends_with(".dk")).unwrap_or(false) {
        return None;
   }
    Some(url)
}
