use clap::Parser;
use sda_rs::{app, config::Config};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let (arguments, bind_address) = sda_rs::config::extract_bind_address(std::env::args_os())?;
    let config = Config::parse_from(arguments);
    let listener = TcpListener::bind((bind_address.as_str(), config.port)).await?;
    axum::serve(listener, app(config)).await?;
    Ok(())
}
