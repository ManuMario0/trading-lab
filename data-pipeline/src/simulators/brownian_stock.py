import argparse
import time
import json
import math
import random
import zmq
import datetime
import sys

class GeometricBrownianMotion:
    def __init__(self, s0: float, mu: float, sigma: float, dt: float):
        self.price = s0
        self.mu = mu
        self.sigma = sigma
        self.dt = dt

    def step(self) -> float:
        # GBM formula: S(t+dt) = S(t) * exp((mu - 0.5 * sigma^2) * dt + sigma * sqrt(dt) * Z)
        # Z ~ N(0, 1)
        z = random.gauss(0.0, 1.0)
        drift = (self.mu - 0.5 * self.sigma**2) * self.dt
        diffusion = self.sigma * math.sqrt(self.dt) * z
        
        self.price = self.price * math.exp(drift + diffusion)
        return self.price

def main():
    parser = argparse.ArgumentParser(description="AAPL Brownian Motion Publisher")
    parser.add_argument("--port", type=int, default=5558, help="ZMQ PUB port to bind to")
    parser.add_argument("--symbol", type=str, default="AAPL", help="Stock symbol")
    parser.add_argument("--exchange", type=str, default="NASDAQ", help="Exchange name")
    parser.add_argument("--price", type=float, default=150.0, help="Initial price")
    parser.add_argument("--spread", type=float, default=0.02, help="Bid/Ask spread")
    parser.add_argument("--mu", type=float, default=0.1, help="Drift (annual)")
    parser.add_argument("--sigma", type=float, default=0.2, help="Volatility (annual)")
    parser.add_argument("--interval", type=float, default=0.1, help="Update interval in seconds")
    
    args = parser.parse_args()

    # ZMQ Setup
    context = zmq.Context()
    socket = context.socket(zmq.PUB)
    try:
        bind_addr = f"tcp://*:{args.port}"
        socket.bind(bind_addr)
        print(f"[Publisher] Bound to {bind_addr}")
    except zmq.ZMQError as e:
        print(f"[Publisher] Error binding to port {args.port}: {e}")
        sys.exit(1)

    # GBM Setup
    # Adjust annual params to per-step (assuming interval is in seconds, 252 trading days is standard but continuous model uses calendar or business time)
    # Simple continuous scaling: dt = interval / (365 * 24 * 3600) or just raw seconds if mu/sigma are per second.
    # Usually inputs are annual. Let's convert interval (seconds) to years.
    dt_years = args.interval / (365.0 * 24.0 * 3600.0)
    
    gbm = GeometricBrownianMotion(s0=args.price, mu=args.mu, sigma=args.sigma, dt=dt_years)

    print(f"[Publisher] Starting simulation for {args.symbol} at {args.price}...")

    try:
        while True:
            new_price = gbm.step()
            
            # Construct message
            timestamp = datetime.datetime.utcnow().isoformat() + "Z"
            
            message = {
                "updates": [
                    {
                        "symbol": args.symbol,
                        "exchange": args.exchange,
                        "price": new_price,
                        "bid": new_price - (args.spread / 2),
                        "ask": new_price + (args.spread / 2)
                    }
                ],
                "timestamp": timestamp
            }
            
            # Send JSON
            # Note: Strategy Lab subscriber listens to "". Sending raw JSON string matches typical ZMQ string recv.
            payload = json.dumps(message)
            socket.send_string(payload)
            
            time.sleep(args.interval)
            
    except KeyboardInterrupt:
        print("\n[Publisher] Stopping...")
    finally:
        socket.close()
        context.term()

if __name__ == "__main__":
    main()
