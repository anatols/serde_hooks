use serde::Serialize;

mod context;
mod hooks;
mod wrapper;

pub use hooks::Hooks;

use context::SerializableWithContext;

pub fn hook<T: Serialize + ?Sized, H: Hooks>(
    serializable: &T,
    hooks: H,
) -> SerializableWithContext<T, H> {
    SerializableWithContext::new(serializable, hooks)
}
