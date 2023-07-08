use std::{cell::Cell, fmt::Display};

use serde::{ser::Impossible, Serialize, Serializer};
use thiserror::Error;

use super::{SerializableWithHooks, SerializerWrapperHooks};
use crate::ser::{hooks::MapAction, path::MapKey};

pub struct SerializeMapWrapper<'h, S: Serializer, H: SerializerWrapperHooks> {
    serialize_map: S::SerializeMap,
    hooks: &'h H,
    actions: Vec<MapAction>,
    entry_index: Cell<usize>,
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> SerializeMapWrapper<'h, S, H> {
    pub(super) fn new(
        serialize_map: S::SerializeMap,
        hooks: &'h H,
        actions: Vec<MapAction>,
    ) -> Self {
        Self {
            serialize_map,
            hooks,
            actions,
            entry_index: Cell::new(0),
        }
    }
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> serde::ser::SerializeMap
    for SerializeMapWrapper<'h, S, H>
{
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        println!("serialize_key");
        self.serialize_map.serialize_key(key)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        println!("serialize_value");
        self.serialize_map.serialize_value(value)
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: Serialize,
        V: Serialize,
    {
        println!("serialize_entry");
        let map_key = match key.serialize(MapKeyCapture {
            entry_index: self.entry_index.get(),
        }) {
            Ok(map_key) => map_key,
            Err(MapKeyCaptureError(map_key)) => map_key,
        };

        //TODO some hashmap for keys would be nice
        let action = self.actions.iter().find(|a| {
            //TODO map key selection boilerplate
            match a {
                MapAction::SkipKey(_k) => false,
            }
        });

        self.entry_index.replace(self.entry_index.get() + 1);

        self.hooks.path_push(map_key.into());

        let res = match action {
            None => self.serialize_map.serialize_entry(
                key,
                &SerializableWithHooks {
                    serializable: value,
                    hooks: self.hooks,
                },
            ),
            Some(MapAction::SkipKey(_k)) => {
                //TODO pop path
                //TODO is this all?
                return Ok(());
            }
        };

        self.hooks.path_pop();
        res
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_map.end()
    }
}

#[derive(Debug, Error)]
#[error("")]
struct MapKeyCaptureError(MapKey);

impl serde::ser::Error for MapKeyCaptureError {
    fn custom<T>(_msg: T) -> Self
    where
        T: Display,
    {
        unimplemented!()
    }
}

struct MapKeyCapture {
    entry_index: usize,
}

impl Serializer for MapKeyCapture {
    type Ok = MapKey;
    type Error = MapKeyCaptureError;
    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(MapKey::Bool(self.entry_index, v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(MapKey::I8(self.entry_index, v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(MapKey::I16(self.entry_index, v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(MapKey::I32(self.entry_index, v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(MapKey::I64(self.entry_index, v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(MapKey::U8(self.entry_index, v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(MapKey::U16(self.entry_index, v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(MapKey::U32(self.entry_index, v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(MapKey::U64(self.entry_index, v))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(MapKey::F32(self.entry_index, v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(MapKey::F64(self.entry_index, v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(MapKey::Char(self.entry_index, v))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(MapKey::Str(self.entry_index, v.to_string()))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(MapKey::Bytes(self.entry_index))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(MapKey::None(self.entry_index))
    }

    fn serialize_some<T: ?Sized>(self, v: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        v.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(MapKey::Unit(self.entry_index))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(MapKey::UnitStruct(self.entry_index))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(MapKey::UnitVariant(self.entry_index))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Ok(MapKey::NewtypeStruct(self.entry_index))
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Ok(MapKey::NewtypeVariant(self.entry_index))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(MapKeyCaptureError(MapKey::Seq(self.entry_index)))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(MapKeyCaptureError(MapKey::Tuple(self.entry_index)))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(MapKeyCaptureError(MapKey::TupleStruct(self.entry_index)))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(MapKeyCaptureError(MapKey::TupleVariant(self.entry_index)))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(MapKeyCaptureError(MapKey::Map(self.entry_index)))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(MapKeyCaptureError(MapKey::Struct(self.entry_index)))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(MapKeyCaptureError(MapKey::StructVariant(self.entry_index)))
    }
}
