use bloomfilter::Bloom;
use reqwest::Url;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub async fn filter_domains(mut raw_receiver: UnboundedReceiver<Url>, filtered_sender: UnboundedSender<Url>) {
    let mut seen: Bloom<String> = Bloom::new_for_fp_rate(1_000_000, 0.01).unwrap();
    
    loop {
        let url = match raw_receiver.recv().await {
            Some(url) => url,
            None => return,
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
