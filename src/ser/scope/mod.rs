mod error;
mod map;
mod seq;
mod start;
mod r#struct;
mod tuple;
mod value;
mod variant;

pub use error::ErrorScope;
pub use map::{MapKeySelector, MapScope};
pub use r#struct::StructScope;
pub use seq::SeqScope;
pub use start::StartScope;
pub use tuple::{TupleScope, TupleStructScope};
pub use value::ValueScope;
pub use variant::EnumVariantScope;

pub type MapKeyScope<'v, S> = ValueScope<'v, S>;
