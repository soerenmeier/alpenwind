mod error;

pub type Result<T> = std::result::Result<T, Error>;

pub use error::Error;