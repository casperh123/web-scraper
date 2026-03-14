use std::collections::HashSet;
use std::sync::{Arc};
use lol_html::{HtmlRewriter, Settings, element};
use reqwest::Url;
use reqwest::Client;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::{Semaphore, mpsc};

#[tokio::main()]
async fn main() {
    let client = Arc::new(Client::new());
    let mut crawled_count = 0;
    let start_domain = Url::parse("https://skadedyrsexperten.dk").expect("Could not parse URL");
    let (raw_tx, raw_rx) = mpsc::unbounded_channel();
    let (filtered_tx, mut filtered_rx) = mpsc::unbounded_channel();

    let _ = raw_tx.send(start_domain);
    
    tokio::spawn(filter_domains(raw_rx, filtered_tx));
    
    let semaphore = Arc::new(Semaphore::new(300));

    let seeds = get_seeds();
    for seed in seeds {
        let _ = raw_tx.send(seed); 
    }

    loop {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let domain_to_crawl: Url = filtered_rx.recv().await.expect("Could not get Url from Orc Channel");
        let raw_sender = raw_tx.clone();
        let client = client.clone();

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

async fn filter_domains(mut raw_receiver: UnboundedReceiver<Url>, filtered_sender: UnboundedSender<Url>) {
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

async fn crawl_domain(client: Arc<Client>, domain: Url, found_domains_channel: UnboundedSender<Url>) {
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

fn should_crawl(url: &Url) -> bool {
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

async fn get_links(client: &Client, url: &Url) -> Vec<Url> {
    let Ok(response) = client.get(url.clone()).send().await else { return vec![] };
    let Ok(body) = response.bytes().await else { return vec![] };

    let mut found_urls: Vec<Url> = Vec::new();
    let urls_ptr = &mut found_urls as *mut Vec<Url>;
    let base_url = url.clone();

    let mut rewriter = HtmlRewriter::new(
        Settings {
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
