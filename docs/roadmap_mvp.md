# 30-Day "Sprint" Roadmap (Detailed)

## Objective
Deliver a verified, risk-managed, algorithm-ready trading platform with "Topology" visualization in 4 weeks.

## Phase 1: Data Foundation & Feature Engineering (Week 1)
**Goal**: Strategies never calculate raw math; they consume "Features".
- **Day 1-2: Raw Data Pipeline**
  - Implement `PolygonWrapper` (or Yahoo) in `data-pipeline`.
  - Storage: Store 1-minute bars in SQLite/Parquet (e.g., `data/raw/AAPL.parquet`).
  - **Deliverable**: Script that downloads last 30 days of SP500 tickers.
- **Day 3-5: The "Feature Store" (Data Crunching)**
  - Create a `DataProcessor` service (Python).
  - **Task**: Load raw Parquet -> Compute Covariance/Eigenvalues/Z-Scores -> Save to `data/derived/`.
  - **Stream**: In Replay Mode, publish BOTH `{Price}` and `{Features}` (e.g., `{"symbol": "PAIR_A_B", "z_score": 2.1}`) over ZMQ.
  - **Deliverable**: A "Replay" that emits price updates AND pre-computed rolling covariance.
- **Day 6-7: Verification**
  - Visualize the "Z-Score" stream in a simple Python plot script to verify data quality.

## Phase 2: Execution Engine & Risk Hardening (Week 2)
**Goal**: "Safe" execution. The engine is the gatekeeper.
- **Day 8-10: Standalone Broker Services**
  - **Architecture**: Decouple Broker logic from Engine.
  - **Interface**: `ExecutionEngine` pushes valid orders to ZMQ (`tcp://*:5570`). Broker Service listens.
  - **Modular Broker Gateways**: We will implement `gateway-ibkr` (Interactive Brokers) and `gateway-paper` as **standalone services**.
  **Goal**: Swapping "IBKR" for "Paper" is just a process switch in the Orchestrator, zero Engine code changes.
  - **Deliverable**: `IBKRService` running as a child process, receiving ZMQ orders.
- **Day 11-13: Policy Engine (The "Guardian")**
  - **Hard Constraints**: `MaxLeverage`, `MaxDrawdown` (Daily), `Whitelist` (Only trade specific symbols).
  - **Logic**: If `Drawdown > 2%`, trigger global `LiquidateAll`.
  - **Deliverable**: Unit tests proving the Engine REJECTS orders that violate risk.
- **Day 14: Latency & Stability**
  - Stress test the loop. Ensure < 1ms internal latency for risk checks.

## Phase 3: Dynamic Topology & Orchestration (Week 3)
**Goal**: The Backend supports the "Graph" vision.
- **Day 15-17: Dynamic Orchestration API**
  - Implement `POST /topology/node/add` (Spawn new strategy process).
  - Implement `POST /topology/link` (Tell Node A to push to Node B's ZMQ port).
  - **Deliverable**: Use `curl` to spawn a new strategy and wire it up *without restarting*.
- **Day 18-20: State Persistence**
  - Save graph layout to `topology.json`.
  - On restart, Orchestrator respawns the previous graph.
- **Day 21: Telemetry**
  - Enhance `/ws` to send "Heartbeats" and "P&L" per node.

## Phase 4: UI & End-to-End Integration (Week 4)
**Goal**: Visual Control.
- **Day 22-25: React Dashboard**
  - Setup React + React Flow.
  - Visualize the nodes (Engine, Mux, Strat) as boxes.
  - Animate the links (ZMQ activity).
- **Day 26-28: Control & Monitoring**
  - Add "Start/Stop" buttons in UI.
  - Display P&L Charts (using Recharts).
- **Day 29-30: Code Freeze & Live Test**
  - Deploy to a cloud server (or dedicated local machine).
  - Run "System A" (Pair Trading) on Paper for 24h.

## Recommendation on "Data Crunching"
**Decision: Python First (MVP), C++ Later.**
- **MVP (Weeks 1-4)**: Use Python (`numpy`/`pandas`) in the `DataProcessor` service. It is fast enough for 1-minute bars and drastically reduces dev time.
- **Post-MVP**: If latency profiling indicates a bottleneck, port the math kernel to C++ (using LAPACK/Eigen) as a dedicated optimization phase.
