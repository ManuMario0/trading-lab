import yfinance as yf
import pandas as pd
import os
import argparse
from datetime import datetime, timedelta

DATA_DIR = os.path.join(os.path.dirname(os.path.dirname(__file__)), 'data', 'raw')

def fetch_data(symbol: str, days: int = 30):
    print(f"[Fetcher] Downloading {symbol} for last {days} days...")
    
    end_date = datetime.now()
    start_date = end_date - timedelta(days=days)
    
    # Let's default to '1h' for >7 days or '1m' for <=7 days.
    interval = '1m' if days <= 7 else '1h'
    
    try:
        df = yf.download(symbol, start=start_date, end=end_date, interval=interval, progress=False)
        
        if df.empty:
            print(f"[Fetcher] Warning: No data found for {symbol}")
            return

        # Normalize Columns
        df.reset_index(inplace=True)
        # YF columns: Date/Datetime, Open, High, Low, Close, Adj Close, Volume
        # We want standard lowercase: timestamp, open, high, low, close, volume
        
        # Renaissance of df.columns often MultiIndex in new yfinance?
        # Check standard flat columns first.
        df.columns = [c[0] if isinstance(c, tuple) else c for c in df.columns] # Flatten if multi-index
        
        df.rename(columns={
            "Date": "timestamp", 
            "Datetime": "timestamp", 
            "Open": "open", 
            "High": "high", 
            "Low": "low", 
            "Close": "close", 
            "Volume": "volume"
        }, inplace=True)
        
        # Clean
        cols = ["timestamp", "open", "high", "low", "close", "volume"]
        df = df[cols]
        
        # Save
        if not os.path.exists(DATA_DIR):
            os.makedirs(DATA_DIR)
            
        path = os.path.join(DATA_DIR, f"{symbol}.csv")
        df.to_csv(path, index=False)
        
        print(f"[Fetcher] Saved {len(df)} rows to {path}")
        
    except Exception as e:
        print(f"[Fetcher] Failed to download {symbol}: {e}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("symbols", help="Ticker symbols (comma-separated, e.g. AAPL,MSFT)")
    parser.add_argument("--days", type=int, default=5, help="Number of days")
    args = parser.parse_args()
    
    ticker_list = args.symbols.split(',')
    for ticker in ticker_list:
        fetch_data(ticker.strip(), args.days)
