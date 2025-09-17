use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, mpsc};
use tokio::time::interval;
use dashmap::DashMap;
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, debug};

use crate::order_book::OrderBook;
use crate::message::{
    SSEMessage, MarketDataUpdate, SSESubscription, DataType,
};

pub type SSEClientSender = mpsc::UnboundedSender<SSEMessage>;

#[derive(Debug)]
pub struct SSEStreamManager {
    order_books: Arc<DashMap<String, Arc<RwLock<OrderBook>>>>,
    subscriptions: Arc<DashMap<String, Vec<SSESubscription>>>,
    clients: Arc<DashMap<Uuid, SSEClientSender>>,
    client_streams: Arc<DashMap<Uuid, Vec<String>>>, // Track which streams each client is subscribed to
}

impl SSEStreamManager {
    pub fn new() -> Self {
        Self {
            order_books: Arc::new(DashMap::new()),
            subscriptions: Arc::new(DashMap::new()),
            clients: Arc::new(DashMap::new()),
            client_streams: Arc::new(DashMap::new()),
        }
    }

    pub async fn start(&self) {
        info!("Starting SSE stream manager");

        // Initialize default symbols
        self.initialize_symbol("BTCUSD").await;
        self.initialize_symbol("ETHUSD").await;
        self.initialize_symbol("ADAUSD").await;

        // Start market simulation
        self.start_market_simulation().await;

        // Start heartbeat
        self.start_heartbeat().await;
    }

    async fn initialize_symbol(&self, symbol: &str) {
        let mut order_book = OrderBook::new(symbol.to_string());
        order_book.initialize_with_sample_data();

        self.order_books.insert(
            symbol.to_string(),
            Arc::new(RwLock::new(order_book))
        );

        info!("Initialized order book for symbol: {}", symbol);
    }

    async fn start_market_simulation(&self) {
        let order_books = Arc::clone(&self.order_books);
        let subscriptions = Arc::clone(&self.subscriptions);
        let clients = Arc::clone(&self.clients);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(300));

            loop {
                interval.tick().await;

                for entry in order_books.iter() {
                    let symbol = entry.key().clone();
                    let order_book_ref = entry.value().clone();

                    // Simulate market activity
                    {
                        let mut order_book = order_book_ref.write().await;
                        order_book.simulate_activity();
                    }

                    // Send updates to subscribed clients
                    if let Some(symbol_subscriptions) = subscriptions.get(&symbol) {
                        for subscription in symbol_subscriptions.iter() {
                            if let Some(client_sender) = clients.get(&subscription.client_id) {
                                let market_data = {
                                    let order_book = order_book_ref.read().await;
                                    match subscription.data_type {
                                        DataType::MBO => {
                                            let (bids, asks) = order_book.get_mbo_data(subscription.max_levels);
                                            MarketDataUpdate::MBO { bids, asks }
                                        }
                                        DataType::MBP => {
                                            let (bids, asks) = order_book.get_mbp_data(subscription.max_levels);
                                            MarketDataUpdate::MBP { bids, asks }
                                        }
                                    }
                                };

                                let message = SSEMessage::MarketData {
                                    stream_id: subscription.stream_id.clone(),
                                    symbol: symbol.clone(),
                                    data: market_data,
                                    sequence: {
                                        let order_book = order_book_ref.read().await;
                                        order_book.get_sequence()
                                    },
                                    timestamp: Utc::now(),
                                };

                                if let Err(_) = client_sender.send(message) {
                                    debug!("Client {} disconnected during market data send", subscription.client_id);
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    async fn start_heartbeat(&self) {
        let clients = Arc::clone(&self.clients);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));

            loop {
                interval.tick().await;

                let heartbeat = SSEMessage::HeartBeat {
                    timestamp: Utc::now(),
                };

                for client in clients.iter() {
                    if let Err(_) = client.send(heartbeat.clone()) {
                        debug!("Client {} disconnected during heartbeat", client.key());
                    }
                }
            }
        });
    }

    pub fn register_client(&self, client_id: Uuid, sender: SSEClientSender) {
        self.clients.insert(client_id, sender);
        self.client_streams.insert(client_id, Vec::new());
        info!("Registered SSE client: {}", client_id);
    }

    pub fn unregister_client(&self, client_id: &Uuid) {
        // Remove all subscriptions for this client
        let client_streams = self.client_streams.remove(client_id);
        if let Some((_, streams)) = client_streams {
            for stream_id in streams {
                self.remove_subscription(client_id, &stream_id);
            }
        }

        self.clients.remove(client_id);
        info!("Unregistered SSE client: {}", client_id);
    }

    pub async fn subscribe_to_streams(
        &self,
        client_id: Uuid,
        stream_definitions: Vec<(String, DataType, u32)>,
    ) -> Result<(), String> {
        for (symbol, data_type, max_levels) in stream_definitions {
            // Ensure the symbol exists
            if !self.order_books.contains_key(&symbol) {
                self.initialize_symbol(&symbol).await;
            }

            let stream_id = format!("{}_{:?}_{}", symbol, data_type, max_levels);

            let subscription = SSESubscription::new(
                stream_id.clone(),
                symbol.clone(),
                data_type.clone(),
                Some(max_levels),
                client_id,
            );

            // Add subscription
            self.subscriptions
                .entry(symbol.clone())
                .or_insert_with(Vec::new)
                .push(subscription);

            // Track this stream for the client
            self.client_streams
                .entry(client_id)
                .or_insert_with(Vec::new)
                .push(stream_id.clone());

            // Send initial snapshot
            if let Some(order_book_ref) = self.order_books.get(&symbol) {
                if let Some(client_sender) = self.clients.get(&client_id) {
                    let market_data = {
                        let order_book = order_book_ref.read().await;
                        match data_type {
                            DataType::MBO => {
                                let (bids, asks) = order_book.get_mbo_data(max_levels);
                                MarketDataUpdate::MBO { bids, asks }
                            }
                            DataType::MBP => {
                                let (bids, asks) = order_book.get_mbp_data(max_levels);
                                MarketDataUpdate::MBP { bids, asks }
                            }
                        }
                    };

                    let initial_message = SSEMessage::MarketData {
                        stream_id: stream_id.clone(),
                        symbol: symbol.clone(),
                        data: market_data,
                        sequence: {
                            let order_book = order_book_ref.read().await;
                            order_book.get_sequence()
                        },
                        timestamp: Utc::now(),
                    };

                    if let Err(_) = client_sender.send(initial_message) {
                        return Err("Failed to send initial snapshot".to_string());
                    }
                }
            }

            info!("Client {} subscribed to {} stream {} ({:?})",
                client_id, symbol, stream_id, data_type
            );
        }

        Ok(())
    }

    fn remove_subscription(&self, client_id: &Uuid, stream_id: &str) {
        for mut entry in self.subscriptions.iter_mut() {
            let initial_len = entry.value().len();
            entry.value_mut().retain(|sub|
                !(sub.client_id == *client_id && sub.stream_id == *stream_id)
            );

            if entry.value().len() != initial_len {
                debug!("Removed subscription for client {} stream {}", client_id, stream_id);
                break;
            }
        }

        // Clean up empty subscription lists
        self.subscriptions.retain(|_, v| !v.is_empty());
    }

    pub async fn get_symbols(&self) -> Vec<String> {
        self.order_books.iter().map(|entry| entry.key().clone()).collect()
    }

    pub fn get_client_sender(&self, client_id: &Uuid) -> Option<dashmap::mapref::one::Ref<Uuid, SSEClientSender>> {
        self.clients.get(client_id)
    }

    pub async fn send_connection_info(&self, client_id: Uuid) {
        let symbols = self.get_symbols().await;

        if let Some(client_sender) = self.clients.get(&client_id) {
            let connection_info = SSEMessage::ConnectionInfo {
                client_id: client_id.to_string(),
                server_time: Utc::now(),
                supported_symbols: symbols,
            };

            if let Err(_) = client_sender.send(connection_info) {
                debug!("Failed to send connection info to client {}", client_id);
            }
        }
    }
}