use std::process;

use argos::{CliArgs, UrlResponse, Config, ChangeMonitor};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // Collect arguments from the console
    let args = CliArgs::parse();

    // Build config from console input
    let config = Config::from(args);

    // Construct the client with default Mozilla User-Agent
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .build()?;

    // Create the monitor object, which contains program state
    let mut monitor = ChangeMonitor::from(config, client);

    loop {
        let url_response = monitor.request_url_response().await.unwrap_or_else(|err| {
            eprintln!("Failed to request URL repsonse {err}");
            UrlResponse::invalid()
        });

        if url_response.is_valid() {
            monitor.reset_failure_count();
            let (site_has_changed, current_hash) = monitor.is_hash_changed(url_response);

            if site_has_changed {
                monitor.set_previous_hash(current_hash);
                println!("Change in website contents since last check");
            } else {
                println!("No change since last time");
            }
        } else {
            monitor.increment_failure_count();
            eprintln!("Request failed with status code: {}", url_response.status.as_str());
        }
        
        if monitor.max_fail_count_reached() {
            eprintln!("Max number of consecutive failures reached. Exiting program");
            process::exit(1);
        }

        monitor.wait_for_check_interval().await;
    }
}
