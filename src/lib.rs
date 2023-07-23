pub mod ser;

mod path;
mod static_str;
mod tests;
mod value;

pub use path::Path;

pub use value::{StaticValue, Value};
