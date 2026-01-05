# Architecture: Live Engine (Distributed Reliability)

> **Goal:** 99.999% Availability & Recoverability.
> **Context:** Mission-critical execution.

## 1. The Microservice Standard
Every component (Feed, Strategy, Execution) is wrapped in a standard **Runner Host**.
*   **Process Isolation:** One Strategy = One OS Process (Docker Container or Binary).
*   **Manifests:** Services expose a `manifest` command describing their inputs (Sim/Live ports) and outputs.
*   **Heartbeats:** Every runner emits a heartbeat to the Supervisor.

## 2. The Orchestrator (The Brain)
*   **Role:** Manages the Topology Graph. It does *not* touch critical data paths (ticks/orders).
*   **SMR (Raft):** The Orchestrator is a cluster of 3 nodes using Raft Consensus.
    *   **Leader:** Accepts Admin commands (Deploy, Stop).
    *   **Followers:** Replicate state. If Leader dies, a Follower is elected instantly.

## 3. High Availability Pattern (Active-Warm Standby)
For critical nodes (Execution Engine, Core Strat):
*   **Active Node:** Consumes data, updates state, **sends orders**.
*   **Warm Standby:** Consumes *same* data, updates *same* state, **suppresses orders**.
*   **Failover Logic:**
    1.  Active Node misses N heartbeats (or sends "Panic" signal).
    2.  Orchestrator sends "Promote" command to Standby.
    3.  Standby opens its Output Gate.
    4.  **Recovery Time:** < 50ms.

## 4. Disaster Recovery Hierarchy

| Tier | Threat | Solution | Recovery Time |
| :--- | :--- | :--- | :--- |
| **App** | Bug / Panic | **Process Isolation** + Auto-Restart | Seconds |
| **Server** | OS Freeze / Hardware | **Local HA** (Active/Standby) | Milliseconds |
| **Region** | Cloud Outage | **Async Replication** (Cross-Region) | Minutes |
