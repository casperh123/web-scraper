use bloomfilter::Bloom;
use reqwest::Url;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub async fn filter_domains(mut raw_receiver: UnboundedReceiver<Url>, filtered_sender: UnboundedSender<Url>) {
    let mut seen: Bloom<String> = Bloom::new_for_fp_rate(1_000_000, 0.01).unwrap();
    
    loop {
        let url = match raw_receiver.recv().await {
            Some(url) => url,
            None => continue,
        };

        let host = match url.host_str() {
            Some(h) => h.trim_start_matches("www.").to_string(),
            None => continue,
        };

        let host_url = match Url::parse(&format!("https://{}", host)) {
            Ok(u) => u,
            Err(_) => continue,
        };

        if !seen.check_and_set(&host) {
            let _ = filtered_sender.send(host_url);
        }
    }
}

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

fn has_crawlable_query(url: &Url) -> bool {
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

fn is_image_or_file(url: &Url) -> bool {
    const FORBIDDEN_EXTENSIONS: [&str; 10] = ["jpg", "jpeg", "png", "gif", "webp", "svg", "ico", "bmp", "pdf", "zip"];

    url.path()
        .rsplit('.')
        .next()
        .map(|ext| FORBIDDEN_EXTENSIONS.iter().any(|&e| ext.eq_ignore_ascii_case(e)))
        .unwrap_or(false)
}
