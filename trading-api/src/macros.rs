#[macro_export]
macro_rules! export_strategy {
    ($strategy_type:ty) => {
        pub fn entry_point() -> Box<dyn $crate::Strategist> {
            Box::new(<$strategy_type>::default())
        }
    };
}
