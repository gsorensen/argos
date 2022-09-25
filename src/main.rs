use std::env;
use std::process;
use reqwest::StatusCode;
use sha2::{Sha224, Digest};
use async_std::task;

use argos::{UrlResponse, Config};

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // Collect arguments from the console
    let args: Vec<String> = env::args().collect();

    // Build config from console input
    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Failed to build config from console: {err}. Exiting");
        process::exit(1);
    });

    // Construct the client with default Mozilla User-Agent
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .build()?;

    // Define state (TODO: Make struct)
    let mut previous_hash = String::new();
    let mut consecutive_failure_count: u64 = 0;

    loop {
        let request = client.get(&config.web_address).send().await?;
        let url_response = UrlResponse::from(request).await?;

        if url_response.status == StatusCode::OK {
            consecutive_failure_count = 0;
            let hasher = Sha224::new();
            let current_hash = url_response.create_hash(hasher);
    
            if previous_hash != current_hash {
                println!("Change in website contents since last check");
            } else {
                println!("No change since last time");
            }
    
            previous_hash = current_hash;
        } else {
            consecutive_failure_count = consecutive_failure_count + 1;
            eprintln!("Request failed with status code: {}", url_response.status.as_str());
        }
        
        if consecutive_failure_count == config.max_fail_count {
            eprintln!("Max number of consecutive failures reached. Exiting program");
            process::exit(1);
        }

        task::sleep(config.check_interval).await;
    }
}
