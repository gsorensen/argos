use std::time::Duration;
use reqwest::{Response, StatusCode};
use sha2::Digest;

pub struct UrlResponse {
    pub status: StatusCode,
    pub body: String,
}

impl UrlResponse {
    pub async fn from(response: Response) -> Result<UrlResponse, reqwest::Error> {
        let url_response = UrlResponse {
            status: response.status(),
            body: response.text().await?,
        };

        Ok(url_response)
    }

    pub fn create_hash<D>(&self, mut hasher: D) -> String
    where
        D: Digest,
        digest::Output<D>: std::fmt::LowerHex,
    {
        hasher.update(&self.body);
        format!("{:x}", hasher.finalize())
    }
}

pub struct Config {
    pub web_address: String,
    pub check_interval: Duration,
    pub max_fail_count: u32
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("No arguments provided");
        } 

        if args.len() == 3 {
            let config = Config {
                web_address: args[1].clone(),
                check_interval: Duration::from_secs(30),
                max_fail_count: 10,
            };
            return Ok(config);
        }

        Err("test")
    }
}