/// A trait for types that can be initialized without arguments.
///
/// This is similar to `Default`, but semantically implies "Constructor for the Plugin".
pub trait Initiable {
    fn init() -> Self;
}
