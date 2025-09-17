use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, error, warn, debug};
use tokio::sync::mpsc;

use crate::stream_manager::StreamManager;
use crate::message::{ClientMessage, ServerMessage};

pub struct WebSocketHandler {
    stream_manager: Arc<StreamManager>,
}

impl WebSocketHandler {
    pub fn new(stream_manager: Arc<StreamManager>) -> Self {
        Self { stream_manager }
    }

    pub async fn start(&self, addr: &str) -> anyhow::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        info!("WebSocket server listening on: {}", addr);

        while let Ok((stream, peer_addr)) = listener.accept().await {
            info!("New connection from: {}", peer_addr);

            let stream_manager = Arc::clone(&self.stream_manager);
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, stream_manager).await {
                    error!("Error handling connection from {}: {}", peer_addr, e);
                }
            });
        }

        Ok(())
    }
}

async fn handle_connection(
    stream: TcpStream,
    stream_manager: Arc<StreamManager>,
) -> anyhow::Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let client_id = Uuid::new_v4();
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Register client with stream manager
    stream_manager.register_client(client_id, tx);

    info!("Client {} connected", client_id);

    // Send welcome message
    let welcome_message = ServerMessage::HeartBeat {
        timestamp: Utc::now(),
    };

    if let Ok(welcome_json) = serde_json::to_string(&welcome_message) {
        if let Err(e) = ws_sender.send(Message::Text(welcome_json)).await {
            error!("Failed to send welcome message to client {}: {}", client_id, e);
        }
    }

    // Spawn task to handle outgoing messages
    let stream_manager_clone = Arc::clone(&stream_manager);
    let client_id_clone = client_id;
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            match serde_json::to_string(&message) {
                Ok(json) => {
                    if let Err(e) = ws_sender.send(Message::Text(json)).await {
                        error!("Failed to send message to client {}: {}", client_id_clone, e);
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to serialize message for client {}: {}", client_id_clone, e);
                }
            }
        }

        // Clean up when client disconnects
        stream_manager_clone.unregister_client(&client_id_clone);
        info!("Client {} disconnected", client_id_clone);
    });

    // Handle incoming messages
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) = handle_message(&text, client_id, &stream_manager).await {
                    error!("Error handling message from client {}: {}", client_id, e);

                    // Send error response
                    if let Some(client_sender) = stream_manager.get_client_sender(&client_id) {
                        let error_message = ServerMessage::Error {
                            code: 400,
                            message: format!("Invalid message: {}", e),
                            stream_id: None,
                        };

                        let _ = client_sender.send(error_message);
                    }
                }
            }
            Ok(Message::Ping(_payload)) => {
                debug!("Received ping from client {}", client_id);
                // WebSocket library handles pong automatically
            }
            Ok(Message::Pong(_)) => {
                debug!("Received pong from client {}", client_id);
            }
            Ok(Message::Close(_)) => {
                info!("Client {} sent close message", client_id);
                break;
            }
            Ok(Message::Binary(_)) => {
                warn!("Received binary message from client {}, ignoring", client_id);
            }
            Ok(Message::Frame(_)) => {
                // Raw frames are handled automatically by the library
                debug!("Received raw frame from client {}", client_id);
            }
            Err(e) => {
                error!("WebSocket error from client {}: {}", client_id, e);
                break;
            }
        }
    }

    stream_manager.unregister_client(&client_id);
    info!("Client {} connection closed", client_id);

    Ok(())
}

async fn handle_message(
    text: &str,
    client_id: Uuid,
    stream_manager: &Arc<StreamManager>,
) -> anyhow::Result<()> {
    let client_message: ClientMessage = serde_json::from_str(text)?;
    debug!("Received message from client {}: {:?}", client_id, client_message);

    match client_message {
        ClientMessage::Subscribe {
            stream_id,
            symbol,
            data_type,
            max_levels,
        } => {
            match stream_manager
                .subscribe(client_id, stream_id.clone(), symbol.clone(), data_type.clone(), max_levels)
                .await
            {
                Ok(()) => {
                    if let Some(client_sender) = stream_manager.get_client_sender(&client_id) {
                        let response = ServerMessage::Subscribed {
                            stream_id,
                            symbol,
                            data_type,
                        };

                        if let Err(e) = client_sender.send(response) {
                            error!("Failed to send subscription confirmation to client {}: {}", client_id, e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to subscribe client {} to {}: {}", client_id, symbol, e);

                    if let Some(client_sender) = stream_manager.get_client_sender(&client_id) {
                        let error_message = ServerMessage::Error {
                            code: 500,
                            message: format!("Subscription failed: {}", e),
                            stream_id: Some(stream_id),
                        };

                        let _ = client_sender.send(error_message);
                    }
                }
            }
        }
        ClientMessage::Unsubscribe { stream_id } => {
            let success = stream_manager.unsubscribe(client_id, &stream_id);

            if let Some(client_sender) = stream_manager.get_client_sender(&client_id) {
                if success {
                    let response = ServerMessage::Unsubscribed { stream_id };
                    let _ = client_sender.send(response);
                } else {
                    let error_message = ServerMessage::Error {
                        code: 404,
                        message: "Stream not found".to_string(),
                        stream_id: Some(stream_id),
                    };
                    let _ = client_sender.send(error_message);
                }
            }
        }
        ClientMessage::Ping { timestamp: _ } => {
            if let Some(client_sender) = stream_manager.get_client_sender(&client_id) {
                let response = ServerMessage::HeartBeat {
                    timestamp: Utc::now(),
                };

                if let Err(e) = client_sender.send(response) {
                    error!("Failed to send ping response to client {}: {}", client_id, e);
                }
            }
        }
    }

    Ok(())
}