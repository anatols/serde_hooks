use std::{borrow::Cow, fmt::Display};

use serde::{Serialize, Serializer};

#[derive(Debug, Clone, PartialEq)]
//TODO should this include None, Unit, UnitStruct?
pub enum PrimitiveValue<'v> {
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Char(char),
    Str(Cow<'v, str>),
}

pub type StaticPrimitiveValue = PrimitiveValue<'static>;

impl Eq for PrimitiveValue<'_> {}

impl Serialize for PrimitiveValue<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            PrimitiveValue::Bool(v) => v.serialize(serializer),
            PrimitiveValue::I8(v) => v.serialize(serializer),
            PrimitiveValue::I16(v) => v.serialize(serializer),
            PrimitiveValue::I32(v) => v.serialize(serializer),
            PrimitiveValue::I64(v) => v.serialize(serializer),
            PrimitiveValue::U8(v) => v.serialize(serializer),
            PrimitiveValue::U16(v) => v.serialize(serializer),
            PrimitiveValue::U32(v) => v.serialize(serializer),
            PrimitiveValue::U64(v) => v.serialize(serializer),
            PrimitiveValue::F32(v) => v.serialize(serializer),
            PrimitiveValue::F64(v) => v.serialize(serializer),
            PrimitiveValue::Char(v) => v.serialize(serializer),
            PrimitiveValue::Str(v) => v.serialize(serializer),
        }
    }
}

impl Display for PrimitiveValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveValue::Bool(v) => Display::fmt(v, f),
            PrimitiveValue::I8(v) => Display::fmt(v, f),
            PrimitiveValue::I16(v) => Display::fmt(v, f),
            PrimitiveValue::I32(v) => Display::fmt(v, f),
            PrimitiveValue::I64(v) => Display::fmt(v, f),
            PrimitiveValue::U8(v) => Display::fmt(v, f),
            PrimitiveValue::U16(v) => Display::fmt(v, f),
            PrimitiveValue::U32(v) => Display::fmt(v, f),
            PrimitiveValue::U64(v) => Display::fmt(v, f),
            PrimitiveValue::F32(v) => Display::fmt(v, f),
            PrimitiveValue::F64(v) => Display::fmt(v, f),
            PrimitiveValue::Char(c) => f.write_fmt(format_args!("'{c}'")),
            PrimitiveValue::Str(s) => f.write_fmt(format_args!("\"{s}\"")),
        }
    }
}

impl<'v> From<&'v str> for PrimitiveValue<'v> {
    fn from(value: &'v str) -> Self {
        PrimitiveValue::Str(Cow::Borrowed(value))
    }
}

impl From<String> for PrimitiveValue<'_> {
    fn from(value: String) -> Self {
        PrimitiveValue::Str(Cow::Owned(value))
    }
}

impl<'v> From<Cow<'v, str>> for PrimitiveValue<'v> {
    fn from(value: Cow<'v, str>) -> Self {
        PrimitiveValue::Str(value)
    }
}

macro_rules! primitive_value_from_type {
    ($variant:ident,$type:ty) => {
        impl From<$type> for PrimitiveValue<'_> {
            fn from(value: $type) -> Self {
                PrimitiveValue::$variant(value)
            }
        }
    };
}

primitive_value_from_type!(Bool, bool);
primitive_value_from_type!(I8, i8);
primitive_value_from_type!(I16, i16);
primitive_value_from_type!(I32, i32);
primitive_value_from_type!(I64, i64);
primitive_value_from_type!(U8, u8);
primitive_value_from_type!(U16, u16);
primitive_value_from_type!(U32, u32);
primitive_value_from_type!(U64, u64);
primitive_value_from_type!(F32, f32);
primitive_value_from_type!(F64, f64);
primitive_value_from_type!(Char, char);

#[derive(Debug)]
pub enum Value<'v> {
    Primitive(PrimitiveValue<'v>),
    Bytes,
    None,
    Unit,
    UnitStruct,
    UnitVariant,
    NewtypeStruct,
    NewtypeVariant,
    Seq,
    Tuple,
    TupleStruct,
    TupleVariant,
    Map,
    Struct,
    StructVariant,
}
