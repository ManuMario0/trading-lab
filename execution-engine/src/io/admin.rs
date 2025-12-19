use crate::engine::Engine;
use crate::models::{AdminCommand, MultiplexerId, StrategyConfig};
use log::{error, info};
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct ZmqAdmin {
    engine: Arc<Mutex<Engine>>,
    port: u16,
    running: Arc<Mutex<bool>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl ZmqAdmin {
    pub fn new(engine: Arc<Mutex<Engine>>, port: u16) -> Self {
        Self {
            engine,
            port,
            running: Arc::new(Mutex::new(false)),
            handle: None,
        }
    }

    pub fn start(&mut self) {
        let engine_clone = self.engine.clone();
        let port = self.port;
        let running_clone = self.running.clone();

        {
            let mut running = self.running.lock().unwrap();
            if *running {
                return;
            }
            *running = true;
        }

        self.handle = Some(thread::spawn(move || {
            let context = zmq::Context::new();
            let responder = context
                .socket(zmq::REP)
                .expect("Failed to create REP socket");
            let addr = format!("tcp://*:{}", port);

            if let Err(e) = responder.bind(&addr) {
                error!("ZmqAdmin failed to bind to {}: {}", addr, e);
                return;
            }

            info!("ZmqAdmin listening on {}", addr);

            while *running_clone.lock().unwrap() {
                // Poll or blocking recv?
                // Using blocking recv for simplicity in this thread.
                // Using poll with timeout to allow checking 'running' flag.
                let mut items = [responder.as_poll_item(zmq::POLLIN)];
                if zmq::poll(&mut items, 100).unwrap() == -1 {
                    break;
                }

                if items[0].get_revents().contains(zmq::POLLIN) {
                    let mut msg = zmq::Message::new();
                    if responder.recv(&mut msg, 0).is_ok() {
                        let msg_str = msg.as_str().unwrap_or("");
                        info!("ZmqAdmin received: {}", msg_str);

                        let response = Self::handle_request(msg_str, &engine_clone);
                        let resp_str = response.to_string();
                        responder
                            .send(&resp_str, 0)
                            .unwrap_or_else(|e| error!("Failed to send reply: {}", e));
                    }
                }
            }
            info!("ZmqAdmin thread exiting.");
        }));
    }

    fn handle_request(req_str: &str, engine: &Arc<Mutex<Engine>>) -> serde_json::Value {
        let req: serde_json::Value = match serde_json::from_str(req_str) {
            Ok(v) => v,
            Err(e) => return json!({"status": "ERROR", "msg": format!("Invalid JSON: {}", e)}),
        };

        let cmd_str = req["cmd"].as_str().unwrap_or("");

        match cmd_str {
            "WALLET" => {
                let engine_guard = engine.lock().unwrap();
                let wallet = engine_guard.portfolios();
                info!("Wallet: {:#?}", wallet);
                json!({"status": "OK", "wallet": wallet})
            }
            "KILL" => {
                // Kill Switch for specific multiplexer or ALL?
                // "Kill switch" usually means ALL in extremis, or specific.
                // User said: "kill switch, add remove multiplexer"
                // Let's assume global kill or specific if ID provided.
                // Assuming specific per existing engine logic 'liquidate_strategy'.
                if let Some(id_str) = req["id"].as_str() {
                    let id = MultiplexerId::new(id_str);
                    // We need to inject a command into the engine. But Engine::process is for Async
                    // Ingress. We can also call methods directly since we have
                    // the Mutex!
                    let mut engine_guard = engine.lock().unwrap();
                    let cmd = AdminCommand::RemoveStrategy { id: id.clone() };
                    // Wait, RemoveStrategy liquidates. Is there a "Kill but keep config"?
                    // Engine::liquidate_strategy is what we want.
                    // Let's use the IngressMessage::Command path to be consistent, or direct calls.
                    // Direct calls are fine.
                    engine_guard.process(crate::models::IngressMessage::Command(cmd));
                    json!({"status": "OK", "msg": format!("Kill signal sent for {}", id)})
                } else {
                    json!({"status": "ERROR", "msg": "Missing 'id'"})
                }
            }
            "ADD" => {
                // Parse StrategyConfig
                let config_val = &req["config"];
                match serde_json::from_value::<StrategyConfig>(config_val.clone()) {
                    Ok(config) => {
                        if let Some(id_str) = req["id"].as_str() {
                            let id = MultiplexerId::new(id_str);
                            let mut engine_guard = engine.lock().unwrap();
                            engine_guard.process(crate::models::IngressMessage::Command(
                                AdminCommand::AddStrategy {
                                    id: id.clone(),
                                    config,
                                },
                            ));
                            json!({"status": "OK", "msg": format!("Added Strategy {}", id)})
                        } else {
                            json!({"status": "ERROR", "msg": "Missing 'id'"})
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse config: {}", e);
                        json!({"status": "ERROR", "msg": format!("Invalid Config: {}", e)})
                    }
                }
            }
            "REMOVE" => {
                if let Some(id_str) = req["id"].as_str() {
                    let id = MultiplexerId::new(id_str);
                    let mut engine_guard = engine.lock().unwrap();
                    engine_guard.process(crate::models::IngressMessage::Command(
                        AdminCommand::RemoveStrategy { id: id.clone() },
                    ));
                    json!({"status": "OK", "msg": format!("Removed {}", id)})
                } else {
                    json!({"status": "ERROR", "msg": "Missing 'id'"})
                }
            }
            "PING" => json!({"status": "PONG"}),
            _ => json!({"status": "ERROR", "msg": "Unknown command"}),
        }
    }
}
