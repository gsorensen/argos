use std::env;
use std::process;
use reqwest::StatusCode;
use sha2::{Sha224, Digest};
use std::time::Duration;
use async_std::task;

use argos::UrlResponse;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // Collect website from console input
    let args: Vec<String> = env::args().collect();

    // Verify that we have recieved some input (should probably also verify URL is valid)
    if args.len() < 2 {
        eprintln!("Need to provide a website to monitor");
        process::exit(1);
    }

    // Extract web address
    let web_address = &args[1];

    // Construct the client with default Mozilla User-Agent
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .build()?;

    let mut previous_hash = String::new();
    
    let check_interval = Duration::from_secs(30);

    let mut consecutive_failure_count = 0;
    let max_failure_count = 10;

    loop {
        let request = client.get(web_address).send().await?;
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
        
        if consecutive_failure_count == max_failure_count {
            eprintln!("Max number of consecutive failures reached. Exiting program");
            process::exit(1);
        }

        task::sleep(check_interval).await;
    }
}
