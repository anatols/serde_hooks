use std::fmt::{Debug, Display, Write};

use smallvec::SmallVec;

use crate::{StaticValue, Value};

#[derive(Debug, Default)]
pub struct Path {
    segments: SmallVec<[PathSegment; 8]>,
}

impl Path {
    pub(crate) fn push_segment(&mut self, segment: PathSegment) {
        self.segments.push(segment);
    }

    pub(crate) fn pop_segment(&mut self) {
        self.segments.pop().expect("unbalanced pop_segment");
    }

    pub fn segments(&self) -> &[PathSegment] {
        &self.segments
    }
}

impl ToString for Path {
    fn to_string(&self) -> String {
        self.segments
            .iter()
            .fold(String::default(), |mut acc, item| {
                match item {
                    PathSegment::StructField(_) => {
                        if acc.is_empty() {
                            write!(&mut acc, "{item}").expect("path concat failed");
                        } else {
                            write!(&mut acc, ".{item}").expect("path concat failed");
                        }
                    }
                    _ => write!(&mut acc, "{item}").expect("path concat failed"),
                }
                acc
            })
    }
}

/// Note: for map keys of type `Value::Bytes` the actual bytes are not stored to avoid
/// allocation on every map key.
#[derive(Debug, Clone)]
pub struct PathMapKey {
    pub index: usize,
    pub value: StaticValue,
}

impl PathMapKey {
    pub(crate) fn new(index: usize, value: StaticValue) -> Self {
        Self { index, value }
    }
}

impl Display for PathMapKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            Value::Bool(_)
            | Value::I8(_)
            | Value::I16(_)
            | Value::I32(_)
            | Value::I64(_)
            | Value::U8(_)
            | Value::U16(_)
            | Value::U32(_)
            | Value::U64(_)
            | Value::F32(_)
            | Value::F64(_)
            | Value::Char(_)
            | Value::Str(_)
            | Value::Unit
            | Value::Some
            | Value::None
            | Value::UnitVariant { .. } => Display::fmt(&self.value, f),
            _ => Display::fmt(&self.index, f),
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
            PathSegment::StructField(field_name) => f.write_str(field_name),
            PathSegment::SeqIndex(index) => f.write_fmt(format_args!("[{index}]")),
        }
    }
}

impl From<PathMapKey> for PathSegment {
    fn from(map_key: PathMapKey) -> Self {
        PathSegment::MapKey(map_key)
    }
}
