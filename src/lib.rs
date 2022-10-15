use std::time::Duration;
use reqwest::{Response, StatusCode, Client};
use sha2::{Digest, Sha224};
use tokio::time;
use clap::Parser;

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

    pub fn invalid() -> UrlResponse {
        UrlResponse { status: StatusCode::NO_CONTENT, body: String::from("") }
    }

    pub fn create_hash<D>(&self, mut hasher: D) -> String
    where
        D: Digest,
        digest::Output<D>: std::fmt::LowerHex,
    {
        hasher.update(&self.body);
        format!("{:x}", hasher.finalize())
    }

    pub fn is_valid(&self) -> bool {
        self.status == StatusCode::OK
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    #[arg(short, long)]
    pub web_address: String,

    #[arg(short, long, default_value_t = 30)]
    pub check_interval_sec: u64,

    #[arg(short, long, default_value_t = 10)]
    pub max_num_of_failures: u64,
}

pub struct Config {
    pub web_address: String,
    pub check_interval: Duration,
    pub max_fail_count: u64
}

impl Config {
    pub fn from(args: CliArgs) -> Config {
        Config { 
            web_address: args.web_address, 
            check_interval: Duration::from_secs(args.check_interval_sec), 
            max_fail_count: args.max_num_of_failures 
        }
    }
}

pub struct ChangeMonitor {
    config: Config,
    client: Client,
    previous_hash: String,
    consecutive_fail_count: u64,
}

impl ChangeMonitor {
    pub fn from(config: Config, client: Client) -> ChangeMonitor {
        ChangeMonitor {
            config,
            client,
            previous_hash: String::new(),
            consecutive_fail_count: 0
        }
    }

    pub async fn request_url_response(&self) -> Result<UrlResponse, reqwest::Error> {
        let request = self.client.get(&self.config.web_address).send().await?;
        let url_response = UrlResponse::from(request).await?;

        Ok(url_response)
    }

    pub fn is_hash_changed(&self, response: UrlResponse) -> (bool, String) {
        let hasher = Sha224::new();
        let current_hash = response.create_hash(hasher);

        let hash_changed = current_hash != self.previous_hash;

        (hash_changed, current_hash)
    }

    pub fn set_previous_hash(&mut self, hash: String) {
        self.previous_hash = hash;
    }

    pub fn reset_failure_count(&mut self) {
        self.consecutive_fail_count = 0;
    }

    pub fn increment_failure_count(&mut self) {
        self.consecutive_fail_count = self.consecutive_fail_count + 1;
    }

    pub async fn wait_for_check_interval(&self) {
        time::sleep(self.config.check_interval).await;
    }

    pub fn max_fail_count_reached(&self) -> bool {
        &self.consecutive_fail_count == &self.config.max_fail_count
    }
}