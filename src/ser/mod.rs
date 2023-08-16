use std::borrow::Cow;

use serde::{Serialize, Serializer};

mod context;
mod scope;
mod value;
mod wrapper;

pub use scope::{
    EnumVariantScope, ErrorScope, MapKeyScope, MapKeySelector, MapScope, SeqScope, StartScope,
    StructScope, TupleScope, TupleStructScope, ValueScope,
};

use context::SerializableWithContext;

use crate::Path;

/// A collection of callback functions (hooks) that are called at specific times during serialization.
///
/// All hooks by default have empty implementations. You only need to implement the ones you're interested in.
///
/// Most hooks take a [`path`](Path) parameter which allows you to figure out where in the
/// serialized data you're called back from.
pub trait Hooks {
    /// Called at the beginning of serialization, before any serializer calls are made.
    fn on_start(&self, start: &mut StartScope) {}

    /// Called after the serialization.
    ///
    /// This hook is called regardless of whether the serialization has succeeded or failed.
    fn on_end(&self) {}

    /// Called before a value is serialized.
    ///
    /// You can use the passed in scope to inspect and modify the value before it's
    /// passed on to the serializer.
    ///
    /// This hook is called for struct field values, map entry values, sequence elements etc.,
    /// but also at the top level serialized value.
    /// Primitive values, like numbers, will have the actual value copied to the scope,
    /// whilst for compound values, like structs, only metadata is available.
    #[allow(unused_variables)]
    fn on_value<S: Serializer>(&self, path: &Path, value: &mut ValueScope<S>) {}

    /// Called before a struct is serialized.
    ///
    /// Using the scope passed in, you can modify the struct by e.g. renaming or skipping
    /// fields.
    ///
    /// This hook will be preceded with a call to [`on_value`](Self::on_value) at the same path.
    #[allow(unused_variables)]
    fn on_struct(&self, path: &Path, st: &mut StructScope) {}

    /// Called before a sequence is serialized.
    ///
    /// Using the scope passed in, you can modify the sequence by e.g. skipping or replacing
    /// elements.
    ///
    /// This hook will be preceded with a call to [`on_value`](Self::on_value) at the same path.
    ///
    /// This hook will also be called for tuples and tuple structs. In this case,
    /// specifying any actions that may change the number of elements in the
    /// sequence (e.g. retaining or skipping elements) will force the tuple to be
    /// serialized as a sequence. Depending on the serializer you use, this might be
    /// totally unsupported or lead to unexpected serialization results.
    #[allow(unused_variables)]
    fn on_seq(&self, path: &Path, seq: &mut SeqScope) {}

    /// Called before a tuple is serialized.
    ///
    /// This hook will be preceded with several other hook calls at the same path,
    /// in the following order:
    /// - [`on_value`](Self::on_value)
    /// - [`on_seq`](Self::on_seq) (passed the same `seq` scope reference)
    ///
    /// Specifying any actions that may change the number of elements in the
    /// sequence (e.g. retaining or skipping elements) will force the tuple to be
    /// serialized as a sequence. Depending on the serializer you use, this might be
    /// totally unsupported or lead to unexpected serialization results.
    #[allow(unused_variables)]
    fn on_tuple(&self, path: &Path, tpl: &mut TupleScope, seq: &mut SeqScope) {}

    /// Called before a tuple struct is serialized.
    ///
    /// This hook will be preceded with several other hook calls at the same path,
    /// in the following order:
    /// - [`on_value`](Self::on_value)
    /// - [`on_seq`](Self::on_seq) (passed the same `seq` scope reference)
    /// - [`on_tuple`](Self::on_tuple) (passed the same `seq` scope reference)
    ///
    /// Specifying any actions that may change the number of elements in the
    /// sequence (e.g. retaining or skipping elements) will force the tuple struct to be
    /// serialized as a sequence. Depending on the serializer you use, this might be
    /// totally unsupported or lead to unexpected serialization results.
    #[allow(unused_variables)]
    fn on_tuple_struct(&self, path: &Path, tpl: &mut TupleStructScope, seq: &mut SeqScope) {}

    /// Called before a map is serialized.
    ///
    /// Using the scope passed in, you can modify the map by e.g. replacing or skipping
    /// map keys and values.
    ///
    /// This hook will be preceded with a call to [`on_value`](Self::on_value) at the same path.
    ///
    /// This hook will be followed by calls to [`on_map_key`](Self::on_map_key) and
    /// [`on_value`](Self::on_value) for each map entry.
    #[allow(unused_variables)]
    fn on_map(&self, path: &Path, map: &mut MapScope) {}

    /// Called before a map key is serialized.
    ///
    /// You can use the passed in scope to inspect and modify the map key before it's
    /// passed on to the serializer.
    #[allow(unused_variables)]
    fn on_map_key<S: Serializer>(&self, path: &Path, map_key: &mut MapKeyScope<S>) {}

    /// Called before an enum variant of any kind is serialized.
    ///
    /// Using the scope passed in, you can modify the variant by e.g. renaming it.
    ///
    /// This hook will be preceded with a call to [`on_value`](Self::on_value) at the same path.
    ///
    /// This hook will be followed by calls to [`on_map_key`](Self::on_map_key) and
    /// [`on_value`](Self::on_value) for each map entry.
    ///
    /// An example of variant kinds in Rust:
    /// ```
    /// enum Enum {
    ///     UnitVariant,
    ///     NewtypeVariant(u8),
    ///     StructVariant { field: i32 },
    ///     TupleVariant(u8, u16),
    /// }
    /// ```
    #[allow(unused_variables)]
    fn on_enum_variant(&self, path: &Path, ev: &mut EnumVariantScope) {}

    /// Called before a struct enum variant is serialized.
    ///
    /// Using the scopes passed in, you can modify the variant by e.g. renaming it; or
    /// modify the struct by e.g. renaming or skipping fields.
    ///
    /// This hook will be preceded with several other hook calls at the same path,
    /// in the following order:
    /// - [`on_value`](Self::on_value)
    /// - [`on_enum_variant`](Self::on_enum_variant) (passed the same `ev` scope reference)
    /// - [`on_struct`](Self::on_struct) (passed the same `st` scope reference)
    ///
    /// An example of a struct variant in Rust:
    /// ```
    /// enum Enum {
    ///     StructVariant { field: i32 },
    /// }
    /// ```
    #[allow(unused_variables)]
    fn on_struct_variant(&self, path: &Path, ev: &mut EnumVariantScope, st: &mut StructScope) {}

    /// Called before a tuple enum variant is serialized.
    ///
    /// Using the scopes passed in, you can modify the variant by e.g. renaming it; or
    /// modify the tuple fields.
    ///
    /// This hook will be preceded with several other hook calls at the same path,
    /// in the following order:
    /// - [`on_value`](Self::on_value)
    /// - [`on_enum_variant`](Self::on_enum_variant) (passed the same `ev` scope reference)
    /// - [`on_seq`](Self::on_seq) (passed the same `seq` scope reference)
    /// - [`on_tuple`](Self::on_tuple) (passed the same `tpl` and `seq` scope references)
    ///
    /// An example of a tuple variant in Rust:
    /// ```
    /// enum Enum {
    ///     TupleVariant(u8, u16),
    /// }
    /// ```
    ///
    /// Specifying any actions that may change the number of elements in the
    /// tuple (e.g. retaining or skipping elements) will force this tuple to be
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

    /// Called when performing an action on a scope results in a recoverable error.
    ///
    /// Calling modification methods on a scope often does not perform an action immediately.
    /// Instead, the action is 'remembered' and performed later on during serialization of
    /// the scope. That action can fail, but the failure can be recoverable. This hook is
    /// called at the moment of failure, and using the passed in `ErrorScope` you can change
    /// the reaction to the failure.
    ///
    /// An example of such action would be trying to rename a structure field that does not exist.
    /// The actual fields that are serialized are only known after the rename action is requested
    /// on the struct scope. When a field, for which a rename was requested, is not found, this
    /// hook will be called, and you can choose to let the error bubble
    /// up and fail serialization, ignore it, or, for example, panic.
    ///
    /// By default, if this hook is not implemented or no action is requested on `err`,
    /// the errors are propagated as custom serialization errors.
    ///
    /// Note, this hook is *not* called when an error is produced by the used serializer.
    /// Serialization errors are not recoverable and are propagated all the way up
    /// to the `serialize` call.
    ///
    /// See [`HooksError`] for the kinds of recoverable errors.
    #[allow(unused_variables)]
    fn on_scope_error(&self, path: &Path, err: &mut ErrorScope) {}
}

/// Kinds of recoverable errors.
///
/// See [`Hooks::on_scope_error`] for more info on handling recoverable errors.
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum HooksError {
    /// Cannot add this key to the map, the key is already present in the map.
    #[error("cannot add key {0}, the key is already present in the map")]
    KeyAlreadyPresent(MapKeySelector),

    /// Cannot add a map entry by specifying an index, please specify key value.
    ///
    /// This error occurs when trying to add a map entry, but specifying an index
    /// instead of an actual key in the map key selector.
    #[error("cannot add entry with an index {0}, please specify key value")]
    CannotAddEntryByIndex(usize),

    /// This key is not found in the map.
    #[error("key {0} not found")]
    KeyNotFound(MapKeySelector),

    /// This field is not found in the struct.
    #[error("field \"{0}\" not found")]
    FieldNotFound(Cow<'static, str>),

    /// This value is not serializable.
    ///
    /// This error occurs when trying to replace a value using compound scope methods
    /// (e.g. on a struct or map scope), and passing in a [`Value`](crate::Value) that is non-primitive.
    /// Non-primitive values, like structs, maps, tuples, are represented by their
    /// metadata, which is obviously not sufficient to serialize them.
    #[error("value is not serializable: {0}")]
    ValueNotSerializable(String),

    /// This index was not found in the sequence.
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
