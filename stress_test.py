import grpc
import order_pb2
import order_pb2_grpc
import random
import time
import uuid

TOTAL_ORDERS = 1000

def random_order():
    return order_pb2.OrderRequest(
        user_id=str(uuid.uuid4())[:8],
        symbol="AAPL",
        side=random.choice(["buy", "sell"]),
        order_type="limit",
        quantity=random.randint(1, 10),
        price=random.randint(145, 155)
    )

def run():
    channel = grpc.insecure_channel("localhost:50051")
    stub = order_pb2_grpc.OrderMatchingStub(channel)

    total_trades = 0
    start = time.time()

    for i in range(TOTAL_ORDERS):
        req = random_order()
        try:
            response = stub.SubmitOrder(req)
            total_trades += len(response.trades)
        except grpc.RpcError as e:
            print(f"❌ gRPC error: {e.code()}")
        if i % 100 == 0:
            print(f"Submitted {i} orders...")

    duration = time.time() - start
    print("\n✅ Stress Test Complete")
    print(f"Total Orders Sent: {TOTAL_ORDERS}")
    print(f"Total Trades Executed: {total_trades}")
    print(f"Total Time: {duration:.2f} sec")
    print(f"Avg Orders/sec: {TOTAL_ORDERS / duration:.2f}")
    print(f"Avg Trades/order: {total_trades / TOTAL_ORDERS:.2f}")

if __name__ == "__main__":
    run()
