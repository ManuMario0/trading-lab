# Trading Bot Architecture

## Overview
This is a high-performance, polyglot automated trading system designed for safety, extensibility, and speed.

## Structure
- **`strategy-lab/` (C++)**: Strategy generation and backtesting factory.
- **`multiplexer/` (C++)**: Capital allocation (Kelly Criterion) and signal aggregation.
- **`execution-engine/` (Rust)**: Core execution logic, risk guard, and internal paper exchange simulation.
- **`broker-gateway/` (Rust)**: Adapter for external broker APIs (IBKR, Alpaca).
- **`data-pipeline/` (Python)**: Historical data processing and analytics.
- **`supervisor-frontend/` (Python)**: Dashboard and sandbox manager.
- **`system-orchestrator/`**: Lifecycle management and CPU pinning.

## Getting Started
Run `make help` to see available commands.
