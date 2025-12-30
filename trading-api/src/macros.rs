#[macro_export]
macro_rules! export_strategy {
    ($strategy_type:ty) => {
        pub fn entry_point() -> Box<dyn $crate::Strategist> {
            Box::new(<$strategy_type as $crate::Initiable>::init())
        }
    };
}

#[macro_export]
macro_rules! export_multiplexer {
    ($multiplexer_type:ty) => {
        pub fn entry_point() -> Box<dyn $crate::Multiplexist> {
            Box::new(<$multiplexer_type as $crate::Initiable>::init())
        }
    };
}

#[macro_export]
macro_rules! export_executor {
    ($executor_type:ty) => {
        pub fn entry_point() -> Box<dyn $crate::Executor> {
            Box::new(<$executor_type as $crate::Initiable>::init())
        }
    };
}

#[macro_export]
macro_rules! export_portfolio {
    ($manager_type:ty) => {
        pub fn entry_point() -> Box<dyn $crate::Manager> {
            Box::new(<$manager_type as $crate::Initiable>::init())
        }
    };
}

#[macro_export]
macro_rules! export_broker {
    ($broker_type:ty) => {
        pub fn entry_point() -> Box<dyn $crate::Broker> {
            Box::new(<$broker_type as $crate::Initiable>::init())
        }
    };
}
