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

        type MarketDataBatch;
        fn get_count(self: &MarketDataBatch) -> usize;
        fn get_update_at(self: &MarketDataBatch, index: usize) -> &PriceUpdate;

        // --- Model: Allocation ---
        type Allocation;
        fn get_id(self: &Allocation) -> usize;
        fn get_source(self: &Allocation) -> &str;
        fn get_timestamp_u64(self: &Allocation) -> u64;
        fn has_position(self: &Allocation, instrument_id: usize) -> bool;
        fn get_position_copy(self: &Allocation, instrument_id: usize) -> Box<Position>;

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

impl Allocation {
    fn get_timestamp_u64(&self) -> u64 {
        self.get_timestamp() as u64
    }

    fn has_position(&self, instrument_id: usize) -> bool {
        self.get_position(instrument_id).is_some()
    }

    fn get_position_copy(&self, instrument_id: usize) -> Box<Position> {
        let pos = self.get_position(instrument_id);
        if let Some(p) = pos {
            Box::new(p.clone())
        } else {
            // Return a default/empty position or handle error better?
            // For now, return a zero-quantity position with ID 0 to indicate not found/safe failure
            // or we could panic, but panic crashes the FFI boundary.
            // Let's assume ID match is checked by caller in robust C++, or return dummy.
            // Correct approach: Result<Box<Position>> but cxx exception handling needed.
            // Simpler: Return dummy.
            Box::new(Position::new(0, 0.0))
        }
    }
}
