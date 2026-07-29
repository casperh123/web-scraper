use bloomfilter::Bloom;
use reqwest::Url;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub async fn filter_domains(mut raw_receiver: UnboundedReceiver<Url>, filtered_sender: UnboundedSender<Url>) {
    let mut seen: Bloom<str> = Bloom::new_for_fp_rate(1_000_000, 0.01).unwrap();
    
    loop {
        let url = match raw_receiver.recv().await {
            Some(url) => url,
            None => return,
        };

        let Some(host) = url.host_str() else { continue };
        let host = host.strip_prefix("www.").unwrap_or(host);
  
        if seen.check_and_set(host) {
            continue;       
        }

        let host_url = match Url::parse(&format!("https://{}", host)) {
            Ok(u) => u,
            Err(_) => continue,
        };

        let _ = filtered_sender.send(host_url);
    }
}
