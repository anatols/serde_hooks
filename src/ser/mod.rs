use serde::Serialize;

mod context;
mod hooks;
mod path;
mod wrapper;

pub use hooks::{Hooks, MapScope, ValueScope};
pub use path::Path;

use context::SerializableWithContext;

pub fn hook<T: Serialize + ?Sized, H: Hooks>(
    serializable: &T,
    hooks: H,
) -> SerializableWithContext<T, H> {
    SerializableWithContext::new(serializable, hooks)
}
