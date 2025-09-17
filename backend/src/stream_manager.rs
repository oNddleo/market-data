use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, mpsc, broadcast};
use tokio::time::interval;
use dashmap::DashMap;
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, debug};

use crate::order_book::OrderBook;
use crate::message::{
    ServerMessage, MarketDataUpdate, Subscription, DataType, OrderActivity,
};

pub type ClientSender = mpsc::UnboundedSender<ServerMessage>;

#[derive(Debug)]
pub struct StreamManager {
    order_books: Arc<DashMap<String, Arc<RwLock<OrderBook>>>>,
    subscriptions: Arc<DashMap<String, Vec<Subscription>>>,
    clients: Arc<DashMap<Uuid, ClientSender>>,
    activity_broadcast: broadcast::Sender<(String, OrderActivity)>,
}

impl StreamManager {
    pub fn new() -> Self {
        let (activity_broadcast, _) = broadcast::channel(1000);

        Self {
            order_books: Arc::new(DashMap::new()),
            subscriptions: Arc::new(DashMap::new()),
            clients: Arc::new(DashMap::new()),
            activity_broadcast,
        }
    }

    pub async fn start(&self) {
        info!("Starting stream manager");

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
        let activity_broadcast = self.activity_broadcast.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(300));

            loop {
                interval.tick().await;

                for entry in order_books.iter() {
                    let symbol = entry.key().clone();
                    let order_book_ref = entry.value().clone();

                    // Simulate market activity
                    let activities = {
                        let mut order_book = order_book_ref.write().await;
                        order_book.simulate_activity()
                    };

                    // Broadcast activities for real-time updates
                    for activity in &activities {
                        let _ = activity_broadcast.send((symbol.clone(), activity.clone()));
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

                                let message = ServerMessage::MarketData {
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

                let heartbeat = ServerMessage::HeartBeat {
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

    pub fn register_client(&self, client_id: Uuid, sender: ClientSender) {
        self.clients.insert(client_id, sender);
        info!("Registered client: {}", client_id);
    }

    pub fn unregister_client(&self, client_id: &Uuid) {
        self.clients.remove(client_id);

        // Remove all subscriptions for this client
        for mut entry in self.subscriptions.iter_mut() {
            entry.value_mut().retain(|sub| sub.client_id != *client_id);
        }

        // Clean up empty subscription lists
        self.subscriptions.retain(|_, v| !v.is_empty());

        info!("Unregistered client: {}", client_id);
    }

    pub async fn subscribe(
        &self,
        client_id: Uuid,
        stream_id: String,
        symbol: String,
        data_type: DataType,
        max_levels: Option<u32>,
    ) -> Result<(), String> {
        // Ensure the symbol exists
        if !self.order_books.contains_key(&symbol) {
            self.initialize_symbol(&symbol).await;
        }

        let subscription = Subscription::new(
            stream_id.clone(),
            symbol.clone(),
            data_type.clone(),
            max_levels,
            client_id,
        );

        // Add subscription
        self.subscriptions
            .entry(symbol.clone())
            .or_insert_with(Vec::new)
            .push(subscription);

        // Send initial snapshot
        if let Some(order_book_ref) = self.order_books.get(&symbol) {
            if let Some(client_sender) = self.clients.get(&client_id) {
                let market_data = {
                    let order_book = order_book_ref.read().await;
                    match data_type {
                        DataType::MBO => {
                            let (bids, asks) = order_book.get_mbo_data(max_levels.unwrap_or(20));
                            MarketDataUpdate::MBO { bids, asks }
                        }
                        DataType::MBP => {
                            let (bids, asks) = order_book.get_mbp_data(max_levels.unwrap_or(20));
                            MarketDataUpdate::MBP { bids, asks }
                        }
                    }
                };

                let initial_message = ServerMessage::MarketData {
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

        info!("Client {} subscribed to {} stream {} ({})",
            client_id, symbol, stream_id,
            match data_type { DataType::MBO => "MBO", DataType::MBP => "MBP" }
        );

        Ok(())
    }

    pub fn unsubscribe(&self, client_id: Uuid, stream_id: &str) -> bool {
        for mut entry in self.subscriptions.iter_mut() {
            let initial_len = entry.value().len();
            entry.value_mut().retain(|sub|
                !(sub.client_id == client_id && sub.stream_id == stream_id)
            );

            if entry.value().len() != initial_len {
                info!("Client {} unsubscribed from stream {}", client_id, stream_id);
                return true;
            }
        }

        false
    }

    pub fn get_activity_receiver(&self) -> broadcast::Receiver<(String, OrderActivity)> {
        self.activity_broadcast.subscribe()
    }

    pub async fn get_symbols(&self) -> Vec<String> {
        self.order_books.iter().map(|entry| entry.key().clone()).collect()
    }

    pub async fn get_order_book_snapshot(&self, symbol: &str, data_type: DataType, max_levels: u32) -> Option<MarketDataUpdate> {
        if let Some(order_book_ref) = self.order_books.get(symbol) {
            let order_book = order_book_ref.read().await;

            let market_data = match data_type {
                DataType::MBO => {
                    let (bids, asks) = order_book.get_mbo_data(max_levels);
                    MarketDataUpdate::MBO { bids, asks }
                }
                DataType::MBP => {
                    let (bids, asks) = order_book.get_mbp_data(max_levels);
                    MarketDataUpdate::MBP { bids, asks }
                }
            };

            Some(market_data)
        } else {
            None
        }
    }

    pub fn get_client_sender(&self, client_id: &Uuid) -> Option<dashmap::mapref::one::Ref<Uuid, ClientSender>> {
        self.clients.get(client_id)
    }
}