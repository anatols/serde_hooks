use serde::Serialize;

mod context;
mod hooks;
mod path;
mod value;
mod wrapper;

pub use hooks::{ErrorScope, Hooks, HooksError, MapKeyScope, MapScope, StructScope, ValueScope};
pub use path::Path;
pub use value::{PrimitiveValue, Value};

use context::SerializableWithContext;

pub fn hook<'s, 'h, T: Serialize + ?Sized, H: Hooks>(
    serializable: &'s T,
    hooks: &'h H,
) -> SerializableWithContext<'s, 'h, T, H> {
    SerializableWithContext::new(serializable, hooks)
}
