use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Option<u64>,
    pub quantity: u64,
    pub timestamp: DateTime<Utc>,
}

impl Order {
    pub fn new(user_id: String, symbol: String, side: Side, order_type: OrderType, price: Option<u64>, quantity: u64) -> Self {
        Order {
            id: Uuid::new_v4().to_string(),
            user_id,
            symbol,
            side,
            order_type,
            price,
            quantity,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub price: u64,
    pub quantity: u64,
    pub buyer: String,
    pub seller: String,
    pub timestamp: DateTime<Utc>,
}
