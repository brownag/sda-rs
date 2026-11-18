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
    let config = Config::from_env();
    let listener = TcpListener::bind(("0.0.0.0", config.port)).await?;
    axum::serve(listener, app(config)).await?;
    Ok(())
}
