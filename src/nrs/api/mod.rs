pub mod error;
pub mod extract;
pub mod user;

pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;
