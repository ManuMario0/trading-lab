# System Orchestrator

## Purpose
This component will manage the lifecycle of the trading bot processes (Execution Engine, Multiplexer, Broker Gateway).

## Future Features
- **CPU Pinning**: Use `taskset` (Linux) or `thread_policy` (macOS) to pin high-frequency components (Multiplexer, Engine) to specific isolated CPU cores to avoid context switching and ensuring consistent latency.
- **Process Supervision**: Restart components if they crash (unless manually killed).
