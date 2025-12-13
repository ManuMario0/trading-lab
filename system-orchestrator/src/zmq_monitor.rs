use crate::config::SystemConfig;
use crate::state::{InstrumentData, PriceUpdate, SharedSystemState};
use log::{debug, error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use zmq::{Context, Socket, POLLIN, SUB};

use tokio::sync::broadcast;

pub struct ZmqMonitor {
    config: SystemConfig,
    state: SharedSystemState,
    running: Arc<AtomicBool>,
    ws_tx: broadcast::Sender<String>,
}

impl ZmqMonitor {
    pub fn new(
        config: SystemConfig,
        state: SharedSystemState,
        ws_tx: broadcast::Sender<String>,
    ) -> Self {
        Self {
            config,
            state,
            running: Arc::new(AtomicBool::new(false)),
            ws_tx,
        }
    }

    pub fn start(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }
        self.running.store(true, Ordering::SeqCst);

        let context = Context::new();
        let running = self.running.clone();
        let config = self.config.clone();
        let state = self.state.clone();
        let ws_tx = self.ws_tx.clone();

        thread::spawn(move || {
            info!("=== ZmqMonitor Loop Starting ===");
            // The context is moved into the thread, so it needs to be created here.
            // let context = Context::new(); // This line was moved outside the thread::spawn in the instruction, but it must be inside for ownership.
            // Reverting to original placement for `context` creation.

            // 1. Subscribe to Market Data
            let data_sub = context.socket(SUB).expect("Failed to create Data SUB");
            data_sub
                .connect(&format!("tcp://127.0.0.1:{}", config.data_port))
                .ok();
            data_sub.set_subscribe(b"").ok();

            // 2. Subscribe to Orders (Engine Output)
            let order_sub = context.socket(SUB).expect("Failed to create Order SUB");
            order_sub
                .connect(&format!("tcp://127.0.0.1:{}", config.order_port))
                .ok();
            order_sub.set_subscribe(b"").ok();

            // 3. Subscribe to Multiplexer Output (Engine Input)
            let mpx_sub = context.socket(SUB).expect("Failed to create Mpx SUB");
            mpx_sub
                .connect(&format!("tcp://127.0.0.1:{}", config.multiplexer_port))
                .ok();
            mpx_sub.set_subscribe(b"").ok();

            let mut items = [
                data_sub.as_poll_item(POLLIN),
                order_sub.as_poll_item(POLLIN),
                mpx_sub.as_poll_item(POLLIN),
            ];

            while running.load(Ordering::SeqCst) {
                // Poll with 100ms timeout
                if let Err(e) = zmq::poll(&mut items, 100) {
                    error!("Zmq Poll Error: {}", e);
                    break;
                }

                // DATA
                if items[0].is_readable() {
                    if let Ok(msg) = data_sub.recv_string(0) {
                        if let Ok(msg) = msg {
                            // Broadcast to WebSockets
                            let _ = ws_tx.send(msg.clone());
                            handle_market_data(&msg, &state);
                        }
                    }
                }

                // ORDERS
                if items[1].is_readable() {
                    if let Ok(msg) = order_sub.recv_string(0) {
                        if let Ok(_msg) = msg {
                            // TODO: Deserialize Order and update state
                            // info!("Monitor saw Order: {}", msg);
                            // For now, minimal impl
                        }
                    }
                }

                // MPX
                if items[2].is_readable() {
                    if let Ok(msg) = mpx_sub.recv_string(0) {
                        if let Ok(_msg) = msg {
                            // info!("Monitor saw TargetPortfolio: {}", msg);
                        }
                    }
                }
            }
            info!("=== ZmqMonitor Loop Stopped ===");
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

fn handle_market_data(json_str: &str, state: &SharedSystemState) {
    // Attempt to parse into PriceUpdate
    // Note: The json sent by brownian_stock might match PriceUpdate directly or need logic
    // We reused the structure from Rust Engine in brownian_stock, so it should match.
    // { "instrument": { "type": "Stock", "data": { "symbol":... } }, "last": ... }

    // We need to map that nested structure to our flat PriceUpdate or struct in state.
    // Let's use serde_json::Value for flexibility first, then map.
    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(v) => {
            // Extract Symbol
            if let Some(symbol) = v
                .pointer("/instrument/data/symbol")
                .and_then(|s| s.as_str())
            {
                if let Some(price) = v.get("last").and_then(|p| p.as_f64()) {
                    // Update State
                    let mut write_guard = state.write().unwrap();

                    // We create a simpler PriceUpdate for the UI
                    let update = PriceUpdate {
                        instrument: InstrumentData {
                            symbol: symbol.to_string(),
                            exchange: "NASDAQ".to_string(), // Simplified
                        },
                        last: price,
                        bid: 0.0, // Extract if needed
                        ask: 0.0,
                        timestamp: 0,
                    };

                    write_guard.last_prices.insert(symbol.to_string(), update);
                    // println!("UPDATED STATE for {}", symbol);
                }
            }
        }
        Err(e) => {
            error!("Monitor failed to parse data: {} from {}", e, json_str);
        }
    }
}
