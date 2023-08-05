use std::borrow::Cow;

use serde::{Serialize, Serializer};

mod context;
mod scope;
mod value;
mod wrapper;

pub use scope::{
    EnumVariantScope, ErrorScope, MapKeyScope, MapKeySelector, MapScope, SeqScope, StructScope,
    TupleScope, TupleStructScope, ValueScope,
};

use context::SerializableWithContext;

use crate::Path;

pub trait Hooks {
    fn on_start(&self) {}

    fn on_end(&self) {}

    #[allow(unused_variables)]
    fn on_error(&self, path: &Path, err: &mut ErrorScope) {}

    #[allow(unused_variables)]
    fn on_map(&self, path: &Path, map: &mut MapScope) {}

    #[allow(unused_variables)]
    fn on_map_key<S: Serializer>(&self, path: &Path, map_key: &mut MapKeyScope<S>) {}

    #[allow(unused_variables)]
    fn on_struct(&self, path: &Path, st: &mut StructScope) {}

    #[allow(unused_variables)]
    fn on_enum_variant(&self, path: &Path, ev: &mut EnumVariantScope) {}

    #[allow(unused_variables)]
    fn on_struct_variant(&self, path: &Path, ev: &mut EnumVariantScope, st: &mut StructScope) {}

    #[allow(unused_variables)]
    fn on_seq(&self, path: &Path, seq: &mut SeqScope) {}

    #[allow(unused_variables)]
    fn on_value<S: Serializer>(&self, path: &Path, value: &mut ValueScope<S>) {}

    /// Specifying any actions that may change the number of elements in the
    /// sequence (e.g. retaining or skipping elements) will force this tuple to be
    /// serialized as a sequence. Depending on the serializer you use, this might be
    /// totally unsupported or lead to unexpected serialization results.
    #[allow(unused_variables)]
    fn on_tuple(&self, path: &Path, tpl: &mut TupleScope, seq: &mut SeqScope) {}

    /// Specifying any actions that may change the number of elements in the
    /// sequence (e.g. retaining or skipping elements) will force this tuple to be
    /// serialized as a sequence. Depending on the serializer you use, this might be
    /// totally unsupported or lead to unexpected serialization results.
    #[allow(unused_variables)]
    fn on_tuple_struct(&self, path: &Path, tpl: &mut TupleStructScope, seq: &mut SeqScope) {}

    /// Specifying any actions that may change the number of elements in the
    /// sequence (e.g. retaining or skipping elements) will force this tuple to be
    /// serialized as a sequence. Depending on the serializer you use, this might be
    /// totally unsupported or lead to unexpected serialization results.
    #[allow(unused_variables)]
    fn on_tuple_variant(
        &self,
        path: &Path,
        ev: &mut EnumVariantScope,
        tpl: &mut TupleScope,
        seq: &mut SeqScope,
    ) {
    }
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum HooksError {
    #[error("cannot add key {0}, the key is already present in the map")]
    KeyAlreadyPresent(MapKeySelector),
    #[error("cannot add entry with an index {0}, please specify key value")]
    CannotAddEntryByIndex(usize),
    #[error("key {0} not found")]
    KeyNotFound(MapKeySelector),
    #[error("field \"{0}\" not found")]
    FieldNotFound(Cow<'static, str>),
    #[error("value is not serializable: {0}")]
    ValueNotSerializable(String),
    #[error("index \"{0}\" not found")]
    IndexNotFound(usize),
}

/// Attach serialization hooks to a serializable value.
///
/// This function returns a new serializable value (i.e. that is `Serialize`)
/// that wraps the `serializable` and calls back on the implementation of
/// [Hooks] on `hooks` at specific moments during serialization.
///
/// Serialization hooks allow inspecting and modifying the serialized value
/// at runtime.
///
/// Note that `hooks` must live at least as long as the `serializable`.
///
/// # Example:
/// ```
/// use serde::Serialize;
/// use serde_hooks::{ser, Path};
///
/// #[derive(Serialize)]
/// struct User {
///     full_name: String,
///     password: String,
/// }
///
/// struct UserHooks {
///     hide_passwords: bool,
/// };
///
/// impl ser::Hooks for UserHooks {
///     // This hook is called on every serialized struct.
///     fn on_struct(&self, _path: &Path, st: &mut ser::StructScope) {
///         // This is similar to #[serde(rename_all="SCREAMING_SNAKE_CASE")].
///         st.rename_all_fields_case("SCREAMING_SNAKE_CASE");
///
///         // This is similar to #[serde(skip)], but the decision to skip is made at runtime.
///         if self.hide_passwords {
///             st.skip_field("password");
///         }
///     }
/// }
///
/// let user = User {
///     full_name: "John Doe".into(),
///     password: "AKJHDKSHD".into(),
/// };
/// let json = serde_json::to_string(&ser::hook(
///     &user,
///     &UserHooks {
///         hide_passwords: true,
///     },
/// ))
/// .unwrap();
///
/// assert_eq!(json, r#"{"FULL_NAME":"John Doe"}"#);
/// ```
/// For more examples, check the documentation of [Hooks].
pub fn hook<'s, 'h: 's, T: Serialize + ?Sized, H: Hooks>(
    serializable: &'s T,
    hooks: &'h H,
) -> impl Serialize + 's {
    SerializableWithContext::new(serializable, hooks)
}
