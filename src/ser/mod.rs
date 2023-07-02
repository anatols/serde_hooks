use serde::Serialize;

mod context;
mod hooks;
mod wrapper;
mod path;

pub use hooks::Hooks;
pub use path::Path;

use context::SerializableWithContext;

pub fn hook<T: Serialize + ?Sized, H: Hooks>(
    serializable: &T,
    hooks: H,
) -> SerializableWithContext<T, H> {
    SerializableWithContext::new(serializable, hooks)
}
