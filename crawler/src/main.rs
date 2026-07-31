use std::{sync::Arc, time::Duration};
use crawler_demo::{crawler::{crawl_result::CrawlResult, crawler::crawl_from_seed, discovery::filter_domains, seeds::get_seeds}};
use database::db::{connection::{add_website, connection}, website_operations::add_website};
use reqwest::{Client};
use sea_orm::DatabaseConnection;
use tokio::sync::mpsc::{self, UnboundedReceiver};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let client = Arc::new(
        Client::builder()
            .timeout(Duration::from_secs(30))
            .hickory_dns(true)
            .build()
            .expect("failed to build reqwest client")
    );

    let db = connection()
        .await
        .expect("Could not connect to DB");

    env_logger::init();   

    let (raw_tx, raw_rx) = mpsc::unbounded_channel();
    let (filtered_tx, filtered_rx) = mpsc::unbounded_channel();
    let (crawled_tx, crawled_rx) = mpsc::unbounded_channel::<CrawlResult>();

    tokio::spawn(filter_domains(raw_rx, filtered_tx));
    tokio::spawn(save_websites(&db, crawled_rx));

    crawl_from_seed(client, raw_tx, filtered_rx, crawled_tx, get_seeds()).await;
}

async fn save_websites(db: &DatabaseConnection, mut crawled: UnboundedReceiver<CrawlResult>) {
    loop {
        let crawled_domain = crawled.recv().await.expect("Error while getting domain from database receiver");
        let _ = add_website(&db, crawled_domain.url, crawled_domain.average_ttfb_ms, crawled_domain.links_crawled).await;
    }
}
