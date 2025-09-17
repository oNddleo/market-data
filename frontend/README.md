# Market Depth Frontend

A React-based real-time market depth visualization application that connects to the Rust WebSocket server for live market data streaming.

## Features

- **Real-time market depth visualization** with professional trading interface
- **Dynamic mode switching** between MBO (Market By Order) and MBP (Market By Price)
- **WebSocket client service** with automatic reconnection
- **Symbol selection** for multiple trading pairs
- **Connection status monitoring** with visual indicators
- **Responsive design** optimized for trading workflows

## Architecture

### Core Components

- **MarketDepth**: Main component with order book visualization
- **WebSocketService**: WebSocket client with reconnection logic
- **Market Data Display**: Real-time bid/ask visualization with depth charts

### Data Visualization

- **MBO Mode**: Individual orders with IDs, timestamps, and age indicators
- **MBP Mode**: Aggregated price levels with quantities and order counts
- **Depth Charts**: Visual representation of market liquidity
- **Spread Calculations**: Real-time spread in basis points

## Prerequisites

- Node.js (16+)
- npm or yarn
- Market Depth Backend server running

## Quick Start

```bash
# Navigate to frontend directory
cd frontend

# Install dependencies
npm install

# Start development server
npm start

# Open browser to http://localhost:3000
```

## Development Commands

```bash
# Start development server
npm start

# Build for production
npm run build

# Run tests
npm test

# Eject from Create React App (irreversible)
npm run eject
```

## Project Structure

```
frontend/
├── public/                 # Static assets
│   ├── index.html         # HTML template
│   └── favicon.ico        # Favicon
├── src/
│   ├── components/        # React components
│   │   ├── MarketDepth.jsx    # Main market depth component
│   │   └── MarketDepth.css    # Component styles
│   ├── services/          # WebSocket and API services
│   │   └── WebSocketService.js # WebSocket client
│   ├── utils/             # Utility functions (legacy)
│   │   └── OrderBook.js   # Original order book implementation
│   ├── App.jsx            # Root application component
│   └── index.js           # Application entry point
├── package.json           # Dependencies and scripts
└── README.md             # This file
```

## Configuration

### WebSocket Connection

The frontend connects to the backend WebSocket server:

```javascript
// Default configuration in WebSocketService.js
const wsUrl = 'ws://127.0.0.1:8080';
```

To change the server URL, modify `src/services/WebSocketService.js`:

```javascript
constructor() {
    this.url = 'ws://your-server-host:port';
}
```

### Supported Symbols

The application supports these trading pairs:
- BTCUSD (Bitcoin/USD)
- ETHUSD (Ethereum/USD)
- ADAUSD (Cardano/USD)

Add more symbols by updating the symbol selector in `MarketDepth.jsx`.

## Features in Detail

### Market Data Modes

#### MBO (Market By Order)
- Shows individual orders with unique IDs
- Displays order age with color coding
- FIFO order priority within price levels
- Real-time order lifecycle tracking

#### MBP (Market By Price)
- Aggregates orders by price level
- Shows total quantity and order count
- Displays cumulative depth visualization
- Traditional market depth view

### WebSocket Integration

The frontend uses a robust WebSocket service with:
- **Automatic reconnection** with exponential backoff
- **Subscription management** for multiple streams
- **Message routing** to appropriate handlers
- **Error handling** and recovery

### User Interface

- **Professional trading theme** with dark background
- **Color-coded price levels** (green for bids, red for asks)
- **Real-time updates** without flicker
- **Responsive layout** for different screen sizes
- **Connection status indicator** in header

## WebSocket Message Handling

The frontend handles these message types:

```javascript
// Market data updates
{
  type: 'MarketData',
  stream_id: 'btc_mbp',
  symbol: 'BTCUSD',
  data: { bids: [...], asks: [...] }
}

// Subscription confirmations
{
  type: 'Subscribed',
  stream_id: 'btc_mbp',
  symbol: 'BTCUSD'
}

// Server heartbeats
{
  type: 'HeartBeat',
  timestamp: '2025-09-16T04:18:26.806069Z'
}
```

## Customization

### Styling

Modify `src/components/MarketDepth.css` to customize:
- Color schemes
- Layout dimensions
- Animation effects
- Responsive breakpoints

### Market Data Display

Update `MarketDepth.jsx` to:
- Add new visualization modes
- Modify update intervals
- Change depth levels displayed
- Add new market indicators

### WebSocket Behavior

Customize `WebSocketService.js` for:
- Different reconnection strategies
- Message transformation
- Error handling policies
- Connection pooling

## Performance Optimization

The application includes several optimizations:
- **React.memo** for order row components
- **Efficient state updates** to prevent unnecessary re-renders
- **Optimized CSS** with hardware acceleration hints
- **Debounced updates** for smooth animations

## Browser Compatibility

Supports all modern browsers with WebSocket API:
- Chrome 16+
- Firefox 11+
- Safari 7+
- Edge 12+

## Troubleshooting

### Connection Issues
1. Ensure backend server is running on ws://127.0.0.1:8080
2. Check browser console for WebSocket errors
3. Verify network connectivity and firewall settings

### Performance Issues
1. Check browser performance tab for bottlenecks
2. Reduce update frequency if needed
3. Limit number of visible order levels

### Display Issues
1. Hard refresh browser (Ctrl+F5)
2. Clear browser cache
3. Check for JavaScript errors in console

## Development

### Adding New Features

1. **New Market Indicators**: Add calculations in `MarketDepth.jsx`
2. **Additional Symbols**: Update symbol selector options
3. **New Visualization**: Create new components in `components/`
4. **Enhanced WebSocket**: Extend `WebSocketService.js`

### Testing

```bash
# Run unit tests
npm test

# Run with coverage
npm test -- --coverage

# Run in watch mode
npm test -- --watch
```

## Build and Deployment

### Production Build

```bash
# Create optimized build
npm run build

# Serve static files
npx serve -s build
```

### Environment Variables

Create `.env` file for configuration:

```
REACT_APP_WS_URL=ws://localhost:8080
REACT_APP_UPDATE_INTERVAL=300
```

## License

ISC