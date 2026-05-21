use oclapp_core::error::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("oClapp CLI starting...");
    // TODO: Implement CLI with clap subcommands (launch, serve, status)
    Ok(())
}
