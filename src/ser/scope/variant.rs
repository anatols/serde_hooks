use std::borrow::Cow;

use crate::{
    ser::wrapper::{VariantAction, VariantActions},
    Case,
};

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

    pub fn enum_name(&self) -> &'static str {
        self.enum_name
    }

    pub fn variant_name(&self) -> &'static str {
        self.variant_name
    }

    pub fn variant_index(&self) -> u32 {
        self.variant_index
    }

    pub fn rename_enum(&mut self, new_enum_name: impl Into<Cow<'static, str>>) -> &mut Self {
        self.actions
            .push(VariantAction::RenameEnum(new_enum_name.into()));
        self
    }

    pub fn rename_enum_case(&mut self, new_enum_case: Case) -> &mut Self {
        self.actions
            .push(VariantAction::RenameEnumCase(new_enum_case));
        self
    }

    pub fn rename_variant(&mut self, new_variant_name: impl Into<Cow<'static, str>>) -> &mut Self {
        self.actions
            .push(VariantAction::RenameVariant(new_variant_name.into()));
        self
    }

    pub fn rename_variant_case(&mut self, new_variant_case: Case) -> &mut Self {
        self.actions
            .push(VariantAction::RenameVariantCase(new_variant_case));
        self
    }

    pub fn change_variant_index(&mut self, new_variant_index: u32) -> &mut Self {
        self.actions
            .push(VariantAction::ChangeVariantIndex(new_variant_index));
        self
    }
}
