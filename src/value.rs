use std::{borrow::Cow, fmt::Display};

/// Value that corresponds to [serde data model](https://serde.rs/data-model.html).
///
/// Primitive values, like numbers, will have the actual value copied,
/// whilst for compound values, like structs, only metadata is available.
#[derive(Debug, Clone, PartialEq)]
pub enum Value<'v> {
    /// `bool` value
    Bool(bool),

    /// `i8` value
    I8(i8),

    /// `i16` value
    I16(i16),

    /// `i32` value
    I32(i32),

    /// `i64` value
    I64(i64),

    /// `i128` value
    I128(i128),

    /// `u8` value
    U8(u8),

    /// `u16` value
    U16(u16),

    /// `u32` value
    U32(u32),

    /// `u64` value
    U64(u64),

    /// `u128` value
    U128(u128),

    /// `f32` value
    F32(f32),

    /// `f64` value
    F64(f64),

    /// `char` value
    Char(char),

    /// String value, borrowed or owned.
    Str(Cow<'v, str>),

    /// Bytes value, borrowed or owned.
    ///
    /// Note that you need to use `serde_bytes` or do custom (de)serialization
    /// in order to get bytes value as a `[u8]` slice instead of a sequence.
    Bytes(Cow<'v, [u8]>),

    /// Unit value, i.e. `()`.
    Unit,

    /// Metadata (marker) for a value of `Option<T>::Some(...)`.
    ///
    /// During serialization, the actual value contained in the `Option` is serialized
    /// in the follow up calls to the serializer. This metadata can thus be seen as a marker,
    /// telling that the actual value serialization will follow.
    Some,

    /// Metadata (marker) for a value of `Option<T>::None`.
    None,

    /// Metadata for a value of unit struct type (e.g. `struct Unit`).
    UnitStruct(
        /// Struct name, e.g. `"Unit"` in `struct Unit`.
        &'static str,
    ),

    /// Metadata for a value of unit variant type (e.g. `E::A` in `enum E { A, B }`).
    UnitVariant {
        /// Enum name.
        ///
        /// For example, this would be `"E"` in `enum E { A, B }`.
        name: &'static str,

        /// Variant index in the enum.
        ///
        /// For example, this would be `0` for `E::A`, `1` for `E::B` in `enum E { A, B }`.
        ///
        /// Note that this is always a sequential index, not the numeric value assigned to the
        /// variant.
        variant_index: u32,

        /// Variant name.
        ///
        /// For example, this would be `"A"` for `E::A` in `enum E { A, B }`.
        variant: &'static str,
    },

    /// Metadata for a value of newtype struct (e.g. `struct Millimeters(u8)`).
    NewtypeStruct(
        /// Struct name, e.g. `"Millimeters"` in `struct Millimeters(u8)`.
        &'static str,
    ),

    /// Metadata for a value of unit variant type (e.g. `E::N` in `enum E { N(u8) }`).
    NewtypeVariant {
        /// Enum name.
        ///
        /// For example, this would be `"E"` in `enum E { N(u8) }`.
        name: &'static str,

        /// Variant index in the enum.
        ///
        /// For example, this would be `0` for `E::N` in `enum E { N(u8) }`.
        variant_index: u32,

        /// Variant name.
        ///
        /// For example, this would be `"N"` for `E::N` in `enum E { N(u8) }`.
        variant: &'static str,
    },

    /// Metadata for a value of sequence type (e.g. `Vec<T>` or `HashSet<T>`).
    Seq(
        /// Length of the sequence, if known during (de)serialization.
        Option<usize>,
    ),

    /// Metadata for a value of tuple type.
    Tuple(
        /// Tuple length.
        ///
        /// Note that unlike `Seq`, the length is not optional and must be known
        /// during (de)serialization.
        usize,
    ),

    /// Metadata for a value of tuple struct type (e.g. `struct Rgb(u8, u8, u8)`).
    TupleStruct {
        /// Struct name, e.g. `"Rgb"` in `struct Rgb(u8, u8, u8)`.
        name: &'static str,

        /// Tuple length, e.g. `3` in `struct Rgb(u8, u8, u8)`.
        len: usize,
    },

    /// Metadata for a value of tuple variant type (e.g. `E::T` in `enum E { T(u8, u8) }`).
    TupleVariant {
        /// Enum name.
        ///
        /// For example, this would be `"E"` in `enum E { T(u8, u8) }`.
        name: &'static str,

        /// Variant index in the enum.
        ///
        /// For example, this would be `0` for `E::T` in `enum E { T(u8, u8) }`.
        variant_index: u32,

        /// Variant name.
        ///
        /// For example, this would be `"T"` for `E::T` in `enum E { T(u8, u8) }`.
        variant: &'static str,

        /// Tuple length, e.g. `2` for `E::T` in `enum E { T(u8, u8) }`.
        len: usize,
    },

    /// Metadata for a value of map type (e.g. `BTreeMap<K, V>`).
    Map(
        /// Number of map entries, if known during (de)serialization.
        Option<usize>,
    ),

    /// Metadata for a value of struct type (e.g. `struct S { r: u8, g: u8, b: u8 }`).
    Struct {
        /// Struct name, e.g. `"S"` in `struct S { r: u8, g: u8, b: u8 }`.
        name: &'static str,

        /// Number of struct fields, e.g. `3` in `struct S { r: u8, g: u8, b: u8 }`.
        ///
        /// Note that unlike `Map`, the number of fields must be known during (de)serialization.
        ///
        /// Some (de)serializers do not distinguish between maps and structures, while some others do.
        len: usize,
    },

    /// Metadata for a value of struct variant type (e.g. `E::S` in `enum E { S { r: u8, g: u8, b: u8 } }`).
    StructVariant {
        /// Enum name.
        ///
        /// For example, this would be `"E"` in `enum E { S { r: u8, g: u8, b: u8 } }`.
        name: &'static str,

        /// Variant index in the enum.
        ///
        /// For example, this would be `0` for `E::S` in `enum E { S { r: u8, g: u8, b: u8 } }`.
        variant_index: u32,

        /// Variant name.
        ///
        /// For example, this would be `"S"` for `E::S` in `enum E { S { r: u8, g: u8, b: u8 } }`.
        variant: &'static str,

        /// Number of struct fields, e.g. `3` for `E::S` in `enum E { S { r: u8, g: u8, b: u8 } }`.
        len: usize,
    },
}

/// A [`Value`] with static lifetime for borrowed data (strings, bytes).
///
/// See [Static strings](crate::ser#static-strings) for more info.
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
            Value::I128(v) => Display::fmt(v, f),
            Value::U8(v) => Display::fmt(v, f),
            Value::U16(v) => Display::fmt(v, f),
            Value::U32(v) => Display::fmt(v, f),
            Value::U64(v) => Display::fmt(v, f),
            Value::U128(v) => Display::fmt(v, f),
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
