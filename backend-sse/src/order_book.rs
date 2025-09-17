use std::collections::{BTreeMap, HashMap};
use chrono::{DateTime, Utc};
use rand::{thread_rng, Rng};

use crate::message::{MBOLevel, MBPLevel, Side, OrderActivity, ActivityType};

#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub price: f64,
    pub quantity: u64,
    pub side: Side,
    pub timestamp: DateTime<Utc>,
    pub original_quantity: u64,
}

impl Order {
    pub fn new(id: String, price: f64, quantity: u64, side: Side) -> Self {
        let timestamp = Utc::now();
        Self {
            id,
            price,
            quantity,
            side,
            timestamp,
            original_quantity: quantity,
        }
    }

    pub fn update_quantity(&mut self, new_quantity: u64) {
        self.quantity = new_quantity;
        self.timestamp = Utc::now();
    }

    pub fn age_ms(&self) -> u64 {
        let now = Utc::now();
        (now - self.timestamp).num_milliseconds().max(0) as u64
    }
}

#[derive(Debug)]
pub struct OrderBook {
    pub symbol: String,
    orders: HashMap<String, Order>,
    bids_by_price: BTreeMap<OrderedFloat, Vec<String>>,
    asks_by_price: BTreeMap<OrderedFloat, Vec<String>>,
    sequence: u64,
}

// Wrapper for f64 to make it Ord for BTreeMap
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct OrderedFloat(f64);

impl Eq for OrderedFloat {}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl From<f64> for OrderedFloat {
    fn from(f: f64) -> Self {
        OrderedFloat(f)
    }
}

impl OrderBook {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            orders: HashMap::new(),
            bids_by_price: BTreeMap::new(),
            asks_by_price: BTreeMap::new(),
            sequence: 0,
        }
    }

    pub fn add_order(&mut self, order: Order) -> bool {
        if self.orders.contains_key(&order.id) {
            self.remove_order(&order.id);
        }

        let price_key = OrderedFloat::from(order.price);
        let order_id = order.id.clone();
        let side = order.side.clone();

        self.orders.insert(order_id.clone(), order);

        match side {
            Side::Bid => {
                self.bids_by_price
                    .entry(price_key)
                    .or_insert_with(Vec::new)
                    .push(order_id);
            }
            Side::Ask => {
                self.asks_by_price
                    .entry(price_key)
                    .or_insert_with(Vec::new)
                    .push(order_id);
            }
        }

        self.sequence += 1;
        true
    }

    pub fn remove_order(&mut self, order_id: &str) -> bool {
        if let Some(order) = self.orders.remove(order_id) {
            let price_key = OrderedFloat::from(order.price);

            match order.side {
                Side::Bid => {
                    if let Some(orders_at_price) = self.bids_by_price.get_mut(&price_key) {
                        orders_at_price.retain(|id| id != order_id);
                        if orders_at_price.is_empty() {
                            self.bids_by_price.remove(&price_key);
                        }
                    }
                }
                Side::Ask => {
                    if let Some(orders_at_price) = self.asks_by_price.get_mut(&price_key) {
                        orders_at_price.retain(|id| id != order_id);
                        if orders_at_price.is_empty() {
                            self.asks_by_price.remove(&price_key);
                        }
                    }
                }
            }

            self.sequence += 1;
            true
        } else {
            false
        }
    }

    pub fn update_order(&mut self, order_id: &str, new_quantity: u64) -> bool {
        if new_quantity == 0 {
            return self.remove_order(order_id);
        }

        if let Some(order) = self.orders.get_mut(order_id) {
            order.update_quantity(new_quantity);
            self.sequence += 1;
            true
        } else {
            false
        }
    }

    pub fn get_mbo_data(&self, max_levels: u32) -> (Vec<MBOLevel>, Vec<MBOLevel>) {
        let bids = self.get_mbo_side(&Side::Bid, max_levels);
        let asks = self.get_mbo_side(&Side::Ask, max_levels);
        (bids, asks)
    }

    fn get_mbo_side(&self, side: &Side, max_levels: u32) -> Vec<MBOLevel> {
        let price_map = match side {
            Side::Bid => &self.bids_by_price,
            Side::Ask => &self.asks_by_price,
        };

        let mut result = Vec::new();
        let mut levels_count = 0;

        let prices: Box<dyn Iterator<Item = _>> = match side {
            Side::Bid => Box::new(price_map.iter().rev()), // Bids: highest to lowest
            Side::Ask => Box::new(price_map.iter()),        // Asks: lowest to highest
        };

        for (&_price_key, order_ids) in prices {
            if levels_count >= max_levels {
                break;
            }

            for order_id in order_ids {
                if let Some(order) = self.orders.get(order_id) {
                    result.push(MBOLevel {
                        order_id: order.id.clone(),
                        price: order.price,
                        quantity: order.quantity,
                        side: order.side.clone(),
                        timestamp: order.timestamp,
                        age_ms: order.age_ms(),
                    });
                }
            }
            levels_count += 1;
        }

        result.truncate((max_levels * 3) as usize); // Limit total orders shown
        result
    }

    pub fn get_mbp_data(&self, max_levels: u32) -> (Vec<MBPLevel>, Vec<MBPLevel>) {
        let bids = self.get_mbp_side(&Side::Bid, max_levels);
        let asks = self.get_mbp_side(&Side::Ask, max_levels);
        (bids, asks)
    }

    fn get_mbp_side(&self, side: &Side, max_levels: u32) -> Vec<MBPLevel> {
        let price_map = match side {
            Side::Bid => &self.bids_by_price,
            Side::Ask => &self.asks_by_price,
        };

        let mut result = Vec::new();
        let mut cumulative_quantity = 0;

        let prices: Box<dyn Iterator<Item = _>> = match side {
            Side::Bid => Box::new(price_map.iter().rev()), // Bids: highest to lowest
            Side::Ask => Box::new(price_map.iter()),        // Asks: lowest to highest
        };

        for (&price_key, order_ids) in prices.take(max_levels as usize) {
            let orders: Vec<&Order> = order_ids
                .iter()
                .filter_map(|id| self.orders.get(id))
                .collect();

            if !orders.is_empty() {
                let quantity: u64 = orders.iter().map(|o| o.quantity).sum();
                let order_count = orders.len() as u32;
                let avg_age_ms = if !orders.is_empty() {
                    orders.iter().map(|o| o.age_ms()).sum::<u64>() / orders.len() as u64
                } else {
                    0
                };

                cumulative_quantity += quantity;

                result.push(MBPLevel {
                    price: price_key.0,
                    quantity,
                    order_count,
                    side: side.clone(),
                    total_quantity: cumulative_quantity,
                    avg_age_ms,
                });
            }
        }

        result
    }

    pub fn get_best_bid_ask(&self) -> (Option<f64>, Option<f64>) {
        let best_bid = self.bids_by_price.keys().next_back().map(|k| k.0);
        let best_ask = self.asks_by_price.keys().next().map(|k| k.0);
        (best_bid, best_ask)
    }

    pub fn get_spread_info(&self) -> (Option<f64>, Option<f64>, Option<f64>) {
        let (best_bid, best_ask) = self.get_best_bid_ask();

        match (best_bid, best_ask) {
            (Some(bid), Some(ask)) => {
                let spread = ask - bid;
                let mid_price = (bid + ask) / 2.0;
                (Some(spread), Some(mid_price), Some(spread / mid_price * 10000.0))
            }
            _ => (None, None, None),
        }
    }

    pub fn simulate_activity(&mut self) -> Vec<OrderActivity> {
        let mut activities = Vec::new();
        let mut rng = thread_rng();

        let num_activities = rng.gen_range(1..=8);

        for _ in 0..num_activities {
            let activity = self.generate_random_activity(&mut rng);
            activities.push(activity.clone());
            self.execute_activity(&activity);
        }

        activities
    }

    fn generate_random_activity(&self, rng: &mut impl Rng) -> OrderActivity {
        let (best_bid, best_ask) = self.get_best_bid_ask();
        let mid_price = match (best_bid, best_ask) {
            (Some(bid), Some(ask)) => (bid + ask) / 2.0,
            _ => 100.0,
        };

        let activity_type_rand = rng.gen::<f64>();
        let side = if rng.gen() { Side::Bid } else { Side::Ask };

        if activity_type_rand < 0.4 {
            // 40% new orders
            let base_price = match (&side, best_bid, best_ask) {
                (Side::Bid, Some(bid), _) => bid,
                (Side::Ask, _, Some(ask)) => ask,
                _ => mid_price,
            };

            let price_variation = (rng.gen::<f64>() - 0.5) * 0.2;
            let price = (base_price + price_variation).max(0.01);
            let quantity = rng.gen_range(1000..=10000);

            OrderActivity {
                activity_type: ActivityType::Add,
                order_id: format!("order_{}_{}", Utc::now().timestamp_millis(), rng.gen::<u32>()),
                symbol: self.symbol.clone(),
                price: Some((price * 100.0).round() / 100.0),
                quantity: Some(quantity),
                side: Some(side),
                timestamp: Utc::now(),
            }
        } else if activity_type_rand < 0.7 && !self.orders.is_empty() {
            // 30% order updates
            let order_ids: Vec<_> = self.orders.keys().cloned().collect();
            let order_id = order_ids[rng.gen_range(0..order_ids.len())].clone();

            if let Some(order) = self.orders.get(&order_id) {
                let new_quantity = (order.quantity as i64 + rng.gen_range(-2000..=1000)).max(0) as u64;

                OrderActivity {
                    activity_type: if new_quantity > 0 { ActivityType::Update } else { ActivityType::Cancel },
                    order_id,
                    symbol: self.symbol.clone(),
                    price: None,
                    quantity: if new_quantity > 0 { Some(new_quantity) } else { None },
                    side: None,
                    timestamp: Utc::now(),
                }
            } else {
                self.generate_random_activity(rng)
            }
        } else if !self.orders.is_empty() {
            // 30% order cancellations
            let order_ids: Vec<_> = self.orders.keys().cloned().collect();
            let order_id = order_ids[rng.gen_range(0..order_ids.len())].clone();

            OrderActivity {
                activity_type: ActivityType::Cancel,
                order_id,
                symbol: self.symbol.clone(),
                price: None,
                quantity: None,
                side: None,
                timestamp: Utc::now(),
            }
        } else {
            self.generate_random_activity(rng)
        }
    }

    fn execute_activity(&mut self, activity: &OrderActivity) {
        match activity.activity_type {
            ActivityType::Add => {
                if let (Some(price), Some(quantity), Some(side)) =
                    (activity.price, activity.quantity, &activity.side) {
                    let order = Order::new(
                        activity.order_id.clone(),
                        price,
                        quantity,
                        side.clone(),
                    );
                    self.add_order(order);
                }
            }
            ActivityType::Update => {
                if let Some(quantity) = activity.quantity {
                    self.update_order(&activity.order_id, quantity);
                }
            }
            ActivityType::Cancel => {
                self.remove_order(&activity.order_id);
            }
        }
    }

    pub fn initialize_with_sample_data(&mut self) {
        let mut rng = thread_rng();
        let base_price = 100.0;

        // Add initial bid orders
        for i in 0..30 {
            let price = base_price - 0.05 - (i as f64 * 0.01);
            let quantity = rng.gen_range(1000..=10000);
            let order = Order {
                id: format!("bid_{}", i),
                price: (price * 100.0).round() / 100.0,
                quantity,
                side: Side::Bid,
                timestamp: Utc::now() - chrono::Duration::milliseconds(rng.gen_range(0..60000)),
                original_quantity: quantity,
            };
            self.add_order(order);
        }

        // Add initial ask orders
        for i in 0..30 {
            let price = base_price + (i as f64 * 0.01);
            let quantity = rng.gen_range(1000..=10000);
            let order = Order {
                id: format!("ask_{}", i),
                price: (price * 100.0).round() / 100.0,
                quantity,
                side: Side::Ask,
                timestamp: Utc::now() - chrono::Duration::milliseconds(rng.gen_range(0..60000)),
                original_quantity: quantity,
            };
            self.add_order(order);
        }
    }

    pub fn get_sequence(&self) -> u64 {
        self.sequence
    }
}