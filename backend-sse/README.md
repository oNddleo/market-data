# Market Depth SSE Server

A high-performance **Server-Sent Events (SSE)** backend for real-time market data streaming with **multiplexed stream support**. Built with Rust and Axum, this server provides an efficient alternative to WebSocket for one-way real-time market depth visualization.

## 🚀 Features

### Core Capabilities
- **🔗 Multiplexed SSE Streaming**: Multiple data streams per client connection via URL parameters
- **📊 Market Data Types**: Support for both MBO (Market By Order) and MBP (Market By Price) formats
- **⚡ Real-time Updates**: 300ms market simulation with realistic order activities
- **🌐 CORS Enabled**: Cross-origin requests supported for web applications
- **💗 Heartbeat System**: 30-second keepalive messages for connection monitoring
- **🛡️ Production Ready**: Structured logging, error handling, and graceful client cleanup

### Market Data Formats

#### MBP (Market By Price)
- Aggregated price levels with total quantities
- Order count per price level
- Cumulative depth calculation
- Traditional market depth view

#### MBO (Market By Order)
- Individual order details with unique IDs
- Order timestamps and age tracking
- FIFO priority within price levels
- Real-time order lifecycle (add/update/cancel)

## 🏗️ Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   SSE Client    │◄───┤  Axum HTTP Server │◄───┤ Stream Manager  │
│  (Browser/App)  │    │   (Port 8081)    │    │   (Multiplexer) │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │                        │
                                │                        ▼
                       ┌──────────────────┐    ┌─────────────────┐
                       │   SSE Handler    │    │   Order Books   │
                       │  (Custom Stream) │    │ (BTCUSD/ETHUSD/ │
                       └──────────────────┘    │    ADAUSD)      │
                                               └─────────────────┘
```

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+
- Cargo

### Installation & Running

1. **Clone and navigate to SSE backend:**
   ```bash
   cd backend-sse
   ```

2. **Install dependencies:**
   ```bash
   cargo check
   ```

3. **Run the server:**
   ```bash
   cargo run --bin sse-server
   ```

4. **Server will start on:**
   ```
   http://127.0.0.1:8081
   ```

### Testing the Server

**Check server status:**
```bash
curl http://127.0.0.1:8081/health
```

**Get API information:**
```bash
curl http://127.0.0.1:8081/api
```

**List available symbols:**
```bash
curl http://127.0.0.1:8081/symbols
```

## 📡 API Endpoints

### REST Endpoints

| Endpoint | Method | Description |
|----------|---------|-------------|
| `/health` | GET | Health check endpoint |
| `/api` | GET | API documentation and capabilities |
| `/symbols` | GET | List of available trading symbols |
| `/stream` | GET | SSE streaming endpoint |

### SSE Streaming Endpoint

**Base URL:** `GET /stream`

#### Query Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `streams` | Comma-separated stream definitions | `BTCUSD:MBP:20,ETHUSD:MBO:10` |
| `symbols` | Comma-separated symbols (uses defaults) | `BTCUSD,ETHUSD` |
| `data_type` | Default data type (MBP/MBO) | `MBP` |
| `max_levels` | Default maximum levels | `20` |

#### Stream Definition Format
```
{SYMBOL}:{DATA_TYPE}:{MAX_LEVELS}
```

**Examples:**
- `BTCUSD:MBP:20` - Bitcoin MBP data with 20 price levels
- `ETHUSD:MBO:10` - Ethereum MBO data with 10 order levels
- `ADAUSD:MBP:5` - Cardano MBP data with 5 price levels

## 🔌 Usage Examples

### 1. Single Stream Connection
```bash
curl -N "http://127.0.0.1:8081/stream?streams=BTCUSD:MBP:10"
```

### 2. Multiple Streams (Multiplexed)
```bash
curl -N "http://127.0.0.1:8081/stream?streams=BTCUSD:MBP:5,ETHUSD:MBO:3,ADAUSD:MBP:10"
```

### 3. Symbol-based with Defaults
```bash
curl -N "http://127.0.0.1:8081/stream?symbols=BTCUSD,ETHUSD&data_type=MBP&max_levels=15"
```

### 4. JavaScript EventSource
```javascript
const eventSource = new EventSource(
  'http://127.0.0.1:8081/stream?streams=BTCUSD:MBP:5,ETHUSD:MBO:3'
);

eventSource.addEventListener('market_data', function(event) {
  const data = JSON.parse(event.data);
  console.log('Market data:', data);
});

eventSource.addEventListener('heartbeat', function(event) {
  console.log('Heartbeat received');
});
```

## 📊 SSE Event Types

### 1. Connection Info
Sent when client connects:
```json
{
  "event": "connection_info",
  "client_id": "550e8400-e29b-41d4-a716-446655440000",
  "server_time": "2024-01-15T10:30:00Z",
  "supported_symbols": ["BTCUSD", "ETHUSD", "ADAUSD"]
}
```

### 2. Market Data (MBP)
```json
{
  "event": "market_data",
  "stream_id": "BTCUSD_MBP_5",
  "symbol": "BTCUSD",
  "data": {
    "format": "MBP",
    "bids": [
      {
        "price": 45000.50,
        "quantity": 1500,
        "order_count": 3,
        "side": "Bid",
        "total_quantity": 1500,
        "avg_age_ms": 1200
      }
    ],
    "asks": [...]
  },
  "sequence": 12345,
  "timestamp": "2024-01-15T10:30:01Z"
}
```

### 3. Market Data (MBO)
```json
{
  "event": "market_data",
  "stream_id": "ETHUSD_MBO_3",
  "symbol": "ETHUSD",
  "data": {
    "format": "MBO",
    "bids": [
      {
        "order_id": "order_1642248601234_567890",
        "price": 3000.25,
        "quantity": 500,
        "side": "Bid",
        "timestamp": "2024-01-15T10:30:01Z",
        "age_ms": 1500
      }
    ],
    "asks": [...]
  },
  "sequence": 12346,
  "timestamp": "2024-01-15T10:30:01Z"
}
```

### 4. Heartbeat
```json
{
  "event": "heartbeat",
  "timestamp": "2024-01-15T10:30:30Z"
}
```

### 5. Error
```json
{
  "event": "error",
  "code": 400,
  "message": "Invalid stream definition",
  "stream_id": "INVALID_STREAM"
}
```

## 🎨 Client Example

A complete HTML/JavaScript client example is included in `example-client.html`. Features:

- **Real-time order book visualization**
- **Multiple stream support**
- **Professional trading interface**
- **Connection status monitoring**
- **Event logging**

To use the client:
```bash
open example-client.html
```

## 🔧 Configuration

### Server Configuration
Default server settings can be modified via command line:

```bash
# Custom address and port
cargo run --bin sse-server -- --addr 0.0.0.0:9000

# Custom log level
cargo run --bin sse-server -- --log-level debug
```

### Available Options
- `--addr, -a`: Server address (default: `127.0.0.1:8081`)
- `--log-level, -l`: Log level (trace, debug, info, warn, error)

## 🏗️ Project Structure

```
backend-sse/
├── src/
│   ├── main.rs              # Server entry point and routing
│   ├── lib.rs               # Library exports
│   ├── message.rs           # SSE message types and parsing
│   ├── order_book.rs        # Order book implementation
│   ├── stream_manager.rs    # Client and stream management
│   └── sse_handler.rs       # SSE endpoint and custom stream
├── Cargo.toml               # Dependencies and project config
├── example-client.html      # Browser-based SSE client
└── README.md               # This file
```

## 🔍 Key Dependencies

- **axum** (0.7): Modern async web framework
- **tokio** (1.40): Async runtime with full features
- **serde** (1.0): Serialization framework
- **uuid** (1.10): UUID generation for client IDs
- **chrono** (0.4): Date and time handling
- **dashmap** (6.1): Concurrent hash map
- **tower-http** (0.5): HTTP middleware (CORS)

## 🚀 Performance

- **Concurrent Clients**: Handles multiple simultaneous SSE connections
- **Memory Efficient**: Lock-free data structures with DashMap
- **Update Frequency**: 300ms market simulation intervals
- **Cleanup**: Automatic client disconnection handling
- **Heartbeat**: 30-second keepalive for connection monitoring

## 🔄 Comparison with WebSocket Backend

| Feature | SSE Backend | WebSocket Backend |
|---------|-------------|-------------------|
| **Direction** | Server → Client | Bidirectional |
| **Protocol** | HTTP/SSE | WebSocket |
| **Multiplexing** | URL Parameters | Message Types |
| **Browser Support** | Universal | Universal |
| **Simplicity** | Higher | Medium |
| **Use Case** | Real-time streaming | Interactive trading |

## 📝 Development

### Building
```bash
cargo build
```

### Testing
```bash
cargo test
```

### Checking
```bash
cargo check
```

### Running with logs
```bash
RUST_LOG=debug cargo run --bin sse-server
```

## 🤝 Integration

This SSE backend integrates with the existing market depth project:

- **Frontend**: Compatible with React frontend (modify WebSocket to EventSource)
- **WebSocket Backend**: Can run alongside on different ports (8080 vs 8081)
- **Order Book**: Shares same order book implementation
- **Message Format**: Similar JSON structure for easy migration

## 📄 License

This project is part of the market-depth visualization system.

---

**Built with ❤️ using Rust and Axum**