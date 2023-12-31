use std::borrow::Cow;

use crate::{
    ser::wrapper::{StructActions, StructFieldAction, StructFieldActions},
    Case, StaticValue,
};

/// Inspect structs and modify their contents.
///
/// See [`Hooks::on_struct`](crate::ser::Hooks::on_struct),
/// [`Hooks::on_struct_variant`](crate::ser::Hooks::on_struct_variant).
pub struct StructScope {
    struct_len: usize,
    struct_name: &'static str,
    struct_actions: StructActions,
    field_actions: StructFieldActions,
}

impl StructScope {
    pub(crate) fn new(struct_len: usize, struct_name: &'static str) -> Self {
        Self {
            struct_len,
            struct_name,
            field_actions: Default::default(),
            struct_actions: StructActions {
                serialize_as_map: false,
            },
        }
    }

    pub(crate) fn into_actions(self) -> (StructActions, StructFieldActions) {
        (self.struct_actions, self.field_actions)
    }

    /// Returns the original number of fields in this struct.
    ///
    /// The returned value is not affected by any retain or skip actions.
    pub fn struct_len(&self) -> usize {
        self.struct_len
    }

    /// Returns the name of the struct.
    pub fn struct_name(&self) -> &'static str {
        self.struct_name
    }

    /// Skips a field during serialization.
    ///
    /// Runtime equivalent to `#[serde(skip)]` or `#[serde(skip_serializing)]`.
    ///
    /// If the field is not found in the struct, [`HooksError::FieldNotFound`](crate::ser::HooksError::FieldNotFound)
    /// is produced _after_ the struct is serialized. You can process or ignore this error in
    /// [`Hooks::on_scope_error`](crate::ser::Hooks::on_scope_error).
    ///
    /// Returns `self` to allow chaining calls.
    pub fn skip_field(&mut self, key: impl Into<Cow<'static, str>>) -> &mut Self {
        self.field_actions.push(StructFieldAction::Skip(key.into()));
        self
    }

    /// Retains a field.
    ///
    /// Calling this method switches processing to a 'retain' mode, in which
    /// all not retained fields are skipped. You can retain multiple fields by
    /// calling this method multiple times.
    ///
    /// There is no equivalent in serde derive, but you can see this as a 'whitelist'
    /// counterpart of `#[serde(skip)]`.
    ///
    /// If the field is not found in the struct, [`HooksError::FieldNotFound`](crate::ser::HooksError::FieldNotFound)
    /// is produced _after_ the struct is serialized. You can process or ignore this error in
    /// [`Hooks::on_scope_error`](crate::ser::Hooks::on_scope_error).
    ///
    /// Returns `self` to allow chaining calls.
    pub fn retain_field(&mut self, key: impl Into<Cow<'static, str>>) -> &mut Self {
        self.field_actions
            .push(StructFieldAction::Retain(key.into()));
        self
    }

    /// Rename a field.
    ///
    /// The `key` refers to the original field key in the struct, even if [`rename_all_fields_case`](Self::rename_all_fields_case)
    /// is called.
    ///
    /// If you use serde's `#[derive(Serialize)]` and `#[serde(rename=...)]` or
    /// `#[serde(rename_all=...)]`, you need to specify the field key as it will be *after* serde renaming.
    ///
    /// If the field is not found in the struct, [`HooksError::FieldNotFound`](crate::ser::HooksError::FieldNotFound)
    /// is produced _after_ the struct is serialized. You can process or ignore this error in
    /// [`Hooks::on_scope_error`](crate::ser::Hooks::on_scope_error).
    ///
    /// Serde expects field names to be known at compile time, and as such, to be static. Passing in a
    /// borrowed `&'static str` for the new key name here fulfills this. However, passing in
    /// an owned `String` leads to special handling described in [Static strings](crate::ser#static-strings).
    ///
    /// Returns `self` to allow chaining calls.
    pub fn rename_field(
        &mut self,
        key: impl Into<Cow<'static, str>>,
        new_key: impl Into<Cow<'static, str>>,
    ) -> &mut Self {
        self.field_actions
            .push(StructFieldAction::Rename(key.into(), new_key.into()));
        self
    }

    /// Rename a field according to the given case convention.
    ///
    /// The `key` refers to the original field key in the struct, even if [`rename_all_fields_case`](Self::rename_all_fields_case)
    /// is called.
    ///
    /// If you use serde's `#[derive(Serialize)]` and `#[serde(rename=...)]` or
    /// `#[serde(rename_all=...)]`, you need to specify the field key as it will be *after* serde renaming.
    ///
    /// If the field is not found in the struct, [`HooksError::FieldNotFound`](crate::ser::HooksError::FieldNotFound)
    /// is produced _after_ the struct is serialized. You can process or ignore this error in
    /// [`Hooks::on_scope_error`](crate::ser::Hooks::on_scope_error).
    ///
    /// The renaming will happen at runtime, which would (most likely) lead to an allocation of a new
    /// String. It is thus more optimal to pass a static string literal into [`rename_field`](Self::rename_field) instead.
    /// See also [Static strings](crate::ser#static-strings) for more info on special handing of strings in serde.
    ///
    /// Returns `self` to allow chaining calls.
    pub fn rename_field_case(
        &mut self,
        key: impl Into<Cow<'static, str>>,
        case: impl Into<Case>,
    ) -> &mut Self {
        let key = key.into();
        let new_key = Case::cow_to_case(&key, case.into());
        self.field_actions
            .push(StructFieldAction::Rename(key, new_key));
        self
    }

    /// Rename all structure fields according to the given case convention.
    ///
    /// If specified multiple times, the last case convention is used.
    ///
    /// Calling [`rename_field`](Self::rename_field) on specific fields will override
    /// this case convention.
    ///
    /// If you use serde's `#[derive(Serialize)]` and `#[serde(rename=...)]` or
    /// `#[serde(rename_all=...)]`, those renames will be applied first. See [`Case`](crate::Case) for more information
    /// and caveats of case conversion.
    ///
    /// Serde expects field names to be known at compile time, and as such, to be static.
    /// Renaming field names with a case convention will produce strings in runtime,
    /// which leads to special handling described in [Static strings](crate::ser#static-strings).
    ///
    /// Returns `self` to allow chaining calls.
    pub fn rename_all_fields_case(&mut self, case: impl Into<Case>) -> &mut Self {
        self.field_actions
            .push(StructFieldAction::RenameAllCase(case.into()));
        self
    }

    /// Replace a value for a field.
    ///
    /// The passed in [`StaticValue`] can represent both primitive and compound value types.
    ///
    /// Primitive values are copied, and are later fed to the serializer instead of the original
    /// struct field values.
    /// For compound values, only metadata is stored, therefore it's not possible to
    /// serialize the actual values from the contents of [`StaticValue`]. Passing in a
    /// compound value here would result in an
    /// [`HooksError::ValueNotSerializable`](crate::ser::HooksError::ValueNotSerializable) error.
    /// The trick to replace a compound value is to replace it in this scope with a primitive one
    /// (e.g. a unit), subscribe to `on_value` hook, and replace the value there again with the
    /// compound one.
    ///
    /// The replacement value does not necessarily need to be of the same type as the
    /// original value in the struct. E.g., you can replace an integer field with a string one.
    /// Some serializers might expect the types to be following a schema, and fail the serialization
    /// if the replacement value is of a wrong type.
    ///
    /// If the field is not found in the struct, [`HooksError::FieldNotFound`](crate::ser::HooksError::FieldNotFound)
    /// is produced _after_ the struct is serialized. You can process or ignore this error in
    /// [`Hooks::on_scope_error`](crate::ser::Hooks::on_scope_error).
    ///
    /// Returns `self` to allow chaining calls.
    pub fn replace_value(
        &mut self,
        key: impl Into<Cow<'static, str>>,
        new_value: impl Into<StaticValue>,
    ) -> &mut Self {
        self.field_actions.push(StructFieldAction::ReplaceValue(
            key.into(),
            new_value.into(),
        ));
        self
    }

    /// Serialize this struct as a map.
    ///
    /// Calling this method makes the struct to be fed to the serializer as a map
    /// with string keys.
    ///
    /// Some serializers, e.g. `serde_json`, do not distinguish between maps
    /// and structs and represent them the same way in the serialized output.
    /// Others however, like `ron`, do represent structs and maps differently.
    /// Some might not support maps where they expect structs.
    ///
    /// You will receive an [`on_map`](crate::ser::Hooks::on_map) hook callback
    /// at the same path before this struct is serialized as a map.
    ///
    /// If you apply modifying actions to both the struct scope here, and to map scope
    /// in `on_map` hook, the actions applied to the struct scope will have precedence.
    /// It generally would lead to confusing effects and is not recommended. Pick one.
    ///
    /// Returns `self` to allow chaining calls.
    pub fn serialize_as_map(&mut self) -> &mut Self {
        self.struct_actions.serialize_as_map = true;
        self
    }

    /// Flatten a field into this structure.
    ///
    /// Runtime equivalent to `#[serde(flatten)]`.
    ///
    /// The field value must be of struct, struct variant or map type, otherwise
    /// [`CannotFlattenUnsupportedDataType`](crate::ser::HooksError::CannotFlattenUnsupportedDataType)
    /// is produced. You can process or ignore this error in
    /// [`Hooks::on_scope_error`](crate::ser::Hooks::on_scope_error).
    ///
    /// This removes one level of hierarchy for the field, adding the struct fields
    /// (map entries) from the field value straight into this struct.
    ///
    /// Flattening any field causes this struct to be serialized as a map with
    /// no length hint to the serializer. Some serializers do not support this.
    /// See [`serialize_as_map`](Self::serialize_as_map) for more details and implications.
    ///
    /// If the field is not found in the struct, [`HooksError::FieldNotFound`](crate::ser::HooksError::FieldNotFound)
    /// is produced _after_ the struct is serialized. You can process or ignore this error in
    /// [`Hooks::on_scope_error`](crate::ser::Hooks::on_scope_error).
    ///
    /// Returns `self` to allow chaining calls.
    pub fn flatten_field(&mut self, key: impl Into<Cow<'static, str>>) -> &mut Self {
        self.field_actions
            .push(StructFieldAction::Flatten(key.into()));
        self
    }
}
