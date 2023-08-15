#![warn(missing_docs)]
#![doc = include_str!("../docs/lib.md")]

/// Serialization with runtime hooks.
pub mod ser;

mod case;
mod path;
mod static_str;
mod value;

pub use case::Case;
pub use path::{Path, PathMapKey, PathSegment};
pub use value::{StaticValue, Value};
