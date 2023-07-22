use std::{
    borrow::Cow,
    fmt::{Debug, Display, Write},
};

use crate::{StaticValue, Value};

#[derive(Debug, Default)]
pub struct Path {
    segments: Vec<PathSegment>,
}

impl Path {
    pub(crate) fn push_segment(&mut self, segment: PathSegment) {
        self.segments.push(segment);
    }

    pub(crate) fn pop_segment(&mut self) {
        self.segments.pop().expect("unbalanced pop_segment");
    }
}

impl ToString for Path {
    fn to_string(&self) -> String {
        self.segments.iter().fold("$".to_string(), |mut acc, item| {
            write!(&mut acc, "{item}").expect("path concat failed");
            acc
        })
    }
}

#[derive(Debug, Clone)]
pub enum PathMapKey {
    Bool(usize, bool),
    I8(usize, i8),
    I16(usize, i16),
    I32(usize, i32),
    I64(usize, i64),
    U8(usize, u8),
    U16(usize, u16),
    U32(usize, u32),
    U64(usize, u64),
    F32(usize, f32),
    F64(usize, f64),
    Char(usize, char),
    Str(usize, Cow<'static, str>),
    Bytes(usize),
    None(usize),
    Unit(usize),
    UnitStruct(usize),
    UnitVariant(usize),
    NewtypeStruct(usize),
    NewtypeVariant(usize),
    Seq(usize),
    Tuple(usize),
    TupleStruct(usize),
    TupleVariant(usize),
    Map(usize),
    Struct(usize),
    StructVariant(usize),
}

impl PathMapKey {
    pub(crate) fn from_index_and_value(index: usize, value: StaticValue) -> Self {
        match value {
            Value::Bool(v) => PathMapKey::Bool(index, v),
            Value::I8(v) => PathMapKey::I8(index, v),
            Value::I16(v) => PathMapKey::I16(index, v),
            Value::I32(v) => PathMapKey::I32(index, v),
            Value::I64(v) => PathMapKey::I64(index, v),
            Value::U8(v) => PathMapKey::U8(index, v),
            Value::U16(v) => PathMapKey::U16(index, v),
            Value::U32(v) => PathMapKey::U32(index, v),
            Value::U64(v) => PathMapKey::U64(index, v),
            Value::F32(v) => PathMapKey::F32(index, v),
            Value::F64(v) => PathMapKey::F64(index, v),
            Value::Char(v) => PathMapKey::Char(index, v),
            Value::Str(v) => PathMapKey::Str(index, v),
            Value::Bytes(_) => PathMapKey::Bytes(index),
            Value::Unit => PathMapKey::Unit(index),
            Value::None => PathMapKey::None(index),
            Value::UnitStruct(_) => PathMapKey::UnitStruct(index),
            Value::UnitVariant { .. } => PathMapKey::UnitVariant(index),
            Value::NewtypeStruct(_) => PathMapKey::NewtypeStruct(index),
            Value::Some => todo!(),
            Value::NewtypeVariant {
                name,
                variant_index,
                variant,
            } => todo!(),
            Value::Seq(_) => todo!(),
            Value::Tuple(_) => todo!(),
            Value::TupleStruct { name, len } => todo!(),
            Value::TupleVariant {
                name,
                variant_index,
                variant,
                len,
            } => todo!(),
            Value::Map(_) => todo!(),
            Value::Struct { name, len } => todo!(),
            Value::StructVariant {
                name,
                variant_index,
                variant,
                len,
            } => todo!(),
        }
    }

    pub(crate) fn index(&self) -> usize {
        match self {
            PathMapKey::Bool(index, _)
            | PathMapKey::I8(index, _)
            | PathMapKey::I16(index, _)
            | PathMapKey::I32(index, _)
            | PathMapKey::I64(index, _)
            | PathMapKey::U8(index, _)
            | PathMapKey::U16(index, _)
            | PathMapKey::U32(index, _)
            | PathMapKey::U64(index, _)
            | PathMapKey::F32(index, _)
            | PathMapKey::F64(index, _)
            | PathMapKey::Char(index, _)
            | PathMapKey::Str(index, _)
            | PathMapKey::Bytes(index)
            | PathMapKey::None(index)
            | PathMapKey::Unit(index)
            | PathMapKey::UnitStruct(index)
            | PathMapKey::UnitVariant(index)
            | PathMapKey::NewtypeStruct(index)
            | PathMapKey::NewtypeVariant(index)
            | PathMapKey::Seq(index)
            | PathMapKey::Tuple(index)
            | PathMapKey::TupleStruct(index)
            | PathMapKey::TupleVariant(index)
            | PathMapKey::Map(index)
            | PathMapKey::Struct(index)
            | PathMapKey::StructVariant(index) => *index,
        }
    }

    pub(crate) fn value(&self) -> Option<Value> {
        match self {
            PathMapKey::Bool(_, value) => Some(Value::Bool(*value)),
            PathMapKey::I8(_, value) => Some(Value::I8(*value)),
            PathMapKey::I16(_, value) => Some(Value::I16(*value)),
            PathMapKey::I32(_, value) => Some(Value::I32(*value)),
            PathMapKey::I64(_, value) => Some(Value::I64(*value)),
            PathMapKey::U8(_, value) => Some(Value::U8(*value)),
            PathMapKey::U16(_, value) => Some(Value::U16(*value)),
            PathMapKey::U32(_, value) => Some(Value::U32(*value)),
            PathMapKey::U64(_, value) => Some(Value::U64(*value)),
            PathMapKey::F32(_, value) => Some(Value::F32(*value)),
            PathMapKey::F64(_, value) => Some(Value::F64(*value)),
            PathMapKey::Char(_, value) => Some(Value::Char(*value)),
            PathMapKey::Str(_, value) => Some(Value::Str(value.clone())),
            _ => None,
        }
    }
}

impl Display for PathMapKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(value) = self.value() {
            Display::fmt(&value, f)
        } else {
            Display::fmt(&self.index(), f)
        }
    }
}

#[derive(Debug, Clone)]
pub enum PathSegment {
    MapKey(PathMapKey),
    StructField(&'static str),
    SeqIndex(usize),
}

impl Display for PathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathSegment::MapKey(key) => f.write_fmt(format_args!("[{key}]")),
            PathSegment::StructField(field_name) => f.write_fmt(format_args!(".{field_name}")),
            PathSegment::SeqIndex(index) => f.write_fmt(format_args!("[{index}]")),
        }
    }
}

impl From<PathMapKey> for PathSegment {
    fn from(map_key: PathMapKey) -> Self {
        PathSegment::MapKey(map_key)
    }
}
