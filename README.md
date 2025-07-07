
# Real-Time Order Matching Engine

## Features

* Matching engine in Rust with FIFO and Pro-Rata logic
* Market, limit, and cancel order support
* gRPC API for order submission
* Redis Streams for trade and PnL broadcasting
* Streamlit dashboard for submitting and tracking orders

## Getting Started

### 1. Clone the repository

```bash
git clone https://github.com/<your-username>/real_time_order_matching_engine.git
cd real_time_order_matching_engine
```

### 2. Start Redis

Ensure Redis is running locally on `127.0.0.1:6379`.

```bash
redis-server
```

### 3. Build and run the Rust engine

```bash
cargo build --release
cargo run -- fifo     # or 'pro' for pro-rata mode
```

### 4. Install Python dependencies

```bash
cd dashboard
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

### 5. Run the Streamlit dashboard

```bash
streamlit run app.py
```
