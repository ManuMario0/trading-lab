# Technical Overview: The Dual-Architecture

> **Core Philosophy:** "A single logic kernel, two runtime environments."

## 1. The Extensible Kernel (Type-Driven Safety)
The system is designed to be fully customizable while preventing catastrophic errors. Users can replace **any** component (Strategy, Multiplexer, Portfolio Manager, Gateway) with their own logic, provided they adhere to our Type Safety Contracts.

### A. The "Impossible State" Philosophy
We leverage Rust's Affine Types and Private Constructors to make invalid financial states **unrepresentable at compile time**. The compiler is the first line of Risk Management.
*   **Example (Execution Safety):** A user cannot manually construct a "Buy 1000 BTC" order to "rogue trade."
*   **The Mechanism:** An `Order` can only be produced by a strict `Diff(CurrentPortfolio, TargetPortfolio)` function.
*   **The Guarantee:** The Execution Engine is mathematically constrained to *only* emit orders that converge the portfolio to its target. It physically cannot diverge.

### B. Core Traits (The API)
All components interact via standard Traits, inheriting the "Dual-Mode" (Backtest/Live) capability automatically.
*   `Strategy`: Consumes Ticks -> Emits Signals (Alpha).
*   `Multiplexer`: Consumes Signals -> Emits Allocation (Weighting/Ensemble).
*   `PortfolioManager`: Consumes Allocation -> Emits Target Portfolio (Risk).
*   `Gateway`: Consumes Orders -> Emits Protocol Messages (FIX/REST).

```rust
pub trait Strategist: Send {
    /// The universal update handler.
    /// - Live Mode: Called with Batch Size = 1 (Low Latency).
    /// - Training Mode: Called with Batch Size = N (SIMD/Cache Friendly).
    /// Input/Output are strict types guarding validity.
    fn on_market_data(&mut self, md: MarketDataBatch) -> AllocationBatch;
}
```

## 2. Runtime A: Distributed Mode (Live)
*   **See:** `02_arch_live.md`
*   **Focus:** Reliability, Isolation, Fault Tolerance.
*   **Structure:** Network of independent processes wire together by ZeroMQ.
*   **Manager:** The **System Orchestrator** (Raft Consensus).

## 3. Runtime B: Monolithic Mode (Backtest)
*   **See:** `03_arch_backtest_engine.md`
*   **Focus:** Throughput, Determinism, Nanosecond Latency.
*   **Structure:** Single-process pipeline where strategies are loaded as libraries.
*   **Manager:** The **Backtest Harness** (Direct Memory Access).

## 4. Shared Standards
To ensure compatibility between both modes, all components adhere to:
*   **Self-Description:** Every service/component exposes a schema (Manifest) declaring its Inputs/Outputs.
*   **Data Types:** All messages (Ticks, Orders) use a compact, zero-copy binary format (`rkyv` or `repr(C)`).
