import React, { useState, useEffect, useRef, useCallback } from 'react';
import { OrderBook } from '../utils/OrderBook';
import './MarketDepth.css';

const MarketDepth = () => {
    const [displayMode, setDisplayMode] = useState('MBP'); // 'MBO' or 'MBP'
    const [bids, setBids] = useState([]);
    const [asks, setAsks] = useState([]);
    const [spread, setSpread] = useState(0);
    const [midPrice, setMidPrice] = useState(0);
    const [spreadBps, setSpreadBps] = useState(0);
    const [orderBook] = useState(() => new OrderBook());
    const [lastUpdate, setLastUpdate] = useState(Date.now());
    const [activityLog, setActivityLog] = useState([]);
    const animationRef = useRef();

    // Initialize order book
    useEffect(() => {
        orderBook.initializeWithSampleData();
        // Initial data load
        const data = orderBook.getMBPData(20);
        setBids(data.bids);
        setAsks(data.asks);

        const spreadInfo = orderBook.getSpreadInfo();
        if (spreadInfo.spread !== null) {
            setSpread(spreadInfo.spread);
            setMidPrice(spreadInfo.midPrice);
            setSpreadBps(spreadInfo.spreadBps);
        }
        setLastUpdate(Date.now());
    }, []);

    // Update display data based on current mode
    const updateDisplayData = useCallback(() => {
        try {
            const data = displayMode === 'MBO' ?
                orderBook.getMBOData(20) :
                orderBook.getMBPData(20);

            setBids(data.bids);
            setAsks(data.asks);

            // Update spread info
            const spreadInfo = orderBook.getSpreadInfo();
            if (spreadInfo.spread !== null) {
                setSpread(spreadInfo.spread);
                setMidPrice(spreadInfo.midPrice);
                setSpreadBps(spreadInfo.spreadBps);
            }

            setLastUpdate(Date.now());
        } catch (error) {
            console.error('Error updating display data:', error);
        }
    }, [displayMode]);

    // Market activity simulation
    useEffect(() => {
        const simulateActivity = () => {
            const activities = orderBook.simulateMarketActivity();
            setActivityLog(prev => [...activities, ...prev].slice(0, 100)); // Keep last 100 activities
            updateDisplayData();
        };

        // Simulate market activity every 300ms
        const interval = setInterval(simulateActivity, 300);

        return () => {
            clearInterval(interval);
        };
    }, [updateDisplayData]);



    // Update display when mode changes
    useEffect(() => {
        updateDisplayData();
    }, [updateDisplayData]);

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

    // Calculate cumulative data for MBP mode
    const calculateCumulative = (entries) => {
        if (displayMode === 'MBO') {
            // For MBO mode, ensure each entry has necessary properties
            return entries.map(entry => ({
                ...entry,
                total: entry.quantity // For MBO, total equals quantity
            }));
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
        ...processedBids.map(b => b.total || b.quantity),
        ...processedAsks.map(a => a.total || a.quantity)
    ) : Math.max(
        ...processedBids.map(b => b.quantity),
        ...processedAsks.map(a => a.quantity)
    );

    return (
        <div className="market-depth">
            <div className="market-depth-header">
                <h2>Market Depth</h2>
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
                <div className="market-stats">
                    <span>Spread: {spread.toFixed(4)} ({spreadBps.toFixed(1)} bps)</span>
                    <span>Mid: {midPrice.toFixed(4)}</span>
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

            {/* Activity Log */}
            <div className="activity-log">
                <h3>Recent Activity</h3>
                <div className="activity-items">
                    {activityLog.slice(0, 10).map((activity, index) => (
                        <div key={index} className={`activity-item ${activity.type}`}>
                            <span className="activity-type">{activity.type.toUpperCase()}</span>
                            <span className="activity-side">{activity.side || 'N/A'}</span>
                            <span className="activity-price">${activity.price?.toFixed(4) || 'N/A'}</span>
                            <span className="activity-quantity">{activity.quantity?.toLocaleString() || 'N/A'}</span>
                            <span className="activity-id">{activity.orderId ? activity.orderId.slice(-6) : 'N/A'}</span>
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
};

// MBP (Market By Price) Row Component
const MBPRow = React.memo(({ level, maxTotal, isBid }) => {
    const percentage = (level.total / maxTotal) * 100;

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
            <span className="total">{level.total.toLocaleString()}</span>
            <span className="order-count">{level.orderCount}</span>
        </div>
    );
});

// MBO (Market By Order) Row Component
const MBORow = React.memo(({ order, maxQuantity, isBid }) => {
    // Defensive checks for order properties
    if (!order) return null;
    
    const quantity = order.quantity || 0;
    const age = order.age || 0;
    const price = order.price || 0;
    
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
            <span className="order-id" title={order.orderId || 'N/A'}>
                {order.orderId ? order.orderId.slice(-8) : 'N/A'}
            </span>
            <span className="age" style={{ color: ageColor }}>
                {formatAge(age)}
            </span>
        </div>
    );
});

export default MarketDepth;