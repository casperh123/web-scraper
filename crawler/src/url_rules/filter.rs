use reqwest::Url;

use crate::url_rules::{extension::is_image_or_file, query::has_crawlable_query};

pub fn resolve_full_url(base: &Url, link: &str) -> Option<Url> {
    Url::parse(link).ok().or_else(|| base.join(&link).ok())
}

pub fn should_crawl(url: &Url) -> bool {
    if url.fragment().is_some() {
        return false;
    }
    if !has_crawlable_query(url) {
        return false;
    }
    if is_image_or_file(url) {
        return false;
    }
    url.host_str().map(|h| h.ends_with(".dk")).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_crawl_positive() {
        let cases = [
            ("https://example.dk", true),
            ("https://example.dk/", true),
            ("https://example.dk/noext", true),
        ];

        for (input, expected) in cases {
            let url = Url::parse(input).unwrap();

            assert_eq!(
                should_crawl(&url), expected,
                "failed for input: {input}"
            );
        }
    }

    #[test]
    fn should_crawl_negative() {
        let cases = [
            ("https://example.dk/hej#meddig", false),
            ("https://example.dk/mannes?info=hej", false),
        ];

        for (input, expected) in cases {
            let url = Url::parse(input).unwrap();

            assert_eq!(
                should_crawl(&url), expected,
                "failed for input: {input}"
            );
        }
    }
}
