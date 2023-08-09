use std::borrow::Cow;

use serde::{Serialize, Serializer};
use smallvec::SmallVec;

mod map;
mod seq;
mod serializer;
mod r#struct;

use super::{HooksError, MapKeySelector};
use crate::{path::PathSegment, Case, StaticValue};

pub(crate) use serializer::SerializerWrapper;

pub(crate) trait SerializerWrapperHooks {
    fn path_push(&self, segment: PathSegment);

    fn path_pop(&self);

    fn on_error<S: Serializer>(&self, error: HooksError) -> Result<(), S::Error>;

    fn on_map(&self, map_len: Option<usize>) -> MapEntryActions;

    fn on_unit_variant(
        &self,
        enum_name: &'static str,
        variant_name: &'static str,
        variant_index: u32,
    ) -> VariantActions;

    fn on_newtype_variant(
        &self,
        enum_name: &'static str,
        variant_name: &'static str,
        variant_index: u32,
    ) -> VariantActions;

    fn on_struct(&self, struct_len: usize, struct_name: &'static str) -> StructFieldActions;

    fn on_struct_variant(
        &self,
        struct_len: usize,
        enum_name: &'static str,
        variant_name: &'static str,
        variant_index: u32,
    ) -> (VariantActions, StructFieldActions);

    fn on_map_key<S: Serializer>(&self, serializer: S, key: crate::Value) -> ValueAction<S>;

    fn on_value<S: Serializer>(&self, serializer: S, value: crate::Value) -> ValueAction<S>;

    fn on_seq(&self, len: Option<usize>) -> SeqElementActions;

    fn on_tuple(&self, len: usize) -> SeqElementActions;

    fn on_tuple_struct(&self, name: &'static str, len: usize) -> SeqElementActions;

    fn on_tuple_variant(
        &self,
        enum_name: &'static str,
        variant_index: u32,
        variant_name: &'static str,
        len: usize,
    ) -> (VariantActions, SeqElementActions);
}

pub(crate) enum StructFieldAction {
    Retain(Cow<'static, str>),
    Skip(Cow<'static, str>),
    Rename(Cow<'static, str>, Cow<'static, str>),
    ReplaceValue(Cow<'static, str>, StaticValue),
    RenameAll(Case),
}

pub(crate) type StructFieldActions = SmallVec<[StructFieldAction; 8]>;

pub(crate) enum MapEntryAction {
    Retain(MapKeySelector),
    Skip(MapKeySelector),
    Add(MapKeySelector, Option<StaticValue>),
    Replace(MapKeySelector, Option<StaticValue>),
    ReplaceOrAdd(MapKeySelector, Option<StaticValue>),
    ReplaceKey(MapKeySelector, StaticValue),
}

pub(crate) type MapEntryActions = SmallVec<[MapEntryAction; 8]>;

pub(crate) enum SeqElementAction {
    Retain(usize),
    Skip(usize),
    ReplaceValue(usize, StaticValue),
}

pub(crate) type SeqElementActions = SmallVec<[SeqElementAction; 8]>;

pub(crate) enum ValueAction<S: Serializer> {
    ContinueSerialization(S),
    ValueReplaced(Result<S::Ok, S::Error>),
}

pub(crate) enum VariantAction {
    RenameEnumCase(Case),
    RenameEnum(Cow<'static, str>),
    RenameVariantCase(Case),
    RenameVariant(Cow<'static, str>),
    ChangeVariantIndex(u32),
}

pub(crate) type VariantActions = SmallVec<[VariantAction; 8]>;

#[derive(Copy, Clone)]
pub(crate) enum SerializableKind {
    Value,
    MapKey,
}

pub(crate) struct SerializableWithHooks<'s, 'h, T: Serialize + ?Sized, H: SerializerWrapperHooks> {
    serializable: &'s T,
    hooks: &'h H,
    kind: SerializableKind,
}

impl<'s, 'h, T: Serialize + ?Sized, H: SerializerWrapperHooks> SerializableWithHooks<'s, 'h, T, H> {
    pub(crate) fn new(serializable: &'s T, hooks: &'h H, kind: SerializableKind) -> Self {
        Self {
            serializable,
            hooks,
            kind,
        }
    }
}

impl<T: Serialize + ?Sized, H: SerializerWrapperHooks> Serialize
    for SerializableWithHooks<'_, '_, T, H>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.serializable
            .serialize(SerializerWrapper::new(serializer, self.hooks, self.kind))
    }
}
