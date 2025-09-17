use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum SSEMessage {
    #[serde(rename = "market_data")]
    MarketData {
        stream_id: String,
        symbol: String,
        data: MarketDataUpdate,
        sequence: u64,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "heartbeat")]
    HeartBeat {
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "connection_info")]
    ConnectionInfo {
        client_id: String,
        server_time: DateTime<Utc>,
        supported_symbols: Vec<String>,
    },
    #[serde(rename = "error")]
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
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Clone)]
pub struct SSESubscription {
    pub stream_id: String,
    pub symbol: String,
    pub data_type: DataType,
    pub max_levels: u32,
    pub client_id: Uuid,
}

impl SSESubscription {
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

// SSE formatting helpers
impl SSEMessage {
    pub fn to_sse_data(&self) -> String {
        match serde_json::to_string(self) {
            Ok(json) => format!("data: {}\n\n", json),
            Err(_) => "data: {\"event\":\"error\",\"message\":\"Serialization failed\"}\n\n".to_string(),
        }
    }

    pub fn to_sse_event(&self) -> String {
        let event_name = match self {
            SSEMessage::MarketData { .. } => "market_data",
            SSEMessage::HeartBeat { .. } => "heartbeat",
            SSEMessage::ConnectionInfo { .. } => "connection_info",
            SSEMessage::Error { .. } => "error",
        };

        match serde_json::to_string(self) {
            Ok(json) => format!("event: {}\ndata: {}\n\n", event_name, json),
            Err(_) => "event: error\ndata: {\"message\":\"Serialization failed\"}\n\n".to_string(),
        }
    }
}

// Query parameters for SSE endpoint
#[derive(Debug, Deserialize)]
pub struct StreamQuery {
    pub streams: Option<String>, // Comma-separated stream definitions: "BTCUSD:MBP:20,ETHUSD:MBO:10"
    pub symbols: Option<String>, // Comma-separated symbols: "BTCUSD,ETHUSD"
    pub data_type: Option<String>, // Default data type: "MBP" or "MBO"
    pub max_levels: Option<u32>, // Default max levels
}

impl StreamQuery {
    pub fn parse_streams(&self) -> Vec<(String, DataType, u32)> {
        let mut streams = Vec::new();

        if let Some(stream_str) = &self.streams {
            for stream_def in stream_str.split(',') {
                let parts: Vec<&str> = stream_def.trim().split(':').collect();
                if parts.len() >= 1 {
                    let symbol = parts[0].to_string();
                    let data_type = if parts.len() >= 2 {
                        match parts[1].to_uppercase().as_str() {
                            "MBO" => DataType::MBO,
                            _ => DataType::MBP,
                        }
                    } else {
                        self.get_default_data_type()
                    };
                    let max_levels = if parts.len() >= 3 {
                        parts[2].parse().unwrap_or(self.max_levels.unwrap_or(20))
                    } else {
                        self.max_levels.unwrap_or(20)
                    };
                    streams.push((symbol, data_type, max_levels));
                }
            }
        } else if let Some(symbols_str) = &self.symbols {
            let data_type = self.get_default_data_type();
            let max_levels = self.max_levels.unwrap_or(20);

            for symbol in symbols_str.split(',') {
                streams.push((symbol.trim().to_string(), data_type.clone(), max_levels));
            }
        }

        streams
    }

    fn get_default_data_type(&self) -> DataType {
        match self.data_type.as_deref() {
            Some("MBO") => DataType::MBO,
            _ => DataType::MBP,
        }
    }
}

#[derive(Debug, Clone)]
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