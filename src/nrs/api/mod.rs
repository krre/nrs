pub mod endpoint;
pub mod error;
pub mod extract;

pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;
