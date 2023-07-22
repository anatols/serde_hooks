use std::{borrow::Cow, fmt::Display};

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'v> {
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
    Some,
    None,
    UnitStruct(&'static str),
    UnitVariant {
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    },
    NewtypeStruct(&'static str),
    NewtypeVariant {
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    },
    Seq(Option<usize>),
    Tuple(usize),
    TupleStruct {
        name: &'static str,
        len: usize,
    },
    TupleVariant {
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    },
    Map(Option<usize>),
    Struct {
        name: &'static str,
        len: usize,
    },
    StructVariant {
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    },
}

pub type StaticValue = Value<'static>;

impl Eq for Value<'_> {}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(v) => Display::fmt(v, f),
            Value::I8(v) => Display::fmt(v, f),
            Value::I16(v) => Display::fmt(v, f),
            Value::I32(v) => Display::fmt(v, f),
            Value::I64(v) => Display::fmt(v, f),
            Value::U8(v) => Display::fmt(v, f),
            Value::U16(v) => Display::fmt(v, f),
            Value::U32(v) => Display::fmt(v, f),
            Value::U64(v) => Display::fmt(v, f),
            Value::F32(v) => Display::fmt(v, f),
            Value::F64(v) => Display::fmt(v, f),
            Value::Char(c) => f.write_fmt(format_args!("'{c}'")),
            Value::Str(s) => f.write_fmt(format_args!("\"{s}\"")),
            Value::Bytes(b) => f.write_fmt(format_args!("[{len} bytes]", len = b.len())),
            Value::Unit => f.write_str("()"),
            Value::None => f.write_str("None"),

            Value::UnitStruct(name) => f.write_fmt(format_args!("unit struct {name}")),

            Value::UnitVariant { name, variant, .. } => {
                f.write_fmt(format_args!("{name}::{variant}"))
            }

            Value::NewtypeStruct(name) => f.write_fmt(format_args!("newtype {name}")),

            Value::Some => f.write_str("Some"),

            Value::NewtypeVariant { name, variant, .. } => {
                f.write_fmt(format_args!("newtype {name}::{variant}"))
            }

            Value::Seq(len) => match len {
                Some(len) => f.write_fmt(format_args!("[{len} items]")),
                None => f.write_str("[? items]"),
            },

            Value::Tuple(len) => f.write_fmt(format_args!("({len} items)")),

            Value::TupleStruct { name, len } => f.write_fmt(format_args!("{name}({len} items)")),

            Value::TupleVariant {
                name, variant, len, ..
            } => f.write_fmt(format_args!("{name}::{variant}({len} items)")),

            Value::Map(len) => match len {
                Some(len) => f.write_fmt(format_args!("{{{len} entries}}")),
                None => f.write_str("{? entries}"),
            },

            Value::Struct { name, len } => f.write_fmt(format_args!("{name}{{{len} fields}}")),

            Value::StructVariant {
                name, variant, len, ..
            } => f.write_fmt(format_args!("{name}::{variant}{{{len} fields}}")),
        }
    }
}

macro_rules! impl_value_from_type {
    ($variant:ident,$type:ty) => {
        impl From<$type> for Value<'_> {
            fn from(value: $type) -> Self {
                Value::$variant(value)
            }
        }
    };
}

impl_value_from_type!(Bool, bool);
impl_value_from_type!(I8, i8);
impl_value_from_type!(I16, i16);
impl_value_from_type!(I32, i32);
impl_value_from_type!(I64, i64);
impl_value_from_type!(U8, u8);
impl_value_from_type!(U16, u16);
impl_value_from_type!(U32, u32);
impl_value_from_type!(U64, u64);
impl_value_from_type!(F32, f32);
impl_value_from_type!(F64, f64);
impl_value_from_type!(Char, char);

impl From<()> for Value<'_> {
    fn from(_: ()) -> Self {
        Value::Unit
    }
}

macro_rules! cow_value_from_type {
    ($variant:ident,$borrowed:ty,$owned:ty) => {
        impl<'v> From<&'v $borrowed> for Value<'v> {
            fn from(value: &'v $borrowed) -> Self {
                Value::$variant(Cow::Borrowed(value))
            }
        }

        impl From<$owned> for Value<'_> {
            fn from(value: $owned) -> Self {
                Value::$variant(Cow::Owned(value))
            }
        }

        impl<'v> From<Cow<'v, $borrowed>> for Value<'v> {
            fn from(value: Cow<'v, $borrowed>) -> Self {
                Value::$variant(value)
            }
        }
    };
}

cow_value_from_type!(Str, str, String);
cow_value_from_type!(Bytes, [u8], Vec<u8>);
