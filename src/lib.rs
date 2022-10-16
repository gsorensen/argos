use clap::Parser;
use reqwest::{Client, Response, StatusCode};
use sha2::{Digest, Sha224};
use std::process;
use std::time::Duration;
use tokio::time;

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
        UrlResponse {
            status: StatusCode::NO_CONTENT,
            body: String::from(""),
        }
    }

    pub fn hash(&self) -> String {
        let mut hasher = Sha224::new();
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
    pub max_fail_count: u64,
}

impl Config {
    pub fn from(args: CliArgs) -> Config {
        Config {
            web_address: args.web_address,
            check_interval: Duration::from_secs(args.check_interval_sec),
            max_fail_count: args.max_num_of_failures,
        }
    }
}

pub struct EyeOfArgos {
    config: Config,
    client: Client,
    previous_hash: String,
    consecutive_fail_count: u64,
}

impl EyeOfArgos {
    pub fn from(config: Config, client: Client) -> EyeOfArgos {
        EyeOfArgos {
            config,
            client,
            previous_hash: String::new(),
            consecutive_fail_count: 0,
        }
    }

    pub async fn watch(&mut self) -> Result<(), reqwest::Error> {
        loop {
            let url_response = self.request_url_response().await.unwrap_or_else(|err| {
                eprintln!("Failed to request URL response {err}");
                UrlResponse::invalid()
            });

            if url_response.is_valid() {
                self.consecutive_fail_count = 0;
                let current_hash = url_response.hash();

                if current_hash != self.previous_hash {
                    self.previous_hash = current_hash;
                    println!("Change in website content since last check!");
                } else {
                    println!("No change since last time");
                }
            } else {
                self.consecutive_fail_count += 1;
                eprintln!(
                    "Request failed with status code {}",
                    url_response.status.as_str()
                );
            }

            if self.consecutive_fail_count == self.config.max_fail_count {
                eprintln!("Max number of consecutive failures reached. Exiting program");
                process::exit(1);
            }

            time::sleep(self.config.check_interval).await;
        }
    }

    async fn request_url_response(&self) -> Result<UrlResponse, reqwest::Error> {
        let request = self.client.get(&self.config.web_address).send().await?;
        let url_response = UrlResponse::from(request).await?;

        Ok(url_response)
    }
}
