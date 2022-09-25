use std::env;
use std::process;
use reqwest::{Response, StatusCode};
use sha2::{Digest, Sha224};

struct UrlResponse {
    status: StatusCode,
    body: String,
}

impl UrlResponse {
    async fn from(response: Response) -> Result<UrlResponse, reqwest::Error> {
        let url_response = UrlResponse {
            status: response.status(),
            body: response.text().await?,
        };

        Ok(url_response)
    }

    fn create_hash<D>(&self, mut hasher: D) -> String
    where
        D: Digest,
        digest::Output<D>: std::fmt::LowerHex,
    {
        hasher.update(&self.body);
        format!("{:x}", hasher.finalize())
    }
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Need to provide a website to monitor");
        process::exit(1);
    }

    let web_address = &args[1];

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .build()?;

    let request = client.get(web_address).send().await?;

    let url_response = UrlResponse::from(request).await?;


    let hasher = Sha224::new();
    let hash = url_response.create_hash(hasher);

    println!("Status: {}", url_response.status);
    println!("Hash: {hash}");
    Ok(())
}
