use std::borrow::Cow;

use smallvec::SmallVec;

use crate::{Path, PrimitiveValue};

#[derive(Debug)]
pub(crate) enum StructFieldAction {
    Retain(Cow<'static, str>),
    Skip(Cow<'static, str>),
    Rename(Cow<'static, str>, Cow<'static, str>),
    ReplaceValue(Cow<'static, str>, PrimitiveValue),
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

    pub fn rename_field(
        &mut self,
        key: impl Into<Cow<'static, str>>,
        new_key: impl Into<Cow<'static, str>>,
    ) -> &mut Self {
        self.actions
            .push(StructFieldAction::Rename(key.into(), new_key.into()));
        self
    }

    pub fn replace_value(
        &mut self,
        key: impl Into<Cow<'static, str>>,
        new_value: impl Into<PrimitiveValue>,
    ) -> &mut Self {
        self.actions.push(StructFieldAction::ReplaceValue(
            key.into(),
            new_value.into(),
        ));
        self
    }
}
