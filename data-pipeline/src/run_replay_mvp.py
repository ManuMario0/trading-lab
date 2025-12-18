import argparse
import os
import sys

# Add src to path so we can import replayer
sys.path.append(os.path.dirname(__file__))

from replayer import replay

# MVP Confiuration
CSV_PATH = os.path.join(os.path.dirname(os.path.dirname(__file__)), 'data', 'processed', 'pair_AAPL_MSFT.csv')
SPEED = 100.0

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=5562, help="ZMQ Pub Port")
    # Orchestrator might pass other args, we can use parse_known_args to ignore them if needed
    args, unknown = parser.parse_known_args()
    
    if not os.path.exists(CSV_PATH):
        print(f"[Error] CSV not found at {CSV_PATH}. Please run processor.py first.")
        sys.exit(1)
        
    print(f"[Wrapper] Launching Replay. File: {CSV_PATH}, Port: {args.port}, Speed: {SPEED}")
    # Force stdout flush
    sys.stdout.flush()
    
    replay(CSV_PATH, speed=SPEED, port=args.port)
