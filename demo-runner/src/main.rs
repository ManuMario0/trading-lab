fn main() {
    let strategy = my_strategy::entry_point();
    trading_core::boot(strategy);
}
