use std::borrow::Cow;

use crate::{
    ser::wrapper::{VariantAction, VariantActions},
    Case,
};

/// Inspect and modify enum variants.
///
/// See [`Hooks::on_enum_variant`](crate::ser::Hooks::on_enum_variant),
/// [`Hooks::on_struct_variant`](crate::ser::Hooks::on_struct_variant),
/// [`Hooks::on_tuple_variant`](crate::ser::Hooks::on_tuple_variant).
pub struct EnumVariantScope {
    enum_name: &'static str,
    variant_name: &'static str,
    variant_index: u32,
    actions: VariantActions,
}

impl EnumVariantScope {
    pub(crate) fn new(
        enum_name: &'static str,
        variant_name: &'static str,
        variant_index: u32,
    ) -> Self {
        Self {
            enum_name,
            variant_name,
            variant_index,
            actions: Default::default(),
        }
    }

    pub(crate) fn into_actions(self) -> VariantActions {
        self.actions
    }

    /// Returns enum name.
    ///
    /// For example, this would be `"E"` in `enum E { A, B }`.
    pub fn enum_name(&self) -> &'static str {
        self.enum_name
    }

    /// Returns variant index in the enum.
    ///
    /// For example, this would be `0` for `E::A`, `1` for `E::B` in `enum E { A, B }`.
    ///
    /// Note that this is always a sequential index, not the numeric value assigned to the
    /// variant.
    pub fn variant_index(&self) -> u32 {
        self.variant_index
    }

    /// Variant name.
    ///
    /// For example, this would be `"A"` for `E::A` in `enum E { A, B }`.
    pub fn variant_name(&self) -> &'static str {
        self.variant_name
    }

    /// Set a new enum name.
    ///
    /// This effectively changes the enum type that is fed into your serializer
    /// and might cause issues if your serializer expect a particular type in the schema.
    ///
    /// Serializers for popular data formats (e.g. `serde_json`) often disregard the enum name
    /// altogether, in which case changing the name has no effect.
    ///
    /// Serde expects enum names to be known at compile time, and as such, to be static. Passing in a
    /// borrowed `&'static str` for the new name here fulfills this. However, passing in
    /// an owned `String` leads to special handling described in [Static strings](crate::ser#static-strings).
    ///
    /// Returns `self` to allow chaining calls.
    pub fn rename_enum(&mut self, new_enum_name: impl Into<Cow<'static, str>>) -> &mut Self {
        self.actions
            .push(VariantAction::RenameEnum(new_enum_name.into()));
        self
    }

    /// Rename enum name according to the given case convention.
    ///
    /// This effectively changes the enum type that is fed into your serializer
    /// and might cause issues if your serializer expect a particular type in the schema.
    ///
    /// Serializers for popular data formats (e.g. `serde_json`) often disregard the enum name
    /// altogether, in which case changing the name has no effect.
    ///
    /// Serde expects enum names to be known at compile time, and as such, to be static.
    /// Renaming enums with a case convention will produce strings in runtime,
    /// which leads to special handling described in [Static strings](crate::ser#static-strings).
    ///
    /// Returns `self` to allow chaining calls.
    pub fn rename_enum_case(&mut self, new_enum_case: impl Into<Case>) -> &mut Self {
        self.actions
            .push(VariantAction::RenameEnumCase(new_enum_case.into()));
        self
    }

    /// Set a new variant name.
    ///
    /// Serde expects variant names to be known at compile time, and as such, to be static. Passing in a
    /// borrowed `&'static str` for the new name here fulfills this. However, passing in
    /// an owned `String` leads to special handling described in [Static strings](crate::ser#static-strings).
    ///
    /// Returns `self` to allow chaining calls.
    pub fn rename_variant(&mut self, new_variant_name: impl Into<Cow<'static, str>>) -> &mut Self {
        self.actions
            .push(VariantAction::RenameVariant(new_variant_name.into()));
        self
    }

    /// Rename variant name according to the given case convention.
    ///
    /// Serde expects enum variant names to be known at compile time, and as such, to be static.
    /// Renaming variants with a case convention will produce strings in runtime,
    /// which leads to special handling described in [Static strings](crate::ser#static-strings).
    ///
    /// Returns `self` to allow chaining calls.
    pub fn rename_variant_case(&mut self, new_variant_case: impl Into<Case>) -> &mut Self {
        self.actions
            .push(VariantAction::RenameVariantCase(new_variant_case.into()));
        self
    }

    /// Set a new variant index.
    ///
    /// Returns `self` to allow chaining calls.
    pub fn change_variant_index(&mut self, new_variant_index: u32) -> &mut Self {
        self.actions
            .push(VariantAction::ChangeVariantIndex(new_variant_index));
        self
    }
}
