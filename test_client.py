import grpc
import order_pb2
import order_pb2_grpc

def run():
    channel = grpc.insecure_channel("localhost:50051")
    stub = order_pb2_grpc.OrderMatchingStub(channel)

    request = order_pb2.OrderRequest(
        user_id="python_buyer",
        symbol="AAPL",
        side="buy",
        order_type="limit",
        quantity=5,
        price=151
    )

    response = stub.SubmitOrder(request)
    print("✅ Trades Executed:")
    for trade in response.trades:
        print(f"{trade.buyer} bought {trade.quantity} from {trade.seller} @ {trade.price}")

if __name__ == "__main__":
    run()
