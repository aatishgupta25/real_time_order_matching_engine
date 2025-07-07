import streamlit as st
import redis
import pandas as pd
from streamlit_autorefresh import st_autorefresh



r = redis.Redis(host='localhost', port=6379, decode_responses=True)

st.set_page_config(page_title="📈 Matching Engine Dashboard", layout="wide")
st.title("📊 Real-Time Trade Feed")

# ⏱ Auto-refresh every 5 sec
st_autorefresh(interval=5000, key="refresh")

import grpc
import order_pb2
import order_pb2_grpc
import uuid

st.header("💸 User PnL Tracker")

user_query = st.text_input("Enter user ID to check PnL", key="pnl_user")

if user_query:
    pnl_key = f"user_pnl:{user_query}"
    pnl_data = r.hgetall(pnl_key)
    
    if pnl_data and "pnl" in pnl_data:
        pnl_val = float(pnl_data["pnl"])
        pnl_color = "🟢" if pnl_val > 0 else ("🟡" if pnl_val == 0 else "🔴")
        st.metric(label=f"Realized PnL for {user_query}", value=f"${pnl_val:.2f}", delta=pnl_color)
    else:
        st.info("No PnL data available for this user yet.")

st.header("📝 Submit New Order")

with st.form("order_form"):
    default_id = st.session_state.get("default_user_id", str(uuid.uuid4())[:8])
    user_id = st.text_input("User ID", value=default_id)
    st.session_state["default_user_id"] = default_id
    side = st.selectbox("Side", ["buy", "sell"])
    order_type = st.selectbox("Order Type", ["limit", "market"])
    quantity = st.number_input("Quantity", min_value=1, step=1)
    price = st.number_input("Price", min_value=1.0, step=0.5) if order_type == "limit" else 0

    submitted = st.form_submit_button("Submit Order")

    if submitted:
        try:
            channel = grpc.insecure_channel("localhost:50051")
            stub = order_pb2_grpc.OrderMatchingStub(channel)

            request = order_pb2.OrderRequest(
                user_id=user_id,
                symbol="AAPL",
                side=side,
                order_type=order_type,
                quantity=int(quantity),
                price=int(price)
            )

            response = stub.SubmitOrder(request)
            st.success(f"✅ Order submitted! Trades executed: {len(response.trades)}")
        except grpc.RpcError as e:
            st.error(f"❌ gRPC error: {e.code()}")


# Get recent trades
trades = r.xrevrange("trades_stream", count=200)

rows = []
for trade_id, fields in reversed(trades):
    rows.append({
        "Price": float(fields["price"]),
        "Qty": int(fields["quantity"]),
        "Buyer": fields["buyer"],
        "Seller": fields["seller"],
        "Time": fields["timestamp"]
    })

df = pd.DataFrame(rows)
st.dataframe(df, use_container_width=True)
