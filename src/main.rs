use argos::{CliArgs, Config, EyeOfArgos};
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
    let mut monitor = EyeOfArgos::from(config, client);

    monitor.watch().await
}
