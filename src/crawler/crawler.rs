use std::{collections::HashSet, sync::Arc, time::Instant};

use lol_html::{HtmlRewriter, Settings, element};
use reqwest::{Client, Url};
use tokio::sync::{Semaphore, mpsc::{UnboundedReceiver, UnboundedSender}};

use crate::crawler::{crawl_result::CrawlResult, filter::should_crawl};

pub async fn crawl_from_seed(client: Arc<Client>, raw_tx: UnboundedSender<Url>, mut filtered_rx: UnboundedReceiver<Url>, crawled: UnboundedSender<CrawlResult>) {
    let semaphore = Arc::new(Semaphore::new(100));
    let mut crawled_count = 0;
    let seeds = get_seeds();
    
    for seed in seeds {
        let _ = raw_tx.send(seed); 
    }

    loop {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let domain_to_crawl: Url = filtered_rx.recv().await.expect("Could not get Url from Orc Channel");
        let raw_sender = raw_tx.clone();
        let crawled = crawled.clone();
        let client = client.clone();

        crawled_count += 1;

        println!("Queued: {}", filtered_rx.len());
        println!("Available permits: {}", semaphore.available_permits());
        println!("Crawled: {}", crawled_count);

        tokio::spawn(async move {
            crawl_domain(client, domain_to_crawl, raw_sender, crawled).await;
            drop(permit);
        });
    }
}

pub async fn crawl_domain(client: Arc<Client>, domain: Url, found_domains_channel: UnboundedSender<Url>, crawled: UnboundedSender<CrawlResult>){
    let mut seen: HashSet<String> = HashSet::new();
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
            
            match should_crawl(domain.authority(), &link) {
                Some(url) => {
                    if url.host() != domain.host() {
                        let _ = found_domains_channel.send(url);
                        continue;
                    } 
                    let path = url.path().to_string();
                    
                    if !seen.contains(&path) {
                        seen.insert(path.clone());
                        links.push(path);
                    }                
                },
                _ => continue
            }
        }
    }
    
    let average_ttfb_ms = match links_crawled {
        0 => 0,
        _ => total_time_ms / links_crawled
    };

    let results = CrawlResult {
        url: domain.to_string(),
        average_ttfb_ms: average_ttfb_ms as i32,
        links_crawled: links_crawled as i32
    };

    let _ = crawled.send(results);
}

async fn get_links(client: &Client, url: &Url) -> Option<(Vec<String>, u128)> {
    let request_begin = Instant::now();

    let Ok(response) = client.get(url.clone()).send().await else { return None };
    let Ok(body) = response.bytes().await else { return None };

    let ttfb = request_begin.elapsed().as_millis();

    let mut found_urls: Vec<String> = Vec::new();
    let urls_ptr = &mut found_urls as *mut Vec<String>;

    let mut rewriter = HtmlRewriter::new(
        Settings{
            element_content_handlers: vec![element!("a[href]", move |el| {
                if let Some(link) = el.get_attribute("href") {
                    unsafe { (*urls_ptr).push(link) };
                }
                Ok(())
            })],
            ..Settings::default()
        },
        |_: &[u8]| (),
    );

    let _ = rewriter.write(&body);
    let _ = rewriter.end();

    Some((found_urls, ttfb))
}

fn get_seeds() -> Vec<Url> {
    vec![
        // News
        "https://dr.dk",
        "https://tv2.dk",
        "https://berlingske.dk",
        "https://politiken.dk",
        "https://ekstrabladet.dk",
        "https://bt.dk",
        "https://information.dk",
        "https://finans.dk",
        "https://arbejderen.dk",
        "https://jyllands-posten.dk",
        "https://kristeligt-dagblad.dk",
        "https://weekendavisen.dk",
        "https://lokalavisen.dk",
        "https://tv2nord.dk",
        "https://tv2ostjylland.dk",
        "https://tv2lorry.dk",
        "https://tv2fyn.dk",
        "https://tv2east.dk",
        "https://tv2kosmopol.dk",
        "https://nordjyske.dk",
        // Tech & Business
        "https://version2.dk",
        "https://computerworld.dk",
        "https://ing.dk",
        "https://itek.dk",
        "https://detdigitaleselskab.dk",
        "https://erhvervsliv.dk",
        "https://borsen.dk",
        "https://finanswatch.dk",
        "https://epn.dk",
        "https://startupcentral.dk",
        // Classifieds & Marketplaces
        "https://dba.dk",
        "https://guloggratis.dk",
        "https://bilbasen.dk",
        "https://autouncle.dk",
        "https://boligsiden.dk",
        "https://edc.dk",
        "https://nybolig.dk",
        "https://home.dk",
        "https://danbolig.dk",
        "https://lejebolig.dk",
        // Forums & Communities
        "https://eksperten.dk",
        "https://hifi4all.dk",
        "https://forums.dk",
        "https://amino.dk",
        "https://babyverden.dk",
        "https://motorsite.dk",
        "https://detektivforumet.dk",
        "https://outdoor.dk",
        "https://sportscykler.dk",
        "https://cykelmagasinet.dk",
        // Government & Public
        "https://denmark.dk",
        "https://politi.dk",
        "https://skat.dk",
        "https://borger.dk",
        "https://sundhed.dk",
        "https://retsinformation.dk",
        "https://dst.dk",
        "https://dtu.dk",
        "https://ku.dk",
        "https://au.dk",
        "https://sdu.dk",
        "https://ruc.dk",
        "https://cbs.dk",
        "https://itu.dk",
        "https://aau.dk",
        // Retail & Shopping
        "https://pricerunner.dk",
        "https://elgiganten.dk",
        "https://power.dk",
        "https://coolshop.dk",
        "https://av-cables.dk",
        "https://komplett.dk",
        "https://computersalg.dk",
        "https://planetz.dk",
        "https://hm.com",
        "https://zalando.dk",
        "https://boozt.com",
        "https://magasin.dk",
        "https://illum.dk",
        // Health & Lifestyle
        "https://helse.dk",
        "https://netdoktor.dk",
        "https://apoteket.dk",
        "https://bodylab.dk",
        "https://medicinraadet.dk",
        "https://cancer.dk",
        "https://hjerteforeningen.dk",
        "https://diabetesforeningen.dk",
        // Sports
        "https://dbu.dk",
        "https://dif.dk",
        "https://bold.dk",
        "https://tipsbladet.dk",
        "https://superliga.dk",
        "https://speedwaynews.dk",
        "https://cykling.dk",
        // Misc high-link-count
        "https://denstoredanske.dk",
        "https://jobindex.dk",
        "https://ofir.dk",
        "https://careerjet.dk",
        "https://rejseplanen.dk",
        "https://danskebank.dk",
        "https://nordea.dk",
        "https://jyskebank.dk",
    ]
    .into_iter()
    .filter_map(|u| Url::parse(u).ok())
    .collect()
}
