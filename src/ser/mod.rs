use serde::Serialize;

mod context;
mod hooks;
mod path;
mod value;
mod wrapper;

pub use hooks::{Hooks, MapScope, ValueScope, MapKeyScope};
pub use path::Path;
pub use value::{PrimitiveValue, Value};

use context::SerializableWithContext;

pub fn hook<T: Serialize + ?Sized, H: Hooks>(
    serializable: &T,
    hooks: H,
) -> SerializableWithContext<T, H> {
    SerializableWithContext::new(serializable, hooks)
}
