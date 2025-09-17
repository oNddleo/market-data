use std::sync::Arc;
use axum::{
    routing::get,
    Router,
};
use clap::Parser;
use tower_http::cors::{CorsLayer, Any};
use tracing::{info, error};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use market_depth_sse_server::{SSEStreamManager, sse_handler, health_check, symbols_handler, api_info};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Server address
    #[arg(short, long, default_value = "127.0.0.1:8081")]
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

    info!("Starting Market Depth SSE Server");
    info!("Log level: {}", args.log_level);

    // Create stream manager
    let stream_manager = Arc::new(SSEStreamManager::new());

    // Start stream manager background tasks
    stream_manager.start().await;

    // Create CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build our application with routes
    let app = Router::new()
        .route("/stream", get(sse_handler))
        .route("/health", get(health_check))
        .route("/symbols", get(symbols_handler))
        .route("/api", get(api_info))
        .route("/", get(api_info))
        .layer(cors)
        .with_state(stream_manager);

    info!("Server starting on: {}", args.addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(&args.addr).await?;
    info!("SSE server listening on: {}", args.addr);

    axum::serve(listener, app).await?;

    Ok(())
}