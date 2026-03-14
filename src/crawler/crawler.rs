use std::{collections::HashSet, sync::Arc};

use lol_html::{HtmlRewriter, Settings, element};
use reqwest::{Client, Url};
use tokio::sync::{Semaphore, mpsc::{UnboundedReceiver, UnboundedSender}};

use crate::crawler::filter::should_crawl;

pub async fn crawl_from_seed(client: Arc<Client>, raw_tx: UnboundedSender<Url>, mut filtered_rx: UnboundedReceiver<Url>, crawled: UnboundedSender<Url>) {
    let semaphore = Arc::new(Semaphore::new(1000));
            let mut crawled_count = 0;

    let seeds = get_seeds();
    for seed in seeds {
        let _ = raw_tx.send(seed); 
    }

    loop {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let domain_to_crawl: Url = filtered_rx.recv().await.expect("Could not get Url from Orc Channel");
        let raw_sender = raw_tx.clone();
        let client = client.clone();

        let _ = crawled.send(domain_to_crawl.clone());

        crawled_count += 1;

        println!("Queued: {}", filtered_rx.len());
        println!("Available permits: {}", semaphore.available_permits());
        println!("Crawled: {}", crawled_count);

        tokio::spawn(async move {
            crawl_domain(client, domain_to_crawl, raw_sender).await;
            drop(permit);
        });
    }
}

pub async fn crawl_domain(client: Arc<Client>, domain: Url, found_domains_channel: UnboundedSender<Url>) {
    let mut seen: HashSet<Url> = HashSet::new();
    let mut links: Vec<Url> = vec![domain.clone()];

    
    while let Some(link_to_crawl) = links.pop() {
        let new_links = get_links(&client, &link_to_crawl).await;
        
        for link in new_links {
            if link.host() != domain.host() {
                let _ = found_domains_channel.send(link);
                continue;
            }

            if should_crawl(&link) && seen.insert(link.clone()) {
                links.push(link);
            }
        }
    }
}

async fn get_links(client: &Client, url: &Url) -> Vec<Url> {
    let Ok(response) = client.get(url.clone()).send().await else { return vec![] };
    let Ok(body) = response.bytes().await else { return vec![] };

    let mut found_urls: Vec<Url> = Vec::new();
    let urls_ptr = &mut found_urls as *mut Vec<Url>;
    let base_url = url.clone();

    let mut rewriter = HtmlRewriter::new(
        Settings{
            element_content_handlers: vec![element!("a[href]", move |el| {
                if let Some(link) = el.get_attribute("href") {
                    let resolved = Url::parse(&link)
                        .ok()
                        .or_else(|| base_url.join(&link).ok());
                    if let Some(resolved_url) = resolved {
                        unsafe { (*urls_ptr).push(resolved_url) };
                    }
                }
                Ok(())
            })],
            ..Settings::default()
        },
        |_: &[u8]| (),
    );

    let _ = rewriter.write(&body);
    let _ = rewriter.end();

    found_urls
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
