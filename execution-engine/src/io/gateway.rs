use crate::gateway::Gateway;
use crate::models::{IngressMessage, Price, TargetPortfolio};
use log::{error, info};
use std::sync::mpsc::{Receiver, TryRecvError}; // Strict usage: Control signals only
use zmq::{Context, Socket};

/// Control commands for the Gateway
pub enum GatewayCommand {
    ConnectMultiplexer(u16),
    ConnectData(u16),
}

pub struct ZmqGateway {
    context: Context,
    data_socket: Socket,
    multiplexer_sockets: Vec<Socket>,
    control_rx: Receiver<GatewayCommand>,
}

impl ZmqGateway {
    pub fn new(
        context: Context,
        data_port: u16,
        multiplexer_ports: Vec<u16>,
        control_rx: Receiver<GatewayCommand>,
    ) -> Self {
        let data_socket = context.socket(zmq::SUB).expect("Failed to create Data SUB");
        let addr = format!("tcp://localhost:{}", data_port);
        data_socket
            .connect(&addr)
            .expect("Failed to connect Data SUB");
        data_socket.set_subscribe(b"").expect("Failed to sub all");
        info!("[ZmqGateway] Connected to Data at {}", addr);

        let mut gateway = Self {
            context,
            data_socket,
            multiplexer_sockets: Vec::new(),
            control_rx,
        };

        // Initialize Poll Items (Data is index 0)
        // Note: poll_items must be rebuilt if we add sockets?
        // zmq::poll takes &mut [PollItem]. We need to reconstruct it.
        // Or keep a vector.

        for port in multiplexer_ports {
            gateway.connect_multiplexer(port);
        }

        gateway
    }

    fn connect_multiplexer(&mut self, port: u16) {
        let socket = self
            .context
            .socket(zmq::SUB)
            .expect("Failed to create MPX SUB");
        let addr = format!("tcp://localhost:{}", port);
        socket.connect(&addr).expect("Failed to connect MPX SUB");
        socket.set_subscribe(b"").expect("Failed to sub all");
        info!("[ZmqGateway] Connected to Multiplexer at {}", addr);
        self.multiplexer_sockets.push(socket);
        // Deferred rebuild
    }

    fn check_control(&mut self) {
        loop {
            match self.control_rx.try_recv() {
                Ok(cmd) => match cmd {
                    GatewayCommand::ConnectMultiplexer(port) => {
                        self.connect_multiplexer(port);
                    }
                    GatewayCommand::ConnectData(port) => {
                        // Reconnect data? Or add?
                        // Implementation simplified: just log for now, or replace socket.
                        info!("[ZmqGateway] Re-connecting Data to {}", port);
                        // Ideally: drop old, create new.
                    }
                },
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
    }
}

impl Gateway for ZmqGateway {
    fn next(&mut self) -> Option<IngressMessage> {
        loop {
            // 1. Process Control Commands (Dynamic Ports)
            self.check_control();

            // 2. Build local Poll Items
            let mut items = Vec::new();
            items.push(self.data_socket.as_poll_item(zmq::POLLIN));
            for s in &self.multiplexer_sockets {
                items.push(s.as_poll_item(zmq::POLLIN));
            }

            // 3. Poll ZMQ
            // Block for 10ms.
            match zmq::poll(&mut items, 10) {
                Ok(-1) => return None, // Error
                Ok(_) => {}            // Continue to check revents (or timeout)
                Err(_) => return None, // Error
            }

            // 4. Check Data Socket (Index 0)
            if items[0].get_revents().contains(zmq::POLLIN) {
                if let Ok(msg) = self.data_socket.recv_string(0) {
                    match msg {
                        Ok(s) => match serde_json::from_str::<Price>(&s) {
                            Ok(price) => return Some(IngressMessage::MarketData(price)),
                            Err(e) => error!("Failed to parse Price: {} from {}", e, s),
                        },
                        Err(_) => {}
                    }
                }
            }

            // 5. Check Multiplexer Sockets (Indices 1..N)
            for i in 0..self.multiplexer_sockets.len() {
                if items[i + 1].get_revents().contains(zmq::POLLIN) {
                    if let Ok(msg) = self.multiplexer_sockets[i].recv_string(0) {
                        match msg {
                            Ok(s) => match serde_json::from_str::<TargetPortfolio>(&s) {
                                Ok(target) => return Some(IngressMessage::TargetPortfolio(target)),
                                Err(e) => error!("Failed to parse Target: {} from {}", e, s),
                            },
                            Err(_) => {}
                        }
                    }
                }
            }

            // Loop continues if no message found
        }
    }
}
