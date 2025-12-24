//! C-compatible Foreign Function Interface (FFI) bindings.
//!
//! This module uses the `cxx` crate to expose Rust functionality to C++.

use crate::args::CommonArgs;
use crate::microservice::registry::{Parameter, Registry};
use crate::model::allocation::{Allocation, Position};
use crate::model::instrument::Stock;
use crate::model::market_data::{MarketDataBatch, PriceUpdate};
use crate::model::order::{Order, OrderSide, OrderType};

#[cxx::bridge(namespace = "trading_core")]
pub mod ffi {
    extern "Rust" {
        // --- Args ---
        type CommonArgs;
        fn get_service_name(self: &CommonArgs) -> String;
        fn get_admin_route_str(self: &CommonArgs) -> String;
        fn get_output_port_str(self: &CommonArgs) -> String;
        fn get_config_dir_str(self: &CommonArgs) -> String;
        fn get_data_dir_str(self: &CommonArgs) -> String;
        fn args_parse(args: Vec<String>) -> Box<CommonArgs>;

        // --- Model: Order ---
        type Order;
        type OrderSide;
        type OrderType;

        fn get_id(self: &Order) -> &str;
        fn get_instrument_id(self: &Order) -> &str;
        fn get_side_i32(self: &Order) -> i32;
        fn get_type_i32(self: &Order) -> i32;
        fn get_price(self: &Order) -> f64;
        fn get_quantity(self: &Order) -> f64;
        fn get_timestamp(self: &Order) -> i64;

        fn new_order(
            id: &str,
            instrument_id: &str,
            side: i32,
            order_type: i32,
            price: f64,
            quantity: f64,
            timestamp: i64,
        ) -> Box<Order>;

        // --- Model: Instrument (Stock) ---
        type Stock;
        fn get_id(self: &Stock) -> usize;
        fn get_symbol(self: &Stock) -> &str;
        fn get_exchange(self: &Stock) -> &str;
        fn get_sector(self: &Stock) -> &str;
        fn get_industry(self: &Stock) -> &str;
        fn get_country(self: &Stock) -> &str;
        fn get_currency(self: &Stock) -> &str;
        fn new_stock(
            id: usize,
            symbol: &str,
            exchange: &str,
            sector: &str,
            industry: &str,
            country: &str,
            currency: &str,
        ) -> Box<Stock>;

        // --- Model: MarketData ---
        type PriceUpdate;
        fn get_instrument_id(self: &PriceUpdate) -> usize;
        fn get_price(self: &PriceUpdate) -> f64;
        fn get_timestamp(self: &PriceUpdate) -> u64;
        fn new_price_update(instrument_id: usize, price: f64, timestamp: u64) -> Box<PriceUpdate>;

        type MarketDataBatch;
        fn new_market_data_batch() -> Box<MarketDataBatch>;
        fn get_count(self: &MarketDataBatch) -> usize;
        fn get_update_at(self: &MarketDataBatch, index: usize) -> &PriceUpdate;

        #[cxx_name = "add_update"]
        fn add_update_boxed(self: &mut MarketDataBatch, update: Box<PriceUpdate>);

        fn clear(self: &mut MarketDataBatch);

        // --- Model: Allocation ---
        type Allocation;
        fn new_allocation(source_name: &str, source_identifier: usize) -> Box<Allocation>;
        fn get_id(self: &Allocation) -> usize;
        fn get_source(self: &Allocation) -> &str;
        fn get_timestamp_u64(self: &Allocation) -> u64;
        // In Allocation
        fn has_position(self: &Allocation, instrument_id: usize) -> bool;
        fn get_position_quantity(self: &Allocation, instrument_id: usize) -> f64;
        fn get_position_copy(self: &Allocation, instrument_id: usize) -> Box<Position>;
        fn update_position(self: &mut Allocation, instrument_id: usize, quantity: f64);

        type Position;
        fn get_instrument_id(self: &Position) -> usize;
        fn get_quantity(self: &Position) -> f64;

        // --- Microservice: Registry ---
        type Parameter;
        fn get_name(self: &Parameter) -> &str;
        fn get_description(self: &Parameter) -> &str;
        fn get_value_as_string(self: &Parameter) -> String;
        fn is_updatable(self: &Parameter) -> bool;

        type Registry;
        fn new_registry() -> Box<Registry>;
        fn get_parameters_list(self: &Registry) -> Vec<String>;

        // --- Service & Config ---
        type Configuration;
        type Microservice;

        fn new_strategy_configuration(callback_ptr: usize, user_data: usize) -> Box<Configuration>;
        fn new_microservice(args: Box<CommonArgs>, config: Box<Configuration>)
            -> Box<Microservice>;
        fn run(self: &mut Microservice);
    }
}

// --- Internal Definitions for Service ---

use crate::microservice::configuration::Configuration as CoreConfiguration;
use crate::microservice::Microservice as CoreMicroservice;

struct FfiState;

pub struct Configuration(CoreConfiguration<FfiState>);
pub struct Microservice(Option<CoreMicroservice<FfiState>>);

fn new_strategy_configuration(callback_ptr: usize, user_data: usize) -> Box<Configuration> {
    let callback: extern "C" fn(*const MarketDataBatch, usize) -> *mut Allocation =
        unsafe { std::mem::transmute(callback_ptr) };

    // Create the Rust closure that calls the C callback
    let market_data_callback = Box::new(
        move |_state: &mut FfiState, batch: &MarketDataBatch| -> Allocation {
            let result_ptr = callback(batch, user_data);
            if result_ptr.is_null() {
                use crate::model::identity::Identity;
                Allocation::new(Identity::new("strategy", "1.0.0", 0))
            } else {
                unsafe { *Box::from_raw(result_ptr) }
            }
        },
    );

    let config = CoreConfiguration::new_strategy(market_data_callback);
    Box::new(Configuration(config))
}

fn new_microservice(args: Box<CommonArgs>, config: Box<Configuration>) -> Box<Microservice> {
    let initial_state = || FfiState;
    let core_config = config.0;

    let ms = CoreMicroservice::new(*args, initial_state, core_config);
    Box::new(Microservice(Some(ms)))
}

impl Microservice {
    fn run(&mut self) {
        if let Some(ms) = self.0.take() {
            ms.run();
        } else {
            eprintln!("Microservice already ran or invalid state.");
        }
    }
}

// --- Implementations for Helper Functions ---

impl CommonArgs {
    fn get_admin_route_str(&self) -> String {
        self.get_admin_route().to_string()
    }
    fn get_output_port_str(&self) -> String {
        self.get_output_port().to_string()
    }
    fn get_config_dir_str(&self) -> String {
        self.get_config_dir().to_string_lossy().to_string()
    }
    fn get_data_dir_str(&self) -> String {
        self.get_data_dir().to_string_lossy().to_string()
    }
}

fn args_parse(args: Vec<String>) -> Box<CommonArgs> {
    Box::new(CommonArgs::parse_args(args))
}

fn new_order(
    id: &str,
    instrument_id: &str,
    side: i32,
    order_type: i32,
    price: f64,
    quantity: f64,
    timestamp: i64,
) -> Box<Order> {
    Box::new(Order::new(
        id,
        instrument_id,
        OrderSide::from_i32(side),
        OrderType::from_i32(order_type),
        price,
        quantity,
        timestamp,
    ))
}

fn new_stock(
    id: usize,
    symbol: &str,
    exchange: &str,
    sector: &str,
    industry: &str,
    country: &str,
    currency: &str,
) -> Box<Stock> {
    Box::new(Stock::new(
        id, symbol, exchange, sector, industry, country, currency,
    ))
}

fn new_price_update(instrument_id: usize, price: f64, timestamp: u64) -> Box<PriceUpdate> {
    Box::new(PriceUpdate::new(instrument_id, price, timestamp))
}

fn new_market_data_batch() -> Box<MarketDataBatch> {
    Box::new(MarketDataBatch::new(Vec::new()))
}

impl MarketDataBatch {
    fn add_update_boxed(&mut self, update: Box<PriceUpdate>) {
        self.add_update(*update);
    }
}

fn new_allocation(source_name: &str, source_identifier: usize) -> Box<Allocation> {
    use crate::model::identity::Identity;
    let identity = Identity::new(source_name, "1.0.0", source_identifier);
    Box::new(Allocation::new(identity))
}

impl Allocation {
    fn get_timestamp_u64(&self) -> u64 {
        self.get_timestamp() as u64
    }

    fn has_position(&self, instrument_id: usize) -> bool {
        self.get_position(instrument_id).is_some()
    }

    fn get_position_quantity(&self, instrument_id: usize) -> f64 {
        self.get_position(instrument_id)
            .map(|p| p.get_quantity())
            .unwrap_or(0.0)
    }

    fn get_position_copy(&self, instrument_id: usize) -> Box<Position> {
        let pos = self.get_position(instrument_id);
        if let Some(p) = pos {
            Box::new(p.clone())
        } else {
            Box::new(Position::new(0, 0.0))
        }
    }
}

fn new_registry() -> Box<Registry> {
    Box::new(Registry::new())
}

impl Registry {
    fn get_parameters_list(&self) -> Vec<String> {
        self.get_parameters()
            .iter()
            .map(|p| p.get_name().to_string())
            .collect()
    }
}
