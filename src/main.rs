use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

mod engine;
mod grpc_server;
mod redis_writer;
mod models;

use crate::engine::{OrderBook, MatchingMode};
use crate::grpc_server::serve;
use crate::redis_writer::RedisWriter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse matching mode from command line
    let mode_str = env::args().nth(1).unwrap_or_else(|| "fifo".to_string());
    let mode = match mode_str.to_lowercase().as_str() {
        "pro" | "prorata" => MatchingMode::ProRata,
        "fifo" | _ => MatchingMode::Fifo,
    };

    println!("🔧 Matching Mode: {:?}", mode);

    // Initialize engine + redis
    let book = Arc::new(Mutex::new(OrderBook::new(mode)));
    let redis = RedisWriter::new("redis://127.0.0.1/");

    // Launch gRPC server
    serve(book, redis).await
}
