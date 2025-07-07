use redis::AsyncCommands;
use crate::models::Trade;

pub struct RedisWriter {
    pub client: redis::Client,
}

impl RedisWriter {
    pub fn new(redis_url: &str) -> Self {
        let client = redis::Client::open(redis_url).expect("Failed to connect to Redis");
        Self { client }
    }

    pub async fn publish_trade(&self, trade: &Trade) {
        let mut conn = match self.client.get_async_connection().await {
            Ok(conn) => conn,
            Err(err) => {
                eprintln!("Redis connection error: {:?}", err);
                return;
            }
        };

        let _: () = conn.xadd(
            "trades_stream",
            "*",
            &[
                ("price", trade.price.to_string()),
                ("quantity", trade.quantity.to_string()),
                ("buyer", trade.buyer.clone()),
                ("seller", trade.seller.clone()),
                ("timestamp", trade.timestamp.to_rfc3339()),
            ],
        ).await.unwrap_or_else(|e| {
            eprintln!("Failed to push to Redis: {:?}", e);
        });
    }

    pub async fn update_user_pnl(&self, trade: &Trade) {
        let mut conn = match self.client.get_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Redis error (PNL): {:?}", e);
                return;
            }
        };
    
        let buyer_key = format!("user_pnl:{}", trade.buyer);
        let seller_key = format!("user_pnl:{}", trade.seller);
        let value = (trade.price * trade.quantity) as f64;
    
        // Use raw redis::cmd to call HINCRBYFLOAT
        let _ = redis::cmd("HINCRBYFLOAT")
            .arg(&buyer_key)
            .arg("pnl")
            .arg(-1.0 * value)
            .query_async::<_, ()>(&mut conn)
            .await;
    
        let _ = redis::cmd("HINCRBYFLOAT")
            .arg(&seller_key)
            .arg("pnl")
            .arg(value)
            .query_async::<_, ()>(&mut conn)
            .await;
    }
    
}
