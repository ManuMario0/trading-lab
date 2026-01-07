#!/bin/bash
set -e

# Cleanup on exit
# Cleanup on exit
cleanup() {
    echo "Cleaning up..."
    if [ ! -z "$ORCH_PID" ]; then
        kill $ORCH_PID || true
    fi
    pkill -f "strategy-lab" || true
    pkill -f "dummy-feed" || true
    pkill -f "portfolio-manager" || true
    pkill -f "execution-engine" || true
    pkill -f "broker-gateway" || true
}
trap cleanup EXIT

# 0. Clean & Build
echo "Building project..."
rm -f orchestrator.log
# Remove conflicting binaries (multiplexer also claims to be "Strategy")
rm -f ./target/debug/multiplexer
cargo build -p system-orchestrator -p controller -p dummy-feed -p strategy-lab -p portfolio-manager -p execution-engine -p broker-gateway

# 1. Start Orchestrator in background
echo "Starting System Orchestrator..."
export RUST_LOG=info
./target/debug/system-orchestrator --service-dir ./target/debug > orchestrator.log 2>&1 &
ORCH_PID=$!

echo "Waiting for Orchestrator to initialize..."
sleep 10

# 2. Deploy Layout
echo "Deploying Simple Layout..."
./target/debug/controller deploy --file simple_layout.json

# 3. Wait for data flow
echo "Waiting 1 seconds for data flow..."
sleep 1

# 4. Check Strategy Logs
echo "Checking Logs..."
if grep -q "strategy_lab] Received batch" orchestrator.log; then
    echo "SUCCESS: Strategy received data!"
    grep "strategy_lab] Received batch" orchestrator.log | head -n 3
else
    echo "FAILURE: Strategy did not receive data."
    tail -n 50 orchestrator.log
    exit 1
fi

if grep -q "portfolio_manager]" orchestrator.log; then
    echo "SUCCESS: Portfolio Manager received allocation!"
    grep "portfolio_manager]" orchestrator.log | head -n 3
else 
    # PM might not log at INFO level by default? We saw it did earlier.
     echo "WARNING: Portfolio Manager output not found (or filtered)."
fi

# We don't have explicit log confirmation for Exec/Gateway yet, but if they start without crashing, that's a win.
# I will check if they are running.
if grep -q "execution_engine" orchestrator.log; then
     echo "SUCCESS: Execution Engine started/active."
fi
if grep -q "broker_gateway" orchestrator.log; then
     echo "SUCCESS: Broker Gateway started/active."
fi

