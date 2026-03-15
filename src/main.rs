use std::sync::Arc;
use reqwest::{Client};
use sea_orm::{Database, DbConn};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use crate::crawler::crawl_result::CrawlResult;
use crate::crawler::{crawler::crawl_from_seed, filter::filter_domains};
use crate::db::operations::add_website;

mod db;
mod crawler;

#[tokio::main]
async fn main() {
    let client = Arc::new(Client::new());

    let db: DbConn = Database::connect("postgresql://crawler:crawler@localhost:5432/crawler")
        .await
        .expect("Database connection failed");

    db.get_schema_registry(module_path!().split("::").next().unwrap())
        .sync(&db)
        .await
        .expect("Failed to sync schema");

    let (raw_tx, raw_rx) = mpsc::unbounded_channel();
    let (filtered_tx, filtered_rx) = mpsc::unbounded_channel();
    let (crawled_tx, crawled_rx) = mpsc::unbounded_channel::<CrawlResult>();

    tokio::spawn(filter_domains(raw_rx, filtered_tx));
    tokio::spawn(save_websites(db, crawled_rx));

    crawl_from_seed(client, raw_tx, filtered_rx, crawled_tx).await;
}

async fn save_websites(db: DbConn, mut crawled: UnboundedReceiver<CrawlResult>) {
    loop {
        let crawled_domain = crawled.recv().await.expect("Error while getting domain from database receiver");
        add_website(&db, crawled_domain.url, crawled_domain.average_ttfb_ms, crawled_domain.links_crawled).await;
    }
}
