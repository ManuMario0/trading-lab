fn main() {
    let _strategy = my_strategy::entry_point();
    let multiplexer = my_multiplexer::entry_point();

    // We can choose which one to boot
    trading_core::boot_multiplexer(multiplexer);
}
