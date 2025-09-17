# Market Depth Server (Backend)

A high-performance WebSocket server built with Rust for real-time market depth data streaming with multiplexed connections.

## Features

- **High-performance WebSocket server** using Tokio and Tungstenite
- **Multiplexed data streams** supporting concurrent MBO/MBP subscriptions
- **Real-time order book simulation** with realistic market activity
- **Message-based protocol** with JSON serialization
- **Multi-symbol support** (BTCUSD, ETHUSD, ADAUSD)
- **Concurrent client handling** with efficient resource management

## Architecture

### Core Components

- **StreamManager**: Manages order books, client subscriptions, and market simulation
- **WebSocketHandler**: Handles WebSocket connections and message routing
- **OrderBook**: Order book implementation with MBO/MBP data generation
- **Message Protocol**: Typed message definitions for client-server communication

### Data Types

- **MBO (Market By Order)**: Individual order tracking with timestamps and age
- **MBP (Market By Price)**: Aggregated price levels with quantities and counts

## Prerequisites

- Rust (latest stable)
- Cargo package manager

## Quick Start

```bash
# Clone and navigate to backend
cd backend

# Run the server
cargo run --bin server

# Server starts on ws://127.0.0.1:8080
```

## Development Commands

```bash
# Check code for errors
cargo check

# Run in development mode
cargo run --bin server

# Build optimized release binary
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Configuration

The server accepts command-line arguments:

```bash
# Custom address and log level
cargo run --bin server -- --addr 0.0.0.0:9000 --log-level debug
```

Options:
- `--addr`: WebSocket server address (default: 127.0.0.1:8080)
- `--log-level`: Logging level (trace, debug, info, warn, error)

## WebSocket Protocol

### Client Messages

#### Subscribe to Market Data
```json
{
  "type": "Subscribe",
  "stream_id": "btc_mbp",
  "symbol": "BTCUSD",
  "data_type": "MBP",
  "max_levels": 20
}
```

#### Unsubscribe from Stream
```json
{
  "type": "Unsubscribe",
  "stream_id": "btc_mbp"
}
```

#### Ping Server
```json
{
  "type": "Ping",
  "timestamp": "2025-09-16T04:18:26.806069Z"
}
```

### Server Messages

#### Market Data Update
```json
{
  "type": "MarketData",
  "stream_id": "btc_mbp",
  "symbol": "BTCUSD",
  "sequence": 560,
  "timestamp": "2025-09-16T04:18:26.806069Z",
  "data": {
    "format": "MBP",
    "bids": [{"price": 102.45, "quantity": 5000, "order_count": 3, "total_quantity": 15000}],
    "asks": [{"price": 97.46, "quantity": 3000, "order_count": 2, "total_quantity": 8000}]
  }
}
```

#### Subscription Confirmation
```json
{
  "type": "Subscribed",
  "stream_id": "btc_mbp",
  "symbol": "BTCUSD",
  "data_type": "MBP"
}
```

#### Heartbeat
```json
{
  "type": "HeartBeat",
  "timestamp": "2025-09-16T04:18:26.806069Z"
}
```

#### Error Response
```json
{
  "type": "Error",
  "code": 400,
  "message": "Invalid message format",
  "stream_id": "btc_mbp"
}
```

## Performance Features

- **Async/await throughout**: Non-blocking I/O operations
- **Efficient data structures**: DashMap for concurrent access
- **Memory optimization**: Streaming without accumulation
- **Connection pooling**: Scalable client management
- **Configurable parameters**: Update intervals and depth limits

## Market Simulation

The server includes realistic market simulation:
- **Order activities**: 40% new orders, 30% updates, 30% cancellations
- **Price movements**: Based on current best bid/ask with realistic spreads
- **Update frequency**: Market data updates every 300ms
- **Multiple symbols**: Independent order books for each trading pair

## Monitoring & Logging

Comprehensive logging for:
- Client connection lifecycle
- Subscription management
- Market data distribution
- Error conditions and recovery
- Performance metrics

Log levels: `trace`, `debug`, `info`, `warn`, `error`

## Dependencies

Key dependencies in `Cargo.toml`:
- `tokio`: Async runtime
- `tokio-tungstenite`: WebSocket implementation
- `serde`: JSON serialization
- `dashmap`: Concurrent hash maps
- `tracing`: Structured logging
- `chrono`: Date/time handling
- `uuid`: Unique identifiers
- `clap`: Command-line parsing

## API Testing

Test the WebSocket API using any WebSocket client:

```javascript
const ws = new WebSocket('ws://127.0.0.1:8080');

ws.onopen = () => {
  // Subscribe to market data
  ws.send(JSON.stringify({
    type: 'Subscribe',
    stream_id: 'test_mbp',
    symbol: 'BTCUSD',
    data_type: 'MBP',
    max_levels: 10
  }));
};

ws.onmessage = (event) => {
  console.log('Received:', JSON.parse(event.data));
};
```

## License

ISC