use serde::{Serialize, Serializer};

mod map;
mod seq;
mod serializer;
mod r#struct;

use super::scope::{
    OnMapEntryActions, OnSeqElementActions, OnStructFieldActions, OnValueAction, OnVariantActions,
};
use super::HooksError;
use crate::path::PathSegment;

pub(crate) use serializer::SerializerWrapper;

pub(crate) trait SerializerWrapperHooks {
    fn path_push(&self, segment: PathSegment);

    fn path_pop(&self);

    fn on_error<S: Serializer>(&self, error: HooksError) -> Result<(), S::Error>;

    fn on_map(&self, map_len: Option<usize>) -> OnMapEntryActions;

    fn on_unit_variant(
        &self,
        enum_name: &'static str,
        variant_name: &'static str,
        variant_index: u32,
    ) -> OnVariantActions;

    fn on_newtype_variant(
        &self,
        enum_name: &'static str,
        variant_name: &'static str,
        variant_index: u32,
    ) -> OnVariantActions;

    fn on_struct(&self, struct_len: usize, struct_name: &'static str) -> OnStructFieldActions;

    fn on_struct_variant(
        &self,
        struct_len: usize,
        enum_name: &'static str,
        variant_name: &'static str,
        variant_index: u32,
    ) -> (OnVariantActions, OnStructFieldActions);

    fn on_map_key<S: Serializer>(&self, serializer: S, key: crate::Value) -> OnValueAction<S>;

    fn on_value<S: Serializer>(&self, serializer: S, value: crate::Value) -> OnValueAction<S>;

    fn on_seq(&self, len: Option<usize>) -> OnSeqElementActions;
}

#[derive(Debug, Copy, Clone)]
pub enum SerializableKind {
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
