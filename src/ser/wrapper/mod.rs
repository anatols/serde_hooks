use serde::{Serialize, Serializer};

mod map_wrapper;
mod struct_wrapper;

use crate::path::PathSegment;

use map_wrapper::SerializeMapWrapper;
use struct_wrapper::SerializeStructWrapper;

use super::HooksError;

use super::scope::{OnMapEntryActions, OnStructFieldActions, OnValueAction};

pub(crate) trait SerializerWrapperHooks {
    fn path_push(&self, segment: PathSegment);

    fn path_pop(&self);

    fn on_error<S: Serializer>(&self, error: HooksError) -> Result<(), S::Error>;

    fn on_map(&self, map_len: Option<usize>) -> OnMapEntryActions;

    fn on_struct(&self, struct_len: usize, struct_name: &'static str) -> OnStructFieldActions;

    fn on_map_key<S: Serializer>(&self, serializer: S, key: crate::Value) -> OnValueAction<S>;

    fn on_value<S: Serializer>(&self, serializer: S, value: crate::Value) -> OnValueAction<S>;
}

pub(super) struct SerializerWrapper<'h, S, H: SerializerWrapperHooks> {
    serializer: S,
    hooks: &'h H,
    kind: SerializableKind,
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> SerializerWrapper<'h, S, H> {
    pub(super) fn new(serializer: S, hooks: &'h H, kind: SerializableKind) -> Self {
        Self {
            serializer,
            hooks,
            kind,
        }
    }
}

macro_rules! wrap_primitive_serialize {
    ($fn:ident, $type:ty) => {
        fn $fn(self, v: $type) -> Result<Self::Ok, Self::Error> {
            let value_action = match self.kind {
                SerializableKind::Value => self.hooks.on_value(
                    self.serializer,
                    crate::Value::Primitive(v.to_owned().into()),
                ),
                SerializableKind::MapKey => self.hooks.on_map_key(
                    self.serializer,
                    crate::Value::Primitive(v.to_owned().into()),
                ),
            };

            match value_action {
                OnValueAction::ContinueSerialization(s) => s.$fn(v),
                OnValueAction::ValueReplaced(r) => r,
            }
        }
    };
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> Serializer for SerializerWrapper<'h, S, H> {
    type Ok = S::Ok;
    type Error = S::Error;
    type SerializeSeq = S::SerializeSeq;
    type SerializeTuple = S::SerializeTuple;
    type SerializeTupleStruct = S::SerializeTupleStruct;
    type SerializeTupleVariant = S::SerializeTupleVariant;
    type SerializeMap = SerializeMapWrapper<'h, S, H>;
    type SerializeStruct = SerializeStructWrapper<'h, S, H>;
    type SerializeStructVariant = S::SerializeStructVariant;

    wrap_primitive_serialize!(serialize_bool, bool);
    wrap_primitive_serialize!(serialize_i8, i8);
    wrap_primitive_serialize!(serialize_i16, i16);
    wrap_primitive_serialize!(serialize_i32, i32);
    wrap_primitive_serialize!(serialize_i64, i64);
    wrap_primitive_serialize!(serialize_u8, u8);
    wrap_primitive_serialize!(serialize_u16, u16);
    wrap_primitive_serialize!(serialize_u32, u32);
    wrap_primitive_serialize!(serialize_u64, u64);
    wrap_primitive_serialize!(serialize_f32, f32);
    wrap_primitive_serialize!(serialize_f64, f64);
    wrap_primitive_serialize!(serialize_char, char);
    wrap_primitive_serialize!(serialize_str, &str);

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_bytes(v)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_none()
    }

    fn serialize_some<T: ?Sized>(self, v: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.serializer.serialize_some(v)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_unit()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_unit_struct(name)
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serializer
            .serialize_unit_variant(name, variant_index, variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.serializer.serialize_newtype_struct(name, value)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.serializer
            .serialize_newtype_variant(name, variant_index, variant, value)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.serializer.serialize_seq(len)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serializer.serialize_tuple(len)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serializer.serialize_tuple_struct(name, len)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serializer
            .serialize_tuple_variant(name, variant_index, variant, len)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        println!("serialize_map {len:?}");

        let actions = self.hooks.on_map(len);
        self.serializer
            .serialize_map(if actions.is_empty() { len } else { None })
            .map(|serialize_map| SerializeMapWrapper::new(serialize_map, self.hooks, actions))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        println!("serialize_struct {name} {len}");

        let actions = self.hooks.on_struct(len, name);
        self.serializer
            .serialize_struct(name, len)
            .map(|serialize_struct| {
                SerializeStructWrapper::new(serialize_struct, self.hooks, actions)
            })
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serializer
            .serialize_struct_variant(name, variant_index, variant, len)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SerializableKind {
    Value,
    MapKey,
}

//TODO give this thing a constructor (or remove constructors from oher internal structs?)
pub(crate) struct SerializableWithHooks<'s, 'h, T: Serialize + ?Sized, H: SerializerWrapperHooks> {
    serializable: &'s T,
    hooks: &'h H,
    kind: SerializableKind,
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
