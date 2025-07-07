use tonic::{Request, Response, Status};
use crate::engine::OrderBook;
use crate::models::{Order, Side, OrderType};
use crate::redis_writer::RedisWriter;
use tokio::sync::Mutex;
use std::sync::Arc;

// Include generated gRPC code
pub mod order {
    tonic::include_proto!("order");
}

use order::order_matching_server::{OrderMatching, OrderMatchingServer};
use order::{OrderRequest, SubmitResponse, Trade};

pub struct OrderService {
    pub book: Arc<Mutex<OrderBook>>,
    pub redis: RedisWriter,
}

#[tonic::async_trait]
impl OrderMatching for OrderService {
    async fn submit_order(&self, request: Request<OrderRequest>) -> Result<Response<SubmitResponse>, Status> {
        let req = request.into_inner();

        // Convert gRPC request to internal Order
        let side = match req.side.to_lowercase().as_str() {
            "buy" => Side::Buy,
            "sell" => Side::Sell,
            _ => return Err(Status::invalid_argument("Invalid side")),
        };

        let order_type = match req.order_type.to_lowercase().as_str() {
            "limit" => OrderType::Limit,
            "market" => OrderType::Market,
            _ => return Err(Status::invalid_argument("Invalid order_type")),
        };

        let order = Order::new(
            req.user_id,
            req.symbol,
            side,
            order_type.clone(), // clone it before it's moved
            if order_type == OrderType::Limit { Some(req.price) } else { None },
            req.quantity,
        );

        // Submit to matching engine
        let mut book = self.book.lock().await;
        let trades = book.submit_order(order.clone());

        // Publish to Redis
        for trade in &trades {
            self.redis.publish_trade(trade).await;
            self.redis.update_user_pnl(trade).await;
            self.redis.publish_trade(trade).await;
        }

        // Return gRPC response
        let response = SubmitResponse {
            trades: trades.into_iter().map(|t| Trade {
                price: t.price,
                quantity: t.quantity,
                buyer: t.buyer,
                seller: t.seller,
                timestamp: t.timestamp.to_rfc3339(),
            }).collect()
        };

        Ok(Response::new(response))
    }
}

// Expose gRPC server runner
pub async fn serve(book: Arc<Mutex<OrderBook>>, redis: RedisWriter) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = OrderService { book, redis };

    println!("🚀 gRPC server running on {}", addr);

    tonic::transport::Server::builder()
        .add_service(OrderMatchingServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
