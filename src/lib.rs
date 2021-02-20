pub mod bitstamp;
pub mod binance;
pub mod order_book;

/// Error returned by most functions.
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// A specialized `Result` type defined as a convenience.
pub type Result<T> = std::result::Result<T, Error>;
