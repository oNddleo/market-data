// Order Book utilities for MBO and MBP implementations

// MBO (Market By Order) - Shows individual orders
// MBP (Market By Price) - Aggregates orders by price level

export class Order {
    constructor(id, price, quantity, side, timestamp = Date.now()) {
        this.id = id;
        this.price = parseFloat(price);
        this.quantity = parseInt(quantity);
        this.side = side; // 'bid' or 'ask'
        this.timestamp = timestamp;
        this.originalQuantity = this.quantity;
    }

    update(newQuantity) {
        this.quantity = parseInt(newQuantity);
        this.timestamp = Date.now();
    }

    getAge() {
        return Date.now() - this.timestamp;
    }
}

export class OrderBook {
    constructor() {
        this.orders = new Map(); // orderId -> Order
        this.bidsByPrice = new Map(); // price -> Set of Order IDs
        this.asksByPrice = new Map(); // price -> Set of Order IDs
        this.sequence = 0;
    }

    // Add or update an order
    addOrder(order) {
        const existingOrder = this.orders.get(order.id);
        if (existingOrder) {
            this.removeOrder(order.id);
        }

        this.orders.set(order.id, order);
        const priceMap = order.side === 'bid' ? this.bidsByPrice : this.asksByPrice;
        
        if (!priceMap.has(order.price)) {
            priceMap.set(order.price, new Set());
        }
        priceMap.get(order.price).add(order.id);
        this.sequence++;
    }

    // Remove an order
    removeOrder(orderId) {
        const order = this.orders.get(orderId);
        if (!order) return false;

        const priceMap = order.side === 'bid' ? this.bidsByPrice : this.asksByPrice;
        const orderSet = priceMap.get(order.price);
        
        if (orderSet) {
            orderSet.delete(orderId);
            if (orderSet.size === 0) {
                priceMap.delete(order.price);
            }
        }
        
        this.orders.delete(orderId);
        this.sequence++;
        return true;
    }

    // Update order quantity
    updateOrder(orderId, newQuantity) {
        const order = this.orders.get(orderId);
        if (!order) return false;

        if (newQuantity <= 0) {
            return this.removeOrder(orderId);
        }

        order.update(newQuantity);
        this.sequence++;
        return true;
    }

    // Get MBO data (Market By Order)
    getMBOData(maxLevels = 20) {
        const bids = this.getMBOSide('bid', maxLevels);
        const asks = this.getMBOSide('ask', maxLevels);
        
        return {
            bids,
            asks,
            sequence: this.sequence,
            timestamp: Date.now()
        };
    }

    getMBOSide(side, maxLevels) {
        const priceMap = side === 'bid' ? this.bidsByPrice : this.asksByPrice;
        const prices = Array.from(priceMap.keys()).sort((a, b) => 
            side === 'bid' ? b - a : a - b // Bids descending, asks ascending
        );

        const result = [];
        let levelCount = 0;

        for (const price of prices) {
            if (levelCount >= maxLevels) break;

            const orderIds = priceMap.get(price);
            const orders = Array.from(orderIds)
                .map(id => this.orders.get(id))
                .filter(order => order)
                .sort((a, b) => a.timestamp - b.timestamp); // FIFO order

            for (const order of orders) {
                result.push({
                    orderId: order.id,
                    price: order.price,
                    quantity: order.quantity,
                    side: order.side,
                    timestamp: order.timestamp,
                    age: order.getAge()
                });
            }
            levelCount++;
        }

        return result.slice(0, maxLevels * 3); // Limit total orders shown
    }

    // Get MBP data (Market By Price)
    getMBPData(maxLevels = 20) {
        const bids = this.getMBPSide('bid', maxLevels);
        const asks = this.getMBPSide('ask', maxLevels);
        
        return {
            bids,
            asks,
            sequence: this.sequence,
            timestamp: Date.now()
        };
    }

    getMBPSide(side, maxLevels) {
        const priceMap = side === 'bid' ? this.bidsByPrice : this.asksByPrice;
        const prices = Array.from(priceMap.keys()).sort((a, b) => 
            side === 'bid' ? b - a : a - b
        );

        return prices.slice(0, maxLevels).map(price => {
            const orderIds = priceMap.get(price);
            const orders = Array.from(orderIds).map(id => this.orders.get(id));
            
            const totalQuantity = orders.reduce((sum, order) => sum + order.quantity, 0);
            const orderCount = orders.length;
            const avgAge = orders.reduce((sum, order) => sum + order.getAge(), 0) / orderCount;

            return {
                price,
                quantity: totalQuantity,
                orderCount,
                avgAge: Math.round(avgAge),
                side
            };
        });
    }

    // Get best bid and ask
    getBestBidAsk() {
        const bestBid = this.bidsByPrice.size > 0 ? 
            Math.max(...this.bidsByPrice.keys()) : null;
        const bestAsk = this.asksByPrice.size > 0 ? 
            Math.min(...this.asksByPrice.keys()) : null;

        return { bestBid, bestAsk };
    }

    // Get spread and mid price
    getSpreadInfo() {
        const { bestBid, bestAsk } = this.getBestBidAsk();
        
        if (!bestBid || !bestAsk) {
            return { spread: null, midPrice: null };
        }

        return {
            spread: bestAsk - bestBid,
            midPrice: (bestBid + bestAsk) / 2,
            spreadBps: ((bestAsk - bestBid) / ((bestBid + bestAsk) / 2)) * 10000
        };
    }

    // Generate realistic market activity
    simulateMarketActivity() {
        const activities = [];
        const numActivities = Math.floor(Math.random() * 10) + 5; // 5-15 activities

        for (let i = 0; i < numActivities; i++) {
            const activity = this.generateRandomActivity();
            activities.push(activity);
            this.executeActivity(activity);
        }

        return activities;
    }

    generateRandomActivity() {
        const { bestBid, bestAsk } = this.getBestBidAsk();
        const midPrice = bestBid && bestAsk ? (bestBid + bestAsk) / 2 : 100;
        
        const activityType = Math.random();
        const side = Math.random() < 0.5 ? 'bid' : 'ask';
        
        if (activityType < 0.4) { // 40% new orders
            const basePrice = side === 'bid' ? (bestBid || midPrice - 0.05) : (bestAsk || midPrice + 0.05);
            const priceVariation = (Math.random() - 0.5) * 0.2; // ±0.10 variation
            const price = Math.max(0.01, basePrice + priceVariation);
            
            return {
                type: 'add',
                orderId: `order_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`,
                price: Math.round(price * 100) / 100,
                quantity: Math.floor(Math.random() * 5000) + 1000,
                side
            };
        } else if (activityType < 0.7) { // 30% order updates
            const existingOrders = Array.from(this.orders.values());
            if (existingOrders.length > 0) {
                const order = existingOrders[Math.floor(Math.random() * existingOrders.length)];
                const newQuantity = Math.max(0, order.quantity + (Math.random() - 0.6) * 1000);
                
                return {
                    type: newQuantity > 0 ? 'update' : 'cancel',
                    orderId: order.id,
                    quantity: Math.floor(newQuantity)
                };
            }
        }
        
        // 30% order cancellations
        const existingOrders = Array.from(this.orders.values());
        if (existingOrders.length > 0) {
            const order = existingOrders[Math.floor(Math.random() * existingOrders.length)];
            return {
                type: 'cancel',
                orderId: order.id
            };
        }

        // Fallback to add order
        return {
            type: 'add',
            orderId: `order_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`,
            price: Math.round((midPrice + (Math.random() - 0.5) * 2) * 100) / 100,
            quantity: Math.floor(Math.random() * 5000) + 1000,
            side
        };
    }

    executeActivity(activity) {
        switch (activity.type) {
            case 'add':
                const order = new Order(
                    activity.orderId,
                    activity.price,
                    activity.quantity,
                    activity.side
                );
                this.addOrder(order);
                break;
            
            case 'update':
                this.updateOrder(activity.orderId, activity.quantity);
                break;
            
            case 'cancel':
                this.removeOrder(activity.orderId);
                break;
        }
    }

    // Initialize with sample data
    initializeWithSampleData() {
        const basePrice = 100;
        
        // Add initial bid orders
        for (let i = 0; i < 30; i++) {
            const price = basePrice - 0.05 - (i * 0.01);
            const quantity = Math.floor(Math.random() * 10000) + 1000;
            const order = new Order(
                `bid_${i}`,
                price,
                quantity,
                'bid',
                Date.now() - Math.random() * 60000 // Orders up to 1 minute old
            );
            this.addOrder(order);
        }

        // Add initial ask orders
        for (let i = 0; i < 30; i++) {
            const price = basePrice + (i * 0.01);
            const quantity = Math.floor(Math.random() * 10000) + 1000;
            const order = new Order(
                `ask_${i}`,
                price,
                quantity,
                'ask',
                Date.now() - Math.random() * 60000
            );
            this.addOrder(order);
        }
    }
}