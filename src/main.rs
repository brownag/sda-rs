use sda_rs::{app, config::Config};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env();
    let listener = TcpListener::bind(("0.0.0.0", config.port)).await?;
    axum::serve(listener, app(config)).await?;
    Ok(())
}
