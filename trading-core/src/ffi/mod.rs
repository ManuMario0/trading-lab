use crate::admin::{ParameterType, GLOBAL_REGISTRY};
use crate::args::CommonArgs;
use crate::comms::ExchangeManager;

#[cxx::bridge(namespace = "trading_core")]
pub mod ffi {
    enum ExchangeType {
        Pub,
        Sub,
        Req,
        Rep,
        Dealer,
        Router,
        Push,
        Pull,
    }

    struct ExchangeConfig {
        name: String,
        endpoint: String,
        socket_type: ExchangeType,
        is_bind: bool,
    }

    // Expose Rust types as opaque C++ types
    extern "Rust" {
        type CommonArgs;
        type ExchangeManager;

        // --- Admin ---
        fn admin_start_server(port: u16);
        fn admin_register_param(
            name: &str,
            description: &str,
            default_value: &str,
            param_type: i32,
        );

        // --- Comms ---
        fn new_exchange_manager() -> Box<ExchangeManager>;

        // Wrapper functions for ExchangeManager methods
        fn exchange_manager_add(mgr: &mut ExchangeManager, config: &ExchangeConfig) -> Result<()>;
        fn exchange_manager_send(
            mgr: &ExchangeManager,
            name: &str,
            data: &[u8],
            flags: i32,
        ) -> Result<()>;
        fn exchange_manager_recv(mgr: &ExchangeManager, name: &str, flags: i32) -> Result<Vec<u8>>;
    }
}

// --- Implementations ---

pub fn admin_start_server(port: u16) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            crate::admin::start_admin_server(port).await;
        });
    });
}

pub fn admin_register_param(name: &str, description: &str, default_value: &str, param_type: i32) {
    let p_type = match param_type {
        0 => ParameterType::String,
        1 => ParameterType::Integer,
        2 => ParameterType::Float,
        3 => ParameterType::Boolean,
        _ => ParameterType::String,
    };

    let mut registry = GLOBAL_REGISTRY.lock().unwrap();
    registry.register(
        name.to_string(),
        description.to_string(),
        p_type,
        default_value.to_string(),
        None,
    );
}

pub fn new_exchange_manager() -> Box<ExchangeManager> {
    Box::new(ExchangeManager::new())
}

// Convert from FFI ExchangeType to internal ExchangeType
impl From<ffi::ExchangeType> for crate::comms::ExchangeType {
    fn from(t: ffi::ExchangeType) -> Self {
        match t {
            ffi::ExchangeType::Pub => crate::comms::ExchangeType::Pub,
            ffi::ExchangeType::Sub => crate::comms::ExchangeType::Sub,
            ffi::ExchangeType::Req => crate::comms::ExchangeType::Req,
            ffi::ExchangeType::Rep => crate::comms::ExchangeType::Rep,
            ffi::ExchangeType::Dealer => crate::comms::ExchangeType::Dealer,
            ffi::ExchangeType::Router => crate::comms::ExchangeType::Router,
            ffi::ExchangeType::Push => crate::comms::ExchangeType::Push,
            ffi::ExchangeType::Pull => crate::comms::ExchangeType::Pull,
            _ => crate::comms::ExchangeType::Pub,
        }
    }
}

pub fn exchange_manager_add(
    mgr: &mut ExchangeManager,
    config: &ffi::ExchangeConfig,
) -> Result<(), anyhow::Error> {
    let internal_config = crate::comms::ExchangeConfig {
        name: config.name.clone(),
        endpoint: config.endpoint.clone(),
        socket_type: config.socket_type.into(), // Use the From impl
        is_bind: config.is_bind,
    };
    mgr.add_exchange(&internal_config)
        .map_err(|e| anyhow::anyhow!(e))
}

pub fn exchange_manager_send(
    mgr: &ExchangeManager,
    name: &str,
    data: &[u8],
    flags: i32,
) -> Result<(), anyhow::Error> {
    mgr.send(name, data, flags).map_err(|e| anyhow::anyhow!(e))
}

pub fn exchange_manager_recv(
    mgr: &ExchangeManager,
    name: &str,
    flags: i32,
) -> Result<Vec<u8>, anyhow::Error> {
    mgr.recv(name, flags).map_err(|e| anyhow::anyhow!(e))
}
