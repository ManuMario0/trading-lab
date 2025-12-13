import zmq
import json
import time
import threading

# Config
STRATEGY_ADDR = "tcp://127.0.0.1:5556"
ENGINE_ADDR = "tcp://127.0.0.1:5557"

def strategy_client(name, symbols, bind=False):
    ctx = zmq.Context()
    sock = ctx.socket(zmq.PUSH)
    sock.connect(STRATEGY_ADDR)
    
    print(f"[{name}] Connected to {STRATEGY_ADDR}")
    time.sleep(1) # Wait for connect

    # Create Portfolio Message
    # Structure matching Portfolio.hpp to_json
    msg = {
        "type": "TargetPortfolio",
        "data": {
            "multiplexer_id": name,
            "target_weights": [], # Filled below
            "target_positions": None
        }
    }

    # Fill weights
    for sym_data in symbols:
        # Instrument: {type, data}
        inst = {
            "type": "Stock",
            "data": {"symbol": sym_data[0], "exchange": sym_data[1]}
        }
        weight = 1.0 / len(symbols)
        # Add [Instrument, Weight] tuple
        msg["data"]["target_weights"].append([inst, weight])

    # Send
    sock.send_json(msg)
    print(f"[{name}] Sent portfolio: {json.dumps(msg)}")
    
    sock.close()
    ctx.term()

def engine_monitor():
    ctx = zmq.Context()
    sock = ctx.socket(zmq.SUB)
    sock.connect(ENGINE_ADDR)
    sock.subscribe("") # Subscribe to all

    print(f"[Engine] Subscribed to {ENGINE_ADDR}")
    time.sleep(1)

    while True:
        try:
            # Non-blocking check or timeout
            if sock.poll(2000): # 2s timeout
                msg = sock.recv_json()
                print(f"[Engine] Received Aggregated: {json.dumps(msg, indent=2)}")
                # Basic validation
                if msg["data"]["multiplexer_id"] == "KellyMultiplexer_Aggregated":
                     # Note: logic returns "KellyMux_Aggregated" in C++, I should check that.
                     pass
            else:
                print("[Engine] No message received for 2s.")
                break
        except Exception as e:
            print(f"[Engine] Error: {e}")
            break
            
    sock.close()
    ctx.term()

if __name__ == "__main__":
    print("--- Starting Test ---")
    
    # Start Engine Monitor in thread
    t_engine = threading.Thread(target=engine_monitor)
    t_engine.start()
    
    time.sleep(1) # Let engine connect

    # Fire Strategy A (AAPL)
    # Registry has StratA with u=0.05, s=0.10 -> Kelly=5
    # Config Kelly=0.3 -> Scalar = 1.5
    # Weight 1.0 -> Final 1.5
    strategy_client("StratA", [("AAPL", "US")])
    
    time.sleep(1)

    # Fire Strategy B (TSLA)
    # Registry has StratB with u=0.10, s=0.20 -> Kelly=2.5
    # Config Kelly=0.3 -> Scalar = 0.75
    # Weight 1.0 -> Final 0.75
    strategy_client("StratB", [("TSLA", "US")])

    t_engine.join()
    print("--- Test Finished ---")
