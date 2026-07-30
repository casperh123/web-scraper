use std::{sync::Arc, time::{Instant}};
use bloomfilter::Bloom;
use reqwest::{Client, Response, Url, header::CONTENT_TYPE};
use tl::ParserOptions;
use tokio::sync::{Semaphore, mpsc::{UnboundedReceiver, UnboundedSender}};

use crate::{crawler::crawl_result::CrawlResult, url_rules::{filter::resolve_full_url, should_crawl}};

pub async fn crawl_from_seed(
    client: Arc<Client>, 
    raw_channel_tx: UnboundedSender<Url>, 
    mut filtered_rx: UnboundedReceiver<Url>, 
    crawled_channel_tx: UnboundedSender<CrawlResult>,
    seeds: Vec<Url>
    ) 
{
    let semaphore = Arc::new(Semaphore::new(300));
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
    let mut seen: Bloom<str> = Bloom::new_for_fp_rate(200_000, 0.01).unwrap();
    let mut links: Vec<String> = vec!["/".to_string()];
    let mut total_time_ms = 0;
    let mut links_crawled = 0;
    
    while let Some(link_to_crawl) = links.pop() {
        let full_link = match domain.join(&link_to_crawl) {
            Ok(url) => url,
            Err(_) => continue,
        };

        let (response, ttfb) = match request_page(&client, &full_link).await {
            Ok(result) => result,
            Err(e) => {
                log::warn!("fetch failed for {full_link}: {e:?}");
                continue;
            }
        };

        if abort_parsing(&response) {
            continue;
        }

        let response_body = match response.text().await {
            Ok(body) => body,
            Err(e) => {
                log::info!("Failed to parse body for {full_link}: {e:?}");
                continue;
            },
        };

        if process_links(&response_body, &domain, &mut seen, &mut links, &found_domains_channel).is_none() {
            continue;
        }

        links_crawled += 1;
        total_time_ms += ttfb;

        if(links_crawled > 50_000) {
            break;
        }
    }
    
    let average_ttfb_ms = match links_crawled {
        0 => 0,
        _ => total_time_ms / links_crawled
    };

    let results = CrawlResult {
        url: domain.to_string(),
        average_ttfb_ms,
        links_crawled,
    };

    let _ = crawled.send(results);
}

async fn request_page(client: &Client, url: &Url) -> Result<(Response, i32), reqwest::Error> {
    let request_begin = Instant::now();
    let response = client.get(url.clone()).send().await;
    let ttfb = request_begin.elapsed().as_millis() as i32; 

    Ok((response?, ttfb))
}

fn process_links(
    body: &str,
    domain: &Url,
    seen: &mut Bloom<str>,
    links: &mut Vec<String>,
    found_domains_channel: &UnboundedSender<Url>,
) -> Option<()> {
    let dom = tl::parse(body, ParserOptions::default()).ok()?;
    let parser = dom.parser();

    for handle in dom.query_selector("a[href]")? {
        let Some(node) = handle.get(parser) else { continue };
        let Some(tag) = node.as_tag() else { continue };
        let Some(Some(href_bytes)) = tag.attributes().get("href") else { continue };
        let href = href_bytes.as_utf8_str(); 
        let Some(resolved_url) = resolve_full_url(domain, &href) else { continue };

        if !should_crawl(&resolved_url) {
            continue;
        }

        if resolved_url.host() != domain.host() {
            let _ = found_domains_channel.send(resolved_url);
        } else {
            let path = resolved_url.path();
            if !seen.check_and_set(path) {
                links.push(path.to_string());            
            }
        }
    }

    Some(())
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
