use crate::exchange::Exchange;
use crate::exchange::ExecutionResult;
use crate::models::Order;

use std::sync::Mutex;

pub struct ZmqExchange {
    pub_socket: Mutex<zmq::Socket>,
}

impl ZmqExchange {
    pub fn new(context: &zmq::Context, port: u16) -> Self {
        let pub_socket = context
            .socket(zmq::PUB)
            .expect("Failed to create PUB socket");
        let addr = format!("tcp://*:{}", port);
        pub_socket
            .bind(&addr)
            .expect("Failed to bind Order Publisher");
        println!("[ZmqExchange] Order Publisher bound to {}", addr);
        Self {
            pub_socket: Mutex::new(pub_socket),
        }
    }
}

impl Exchange for ZmqExchange {
    fn submit_order(&mut self, order: &Order, market_price: f64) -> ExecutionResult {
        // Serialize Order to JSON
        let json = serde_json::to_string(&order).unwrap_or_default();

        // Publish
        // Topic? Usually just Publish everything. or Topic="ORDER".
        // Let's assume no topic envelope for now, or empty envelope.
        let socket = self.pub_socket.lock().unwrap();
        socket
            .send(&json, 0)
            .unwrap_or_else(|e| println!("Failed to publish order: {}", e));

        // Simulate Immediate Fill for now (Paper Trading Mode within Engine)
        // In real IO, we might wait for Fill report.
        // User asked for "publish orders", implying the execution happens elsewhere or
        // this IS the execution report? "it should publish the orders on a
        // port". The Engine core expects a Result immediately currently (sync
        // trait). So we fake a fill.

        ExecutionResult {
            order_id: order.id(),
            instrument: order.instrument().clone(),
            side: order.side(),
            quantity: order.quantity(),
            price: market_price, // Fill at current price
            fee: 0.0,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }
}
