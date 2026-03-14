use std::sync::{Arc};
use reqwest::Client;
use tokio::sync::{mpsc};
use crate::crawler::crawler::crawl_from_seed;
use crate::crawler::filter::{filter_domains};

mod crawler;

#[tokio::main()]
async fn main() {
    let client = Arc::new(Client::new());
    let (raw_tx, raw_rx) = mpsc::unbounded_channel();
    let (filtered_tx, filtered_rx) = mpsc::unbounded_channel();

    tokio::spawn(filter_domains(raw_rx, filtered_tx));

    crawl_from_seed(client, raw_tx, filtered_rx).await;
}


