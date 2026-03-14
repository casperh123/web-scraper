use std::collections::HashSet;

use reqwest::Url;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub async fn filter_domains(mut raw_receiver: UnboundedReceiver<Url>, filtered_sender: UnboundedSender<Url>) {
    let mut seen: HashSet<String> = HashSet::new();
    
    loop {
        let url = match raw_receiver.recv().await {
            Some(url) => url,
            None => return,
        };

        let host = match url.host_str() {
            Some(h) => h.to_string(),
            None => continue,
        };

        let host_url = match Url::parse(&format!("https://{}", host)) {
            Ok(u) => u,
            Err(_) => continue,
        };

        if ! should_crawl(&host_url) {
            continue;
        }

        if seen.insert(host) {
            let _ = filtered_sender.send(host_url);
        }
    }
}

pub fn should_crawl(url: &Url) -> bool {
    if url.fragment().is_some() {
        return false;
    }
    if url.query().is_some() {
        return false;
    }
    if is_image_or_file(url) {
        return false;
    }
    if !url.host_str().map(|h| h.ends_with(".dk")).unwrap_or(false) {
        return false;
   }
    true
}

fn is_image_or_file(url: &Url) -> bool {
    const FORBIDDEN_EXTENSIONS: [&str; 10] = ["jpg", "jpeg", "png", "gif", "webp", "svg", "ico", "bmp", "pdf", "zip"];

    url.path()
        .rsplit('.')
        .next()
        .map(|ext| FORBIDDEN_EXTENSIONS.iter().any(|&e| ext.eq_ignore_ascii_case(e)))
        .unwrap_or(false)
}
