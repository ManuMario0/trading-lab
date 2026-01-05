# Architecture: Backtest Engine (Performance Kernel)

> **Goal:** < 50ns per tick processing.
> **Context:** Deterministic historical replay & High-Throughput Training.

## 1. The Monolithic Pipeline
In Backtest Mode, we abandon network sockets for **Direct Memory Access**.
*   **Structure:** A single OS process.
*   **Linking:** Strategy logic is compiled as a static/dynamic library and loaded into the Engine's address space.

## 2. Memory Strategy (Zero-Copy)
*   **Loader:** Uses `mmap` or `io_uring` to map huge Parquet/Binary files into RAM.
*   **Transformer:** Casts raw bytes into `Tick` structs using `rkyv` (Zero-Copy Deserialization).
*   **Feed:** The Engine passes a **Reference** (`&Tick`) to the Strategy. No copying, no allocation.

## 3. Determinism & Time
*   **Event Sourcing:** The engine maintains a global monotonic clock.
*   **Strict Ordering:** Events are processed sequentially.
*   **Result:** Re-running the same seed produces bit-identical results, every time.

## 4. The "Gym" Interface (AI Training)
*   **Batching:** For RL, the engine can step in "Batches" (e.g., Run 10,000 ticks, pause, return state tensor).
*   **Parallelism:** The engine can spawn N threads, each running a separate market simulation, to saturate the CPU.
