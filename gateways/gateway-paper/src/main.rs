use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
// use zmq::{Context, Socket}; // Using blocking ZMQ in a managed thread or just polling
// Actually for a simple REP service, blocking loop is fine if we are single threaded.
// But we want to simulate delay.
// Let's use basic loop for MVP.

#[derive(Debug, Deserialize, Serialize)]
struct InstrumentData {
    symbol: String,
    exchange: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Instrument {
    #[serde(rename = "type")]
    type_: String,
    data: InstrumentData,
}

#[derive(Debug, Deserialize, Serialize)]
struct Order {
    id: String,
    instrument: Instrument,
    side: String,
    quantity: f64,
    order_type: String,
}

#[derive(Debug, Serialize)]
struct ExecutionReport {
    order_id: String,
    status: String, // "Filled"
    filled_qty: f64,
    filled_price: f64,
    fee: f64,
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("=== Gateway Paper Starting ===");

    let context = zmq::Context::new();
    let socket = context.socket(zmq::REP)?;

    // Bind to 5570 (Standard Gateway Port for this MVP)
    let port = 5570;
    let addr = format!("tcp://*:{}", port);
    socket.bind(&addr)?;
    log::info!("Bound to {}", addr);

    loop {
        // 1. Wait for Order Request
        let msg = socket.recv_string(0)?;
        match msg {
            Ok(json_str) => {
                log::info!("Received Order: {}", json_str);

                match serde_json::from_str::<Order>(&json_str) {
                    Ok(order) => {
                        // 2. Simulate Execution Logic
                        // In a real async system, we would return "Pending" and send "Fill" later via another channel (PUB).
                        // But Engine currently calls `submit_order` synchronously or expects immediate return?
                        // Let's assume the Engine sends REQ and waits for REP.
                        // We will check Engine implementation.
                        // Assuming Sync REQ-REP for MVP step 1.

                        // Mock Price (We don't know the market price here unless we sub to data!)
                        // For Paper Gateway, we trust the "Limit Price" or we need Data.
                        // If Market Order, we need a price.
                        // Hack: The Order struct usually comes with a "price" hint if the Engine sent it,
                        // OR the Engine expects us to fill at the next tick?

                        // Minimal MVP: Fill at 150.0 hardcoded or random, just to test flow.
                        // OR better: The Engine passed the `market_price` in the function call `submit_order(&order, price)`.
                        // Does it send that over ZMQ?
                        // We need to define the ZMQ payload.

                        // Let's assume payload includes "limit_price" or "expected_price".
                        // If not, we just fill at 100.0 for now.
                        let fill_price = 100.0;

                        let report = ExecutionReport {
                            order_id: order.id,
                            status: "Filled".to_string(),
                            filled_qty: order.quantity,
                            filled_price: fill_price,
                            fee: 1.0,
                        };

                        let response = serde_json::to_string(&report)?;
                        // Simulate latency?
                        // std::thread::sleep(Duration::from_millis(100));

                        socket.send(&response, 0)?;
                        log::info!("Sent Fill: {:?}", report);
                    }
                    Err(e) => {
                        log::error!("Failed to parse order: {}", e);
                        socket.send("ERROR: Invalid JSON", 0)?;
                    }
                }
            }
            Err(bytes) => {
                log::error!("Received non-UTF8 message: {:?}", bytes);
            }
        }
    }
}
