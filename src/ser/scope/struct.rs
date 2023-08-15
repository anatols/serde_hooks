use std::borrow::Cow;

use crate::{
    ser::wrapper::{StructFieldAction, StructFieldActions},
    Case, StaticValue,
};

//TODO add support for flatten and serialize_as_map
//TODO add rename_field_case
//TODO document errors

/// Inspects structs and modify their contents.
///
/// See [`Hooks::on_struct`](crate::ser::Hooks::on_struct),
/// [`Hooks::on_struct_variant`](crate::ser::Hooks::on_struct_variant).
pub struct StructScope {
    struct_len: usize,
    struct_name: &'static str,
    actions: StructFieldActions,
}

impl StructScope {
    pub(crate) fn new(struct_len: usize, struct_name: &'static str) -> Self {
        Self {
            struct_len,
            struct_name,
            actions: Default::default(),
        }
    }

    pub(crate) fn into_actions(self) -> StructFieldActions {
        self.actions
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
    /// Returns `self` to allow chaining calls.
    pub fn skip_field(&mut self, key: impl Into<Cow<'static, str>>) -> &mut Self {
        self.actions.push(StructFieldAction::Skip(key.into()));
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
    /// Returns `self` to allow chaining calls.
    pub fn retain_field(&mut self, key: impl Into<Cow<'static, str>>) -> &mut Self {
        self.actions.push(StructFieldAction::Retain(key.into()));
        self
    }

    //TODO better docs - explain about static strings
    /// Rename a field.
    ///
    /// The `key` refers to the original field key in the struct, even if [`rename_all_fields_case`](Self::rename_all_fields_case)
    /// is called.
    ///
    /// If you use serde's `#[derive(Serialize)]` and `#[serde(rename=...)]` or
    /// `#[serde(rename_all=...)]`, you need to specify the field key as it will be *after* serde renaming.
    ///
    /// Returns `self` to allow chaining calls.
    pub fn rename_field(
        &mut self,
        key: impl Into<Cow<'static, str>>,
        new_key: impl Into<Cow<'static, str>>,
    ) -> &mut Self {
        self.actions
            .push(StructFieldAction::Rename(key.into(), new_key.into()));
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
    /// Returns `self` to allow chaining calls.
    pub fn rename_all_fields_case(&mut self, case: impl Into<Case>) -> &mut Self {
        self.actions.push(StructFieldAction::RenameAll(case.into()));
        self
    }

    //TODO document, explain replacing non-trivial values
    ///
    ///
    /// Returns `self` to allow chaining calls.
    pub fn replace_value(
        &mut self,
        key: impl Into<Cow<'static, str>>,
        new_value: impl Into<StaticValue>,
    ) -> &mut Self {
        self.actions.push(StructFieldAction::ReplaceValue(
            key.into(),
            new_value.into(),
        ));
        self
    }
}
