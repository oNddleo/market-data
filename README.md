# Market Depth Visualization - Full Stack Application

A high-performance market depth visualization system with separated backend and frontend applications, demonstrating real-time financial market data streaming with multiplexed WebSocket connections.

## Project Structure

```
market-depth/
├── backend/                # Rust WebSocket Server
│   ├── src/               # Rust source code
│   ├── Cargo.toml         # Rust dependencies
│   └── README.md          # Backend documentation
├── frontend/              # React Application
│   ├── src/               # React source code
│   ├── public/            # Static assets
│   ├── package.json       # Node.js dependencies
│   └── README.md          # Frontend documentation
└── ROOT_README.md         # This file
```

## Architecture Overview

### Backend (Rust)
- **High-performance WebSocket server** using Tokio and Tungstenite
- **Multiplexed data streams** supporting concurrent MBO/MBP subscriptions
- **Real-time order book simulation** with realistic market activity
- **JSON-based message protocol** for client communication
- **Multi-symbol support** with independent order books

### Frontend (React)
- **Real-time visualization** of market depth data
- **Professional trading interface** with MBO/MBP modes
- **WebSocket client** with automatic reconnection
- **Responsive design** optimized for financial workflows
- **Symbol selection** and connection monitoring

## Quick Start (Both Applications)

### 1. Start Backend Server

```bash
# Navigate to backend
cd backend

# Run Rust server
cargo run --bin server

# Server starts on ws://127.0.0.1:8080
```

### 2. Start Frontend Application

```bash
# Open new terminal and navigate to frontend
cd frontend

# Install dependencies (first time only)
npm install

# Start React development server
npm start

# Open browser to http://localhost:3000
```

## Individual Setup

### Backend Only

```bash
cd backend
cargo run --bin server
```

The WebSocket server will be available at `ws://127.0.0.1:8080` and can be accessed by any WebSocket client.

### Frontend Only

```bash
cd frontend
npm install
npm start
```

The React application will start at `http://localhost:3000` and attempt to connect to the backend server.

## Development Workflow

### Backend Development

```bash
cd backend
cargo check          # Check for compilation errors
cargo test           # Run tests
cargo clippy         # Lint code
cargo fmt            # Format code
cargo run --bin server  # Run development server
```

### Frontend Development

```bash
cd frontend
npm start            # Development server with hot reload
npm test             # Run tests
npm run build        # Production build
```

## Features

### Real-time Market Data
- **Market By Order (MBO)**: Individual order tracking with IDs and timestamps
- **Market By Price (MBP)**: Aggregated price levels with quantities
- **Multiple symbols**: BTCUSD, ETHUSD, ADAUSD
- **Live updates**: 300ms intervals with sequence numbers

### WebSocket Protocol
- **Subscription management**: Multiple concurrent streams per client
- **Multiplexed connections**: Different data types on same connection
- **Automatic reconnection**: Client-side resilience
- **Error handling**: Comprehensive error reporting

### Performance
- **Rust backend**: Zero-cost abstractions and memory safety
- **Async I/O**: Non-blocking operations throughout
- **React optimization**: Memoized components and efficient updates
- **Scalable architecture**: Concurrent client handling

## API Documentation

### WebSocket Endpoints

| Endpoint | Description |
|----------|-------------|
| `ws://127.0.0.1:8080` | Main WebSocket endpoint |

### Message Types

| Client → Server | Server → Client |
|----------------|-----------------|
| Subscribe | MarketData |
| Unsubscribe | Subscribed |
| Ping | HeartBeat |
| | Error |

See individual README files for detailed protocol documentation.

## Configuration

### Backend Configuration

```bash
# Custom server address and logging
cargo run --bin server -- --addr 0.0.0.0:9000 --log-level debug
```

### Frontend Configuration

Update WebSocket URL in `frontend/src/services/WebSocketService.js`:

```javascript
this.url = 'ws://your-server-host:port';
```

## Testing

### Backend Testing

```bash
cd backend
cargo test                    # Unit tests
cargo run --bin server       # Manual testing
```

### Frontend Testing

```bash
cd frontend
npm test                      # React component tests
npm start                     # Manual testing in browser
```

### Integration Testing

1. Start backend server
2. Start frontend application
3. Verify WebSocket connection in browser console
4. Test symbol switching and mode changes
5. Monitor real-time data updates

## Production Deployment

### Backend Deployment

```bash
cd backend
cargo build --release
./target/release/server --addr 0.0.0.0:8080
```

### Frontend Deployment

```bash
cd frontend
npm run build
# Deploy 'build' directory to web server
```

## Dependencies

### Backend Dependencies
- Tokio (async runtime)
- Tungstenite (WebSocket)
- Serde (JSON serialization)
- DashMap (concurrent collections)
- Tracing (logging)

### Frontend Dependencies
- React 18+
- React Scripts (build tools)
- Modern browser with WebSocket support

## Contributing

1. **Backend changes**: Work in `backend/` directory
2. **Frontend changes**: Work in `frontend/` directory
3. **Protocol changes**: Update both backend and frontend
4. **Documentation**: Update relevant README files

## License

ISC

## Support

For issues:
- **Backend**: Check `backend/README.md`
- **Frontend**: Check `frontend/README.md`
- **Integration**: Verify both services are running and network connectivity