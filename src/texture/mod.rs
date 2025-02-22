mod error;

pub use error::{Error, Result};

// #[cfg(feature = "texture-atlas")]
pub mod atlas;
// #[cfg(feature = "texture-png")]
pub mod png;
