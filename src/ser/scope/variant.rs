use std::borrow::Cow;

use smallvec::SmallVec;

use crate::{Case, Path};

pub(crate) enum VariantAction {
    RenameEnumCase(Case),
    RenameEnum(Cow<'static, str>),
    RenameVariantCase(Case),
    RenameVariant(Cow<'static, str>),
    ChangeVariantIndex(u32),
}

pub(crate) type OnVariantActions = SmallVec<[VariantAction; 8]>;

pub struct EnumVariantScope<'p> {
    path: &'p Path,
    enum_name: &'static str,
    variant_name: &'static str,
    variant_index: u32,
    actions: OnVariantActions,
}

impl<'p> EnumVariantScope<'p> {
    pub(crate) fn new(
        path: &'p Path,
        enum_name: &'static str,
        variant_name: &'static str,
        variant_index: u32,
    ) -> Self {
        Self {
            path,
            enum_name,
            variant_name,
            variant_index,
            actions: Default::default(),
        }
    }

    pub(crate) fn into_actions(self) -> OnVariantActions {
        self.actions
    }

    pub fn path(&self) -> &Path {
        self.path
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
