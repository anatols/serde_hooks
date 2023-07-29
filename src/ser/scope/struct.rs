use std::borrow::Cow;

use smallvec::SmallVec;

use crate::{Case, Path, StaticValue};

pub(crate) enum StructFieldAction {
    Retain(Cow<'static, str>),
    Skip(Cow<'static, str>),
    Rename(Cow<'static, str>, Cow<'static, str>),
    ReplaceValue(Cow<'static, str>, StaticValue),
    RenameAll(Case),
}

pub(crate) type OnStructFieldActions = SmallVec<[StructFieldAction; 8]>;

pub struct StructScope<'p> {
    path: &'p Path,
    struct_len: usize,
    struct_name: &'static str,
    actions: OnStructFieldActions,
}

impl<'p> StructScope<'p> {
    pub(crate) fn new(path: &'p Path, struct_len: usize, struct_name: &'static str) -> Self {
        Self {
            path,
            struct_len,
            struct_name,
            actions: Default::default(),
        }
    }

    pub(crate) fn into_actions(self) -> OnStructFieldActions {
        self.actions
    }

    pub fn path(&self) -> &Path {
        self.path
    }

    pub fn struct_len(&self) -> usize {
        self.struct_len
    }

    pub fn struct_name(&self) -> &'static str {
        self.struct_name
    }

    pub fn retain_field(&mut self, key: impl Into<Cow<'static, str>>) -> &mut Self {
        self.actions.push(StructFieldAction::Retain(key.into()));
        self
    }

    pub fn skip_field(&mut self, key: impl Into<Cow<'static, str>>) -> &mut Self {
        self.actions.push(StructFieldAction::Skip(key.into()));
        self
    }

    //TODO better docs - explain about static strings
    /// Rename a field.
    ///
    /// The `key` refers to the original field key in the struct, even if [rename_all](StructManipulation::rename_all)
    /// is called.
    ///
    /// If you use serde's `#[derive(Serialize)]` and `#[serde(rename=...)]` or
    /// `#[serde(rename_all=...)]`, you need to specify the field key as it will be *after* serde renaming.
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
    /// Calling [rename_field](StructManipulation::rename_field) on specific fields will override
    /// this case convention.
    ///
    /// If you use serde's `#[derive(Serialize)]` and `#[serde(rename=...)]` or
    /// `#[serde(rename_all=...)]`, those renames will be applied first. See [Case](crate::Case) for more information
    /// and caveats of case conversion.
    pub fn rename_all_fields(&mut self, case: Case) -> &mut Self {
        self.actions.push(StructFieldAction::RenameAll(case));
        self
    }

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
