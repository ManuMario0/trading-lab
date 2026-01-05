# Implementation Roadmap

> **Status:** Active Execution (Jan 2026)
> **Priorities:** Verify Core Pipeline -> Backtest Engine -> UI

## 1. Immediate Sprint: The "Hello World" (Due: Jan 31st)
**Goal:** A verifiable, end-to-end data flow with "Self-Describing" services.

### Task 1: Orchestrator Logic & Connectivity [Completed]
*   [x] Finish `resolve_node_config` in `layout/engine.rs`.
*   [x] Verify binding generation.

### Task 2: Data & Pipeline Verification [Active]
*   **Spec:** Ensure `dummy-feed` writes to `strategy-lab`.
*   [ ] Add `DataFeed` trait to `trading-api`.
*   [ ] Update `simple_layout.json` to include `dummy-feed`.
*   [ ] **Verification:** Run the pipeline. Expect logs showing `Strategy received Tick { price: ... }`.

### Task 3: Manifests (Self-Describing Services)
*   **Spec:** The Orchestrator must not hardcode ports.
*   [ ] Services implement `--manifest` returning JSON (ports, types, version).
*   [ ] Orchestrator discovers capability at startup.

---

## 2. Phase 2: The Backtest Engine (Feb 1st - Feb 15th)
**Goal:** Validated performance parity.

*   **The "Compiler":** CLI tool (`cargo generate`) to bootstrap new strategies from a template.
*   **CsvFeeder:** A Replay feeder that reads `data/raw/*.parquet` (via Polars/Arrow) and emits ZMQ ticks at max speed.
*   **In-Process Runner:** Linking the Strategy Trait directly to the Feeder for IO-less backtesting.
*   **Benchmark:** Prove 100x speedup over Python.

---

## 3. Phase 3: Risk & Execution Hardening (Feb 15th - Feb 28th)
**Goal:** Unshakeable Safety.

*   **Policy Engine:**
    *   Implement "Hard Limits" (Max Leverage, Max Drawdown) in the Multiplexer.
    *   **Test:** Send an illegal order and verify rejection.
*   **Persistence (The Kill Switch):**
    *   Implement `JSON` state dumping for `PortfolioManager`.
    *   **Demo:** Kill process, restart, verify state is restored.

---

## 4. Phase 4: UI & Operations (March)
**Goal:** The "Control Board" Dashboard.

*   **API:** Orchestrator exposes HTTP/WebSocket for "Graph State".
*   **Frontend:** React Flow based dashboard.
    *   Visual Node Graph.
    *   Green/Red status lights.
    *   Start/Stop buttons.

---

## Technical Debt & Refactoring
*   [ ] **Error Handling:** Migrate `anyhow::Result` to explicit `thiserror` enums in core paths.
*   [ ] **ZMQ Safety:** Remove `unwrap()` in network handling code.
*   [ ] **Docs:** Ensure all public API traits (`Strategy`, `Microservice`) have rustdoc.
