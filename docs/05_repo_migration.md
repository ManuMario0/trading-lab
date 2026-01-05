# Repository Migration Roadmap

**Status:** Draft / Proposed
**Target:** Q2 2026 (Post-MVP)
**Objective:** Decompose the Monorepo into a Public SDK and Proprietary Platform/Engine to enable "Inversion of Control" SaaS model.

---

## 1. Target Architecture: The "Unity Engine" Model

The goal is to separate the **Contract** (what users write) from the **Engine** (how it runs).

### Repository A: `open-trading-sdk` (Public / Open Source)
> **Access:** Public (Github) / Crates.io (Eventually)
> **Audience:** End Users (Quants, Developers)
> **Content:**
> - `trading-api`: The *only* crate imported by users. Contains traits (`Strategist`) and data models (`Order`).
> - `trading-utils` (Optional): Helpers for math, indicators (non-IP sensitive).

### Repository B: `proprietary-engine` (Private / Core IP)
> **Access:** Core Team Internal
> **Audience:** The Platform
> **Content:**
> - `trading-core`: The Runtime. Wraps user logic in ZMQ networking or Backtest loops.
> - `orchestrator-protocol`: Internal control plane definitions.
> - `generic-runners`: Binaries that can load user code dynamically (e.g., `strategy-runner`, `multiplexer-runner`).

### Repository C: `proprietary-platform` (Private / Infrastructure)
> **Access:** DevOps / Infra
> **Content:**
> - `system-orchestrator`: The daemon managing the fleet.
> - `supervisor-frontend`: The Web UI.
> - `controller`: The CLI.

---

## 2. The User Experience (Inversion of Control)

**Deployment Model:** The user provides *Code*, the Platform provides the *Runtime*.

### A. Development Flow (Local)
1.  User `git clones` a starter template (uses `open-trading-sdk`).
2.  User implements `trait Strategist`.
3.  User runs `cargo test` (Standard Rust unit tests).
4.  **Local Run:** User runs a "Dev Runner" (Docker container provided by you) that mounts their code and recompiles it against the proprietary engine.
    *   *Why Docker?* It solves the Rust ABI stability issue. You control the compiler version inside the container.

### B. Deployment Flow (SaaS)
1.  **Submission:** User pushes code to a Private Git Repo (connected to your platform) OR uploads a Zip.
2.  **Build (CI/CD):** Your Server (Builder):
    - Clones user code.
    - Mounts strict `Cargo.toml` overrides.
    - Compiles user code as a `cdylib` (Dynamic Library) or static link against the Runner.
3.  **Deploy:** Your Orchestrator launches a `strategy-runner` container and injects the user's compiled logic.

---

## 3. Migration Roadmap

### Phase 1: The Public Interface (Immediate)
**Goal:** Isolate `trading-api` so it *feels* like an external library.
1.  **Strict Audit:** Ensure `trading-api` has ZERO dependencies on `trading-core`.
2.  **Dependency Inversion:** `trading-core` must depend on `trading-api`.
3.  **Trait Stabilization:** The `Strategist` trait must be "frozen". Changes here break every user's code.

### Phase 2: The Generic Runner (The "Player")
**Goal:** Stop writing `main.rs` for every strategy.
1.  **Refactor `strategy-lab`:** Convert it from a binary to a library that implements `Strategist`.
2.  **Create `strategy-runner`:** A new binary in `trading-core` that:
    - Accepts a "User Lib" path (Dynamic Loading) OR uses conditional compilation to link a specific strategy.
    - *Note on Rust Dynamic Loading:* It is unsafe/unstable. **Preferred approach for SaaS:** The "Builder" pattern where you take user source + your runner source -> compile one binary.

### Phase 3: The Split (Post-MVP)
**Action:** Physically move directories to new repos.
1.  Create `open-trading-sdk` repo. Publish `trading-api`.
2.  Create `proprietary-engine` repo. Update `Cargo.toml` to depend on `open-trading-sdk`.

---

## 4. Key Risks & Technical Decisions

### The "Rust ABI" Problem
Rust does not have a stable ABI (unlike C). You cannot compile your Engine with Rust 1.80 and run a User Plugin compiled with Rust 1.81. They will crash.

**Solution 1 (The Docker Builder - Recommended):**
All "User Code" is compiled *by your infrastructure* at deploy time.
- **Pros:** Safety, Optimization (LTO), no ABI issues.
- **Cons:** Higher compute cost on deploy (recompiling).

**Solution 2 (WASM - WebAssembly):**
Run user strategies as WASM modules.
- **Pros:** Perfect sandboxing (safety), stable ABI, easy to distribute.
- **Cons:** Performance hit (near-native, but not native), harder to do complex things (threads, networking) inside the sandbox. **Likely too slow for HFT aspirations.**

**Recommendation:** Stick to **Solution 1 (Cloud Build)**.
- User pushes Git.
- Your CI builds the binary using your pinned `trading-core` version.
- Resulting Docker Image is deployed.

---

## 5. Next Steps (Pre-Split)

1.  **Cleanup `trading-api`:** Review strictly. Is everything there necessary for the user? Is anything missing?
2.  **Define the "Compilation Boundary":** Decide if you want to support Dynamic Libraries (`.so`) or if you will monolithically compile user code with your runner. (Hint: Monolithic compilation is safer for Rust).

