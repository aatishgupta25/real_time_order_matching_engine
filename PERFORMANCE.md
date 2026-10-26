# Matching-engine performance and CPU profiling

These measurements cover **the in-memory FIFO order matching function** (not gRPC, Redis,
the dashboard, or end-to-end trade latency). Benchmark code: `src/bin/profile_matching.rs`.

## Reproducible benchmarks

Requires the Rust toolchain and `protoc` (the existing project generates gRPC code at build time).

```bash
cargo run --release --bin profile_matching -- --scenario all --iterations 10000 --warmup 1000 --depth 64
```

The executable prints a CSV header and one row per scenario:
`scenario,iterations,depth,mean_ns,p50_ns,p95_ns,p99_ns,matching_ops_per_sec`.

- **single_fill:** one resting sell and one crossing market buy (one fill).
- **sweep_levels:** one resting sell at each of N price levels; one market buy sweeps N fills.
- **non_crossing:** N resting ask price levels; one non-crossing bid adds a resting order.

Setup/book initialization is **excluded** from each timed measurement. Each run creates a fresh
order book to keep state and workloads reproducible. Matching throughput is calculated as
`1e9 / mean_ns`, not measured end-to-end system throughput. Values depend on hardware, system
load, Rust version, and depth; record the environment before comparing runs.

## CPU flamegraph (Linux)

Install the Rust flamegraph utility and ensure `perf` can collect CPU samples:

```bash
cargo install flamegraph
cargo flamegraph --release --bin profile_matching -o matching-flamegraph.svg -- --scenario sweep --iterations 100000 --warmup 0 --depth 128
```

The Cargo release profile retains minimal debug symbols for readable stack traces. If Linux
`perf_event_paranoid` prevents sampling, configure permissions on a machine you control.
On macOS, use Instruments / Time Profiler on the release-built `profile_matching` binary.

**Interpret the flamegraph carefully:** the timer measures only matching, but the sampled
process also executes workload construction and data collection. Inspect frames belonging to
`OrderBook::submit_order`, `match_buy`, BTreeMap operations, and allocations; construction frames
are not matching hot spots. To isolate the matching hot path in CPU samples, use a narrower
profiling configuration or dedicated sampling run.

## Results

No benchmark timings or CPU flamegraph have been recorded in this repository yet.
Before claiming measured throughput or a discovered bottleneck on a resume, run the commands,
inspect the flamegraph, and record:

- CPU and operating system, Rust version, build profile
- scenario, depth, iterations, p50/p95/p99 and operations/s
- observed hottest matching function(s), percentage of CPU samples if known, and before/after
  numbers for any optimization

Do not mix in Redis or gRPC latency when quoting these results.
