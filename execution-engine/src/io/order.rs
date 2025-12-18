use crate::exchange::Exchange;
use crate::exchange::ExecutionResult;
use crate::models::{Order, Side};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Deserialize)]
struct GatewayResponse {
    order_id: String,
    status: String,
    filled_qty: f64,
    filled_price: f64,
    fee: f64,
}

pub struct ZmqExchange {
    req_socket: Mutex<zmq::Socket>,
}

impl ZmqExchange {
    pub fn new(context: &zmq::Context, port: u16) -> Self {
        // We use REQ to push orders and wait for confirmation (Ack/Fill)
        let req_socket = context
            .socket(zmq::REQ)
            .expect("Failed to create REQ socket");

        // Connect to the Gateway Service (e.g. gateway-paper at 5570)
        // NOTE: The 'port' argument passed here should be 5570.
        // If main.rs passes 5563 (Order Port), we might need to change main.rs config
        // OR rely on main.rs passing the correct Gateway Port.
        // For MVP, let's assume 'port' IS the gateway port.
        let addr = format!("tcp://localhost:{}", port);
        req_socket
            .connect(&addr)
            .expect("Failed to connect to Gateway");

        println!("[ZmqExchange] Connected to Gateway at {}", addr);

        Self {
            req_socket: Mutex::new(req_socket),
        }
    }
}

impl Exchange for ZmqExchange {
    fn submit_order(&mut self, order: &Order, _market_price: f64) -> ExecutionResult {
        // Serialize Order to JSON
        // We rely on Order implementing Serialize matching the Gateway's expected
        // format.
        let json = serde_json::to_string(&order).unwrap_or_default();

        let socket = self.req_socket.lock().unwrap();

        // 1. Send Order
        if let Err(e) = socket.send(&json, 0) {
            println!("Failed to send order: {}", e);
            // Return empty/failed result?
            // For MVP panic or return zero fill.
            return ExecutionResult {
                order_id: order.id(),
                instrument: order.instrument().clone(),
                side: order.side(),
                quantity: 0.0,
                price: 0.0,
                fee: 0.0,
                timestamp: chrono::Utc::now().timestamp_millis(),
            };
        }

        // 2. Wait for Response (Sync)
        match socket.recv_string(0) {
            Ok(Ok(response_json)) => {
                match serde_json::from_str::<GatewayResponse>(&response_json) {
                    Ok(resp) => {
                        // Map Gateway Response to ExecutionResult
                        ExecutionResult {
                            order_id: order.id(), // Assume Gateway echoes ID or we rely on our ID
                            instrument: order.instrument().clone(),
                            side: order.side(),
                            quantity: resp.filled_qty,
                            price: resp.filled_price,
                            fee: resp.fee,
                            timestamp: chrono::Utc::now().timestamp_millis(),
                        }
                    }
                    Err(e) => {
                        println!(
                            "Failed to parse Gateway Response: {} | {}",
                            e, response_json
                        );
                        ExecutionResult {
                            order_id: order.id(),
                            instrument: order.instrument().clone(),
                            side: order.side(),
                            quantity: 0.0,
                            price: 0.0,
                            fee: 0.0,
                            timestamp: chrono::Utc::now().timestamp_millis(),
                        }
                    }
                }
            }
            _ => {
                println!("Failed to receive response from Gateway");
                ExecutionResult {
                    order_id: order.id(),
                    instrument: order.instrument().clone(),
                    side: order.side(),
                    quantity: 0.0,
                    price: 0.0,
                    fee: 0.0,
                    timestamp: chrono::Utc::now().timestamp_millis(),
                }
            }
        }
    }
}
