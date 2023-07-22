use std::{borrow::Cow, fmt::Display};

use serde::{Serialize, Serializer};

#[derive(Debug, Clone, PartialEq)]
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
    Bytes(Cow<'v, [u8]>),
    Unit,
    None,
    UnitStruct(&'static str),
    UnitVariant {
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    },
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
            PrimitiveValue::Bytes(v) => serializer.serialize_bytes(v),
            PrimitiveValue::Unit => serializer.serialize_unit(),
            PrimitiveValue::None => serializer.serialize_none(),
            PrimitiveValue::UnitStruct(name) => serializer.serialize_unit_struct(name),
            PrimitiveValue::UnitVariant {
                name,
                variant_index,
                variant,
            } => serializer.serialize_unit_variant(name, *variant_index, variant),
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
            PrimitiveValue::Bytes(b) => f.write_fmt(format_args!("[{len} bytes]", len = b.len())),
            PrimitiveValue::Unit => f.write_str("()"),
            PrimitiveValue::None => f.write_str("None"),
            PrimitiveValue::UnitStruct(name) => f.write_fmt(format_args!("{name}{{}}")),
            PrimitiveValue::UnitVariant { name, variant, .. } => {
                f.write_fmt(format_args!("{name}::{variant}"))
            }
        }
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

impl From<()> for PrimitiveValue<'_> {
    fn from(_: ()) -> Self {
        PrimitiveValue::Unit
    }
}

macro_rules! cow_value_from_type {
    ($variant:ident,$borrowed:ty,$owned:ty) => {
        impl<'v> From<&'v $borrowed> for PrimitiveValue<'v> {
            fn from(value: &'v $borrowed) -> Self {
                PrimitiveValue::$variant(Cow::Borrowed(value))
            }
        }

        impl From<$owned> for PrimitiveValue<'_> {
            fn from(value: $owned) -> Self {
                PrimitiveValue::$variant(Cow::Owned(value))
            }
        }

        impl<'v> From<Cow<'v, $borrowed>> for PrimitiveValue<'v> {
            fn from(value: Cow<'v, $borrowed>) -> Self {
                PrimitiveValue::$variant(value)
            }
        }
    };
}

cow_value_from_type!(Str, str, String);
cow_value_from_type!(Bytes, [u8], Vec<u8>);

#[derive(Debug)]
pub enum Value<'v> {
    Primitive(PrimitiveValue<'v>),
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
