use std::fmt::{Debug, Display, Write};

use super::hooks::PrimitiveValue;

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
        self.segments.iter().fold(String::new(), |mut acc, item| {
            if !acc.is_empty() {
                acc.push('.');
            }
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
    Str(usize, String),
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

    pub(crate) fn primitive_value(&self) -> Option<PrimitiveValue> {
        match self {
            PathMapKey::Bool(_, value) => Some(PrimitiveValue::Bool(*value)),
            PathMapKey::I8(_, value) => Some(PrimitiveValue::I8(*value)),
            PathMapKey::I16(_, value) => Some(PrimitiveValue::I16(*value)),
            PathMapKey::I32(_, value) => Some(PrimitiveValue::I32(*value)),
            PathMapKey::I64(_, value) => Some(PrimitiveValue::I64(*value)),
            PathMapKey::U8(_, value) => Some(PrimitiveValue::U8(*value)),
            PathMapKey::U16(_, value) => Some(PrimitiveValue::U16(*value)),
            PathMapKey::U32(_, value) => Some(PrimitiveValue::U32(*value)),
            PathMapKey::U64(_, value) => Some(PrimitiveValue::U64(*value)),
            PathMapKey::F32(_, value) => Some(PrimitiveValue::F32(*value)),
            PathMapKey::F64(_, value) => Some(PrimitiveValue::F64(*value)),
            PathMapKey::Char(_, value) => Some(PrimitiveValue::Char(*value)),
            PathMapKey::Str(_, value) => Some(PrimitiveValue::Str(value.clone())),
            _ => None,
        }
    }
}

impl Display for PathMapKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //TODO proper formatting
        f.write_fmt(format_args!("{:?}", self))
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
        //TODO proper formatting
        f.write_fmt(format_args!("{:?}", self))
    }
}

impl From<PathMapKey> for PathSegment {
    fn from(map_key: PathMapKey) -> Self {
        PathSegment::MapKey(map_key)
    }
}