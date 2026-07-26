use std::{sync::Arc, time::{Duration, Instant}};
use bloomfilter::Bloom;
use reqwest::{Client, Response, Url, header::CONTENT_TYPE};
use tl::ParserOptions;
use tokio::sync::{Semaphore, mpsc::{UnboundedReceiver, UnboundedSender}};

use crate::{crawler::crawl_result::CrawlResult, url_rules::{filter::resolve, should_crawl}};

pub async fn crawl_from_seed(
    client: Arc<Client>, 
    raw_channel_tx: UnboundedSender<Url>, 
    mut filtered_rx: UnboundedReceiver<Url>, 
    crawled_channel_tx: UnboundedSender<CrawlResult>,
    seeds: Vec<Url>
    ) 
{
    let semaphore = Arc::new(Semaphore::new(1000));
    let mut crawled_count = 0;
    
    for seed in seeds {
        let _ = raw_channel_tx.send(seed); 
    }

    loop {
        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .unwrap();
        
        let domain_to_crawl: Url = filtered_rx.recv().await.expect("Could not get Url from Orc Channel");
        let raw_tx = raw_channel_tx.clone();
        let crawled_tx = crawled_channel_tx.clone();
        let client = client.clone();

        crawled_count += 1;

        log::info!("Total crawled: {}, Permit granted for: {}", crawled_count, domain_to_crawl);

        tokio::spawn(async move {
            crawl_domain(client, domain_to_crawl, raw_tx, crawled_tx).await;
            drop(permit);
        });
    }
}

pub async fn crawl_domain(client: Arc<Client>, domain: Url, found_domains_channel: UnboundedSender<Url>, crawled: UnboundedSender<CrawlResult>){
    let mut seen: Bloom<String> = Bloom::new_for_fp_rate(100_000, 0.01).unwrap();
    let mut links: Vec<String> = vec!["/".to_string()];
    let mut total_time_ms = 0;
    let mut links_crawled = 0;
    
    while let Some(link_to_crawl) = links.pop() {
        let full_link = match domain.join(&link_to_crawl) {
            Ok(url) => url,
            Err(_) => continue,
        };

        let (new_links, ttfb) = match get_links(&client, &full_link).await {
            Some((links, ttfb)) => (links, ttfb),
            None => continue
        };

        links_crawled += 1;
        total_time_ms += ttfb;
        
        for link in new_links {

            let Some(resolved_url) = resolve(&domain, &link) else { continue };

            if !should_crawl(&resolved_url) {
                continue;
            }

            if resolved_url.host() != domain.host() {
                let _ = found_domains_channel.send(resolved_url);
            } else {
                let path = resolved_url.path().to_string();
            
                if links.len() < 1000 && !seen.check_and_set(&path) {
                    links.push(path);
                }
            }
        }

        if links_crawled > 50000 {
            break;
        }
    }
    
    let average_ttfb_ms = match links_crawled {
        0 => 0,
        _ => total_time_ms / links_crawled
    };

    let results = CrawlResult {
        url: domain.to_string(),
        average_ttfb_ms: average_ttfb_ms,
        links_crawled: links_crawled
    };

    let _ = crawled.send(results);
}

async fn get_links(client: &Client, url: &Url) -> Option<(Vec<String>, i32)> {
    let request_begin = Instant::now();
   let response = match client.get(url.clone()).send().await {
    Ok(resp) => resp,
    Err(e) => {
        log::warn!("fetch failed for {url}: {e:?}");
        return None;
    }
};
    let ttfb = request_begin.elapsed().as_millis() as i32;

    if abort_parsing(&response) {
        return None
    }

    let Ok(body) = response.text().await else { return None };

    let Ok(dom) = tl::parse(&body, ParserOptions::default()) else { return None };
    let parser = dom.parser();

    let found_urls = dom
        .query_selector("a[href]")?
        .filter_map(|handle| {
            let node = handle.get(parser)?;
            let tag = node.as_tag()?;
            let href = tag.attributes().get("href")??.as_utf8_str().to_string();
            Some(href)
        })
        .collect::<Vec<String>>();

    Some((found_urls, ttfb))
}

fn abort_parsing(response: &Response) -> bool {
    let content_length = response.content_length().unwrap_or(1024 * 1024);
    let content_type = response.headers().get(CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("");

    if content_length > 1024 * 1024 * 1024 {
        log::warn!("Aborted: Too large");
        return true
    }

    if !content_type.contains("text/html") {
        log::warn!("Aborted: Not HTML, but {}", content_type);
        return true
    }

    false
}
