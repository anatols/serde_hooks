// #![warn(missing_docs)]
#![doc = include_str!("../README.md")]
#![doc = include_str!("../docs/lib.md")]

/// Serialization with runtime hooks.
pub mod ser;

mod case;
mod path;
mod value;

pub use case::Case;
pub use path::{Path, PathMapKey, PathSegment};
pub use value::{StaticValue, Value};
