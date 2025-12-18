import pandas as pd
import numpy as np
import os
import argparse

DATA_RAW = os.path.join(os.path.dirname(os.path.dirname(__file__)), 'data', 'raw')
DATA_PROCESSED = os.path.join(os.path.dirname(os.path.dirname(__file__)), 'data', 'processed')

def compute_features(sym_a: str, sym_b: str, window: int = 60):
    print(f"[Processor] Computing features for {sym_a} vs {sym_b} (Window={window})...")
    
    # 1. Load Data
    path_a = os.path.join(DATA_RAW, f"{sym_a}.csv")
    path_b = os.path.join(DATA_RAW, f"{sym_b}.csv")
    
    if not os.path.exists(path_a) or not os.path.exists(path_b):
        print("[Processor] Error: Raw data not found. Please run fetcher.py first.")
        return

    df_a = pd.read_csv(path_a)
    df_b = pd.read_csv(path_b)
    
    # 2. Align Data (Merge on timestamp)
    # Ensure timestamps are parsed
    df_a['timestamp'] = pd.to_datetime(df_a['timestamp'])
    df_b['timestamp'] = pd.to_datetime(df_b['timestamp'])
    
    merged = pd.merge(df_a, df_b, on='timestamp', suffixes=(f'_{sym_a}', f'_{sym_b}'))
    merged.sort_values('timestamp', inplace=True)
    
    if merged.empty:
        print("[Processor] Error: No overlapping timestamps found.")
        return

    # 3. Compute Features (Pair Trading Logic)
    # Simple Ratio for MVP: A / B
    # Ideally we use OLS for Beta, but rolling Z-Score of ratio is a good start.
    
    price_a = merged[f'close_{sym_a}']
    price_b = merged[f'close_{sym_b}']
    
    # Feature 1: Ratio
    merged['ratio'] = price_a / price_b
    
    # Feature 2: Rolling Mean & Std (of Ratio)
    rolling_mean = merged['ratio'].rolling(window=window).mean()
    rolling_std = merged['ratio'].rolling(window=window).std()
    
    # Feature 3: Z-Score
    merged['z_score'] = (merged['ratio'] - rolling_mean) / rolling_std
    
    # Drop NaN (startup period)
    merged.dropna(inplace=True)
    
    # 4. Save
    if not os.path.exists(DATA_PROCESSED):
        os.makedirs(DATA_PROCESSED)
        
    out_path = os.path.join(DATA_PROCESSED, f"pair_{sym_a}_{sym_b}.csv")
    
    # Keep only relevant columns for the stream
    # timestamp, close_A, close_B, z_score
    output_df = merged[['timestamp', f'close_{sym_a}', f'close_{sym_b}', 'z_score']].copy()
    
    output_df.to_csv(out_path, index=False)
    print(f"[Processor] Saved {len(output_df)} rows to {out_path}")
    print(f"[Processor] Last Z-Score: {output_df.iloc[-1]['z_score']:.4f}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("sym_a", help="First Symbol (e.g. AAPL)")
    parser.add_argument("sym_b", help="Second Symbol (e.g. MSFT)")
    parser.add_argument("--window", type=int, default=60, help="Rolling window size (default 60)")
    args = parser.parse_args()
    
    compute_features(args.sym_a, args.sym_b, args.window)
