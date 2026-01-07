# Architecture: High-Performance Backtest Engine

> **The Vision:** "The ease of Python (Pandas) with the speed of High-Frequency Trading (C++/FPGA)."

## 1. Core Philosophy: The "One Language" Problem
Industry Standard involves a fatal disconnect: Quants write in Python (slow, safe), Devs rewrite in C++ (fast, complex), leading to implementation drift.
**Our Solution:** **"Kernel Fusion"**.
*   **User Interface:** A Python library (`trading_dsl`) that mimics `numpy`/`pandas`. It does **lazy evaluation** to build an Abstract Syntax Tree (AST).
*   **Execution:** A JIT Transpiler converts that AST into highly optimized Rust code, compiles it, and runs it as a native binary.

---

## 2. Performance Tiers & Goals

| Tier | Technology | Time (10y/1k Tickers) | Throughput (Bars/Sec) | Description |
| :--- | :--- | :--- | :--- | :--- |
| **Tier 1** | Python (Pandas) | ~3 Hours | ~2 Million | Bottleneck: Memory Bandwidth (RAM Thrashing). |
| **Tier 2** | Python (Numpy) | ~45 Minutes | ~8 Million | Better layout, but intermediate Python object allocations. |
| **Tier 3** | Rust/C++ (Naive) | ~10 Minutes | ~40 Million | Native speed, but naive `Vec<f64>` loops (RAM heavy). |
| **Tier 4** | **Target (Stream Fusion)** | **< 60 Seconds** | **~500 Million** | **Our Goal.** Zero-RAM writes. Computation stays in CPU Registers (L1/L2 Cache). |
| **Tier 5** | HFT (KDB+/FPGA) | ~5 Seconds | ~4 Billion+ | Dedicated hardware, manual ASM, AVX-512 hand-tuning. Expensive ($1M+). |

**Value Proposition:** We deliver **Tier 4** performance (45x faster than Tier 1) with **Tier 1** usability.

---

## 3. The "Numpy Shadow" Architecture
How we bridge the gap between Python ease and Rust speed.

### A. The Definition Layer (Python)
The user writes code that *looks* like data analysis:
```python
# Looks like immediate calculation, but is actually building a DAG/AST
rsi = market.col("close").rsi(14)
signal = (rsi > 70) & (market.col("volume") > 1_000_000)
```

### B. The Transpilation Layer (Rust)
We do NOT interpret the Python. We generate Rust code using **Stream Fusion (Lazy Iterators)**.
Instead of creating 3 vectors (RSI, Boolean, Volume Boolean), the compiler fuses them into a single loop:
```rust
// Generated "Phantom" Stream
prices.iter().zip(volumes.iter())
    .map(|(p, v)| {
        let r_rsi = calculate_rsi_step(p, &mut state); // Input Register
        let r_sig = r_rsi > 70.0 && v > 1_000_000.0;   // Output Register
        r_sig // No RAM write
    })
```

---

## 4. Data Foundation: The "Cache" Strategy

### A. Storage (The Source of Truth)
*   **Format:** **Apache Parquet** (Snappy Compressed).
*   **Location:** AWS S3 / Cold Storage.
*   **Why:** Industry standard compatibility (Data Engineering can use Spark/Snowflake).

### B. Hot Path (The Speed Layer)
When a backtest starts, we do NOT parse Parquet rows CPU-by-CPU.
*   **Mechanism:** `mmap` (Memory Mapped Files).
*   **Format:** **Raw Binary Dump** (`#[repr(C)]` or `rkyv`).
*   **Process:**
    1.  Downloads/Reads Parquet.
    2.  Converts to Raw Layout (if not cached).
    3.  `mmap` the Raw file.
    4.  Cast pointer `*u8` to `&MarketDataBatch`.
    5.  **Cost:** 0ns parsing time. The OS handles paging.

---

## 5. Physical Optimizations

### A. CPU Pinning
*   **Problem:** OS Scheduler moving threads destroys L1/L2 cache locality (~20µs penalties).
*   **Solution:** For large batch runs, we pin the "Market Simulation" thread to a specific isolated Core (e.g., Core 7) using `core_affinity`.

### B. Linker Optimization (`mold`)
*   **Problem:** Compiling user logic is fast, but *Linking* against the robust `trading-core` library takes 10+ seconds with `ld`.
*   **Solution:** Use `mold` (Modern Linker) to reduce link times to < 1 second, enabling the "Interactive Feel".

---

## 6. Roadmap Summary

1.  **Phase 1 (MVP Nuance):** Integrate `parquet` crate. Implement standard loading (Tier 3 Speed).
2.  **Phase 2 (Mmap):** Implement `rkyv` Zero-Copy loader (Fast Start).
3.  **Phase 3 (Fusion):** Implement `Iterator` traits for `MarketDataBatch` math (Tier 4 Speed).
4.  **Phase 4 (Frontend):** Build the "Numpy Shadow" Python lib and AST-to-Rust generator.
