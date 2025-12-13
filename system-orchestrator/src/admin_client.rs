use crate::config::SystemConfig;
use log::info;
use std::sync::{Arc, Mutex};
use zmq::{Context, Socket, REQ};

pub struct AdminClient {
    socket: Arc<Mutex<Socket>>,
}

impl AdminClient {
    pub fn new(config: &SystemConfig) -> anyhow::Result<Self> {
        let context = Context::new();
        let socket = context.socket(REQ)?;
        let addr = format!("tcp://127.0.0.1:{}", config.admin_port);
        socket.connect(&addr)?;
        info!("AdminClient connected to {}", addr);

        Ok(Self {
            socket: Arc::new(Mutex::new(socket)),
        })
    }

    pub fn send_command(&self, cmd: &str) -> anyhow::Result<String> {
        let socket = self.socket.lock().unwrap();
        socket.send(cmd, 0)?;

        // Blocking receive for reply
        let reply = socket
            .recv_string(0)?
            .map_err(|bytes| anyhow::anyhow!("Invalid UTF-8 in reply: {:?}", bytes))?;
        Ok(reply)
    }
}
