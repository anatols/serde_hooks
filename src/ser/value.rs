use std::fmt::Display;

use serde::{ser::Impossible, Serialize, Serializer};

use crate::Value;

use super::HooksError;

impl Serialize for Value<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Bool(v) => v.serialize(serializer),
            Value::I8(v) => v.serialize(serializer),
            Value::I16(v) => v.serialize(serializer),
            Value::I32(v) => v.serialize(serializer),
            Value::I64(v) => v.serialize(serializer),
            Value::U8(v) => v.serialize(serializer),
            Value::U16(v) => v.serialize(serializer),
            Value::U32(v) => v.serialize(serializer),
            Value::U64(v) => v.serialize(serializer),
            Value::F32(v) => v.serialize(serializer),
            Value::F64(v) => v.serialize(serializer),
            Value::Char(v) => v.serialize(serializer),
            Value::Str(v) => v.serialize(serializer),
            Value::Bytes(v) => serializer.serialize_bytes(v),
            Value::Unit => serializer.serialize_unit(),
            Value::None => serializer.serialize_none(),
            Value::UnitStruct(name) => serializer.serialize_unit_struct(name),
            Value::UnitVariant {
                name,
                variant_index,
                variant,
            } => serializer.serialize_unit_variant(name, *variant_index, variant),
            _ => Err(serde::ser::Error::custom(format!(
                "{self} cannot be represented fully in Value"
            ))),
        }
    }
}

impl Value<'_> {
    pub(crate) fn check_if_can_serialize(&self) -> Result<(), HooksError> {
        struct FauxSerializer;

        #[derive(Debug, thiserror::Error)]
        #[error("{0}")]
        struct FauxError(String);

        impl serde::ser::Error for FauxError {
            fn custom<T>(msg: T) -> Self
            where
                T: Display,
            {
                Self(msg.to_string())
            }
        }

        impl Serializer for FauxSerializer {
            type Ok = ();
            type Error = FauxError;
            type SerializeSeq = Impossible<Self::Ok, Self::Error>;
            type SerializeTuple = Impossible<Self::Ok, Self::Error>;
            type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
            type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
            type SerializeMap = Impossible<Self::Ok, Self::Error>;
            type SerializeStruct = Impossible<Self::Ok, Self::Error>;
            type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

            fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }

            fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }

            fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }

            fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }

            fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }

            fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }

            fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }

            fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }

            fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }

            fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }

            fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }

            fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }

            fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }

            fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }

            fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }

            fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
            where
                T: Serialize,
            {
                unreachable!()
            }

            fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }

            fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }

            fn serialize_unit_variant(
                self,
                _name: &'static str,
                _variant_index: u32,
                _variant: &'static str,
            ) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }

            fn serialize_newtype_struct<T: ?Sized>(
                self,
                _name: &'static str,
                _value: &T,
            ) -> Result<Self::Ok, Self::Error>
            where
                T: Serialize,
            {
                unreachable!()
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
                unreachable!()
            }

            fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
                unreachable!()
            }

            fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
                unreachable!()
            }

            fn serialize_tuple_struct(
                self,
                _name: &'static str,
                _len: usize,
            ) -> Result<Self::SerializeTupleStruct, Self::Error> {
                unreachable!()
            }

            fn serialize_tuple_variant(
                self,
                _name: &'static str,
                _variant_index: u32,
                _variant: &'static str,
                _len: usize,
            ) -> Result<Self::SerializeTupleVariant, Self::Error> {
                unreachable!()
            }

            fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
                unreachable!()
            }

            fn serialize_struct(
                self,
                _name: &'static str,
                _len: usize,
            ) -> Result<Self::SerializeStruct, Self::Error> {
                unreachable!()
            }

            fn serialize_struct_variant(
                self,
                _name: &'static str,
                _variant_index: u32,
                _variant: &'static str,
                _len: usize,
            ) -> Result<Self::SerializeStructVariant, Self::Error> {
                unreachable!()
            }
        }

        self.serialize(FauxSerializer)
            .map_err(|err| HooksError::ValueNotSerializable(err.0))
    }
}
