use std::sync::Arc;
use std::time::Duration;
use axum::{
    extract::{Query, State},
    response::Sse,
    http::StatusCode,
};
use axum::response::sse::{Event, KeepAlive};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use tracing::{info, debug, error};
use futures::stream::Stream;
use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::stream_manager::SSEStreamManager;
use crate::message::{SSEMessage, StreamQuery};

#[pin_project]
pub struct SSEStream {
    #[pin]
    inner: UnboundedReceiverStream<SSEMessage>,
    client_id: Uuid,
    stream_manager: Arc<SSEStreamManager>,
}

impl SSEStream {
    pub fn new(
        receiver: mpsc::UnboundedReceiver<SSEMessage>,
        client_id: Uuid,
        stream_manager: Arc<SSEStreamManager>,
    ) -> Self {
        Self {
            inner: UnboundedReceiverStream::new(receiver),
            client_id,
            stream_manager,
        }
    }
}

impl Stream for SSEStream {
    type Item = Result<Event, axum::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match this.inner.poll_next(cx) {
            Poll::Ready(Some(message)) => {
                let event = match &message {
                    SSEMessage::MarketData { stream_id, .. } => {
                        Event::default()
                            .event("market_data")
                            .data(serde_json::to_string(&message).unwrap_or_default())
                            .id(stream_id)
                    }
                    SSEMessage::HeartBeat { .. } => {
                        Event::default()
                            .event("heartbeat")
                            .data(serde_json::to_string(&message).unwrap_or_default())
                    }
                    SSEMessage::ConnectionInfo { .. } => {
                        Event::default()
                            .event("connection_info")
                            .data(serde_json::to_string(&message).unwrap_or_default())
                    }
                    SSEMessage::Error { .. } => {
                        Event::default()
                            .event("error")
                            .data(serde_json::to_string(&message).unwrap_or_default())
                    }
                };
                Poll::Ready(Some(Ok(event)))
            }
            Poll::Ready(None) => {
                // Stream ended, clean up
                this.stream_manager.unregister_client(this.client_id);
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}


pub async fn sse_handler(
    Query(query): Query<StreamQuery>,
    State(stream_manager): State<Arc<SSEStreamManager>>,
) -> Result<Sse<SSEStream>, StatusCode> {
    let client_id = Uuid::new_v4();
    let (tx, rx) = mpsc::unbounded_channel();

    // Register the client
    stream_manager.register_client(client_id, tx);

    // Send connection info
    stream_manager.send_connection_info(client_id).await;

    // Parse and subscribe to requested streams
    let stream_definitions = query.parse_streams();
    if !stream_definitions.is_empty() {
        match stream_manager
            .subscribe_to_streams(client_id, stream_definitions)
            .await
        {
            Ok(()) => {
                info!("Client {} subscribed to requested streams", client_id);
            }
            Err(e) => {
                error!("Failed to subscribe client {} to streams: {}", client_id, e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    } else {
        // If no specific streams requested, subscribe to default BTCUSD MBP
        let default_streams = vec![("BTCUSD".to_string(), crate::message::DataType::MBP, 20)];
        if let Err(e) = stream_manager
            .subscribe_to_streams(client_id, default_streams)
            .await
        {
            error!("Failed to subscribe client {} to default streams: {}", client_id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    let sse_stream = SSEStream::new(rx, client_id, Arc::clone(&stream_manager));

    Ok(Sse::new(sse_stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
}

pub async fn health_check() -> &'static str {
    "SSE Market Depth Server is running"
}

pub async fn symbols_handler(
    State(stream_manager): State<Arc<SSEStreamManager>>,
) -> Result<axum::Json<Vec<String>>, StatusCode> {
    let symbols = stream_manager.get_symbols().await;
    Ok(axum::Json(symbols))
}

pub async fn api_info() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "name": "Market Depth SSE Server",
        "version": "0.1.0",
        "endpoints": {
            "/stream": {
                "method": "GET",
                "description": "SSE endpoint for market data streams",
                "parameters": {
                    "streams": "Comma-separated stream definitions (symbol:type:levels): BTCUSD:MBP:20,ETHUSD:MBO:10",
                    "symbols": "Comma-separated symbols: BTCUSD,ETHUSD (uses default type and levels)",
                    "data_type": "Default data type: MBP or MBO (default: MBP)",
                    "max_levels": "Default max levels (default: 20)"
                },
                "examples": [
                    "/stream?streams=BTCUSD:MBP:20,ETHUSD:MBO:10",
                    "/stream?symbols=BTCUSD,ETHUSD&data_type=MBP&max_levels=15",
                    "/stream?symbols=BTCUSD"
                ]
            },
            "/health": {
                "method": "GET",
                "description": "Health check endpoint"
            },
            "/symbols": {
                "method": "GET",
                "description": "List available symbols"
            },
            "/api": {
                "method": "GET",
                "description": "API information (this endpoint)"
            }
        },
        "supported_symbols": ["BTCUSD", "ETHUSD", "ADAUSD"],
        "data_types": ["MBO", "MBP"],
        "sse_events": [
            "market_data",
            "heartbeat",
            "connection_info",
            "error"
        ]
    }))
}