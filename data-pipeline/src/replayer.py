import pandas as pd
import zmq
import time
import json
import argparse
import os

def replay(csv_path, speed=1.0, port=5562):
    print(f"[Replayer] Loading {csv_path}...")
    df = pd.read_csv(csv_path)
    
    # Sort by time just in case
    df['timestamp'] = pd.to_datetime(df['timestamp'])
    df.sort_values('timestamp', inplace=True)
    
    context = zmq.Context()
    socket = context.socket(zmq.PUB)
    print(f"[Replayer] Binding to tcp://*:{port}")
    socket.bind(f"tcp://*:{port}")
    
    print(f"[Replayer] Starting Replay in 3 seconds... (Speed: {speed}x)")
    time.sleep(3)
    
    records = df.to_dict('records')
    prev_time = records[0]['timestamp']
    
    for row in records:
        curr_time = row['timestamp']
        
        # Calculate delay
        if speed > 0:
            diff_sec = (curr_time - prev_time).total_seconds()
            sleep_time = diff_sec / speed
            if sleep_time > 0:
                time.sleep(sleep_time)
        
        prev_time = curr_time
        
        # Publish AAPL Price
        # Using the standard JSON format your System Orchestrator expects
        # {"instrument": {"type": "Stock", "data": {"symbol": "AAPL", ...}}, "last": ...}
        
        # 1. AAPL
        msg_aapl = {
            "instrument": {
                "type": "Stock",
                "data": {"symbol": "AAPL", "exchange": "NASDAQ"}
            },
            "last": row['close_AAPL'],
            "bid": row['close_AAPL'] - 0.01, # Mock spread
            "ask": row['close_AAPL'] + 0.01,
            "timestamp": int(curr_time.timestamp() * 1000)
        }
        socket.send_string(json.dumps(msg_aapl))
        
        # 2. MSFT
        msg_msft = {
            "instrument": {
                "type": "Stock",
                "data": {"symbol": "MSFT", "exchange": "NASDAQ"}
            },
            "last": row['close_MSFT'],
            "bid": row['close_MSFT'] - 0.01,
            "ask": row['close_MSFT'] + 0.01,
            "timestamp": int(curr_time.timestamp() * 1000)
        }
        socket.send_string(json.dumps(msg_msft))
        
        # 3. FEATURE (Z-Score)
        # We need a schema for features. For now, let's treat it as a special instrument or topic.
        # But since the current Orchestrator might only parse "instrument", let's wrap it similarly or use a different topic?
        # Your Orchestrator/Strategy expects JSON. Let's send a customized JSON for features.
        msg_feature = {
            "type": "feature",
            "symbol": "AAPL_MSFT",
            "z_score": row['z_score'],
            "timestamp": int(curr_time.timestamp() * 1000)
        }
        socket.send_string(json.dumps(msg_feature))
        
        # Visual feedback every 10 rows
        if int(curr_time.timestamp()) % 60 == 0: 
             print(f"[Replay] {curr_time} -> AAPL: {row['close_AAPL']}, Z: {row['z_score']:.4f}")

    print("[Replayer] Done.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("file", help="Path to processed CSV")
    parser.add_argument("--speed", type=float, default=100.0, help="Replay speed multiplier (default 100x)")
    args = parser.parse_args()
    
    replay(args.file, args.speed)
