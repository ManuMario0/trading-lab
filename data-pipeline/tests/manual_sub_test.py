import zmq
import json
import time
import argparse

def main():
    parser = argparse.ArgumentParser(description="Manual Subscriber Test")
    parser.add_argument("--port", type=int, default=5558, help="ZMQ SUB port to connect to")
    args = parser.parse_args()

    context = zmq.Context()
    socket = context.socket(zmq.SUB)
    
    # Connect to the publisher
    connect_addr = f"tcp://127.0.0.1:{args.port}"
    socket.connect(connect_addr)
    
    # Subscribe to all topics (empty string)
    socket.setsockopt_string(zmq.SUBSCRIBE, "")
    
    print(f"[Subscriber] Connected to {connect_addr}. Waiting for data...")
    
    messages_received = 0
    max_messages = 5
    
    try:
        while messages_received < max_messages:
            # Receive message
            msg = socket.recv_string()
            print(f"[Subscriber] Received: {msg}")
            
            # Validate JSON
            try:
                data = json.loads(msg)
                if "updates" in data and "timestamp" in data:
                    item = data['updates'][0]
                    if "bid" in item and "ask" in item:
                         print(f"  -> Valid JSON format. Symbol: {item['symbol']}, Price: {item['price']:.2f}, Bid: {item['bid']:.2f}, Ask: {item['ask']:.2f}")
                    else:
                         print(f"  -> Valid JSON format but missing Bid/Ask. Symbol: {item['symbol']}, Price: {item['price']:.2f}")
                    messages_received += 1
                else:
                    print("  -> Invalid JSON structure")
            except json.JSONDecodeError:
                print("  -> Not a valid JSON")
                
    except KeyboardInterrupt:
        print("Interrupted")
    finally:
        socket.close()
        context.term()
        print("[Subscriber] Test finished.")

if __name__ == "__main__":
    main()
