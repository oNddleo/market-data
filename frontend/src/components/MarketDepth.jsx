import React, { useState, useEffect, useRef, useCallback } from 'react';
import WebSocketService from '../services/WebSocketService';
import './MarketDepth.css';

const MarketDepth = () => {
    const [displayMode, setDisplayMode] = useState('MBP'); // 'MBO' or 'MBP'
    const [symbol, setSymbol] = useState('BTCUSD');
    const [bids, setBids] = useState([]);
    const [asks, setAsks] = useState([]);
    const [spread, setSpread] = useState(0);
    const [midPrice, setMidPrice] = useState(0);
    const [spreadBps, setSpreadBps] = useState(0);
    const [lastUpdate, setLastUpdate] = useState(Date.now());
    const [activityLog, setActivityLog] = useState([]);
    const [connectionStatus, setConnectionStatus] = useState('Disconnected');
    const [sequence, setSequence] = useState(0);
    const wsService = useRef(new WebSocketService());
    const animationRef = useRef();

    // Initialize WebSocket connection
    useEffect(() => {
        const connectWebSocket = async () => {
            try {
                setConnectionStatus('Connecting...');
                await wsService.current.connect();
                setConnectionStatus('Connected');
            } catch (error) {
                console.error('Failed to connect to WebSocket:', error);
                setConnectionStatus('Connection Failed');
            }
        };

        connectWebSocket();

        return () => {
            wsService.current.disconnect();
        };
    }, []);

    // Handle market data from WebSocket
    const handleMarketData = useCallback((message) => {
        try {
            const { data, sequence: msgSequence, timestamp } = message;

            if (data.format === 'MBO' && displayMode === 'MBO') {
                setBids(data.bids || []);
                setAsks(data.asks || []);
            } else if (data.format === 'MBP' && displayMode === 'MBP') {
                setBids(data.bids || []);
                setAsks(data.asks || []);
            }

            // Calculate spread from best bid/ask
            const bestBid = data.bids?.[0]?.price;
            const bestAsk = data.asks?.[0]?.price;

            if (bestBid && bestAsk) {
                const spreadValue = bestAsk - bestBid;
                const midPriceValue = (bestBid + bestAsk) / 2;
                const spreadBpsValue = (spreadValue / midPriceValue) * 10000;

                setSpread(spreadValue);
                setMidPrice(midPriceValue);
                setSpreadBps(spreadBpsValue);
            }

            setSequence(msgSequence);
            setLastUpdate(new Date(timestamp).getTime());
        } catch (error) {
            console.error('Error processing market data:', error);
        }
    }, [displayMode]);

    // Subscribe to market data when mode or symbol changes
    useEffect(() => {
        if (wsService.current.getConnectionStatus().isConnected) {
            // Unsubscribe from previous stream
            const oldStreamId = `${symbol}_${displayMode === 'MBO' ? 'MBP' : 'MBO'}`;
            wsService.current.unsubscribe(oldStreamId);

            // Subscribe to new stream
            const streamId = `${symbol}_${displayMode}`;
            wsService.current.subscribe(
                streamId,
                symbol,
                displayMode,
                20,
                handleMarketData
            );
        }
    }, [displayMode, symbol, handleMarketData]);



    // Monitor connection status
    useEffect(() => {
        const interval = setInterval(() => {
            const status = wsService.current.getConnectionStatus();
            if (status.isConnected) {
                setConnectionStatus('Connected');
            } else if (status.reconnectAttempts > 0) {
                setConnectionStatus(`Reconnecting... (${status.reconnectAttempts})`);
            } else {
                setConnectionStatus('Disconnected');
            }
        }, 1000);

        return () => clearInterval(interval);
    }, []);

    // Animation loop for smooth updates
    useEffect(() => {
        const animate = () => {
            // Process any pending updates
            animationRef.current = requestAnimationFrame(animate);
        };

        animationRef.current = requestAnimationFrame(animate);

        return () => {
            if (animationRef.current) {
                cancelAnimationFrame(animationRef.current);
            }
        };
    }, []);

    // Format time ago
    const formatTimeAgo = (timestamp) => {
        const seconds = Math.floor((Date.now() - timestamp) / 1000);
        if (seconds < 60) return `${seconds}s`;
        if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
        return `${Math.floor(seconds / 3600)}h`;
    };

    // Calculate cumulative data for display
    const calculateCumulative = (entries) => {
        if (displayMode === 'MBO') {
            // For MBO mode, no cumulative calculation needed
            return entries.map(entry => ({
                ...entry,
                total: entry.quantity
            }));
        }

        // For MBP mode, use total_quantity if available, otherwise calculate cumulative
        if (entries.length > 0 && entries[0].total_quantity !== undefined) {
            return entries; // Already has cumulative data from server
        }

        let total = 0;
        return entries.map(entry => {
            total += entry.quantity;
            return { ...entry, total };
        });
    };

    const processedBids = calculateCumulative(bids);
    const processedAsks = calculateCumulative(asks);

    const maxTotal = displayMode === 'MBP' ? Math.max(
        ...processedBids.map(b => b.total_quantity || b.total || b.quantity || 0),
        ...processedAsks.map(a => a.total_quantity || a.total || a.quantity || 0),
        1 // Ensure we have at least 1 to avoid division by zero
    ) : Math.max(
        ...processedBids.map(b => b.quantity || 0),
        ...processedAsks.map(a => a.quantity || 0),
        1 // Ensure we have at least 1 to avoid division by zero
    );

    return (
        <div className="market-depth">
            <div className="market-depth-header">
                <h2>Market Depth - Rust Server</h2>
                <div className="controls-section">
                    <div className="symbol-selector">
                        <label>Symbol:</label>
                        <select
                            value={symbol}
                            onChange={(e) => setSymbol(e.target.value)}
                            className="symbol-select"
                        >
                            <option value="BTCUSD">BTC/USD</option>
                            <option value="ETHUSD">ETH/USD</option>
                            <option value="ADAUSD">ADA/USD</option>
                        </select>
                    </div>
                    <div className="mode-controls">
                        <button
                            className={`mode-btn ${displayMode === 'MBP' ? 'active' : ''}`}
                            onClick={() => setDisplayMode('MBP')}
                        >
                            MBP
                        </button>
                        <button
                            className={`mode-btn ${displayMode === 'MBO' ? 'active' : ''}`}
                            onClick={() => setDisplayMode('MBO')}
                        >
                            MBO
                        </button>
                    </div>
                </div>
                <div className="market-stats">
                    <span className={`connection-status ${connectionStatus.toLowerCase().replace(/[^a-z]/g, '')}`}>
                        Status: {connectionStatus}
                    </span>
                    <span>Spread: {spread.toFixed(4)} ({spreadBps.toFixed(1)} bps)</span>
                    <span>Mid: {midPrice.toFixed(4)}</span>
                    <span>Seq: {sequence}</span>
                    <span>Updated: {formatTimeAgo(lastUpdate)}</span>
                </div>
            </div>

            <div className="depth-container">
                <div className="depth-side bids">
                    <div className="depth-header">
                        <span>Price</span>
                        <span>Size</span>
                        {displayMode === 'MBP' && <span>Total</span>}
                        {displayMode === 'MBO' && <span>Order ID</span>}
                        {displayMode === 'MBO' && <span>Age</span>}
                        {displayMode === 'MBP' && <span>Orders</span>}
                    </div>
                    {processedBids.map((bid, index) => (
                        displayMode === 'MBO' ? (
                            <MBORow
                                key={bid.orderId || index}
                                order={bid}
                                maxQuantity={maxTotal}
                                isBid={true}
                            />
                        ) : (
                            <MBPRow
                                key={index}
                                level={bid}
                                maxTotal={maxTotal}
                                isBid={true}
                            />
                        )
                    ))}
                </div>

                <div className="depth-side asks">
                    <div className="depth-header">
                        <span>Price</span>
                        <span>Size</span>
                        {displayMode === 'MBP' && <span>Total</span>}
                        {displayMode === 'MBO' && <span>Order ID</span>}
                        {displayMode === 'MBO' && <span>Age</span>}
                        {displayMode === 'MBP' && <span>Orders</span>}
                    </div>
                    {processedAsks.map((ask, index) => (
                        displayMode === 'MBO' ? (
                            <MBORow
                                key={ask.orderId || index}
                                order={ask}
                                maxQuantity={maxTotal}
                                isBid={false}
                            />
                        ) : (
                            <MBPRow
                                key={index}
                                level={ask}
                                maxTotal={maxTotal}
                                isBid={false}
                            />
                        )
                    ))}
                </div>
            </div>

            {/* Connection Info */}
            <div className="connection-info">
                <div className="connection-details">
                    <h3>Server Connection</h3>
                    <div className="connection-stats">
                        <span>WebSocket URL: ws://127.0.0.1:8080</span>
                        <span>Current Symbol: {symbol}</span>
                        <span>Data Type: {displayMode}</span>
                        <span>Status: {connectionStatus}</span>
                    </div>
                </div>
            </div>
        </div>
    );
};

// MBP (Market By Price) Row Component
const MBPRow = React.memo(({ level, maxTotal, isBid }) => {
    const total = level.total_quantity || level.total || level.quantity;
    const percentage = (total / maxTotal) * 100;

    return (
        <div className={`depth-row mbp-row ${isBid ? 'bid' : 'ask'}`}>
            <div
                className="depth-background"
                style={{
                    width: `${percentage}%`,
                    backgroundColor: isBid ? 'rgba(0, 255, 0, 0.1)' : 'rgba(255, 0, 0, 0.1)'
                }}
            />
            <span className="price">{level.price.toFixed(4)}</span>
            <span className="quantity">{level.quantity.toLocaleString()}</span>
            <span className="total">{total.toLocaleString()}</span>
            <span className="order-count">{level.order_count || level.orderCount || 0}</span>
        </div>
    );
});

// MBO (Market By Order) Row Component
const MBORow = React.memo(({ order, maxQuantity, isBid }) => {
    // Defensive checks for order properties
    if (!order) return null;

    const quantity = order.quantity || 0;
    const age = order.age_ms || order.age || 0;
    const price = order.price || 0;
    const orderId = order.order_id || order.orderId;

    const percentage = (quantity / maxQuantity) * 100;
    const ageColor = age < 10000 ? '#00ff00' :
        age < 30000 ? '#ffff00' : '#ff6600';

    const formatAge = (age) => {
        if (age < 1000) return `${Math.floor(age)}ms`;
        if (age < 60000) return `${Math.floor(age / 1000)}s`;
        return `${Math.floor(age / 60000)}m`;
    };

    return (
        <div className={`depth-row mbo-row ${isBid ? 'bid' : 'ask'}`}>
            <div
                className="depth-background"
                style={{
                    width: `${percentage}%`,
                    backgroundColor: isBid ? 'rgba(0, 255, 0, 0.05)' : 'rgba(255, 0, 0, 0.05)'
                }}
            />
            <span className="price">{price.toFixed(4)}</span>
            <span className="quantity">{quantity.toLocaleString()}</span>
            <span className="order-id" title={orderId || 'N/A'}>
                {orderId ? orderId.slice(-8) : 'N/A'}
            </span>
            <span className="age" style={{ color: ageColor }}>
                {formatAge(age)}
            </span>
        </div>
    );
});

export default MarketDepth;