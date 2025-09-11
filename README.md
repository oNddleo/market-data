# OrderBook Implementation

A high-performance order book implementation for financial market data processing, supporting both Market By Order (MBO) and Market By Price (MBP) data models.

## Overview

This order book implementation provides a complete solution for managing and visualizing real-time market depth data. It supports both individual order tracking (MBO) and aggregated price level data (MBP), making it suitable for various trading applications.

## Classes

### Order Class

Represents an individual order in the order book.

#### Constructor
```javascript
new Order(id, price, quantity, side, timestamp = Date.now())
```

#### Properties
- `id` (string): Unique order identifier
- `price` (number): Order price (automatically parsed to float)
- `quantity` (number): Order quantity (automatically parsed to integer)
- `side` (string): Order side - either 'bid' or 'ask'
- `timestamp` (number): Order creation/update timestamp
- `originalQuantity` (number): Initial order quantity

#### Methods
- `update(newQuantity)`: Updates order quantity and timestamp
- `getAge()`: Returns order age in milliseconds

### OrderBook Class

Main order book implementation managing all orders and providing market data.

#### Constructor
```javascript
new OrderBook()
```

#### Properties
- `orders` (Map): Map of orderId → Order objects
- `bidsByPrice` (Map): Map of price → Set of Order IDs for bid side
- `asksByPrice` (Map): Map of price → Set of Order IDs for ask side
- `sequence` (number): Monotonic sequence number for updates

## Key Methods

### Order Management

#### `addOrder(order)`
Adds or updates an order in the book.
- **Parameters**: `order` (Order) - Order instance to add
- **Behavior**: If order ID exists, removes old order first

#### `removeOrder(orderId)`
Removes an order from the book.
- **Parameters**: `orderId` (string) - Order ID to remove
- **Returns**: boolean - true if order was found and removed

#### `updateOrder(orderId, newQuantity)`
Updates order quantity.
- **Parameters**: 
  - `orderId` (string) - Order ID to update
  - `newQuantity` (number) - New quantity (order removed if ≤ 0)
- **Returns**: boolean - true if order was found and updated

### Market Data Access

#### `getMBOData(maxLevels = 20)`
Returns Market By Order data showing individual orders.
- **Parameters**: `maxLevels` (number) - Maximum price levels to return
- **Returns**: Object with structure:
```javascript
{
  bids: [
    {
      orderId: "order_123",
      price: 99.95,
      quantity: 1000,
      side: "bid",
      timestamp: 1642684800000,
      age: 5000
    }
  ],
  asks: [...],
  sequence: 150,
  timestamp: 1642684805000
}
```

#### `getMBPData(maxLevels = 20)`
Returns Market By Price data with aggregated price levels.
- **Parameters**: `maxLevels` (number) - Maximum price levels to return
- **Returns**: Object with structure:
```javascript
{
  bids: [
    {
      price: 99.95,
      quantity: 5000,
      orderCount: 3,
      avgAge: 15000,
      side: "bid"
    }
  ],
  asks: [...],
  sequence: 150,
  timestamp: 1642684805000
}
```

### Market Information

#### `getBestBidAsk()`
Gets best bid and ask prices.
- **Returns**: `{ bestBid: number|null, bestAsk: number|null }`

#### `getSpreadInfo()`
Calculates spread and mid price.
- **Returns**: Object with:
```javascript
{
  spread: 0.05,           // Absolute spread
  midPrice: 99.975,       // Mid price
  spreadBps: 50.03        // Spread in basis points
}
```

### Market Simulation

#### `simulateMarketActivity()`
Generates and executes random market activities.
- **Returns**: Array of executed activities
- **Behavior**: Executes 5-15 random activities (40% adds, 30% updates, 30% cancels)

#### `initializeWithSampleData()`
Populates order book with initial sample data.
- **Behavior**: Creates 30 bid and 30 ask orders around $100 base price

## Usage Examples

### Basic Order Book Operations

```javascript
import { OrderBook, Order } from './src/utils/OrderBook.js';

// Create order book
const orderBook = new OrderBook();

// Add orders
const buyOrder = new Order('buy_1', 99.50, 1000, 'bid');
const sellOrder = new Order('sell_1', 100.50, 1500, 'ask');

orderBook.addOrder(buyOrder);
orderBook.addOrder(sellOrder);

// Get market data
const mboData = orderBook.getMBOData(10);
const mbpData = orderBook.getMBPData(10);

// Get market info
const { bestBid, bestAsk } = orderBook.getBestBidAsk();
const spreadInfo = orderBook.getSpreadInfo();

console.log(`Best Bid: ${bestBid}, Best Ask: ${bestAsk}`);
console.log(`Spread: ${spreadInfo.spread} (${spreadInfo.spreadBps} bps)`);
```

### Real-time Market Simulation

```javascript
// Initialize with sample data
orderBook.initializeWithSampleData();

// Simulate market activity
setInterval(() => {
  const activities = orderBook.simulateMarketActivity();
  const marketData = orderBook.getMBPData();
  
  // Update UI with new market data
  updateMarketDisplay(marketData);
  logActivities(activities);
}, 300); // 300ms intervals
```

### Order Lifecycle Management

```javascript
// Add new order
const order = new Order('trader_001', 99.75, 2000, 'bid');
orderBook.addOrder(order);

// Update order quantity
orderBook.updateOrder('trader_001', 1500);

// Check order age
const orderObj = orderBook.orders.get('trader_001');
console.log(`Order age: ${orderObj.getAge()}ms`);

// Cancel order
orderBook.removeOrder('trader_001');
```

## Data Models

### MBO (Market By Order)
- Shows individual orders with unique IDs
- Maintains FIFO order priority within price levels
- Includes order age for visualization
- Suitable for detailed order flow analysis

### MBP (Market By Price)
- Aggregates orders by price level
- Shows total quantity and order count per level
- Calculates average order age per level
- Traditional market depth view

## Performance Features

- Efficient data structures using Maps and Sets
- O(log n) price level ordering
- Memory-efficient order tracking
- Optimized for high-frequency updates

## Integration

This order book is designed to work with React components and supports:
- Real-time market data updates
- Professional trading interface visualization
- WebAssembly integration for enhanced performance
- Responsive design for multiple screen sizes

## Dependencies

- Pure JavaScript implementation (ES6+)
- No external dependencies required
- Compatible with modern browsers and Node.js