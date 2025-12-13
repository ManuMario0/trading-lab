import zmq
import json
import subprocess
import time
import os
import signal

def test_strategy_lab():
    context = zmq.Context()

    # Input Socket (Publisher from test perspective, essentially pushing data)
    # Strategy listens on SUB, so we can use PUB.
    # But wait, Strategy uses SUB. So we must use PUB.
    # Bind the publisher so strategy can connect.
    pub_socket = context.socket(zmq.PUB)
    pub_url = "tcp://127.0.0.1:5555"
    pub_socket.bind(pub_url)

    # Output Socket (Puller to receive strategy output)
    # Strategy uses PUSH. So we must use PULL.
    # Bind the puller so strategy can connect.
    pull_socket = context.socket(zmq.PULL)
    pull_url = "tcp://127.0.0.1:5556"
    pull_socket.bind(pull_url)

    # Admin Socket (Req)
    # Strategy binds REP. We connect REQ.
    admin_socket = context.socket(zmq.REQ)
    admin_url = "tcp://127.0.0.1:5557"
    # Strategy binds to *:5557. We connect to localhost.
    
    print(f"[Test] Sockets bound/ready. Launching Strategy Lab...")
    
    # Launch Strategy Lab
    # Usage: ./strategy_lab [input_addr] [output_addr] [admin_addr]
    # Strategy connects to input/output, binds admin.
    
    # Wait a bit for bind
    time.sleep(0.5)

    build_dir = os.path.join(os.getcwd(), "build")
    exe_path = os.path.join(build_dir, "strategy_lab")
    
    process = subprocess.Popen(
        [exe_path, pub_url, pull_url, admin_url],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    
    # Wait for startup
    time.sleep(2)
    # Check if alive
    if process.poll() is not None:
        print("[Test] Process died early!")
        print(process.stdout.read())
        print(process.stderr.read())
        return

    admin_socket.connect(admin_url)
    
    try:
        # 1. Send Admin Ping
        print("[Test] Sending Admin Ping...")
        admin_socket.send_string("PING")
        reply = admin_socket.recv_string()
        print(f"[Test] Admin Reply: {reply}")
        assert reply == "OK"

        # 2. Send Market Data (Tick 1 - Init)
        print("[Test] Sending Tick 1 (AAPL 150.0)...")
        tick1 = {
            "updates": [
                {"symbol": "AAPL", "exchange": "NASDAQ", "price": 150.0}
            ],
            "timestamp": "2023-01-01T10:00:00Z"
        }
        pub_socket.send_string(json.dumps(tick1))
        time.sleep(0.5)
        
        # Expect NO output on first tick (logic: "First tick, just record it")
        try:
            msg = pull_socket.recv_string(flags=zmq.NOBLOCK)
            print(f"[Test] Unexpected output on Tick 1: {msg}")
        except zmq.Again:
            print("[Test] Correctly no output on Tick 1")

        # 3. Send Market Data (Tick 2 - Price Up -> Buy)
        print("[Test] Sending Tick 2 (AAPL 155.0)...")
        tick2 = {
            "updates": [
                {"symbol": "AAPL", "exchange": "NASDAQ", "price": 155.0}
            ],
            "timestamp": "2023-01-01T10:00:01Z"
        }
        pub_socket.send_string(json.dumps(tick2))
        
        # Expect Portfolio Output
        print("[Test] Waiting for Portfolio (Buy)...")
        msg = pull_socket.recv_string() # Blocking
        print(f"[Test] Received: {msg}")
        data = json.loads(msg)
        
        # Verify
        # Format: {"type": "TargetPortfolio", "data": { "multiplexer_id": ..., "target_weights": [[{"type":..., "data":...}, 1.0]] }}
        assert data["type"] == "TargetPortfolio"
        weights = data["data"]["target_weights"]
        # weights is list of [key, val]
        found_aapl = False
        for item in weights:
            # item[0] is Instrument, item[1] is weight
            inst = item[0]
            weight = item[1]
            if inst["data"]["symbol"] == "AAPL":
                assert weight == 1.0
                found_aapl = True
        assert found_aapl
        print("[Test] Tick 2 Verified: AAPL Buy 1.0")

        # 4. Send Market Data (Tick 3 - Price Down -> Sell)
        print("[Test] Sending Tick 3 (AAPL 140.0)...")
        tick3 = {
            "updates": [
                {"symbol": "AAPL", "exchange": "NASDAQ", "price": 140.0}
            ],
            "timestamp": "2023-01-01T10:00:02Z"
        }
        pub_socket.send_string(json.dumps(tick3))
        
        # Expect Portfolio Output
        print("[Test] Waiting for Portfolio (Sell)...")
        msg = pull_socket.recv_string()
        data = json.loads(msg)
        weights = data["data"]["target_weights"]
        found_aapl = False
        for item in weights:
            if item[0]["data"]["symbol"] == "AAPL":
                assert weight == -1.0 or item[1] == -1.0
                found_aapl = True
        assert found_aapl
        print("[Test] Tick 3 Verified: AAPL Sell -1.0")

        print("[Test] integration_test PASSED")

    except Exception as e:
        print(f"[Test] FAILED: {e}")
        if process.poll() is not None:
             print("Process stdout:", process.stdout.read())
             print("Process stderr:", process.stderr.read())

    finally:
        print("[Test] Cleaning up...")
        process.terminate()
        process.wait()
        context.term()

if __name__ == "__main__":
    test_strategy_lab()
