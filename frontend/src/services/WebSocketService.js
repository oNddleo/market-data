class WebSocketService {
    constructor() {
        this.ws = null;
        this.subscriptions = new Map();
        this.messageHandlers = new Map();
        this.isConnected = false;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.reconnectDelay = 1000;
        this.url = 'ws://127.0.0.1:8080';
    }

    connect() {
        return new Promise((resolve, reject) => {
            try {
                this.ws = new WebSocket(this.url);

                this.ws.onopen = () => {
                    console.log('WebSocket connected to', this.url);
                    this.isConnected = true;
                    this.reconnectAttempts = 0;
                    resolve();
                };

                this.ws.onmessage = (event) => {
                    try {
                        const message = JSON.parse(event.data);
                        this.handleMessage(message);
                    } catch (error) {
                        console.error('Failed to parse WebSocket message:', error);
                    }
                };

                this.ws.onclose = (event) => {
                    console.log('WebSocket connection closed:', event.code, event.reason);
                    this.isConnected = false;
                    this.handleReconnect();
                };

                this.ws.onerror = (error) => {
                    console.error('WebSocket error:', error);
                    if (!this.isConnected) {
                        reject(error);
                    }
                };

            } catch (error) {
                reject(error);
            }
        });
    }

    handleReconnect() {
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            this.reconnectAttempts++;
            console.log(`Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`);

            setTimeout(() => {
                this.connect().then(() => {
                    // Resubscribe to all active subscriptions
                    this.resubscribe();
                }).catch(error => {
                    console.error('Reconnection failed:', error);
                });
            }, this.reconnectDelay * this.reconnectAttempts);
        } else {
            console.error('Max reconnection attempts reached');
        }
    }

    resubscribe() {
        for (const [streamId, subscription] of this.subscriptions) {
            this.send({
                type: 'Subscribe',
                stream_id: streamId,
                symbol: subscription.symbol,
                data_type: subscription.dataType,
                max_levels: subscription.maxLevels
            });
        }
    }

    handleMessage(message) {
        console.log('Received message:', message);

        switch (message.type) {
            case 'MarketData':
                this.handleMarketData(message);
                break;
            case 'Subscribed':
                this.handleSubscribed(message);
                break;
            case 'Unsubscribed':
                this.handleUnsubscribed(message);
                break;
            case 'HeartBeat':
                this.handleHeartBeat(message);
                break;
            case 'Error':
                this.handleError(message);
                break;
            default:
                console.warn('Unknown message type:', message.type);
        }
    }

    handleMarketData(message) {
        const handler = this.messageHandlers.get(message.stream_id);
        if (handler) {
            handler(message);
        }
    }

    handleSubscribed(message) {
        console.log(`Subscribed to stream: ${message.stream_id} for ${message.symbol}`);
    }

    handleUnsubscribed(message) {
        console.log(`Unsubscribed from stream: ${message.stream_id}`);
        this.subscriptions.delete(message.stream_id);
        this.messageHandlers.delete(message.stream_id);
    }

    handleHeartBeat(message) {
        console.debug('Received heartbeat:', message.timestamp);
    }

    handleError(message) {
        console.error('Server error:', message.code, message.message);
        if (message.stream_id) {
            console.error('Error for stream:', message.stream_id);
        }
    }

    send(message) {
        if (this.ws && this.isConnected && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message));
        } else {
            console.error('WebSocket is not connected');
        }
    }

    subscribe(streamId, symbol, dataType, maxLevels = 20, handler) {
        if (this.subscriptions.has(streamId)) {
            console.warn(`Already subscribed to stream: ${streamId}`);
            return;
        }

        const subscription = {
            symbol,
            dataType,
            maxLevels
        };

        this.subscriptions.set(streamId, subscription);
        this.messageHandlers.set(streamId, handler);

        this.send({
            type: 'Subscribe',
            stream_id: streamId,
            symbol,
            data_type: dataType,
            max_levels: maxLevels
        });

        console.log(`Subscribing to ${dataType} stream for ${symbol} (${streamId})`);
    }

    unsubscribe(streamId) {
        if (!this.subscriptions.has(streamId)) {
            console.warn(`Not subscribed to stream: ${streamId}`);
            return;
        }

        this.send({
            type: 'Unsubscribe',
            stream_id: streamId
        });

        console.log(`Unsubscribing from stream: ${streamId}`);
    }

    ping() {
        this.send({
            type: 'Ping',
            timestamp: new Date().toISOString()
        });
    }

    disconnect() {
        if (this.ws) {
            this.ws.close();
            this.ws = null;
            this.isConnected = false;
            this.subscriptions.clear();
            this.messageHandlers.clear();
        }
    }

    getConnectionStatus() {
        return {
            isConnected: this.isConnected,
            reconnectAttempts: this.reconnectAttempts,
            subscriptionCount: this.subscriptions.size
        };
    }
}

export default WebSocketService;