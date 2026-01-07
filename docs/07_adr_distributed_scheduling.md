# Architecture Decision Record: Distributed Resource Scheduling "The Hive"

**Date:** 2026-01-06
**Status:** PROPOSED (Draft)
**Context:** Transitioning from Single-Node Orchestrator to Multi-Node/Cloud-Native Architecture.

---

## 1. The Challenge: Heterogeneous Workloads
We need to support two distinct workload profiles, often running on specialized hardware:
1.  **"The Heavy Lifters" (Backtest/Data Nodes):**
    *   **Profile:** Memory Intensive, High I/O.
    *   **Hardware:** Machines with massive RAM (e.g., AWS `r6g` instances) to hold historical data in-memory (RAM Disks) for instant replay.
    *   **Role:** Running the "Time Travel" simulations.
2.  **"The Racers" (Live/Network Nodes):**
    *   **Profile:** CPU Latency Sensitive, Network Intensive.
    *   **Hardware:** Compute optimized, high clock speed, clean network paths (e.g., AWS `c6gn` instances).
    *   **Role:** Handling real-time Tick-to-Trade loops.

**Constraint:** The `System Orchestrator` must intelligently partition these workloads. 

**Clarification on Physical Partitioning:**
We strictly enforce **Hardware Partitioning**.
*   **Live Nodes:** Dedicated solely to production trading. NO backtests allow.
*   **Backtest/Data Nodes:** Dedicated to simulation. They keep datasets permanently loaded in RAM ("Hot Data") to minimize startup time.
*   *These are physically distinct machines/instances, never shared.*

---

## 2. Key Engineering Challenges (To Be Solved)

### A. State Synchronization (The "Split-Brain" Risk)
*   **Risk:** If the Orchestrator loses connection to a Live Runner, it might assume the Runner is dead and spawn a replacement. If the original Runner is still alive (just a network partition), we have two strategies trading the same account. **This is unacceptable.**
*   **Solution (Fencing at the Gateway):** 
    *   We implement **Token-Based Filtering** at the Broker Gateway.
    *   Every Strategy instance is assigned a unique `DeploymentID`.
    *   The Gateway is configured to *only* accept orders from the current active `DeploymentID`.
    *   **Failover:** If we switch to a warm backup, the Orchestrator tells the Gateway: "Drop packets from UUID-OLD, accept packets from UUID-NEW". This guarantees no double-execution even if the old node keeps sending orders.

### B. Scalability of the "Mesh"
*   **Risk:** As the number of nodes grows, managing N*N ZMQ connections via the Orchestrator becomes fragile. Each toplogy change requires updating every peer's configuration.
*   **Solution (Sidecar Proxy Pattern):**
    *   The `Runner` daemon acts as a **Local "Post Office" (ZMQ Forwarder)**.
    *   **The Strategy:** Is "dumb" and only connects to local IPC sockets (e.g., `ipc:///tmp/strategies/strat-1/market_data`). It knows nothing about the network.
    *   **The Runner:** Manages the bridge between this local IPC socket and the actual remote IP addresses (TCP).
    *   **Benefit:** We can reconfigure physical network topology (move a Feeder to a new IP) by simply updating the Runner's routing table. The Strategy process sees zero downtime/disconnects.

### C. Data Locality
*   **Optimization:** The Scheduler must prioritize "Data Awareness". It's not enough to find a node with RAM; it must find the node that *already has the specific dataset heated in memory* to avoid massive network transfer penalties.

---

## 3. The Solution: "Scheduler & Runners" (The Hive)

We decouple **Piping** (Logic) from **Placement** (Physics).

### A. The Components

#### 1. The Runner (The "Muscle")
A lightweight daemon (`trading-runner`) installed on every node.
*   **Capabilities Reporting:** On startup, it reports its "Specs" (Tags) to the Orchestrator.
    *   `tags: ["ram_128gb", "data_node", "region_us_east"]`
    *   `tags: ["low_latency", "live_node", "region_tokyo"]`
*   **Resource Isolation:** It manages the `cgroups` or Docker containers on its host.
*   **Port Management:** It owns the port range on its host. It returns the *actual* IPs/Ports used.

#### 2. The Scheduler (The "Brain")
A module within the Orchestrator's Core.
*   **Placement Logic:** Instead of `spawn(service)`, it performs `match_node(requirements)`.
    *   *Request:* "I need a `MomentumStrategy` for `Backtest`."
    *   *Logic:* Look for connected Runners with tag `data_node`.
    *   *Result:* Assigns to Runner ID `node-05` (High RAM).
*   **Load Balancing:**
    *   If 3 Runners match, use the one with `lowest_cpu_load`.

#### 3. The Layout Engine (The "Electrician")
Remains focused on **Topology**.
*   It receives the `ServiceMetadata` (IPs/Ports) from the Scheduler.
*   It calculates the wiring: "Connect `node-05` (Backtest Engine) to `node-02` (Results Database)."

---

## 3. The Data Flow (Future State)

1.  **Deployment Request**: User submits a Layout for "Backtest Simulation".
2.  **Constraint Solving (Scheduler)**:
    *   Analyzer reads `ServiceManifest`. Metadata says: `requires: ["high_ram"]`.
    *   Scheduler scans **Runner Registry**. Finds `Runner-B` (Tag: `high_ram`).
3.  **Instruction (RPC)**:
    *   Orchestrator -> `Runner-B`: "Spawn `strategy_v1` binary."
4.  **Execution (Runner)**:
    *   `Runner-B` spawns process.
    *   Allocates local ports: `10.0.1.5:8000`.
5.  **Acknowledgement**:
    *   `Runner-B` -> Orchestrator: "Spawned. Listening on `10.0.1.5:8000`."
6.  **Piping (Layout Engine)**:
    *   Orchestrator updates all peers: "Send traffic for `strategy_v1` to `10.0.1.5:8000`."

---

## 4. Current Implementation Plan (Bridging the Gap)

To prepare for this without over-engineering now:

1.  **Abstract the `ServiceProvider` Trait:**
    *   Currently: `LocalRuntime` (spawns processes directly).
    *   Target: `ClusterRuntime` (talks to Runners).
    *   **Action:** Ensure our `Supervisor` uses a strict trait that returns `Future<AddressMetadata>`.

2.  **Manifest Metadata:**
    *   Add `requirements` field to `ServiceManifest` (e.g., `labels: ["high_memory"]`).

3.  **Standardized "Runner" Protocol:**
    *   Define the protobuf/structs for `SpawnRequest` and `SpawnResponse` now, even if we only use them locally.

---

**Next Steps:**
1.  Continue stabilizing the `LocalRuntime` (Task 1.3 completed).
2.  Design the "Runner Protocol" (Interface Definition).
3.  Implement a "Mock Runner" to simulate a remote node in tests.
