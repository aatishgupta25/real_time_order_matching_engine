//! Reproducible latency benchmarks and a CPU-profiling workload for the FIFO matching engine.
//! Run: cargo run --release --bin profile_matching -- --scenario all --iterations 10000 --depth 64
//! CPU flamegraph: cargo flamegraph --release --bin profile_matching -- --scenario sweep --iterations 100000 --depth 128

#[path = "../models.rs"]
mod models;
#[path = "../engine.rs"]
mod engine;

use engine::{MatchingMode, OrderBook};
use models::{Order, OrderType, Side};
use std::env;
use std::hint::black_box;
use std::time::Instant;

#[derive(Clone, Copy)]
enum Scenario {
    SingleFill,
    Sweep,
    NonCrossing,
}

impl Scenario {
    fn name(self) -> &'static str {
        match self {
            Self::SingleFill => "single_fill",
            Self::Sweep => "sweep_levels",
            Self::NonCrossing => "non_crossing",
        }
    }
}

fn order(user: String, side: Side, kind: OrderType, price: Option<u64>, qty: u64) -> Order {
    Order::new(user, "AAPL".to_owned(), side, kind, price, qty)
}

// Book construction happens OUTSIDE the timed interval.
fn workload(case: Scenario, depth: usize) -> (OrderBook, Order, usize) {
    let mut book = OrderBook::new(MatchingMode::Fifo);
    match case {
        Scenario::SingleFill => {
            book.submit_order(order("seller".into(), Side::Sell, OrderType::Limit, Some(100), 1));
            (book, order("buyer".into(), Side::Buy, OrderType::Market, None, 1), 1)
        }
        Scenario::Sweep => {
            for i in 0..depth {
                book.submit_order(order(
                    format!("seller-{i}"), Side::Sell, OrderType::Limit,
                    Some(100 + i as u64), 1,
                ));
            }
            (book, order("buyer".into(), Side::Buy, OrderType::Market, None, depth as u64), depth)
        }
        Scenario::NonCrossing => {
            for i in 0..depth {
                book.submit_order(order(
                    format!("seller-{i}"), Side::Sell, OrderType::Limit,
                    Some(200 + i as u64), 1,
                ));
            }
            (book, order("buyer".into(), Side::Buy, OrderType::Limit, Some(100), 1), 0)
        }
    }
}

fn percentile(sorted: &[u128], percentile: usize) -> u128 {
    // Nearest-rank method. Report this consistently for cross-run comparison.
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn benchmark(case: Scenario, iterations: usize, warmup: usize, depth: usize) {
    let mut samples = Vec::with_capacity(iterations);
    let mut total_nanos: u128 = 0;

    for run in 0..(iterations + warmup) {
        let (mut book, incoming, expected_trades) = workload(case, depth);
        let start = Instant::now();
        let trades = book.submit_order(black_box(incoming));
        let elapsed = start.elapsed().as_nanos();

        assert_eq!(trades.len(), expected_trades, "incorrect benchmark workload");
        black_box(trades);

        if run >= warmup {
            samples.push(elapsed);
            total_nanos += elapsed;
        }
    }

    samples.sort_unstable();
    let mean = total_nanos as f64 / iterations as f64;
    let ops_per_sec = 1_000_000_000.0 / mean;
    println!(
        "{},{},{},{:.1},{},{},{},{:.0}",
        case.name(), iterations, depth, mean,
        percentile(&samples, 50),
        percentile(&samples, 95),
        percentile(&samples, 99),
        ops_per_sec
    );
}

fn main() {
    let mut scenario = String::from("all");
    let mut iterations = 10_000usize;
    let mut warmup = 1_000usize;
    let mut depth = 64usize;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        let value = args.next().unwrap_or_else(|| {
            eprintln!("Missing value for {arg}");
            std::process::exit(2);
        });
        let parsed = || value.parse::<usize>().unwrap_or_else(|_| {
            eprintln!("Expected a positive integer for {arg}, got {value}");
            std::process::exit(2);
        });
        match arg.as_str() {
            "--scenario" => scenario = value,
            "--iterations" => iterations = parsed(),
            "--warmup" => warmup = parsed(),
            "--depth" => depth = parsed(),
            _ => {
                eprintln!("Unknown option: {arg}");
                std::process::exit(2);
            }
        }
    }

    if iterations == 0 || depth == 0 || depth > 1_000_000 {
        eprintln!("--iterations and --depth must be > 0; --depth must be <= 1,000,000");
        std::process::exit(2);
    }
    if depth > (u64::MAX - 200) as usize {
        eprintln!("--depth too large");
        std::process::exit(2);
    }

    let cases: &[Scenario] = match scenario.as_str() {
        "all" => &[Scenario::SingleFill, Scenario::Sweep, Scenario::NonCrossing],
        "single" => &[Scenario::SingleFill],
        "sweep" => &[Scenario::Sweep],
        "noncrossing" => &[Scenario::NonCrossing],
        _ => {
            eprintln!("Invalid --scenario: {scenario}. Use all|single|sweep|noncrossing");
            std::process::exit(2);
        }
    };
    println!("scenario,iterations,depth,mean_ns,p50_ns,p95_ns,p99_ns,matching_ops_per_sec");
    for &case in cases {
        benchmark(case, iterations, warmup, depth);
    }
}
