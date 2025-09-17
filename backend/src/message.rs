use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Subscribe {
        stream_id: String,
        symbol: String,
        data_type: DataType,
        max_levels: Option<u32>,
    },
    Unsubscribe {
        stream_id: String,
    },
    Ping {
        timestamp: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Subscribed {
        stream_id: String,
        symbol: String,
        data_type: DataType,
    },
    Unsubscribed {
        stream_id: String,
    },
    MarketData {
        stream_id: String,
        symbol: String,
        data: MarketDataUpdate,
        sequence: u64,
        timestamp: DateTime<Utc>,
    },
    HeartBeat {
        timestamp: DateTime<Utc>,
    },
    Error {
        code: u32,
        message: String,
        stream_id: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    MBO, // Market By Order
    MBP, // Market By Price
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "format")]
pub enum MarketDataUpdate {
    MBO {
        bids: Vec<MBOLevel>,
        asks: Vec<MBOLevel>,
    },
    MBP {
        bids: Vec<MBPLevel>,
        asks: Vec<MBPLevel>,
    },
    OrderActivity {
        activity: OrderActivity,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MBOLevel {
    pub order_id: String,
    pub price: f64,
    pub quantity: u64,
    pub side: Side,
    pub timestamp: DateTime<Utc>,
    pub age_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MBPLevel {
    pub price: f64,
    pub quantity: u64,
    pub order_count: u32,
    pub side: Side,
    pub total_quantity: u64,
    pub avg_age_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderActivity {
    pub activity_type: ActivityType,
    pub order_id: String,
    pub symbol: String,
    pub price: Option<f64>,
    pub quantity: Option<u64>,
    pub side: Option<Side>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
    Add,
    Update,
    Cancel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Clone)]
pub struct Subscription {
    pub stream_id: String,
    pub symbol: String,
    pub data_type: DataType,
    pub max_levels: u32,
    pub client_id: Uuid,
}

impl Subscription {
    pub fn new(
        stream_id: String,
        symbol: String,
        data_type: DataType,
        max_levels: Option<u32>,
        client_id: Uuid,
    ) -> Self {
        Self {
            stream_id,
            symbol,
            data_type,
            max_levels: max_levels.unwrap_or(20),
            client_id,
        }
    }
}