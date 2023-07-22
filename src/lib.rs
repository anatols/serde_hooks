pub mod ser;

mod path;
mod tests;
mod value;

pub use path::Path;

pub use value::{PrimitiveValue, StaticPrimitiveValue, Value};
