# Long-Term Vision: Node-Based Strategy Builder & Ops Dashboard

## 1. Executive Summary
The goal is to build a **low-code, visual trading platform** that empowers a single developer to:
1.  **Rapidly Prototype**: Assemble strategies visually using pre-built logic blocks without writing boilerplate.
2.  **Seamlessly Deploy**: Push strategies to paper trading or live environments with one click.
3.  **Visually Monitor**: Instantly assess system health and P&L through a unified "Control Board."

## 2. Core Value Proposition
- **Practicability**: By abstracting complex systems (ZMQ wiring, process management) into visual nodes, you reduce the cognitive load of running a multi-process system.
- **Speed**: "Visual Scripting" allows you to focus on *alpha generation* (logic) rather than *infrastructure* (coding sockets).
- **Peace of Mind**: A visual dashboard provides "at-a-glance" observability, essential for running automated systems 24/7.

## 3. User Interface (UI) Vision
### The "Control Board"
- **Aesthetic**: Dark Mode, "Sci-Fi/Cyberpunk" theme (Deep Blue/Neon).
- **Graph-Based Layout**:
    - **Data Nodes**: Sources of market data (Sim, Live, Replay).
    - **Logic Nodes**: Strategies, Signals (e.g., "RSI > 70"), Filters.
    - **Execution Nodes**: Multiplexers (Risk Management), Execution Engines.
- **Visual Feedback**:
    - **Edges (Spines)**: Animate to show data flow volume/latency.
    - **Node Status**:
        - 🟢 **Healthy**: Running normally.
        - 🔴 **Error**: Disconnected or crashed.
        - 🟡 **Warning**: High latency or close to risk limits.
    - **P&L Overlays**: Real-time P&L badges directly on Strategy/Multiplexer nodes.

## 4. Workflows
### A. Rapid Prototyping (The "Idea" Phase)
1.  **Drag & Drop**: User drags a "Mean Reversion" template onto the canvas.
2.  **Configure**: Clicks the node to set parameters (Window: 20, StdDev: 2) in a sidebar form.
3.  **Wire**: Connects "Market Data" output to Strategy input, and Strategy output to "Paper Multiplexer".
4.  **Simulate**: Presses "Run Backtest" (allocates temporary resources) to validate logic instantly.

### B. Daily Operations (The "Check" Phase)
1.  **Nightly Check**: User opens the dashboard.
2.  **Visual Scan**: Checks that all spines are pulsing (data flowing) and nodes are Green.
3.  **P&L Review**: Sees "+$500" on the Multiplexer node and "-$50" on a specific experimental strategy.
4.  **Action**: Clicks "Stop" on the losing strategy to kill that specific process without affecting others.

## 5. Technical Architecture Requirements
### Dynamic Orchestration (The Enabler)
To support this UI, the underlying System Orchestrator must support:
- **Hot-Swapping**: Ability to add/remove strategies without restarting the Engine or Data Pipeline.
- **Dynamic Configuration**: Strategies must accept formatted JSON config strings via ZMQ Admin commands.
- **Telemetry Stream**: A high-frequency WebSocket stream broadcasting not just prices, but "Heartbeats", "P&L", and "Position" snapshots for every node.

### Component Design
- **Strategy Blocks**: Must be generic execution containers that load specific logic (like a DLL or Python script) based on UI config.
- **Multiplexer**: Acts as the central "Risk Router," aggregating inputs and enforcing the visual connections.

## 6. Implementation Gap Analysis
| Feature | Current State | Required for Vision |
| :--- | :--- | :--- |
| **API** | Basic Control (Start/Stop) | Full "Graph" API (Create/Link Nodes) |
| **Observability** | Console Logs | Structured Telemetry Stream (P&L, Latency) |
| **Configuration** | Command-line Args | Dynamic JSON Config Injection |

## 7. Future Expansion: Visual-to-Native Compilation
### The "Compilable Strategy" Concept
The ultimate evolution is to bridge the gap between "Ease of Design" and "Execution Speed":
1.  **Visual DSL**: You build logic using nodes (e.g., `Price` -> `Minus(MovingAvg)` -> `If(> 0)` -> `Buy`).
2.  **Code Generation**: The UI sends this graph definition to the Orchestrator.
3.  **Transpilation**: The Orchestrator (or a build service) translates the JSON Graph into raw **C++ Source Code** (e.g., generating a `GeneratedStrategy.cpp` that inherits from `IStrategy`).
4.  **Auto-Compilation**: The system invokes `cmake` and `make` in the background to produce a highly optimized binary.
5.  **Hot-Deploy**: The new binary is automatically spawned and wired into the Multiplexer.

### Why this is powerful
- **Zero Overhead**: Unlike interpreted languages (Python), your final strategy runs as native machine code.
- **Safety**: The Visual Editor prevents syntax errors; the compiler catches type errors before deployment.
- **Iteration**: You get the prototyping speed of a GUI with the production performance of C++.
