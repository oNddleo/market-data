use std::sync::Arc;
use clap::Parser;
use tracing::{info, error};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use market_depth_server::{StreamManager, WebSocketHandler};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// WebSocket server address
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    addr: String,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize tracing
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&args.log_level));

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Market Depth Server");
    info!("Log level: {}", args.log_level);

    // Create stream manager
    let stream_manager = Arc::new(StreamManager::new());

    // Start stream manager background tasks
    stream_manager.start().await;

    // Create and start WebSocket handler
    let ws_handler = WebSocketHandler::new(Arc::clone(&stream_manager));

    info!("Server starting on: {}", args.addr);

    // Start the server
    if let Err(e) = ws_handler.start(&args.addr).await {
        error!("Server error: {}", e);
        return Err(e);
    }

    Ok(())
}