use serde::{Serialize, Serializer};

mod context;
mod hooks;
mod wrapper;

pub use hooks::Hooks;

use context::Context;

pub fn hook<T: Serialize + ?Sized, H: Hooks>(
    serializable: &T,
    hooks: H,
) -> wrapper::SerializableWithHooks<T, Context<H>> {
    wrapper::SerializableWithHooks::new(serializable, Context::new(hooks))
}
