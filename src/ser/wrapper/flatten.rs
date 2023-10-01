use std::fmt::Display;

use serde::{
    ser::{Impossible, SerializeMap},
    Serializer,
};

#[derive(Debug, thiserror::Error)]
pub(super) enum FlattenError<E: serde::ser::Error> {
    #[error("cannot flatten unsupported data type \"{0}\"")]
    UnsupportedDataType(&'static str),
    #[error("{0}")]
    SerializerError(E),
}

impl<E: serde::ser::Error> serde::ser::Error for FlattenError<E> {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::SerializerError(E::custom(msg))
    }
}

pub(super) struct FlattenSerializer<'s, S: SerializeMap> {
    serialize_map: &'s mut S,
}

impl<'s, S: SerializeMap> FlattenSerializer<'s, S> {
    pub(super) fn new(serialize_map: &'s mut S) -> Self {
        Self { serialize_map }
    }
}

impl<'s, S: SerializeMap> Serializer for FlattenSerializer<'s, S> {
    type Ok = ();
    type Error = FlattenError<S::Error>;
    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(FlattenError::UnsupportedDataType("bool"))
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(FlattenError::UnsupportedDataType("i8"))
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(FlattenError::UnsupportedDataType("i16"))
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(FlattenError::UnsupportedDataType("i32"))
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(FlattenError::UnsupportedDataType("i64"))
    }

    fn serialize_i128(self, _v: i128) -> Result<Self::Ok, Self::Error> {
        Err(FlattenError::UnsupportedDataType("i128"))
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(FlattenError::UnsupportedDataType("u8"))
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(FlattenError::UnsupportedDataType("u16"))
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(FlattenError::UnsupportedDataType("u32"))
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(FlattenError::UnsupportedDataType("u64"))
    }

    fn serialize_u128(self, _v: u128) -> Result<Self::Ok, Self::Error> {
        Err(FlattenError::UnsupportedDataType("u128"))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(FlattenError::UnsupportedDataType("f32"))
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(FlattenError::UnsupportedDataType("f64"))
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(FlattenError::UnsupportedDataType("char"))
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(FlattenError::UnsupportedDataType("str"))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(FlattenError::UnsupportedDataType("bytes"))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(FlattenError::UnsupportedDataType("none"))
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        Err(FlattenError::UnsupportedDataType("some"))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(FlattenError::UnsupportedDataType("unit"))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(FlattenError::UnsupportedDataType("unit struct"))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(FlattenError::UnsupportedDataType("unit variant"))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        Err(FlattenError::UnsupportedDataType("newtype struct"))
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        Err(FlattenError::UnsupportedDataType("newtype variant"))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(FlattenError::UnsupportedDataType("seq"))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(FlattenError::UnsupportedDataType("tuple"))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(FlattenError::UnsupportedDataType("tuple struct"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(FlattenError::UnsupportedDataType("tuple variant"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(self)
    }
}

impl<'s, S: SerializeMap> serde::ser::SerializeMap for FlattenSerializer<'s, S> {
    type Ok = ();
    type Error = FlattenError<S::Error>;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        self.serialize_map
            .serialize_key(key)
            .map_err(FlattenError::SerializerError)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        self.serialize_map
            .serialize_value(value)
            .map_err(FlattenError::SerializerError)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // ignore the end call, the outer map will do this
        Ok(())
    }
}

impl<'s, S: SerializeMap> serde::ser::SerializeStruct for FlattenSerializer<'s, S> {
    type Ok = ();
    type Error = FlattenError<S::Error>;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        self.serialize_map
            .serialize_key(key)
            .map_err(FlattenError::SerializerError)?;
        self.serialize_map
            .serialize_value(value)
            .map_err(FlattenError::SerializerError)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // ignore the end call, the outer map will do this
        Ok(())
    }
}

impl<'s, S: SerializeMap> serde::ser::SerializeStructVariant for FlattenSerializer<'s, S> {
    type Ok = ();
    type Error = FlattenError<S::Error>;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        self.serialize_map
            .serialize_key(key)
            .map_err(FlattenError::SerializerError)?;
        self.serialize_map
            .serialize_value(value)
            .map_err(FlattenError::SerializerError)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // ignore the end call, the outer map will do this
        Ok(())
    }
}
