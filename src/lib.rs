use std::{time::Duration, num::ParseIntError};
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
    pub max_fail_count: u64
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("No arguments provided");
        } 

        // Only address specified, use with default parameters
        if args.len() == 3 {
            let config = Config {
                web_address: args[1].clone(),
                check_interval: Duration::from_secs(30),
                max_fail_count: 10,
            };
            return Ok(config);
        }
        
        let mut web_address = String::new();
        let mut check_interval_secs = Duration::from_secs(30);
        let mut max_fail_count = 10;

        for idx in (1..args.len()).step_by(2) {
            let flag = &args[idx];
            
            if flag == "-w" {
                web_address = String::from(&args[idx + 1]);
                println!("Address: {web_address}");
                continue;
            }

            if flag == "-d" {
                let parsed_duration: Result<u64, ParseIntError> = args[idx + 1].parse();

                match parsed_duration {
                    Ok(duration) => check_interval_secs = Duration::from_secs(duration),
                    Err(_) => return Err("Failed to parse valid check duration"),
                }

                continue;
            }

            if flag == "-n" {
                let parsed_max_fail_count: Result<u64, ParseIntError> = args[idx + 1].parse();

                match parsed_max_fail_count {
                    Ok(count) => max_fail_count = count,
                    Err(_) => return Err("Failed to parse valid max fail count"),
                }

                continue;
            }
        }

        Ok(Config {
            web_address: web_address,
            check_interval: check_interval_secs,
            max_fail_count: max_fail_count,
        })
    }
}