use std::fmt::{Display, Debug, Write};

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
pub enum MapKey {
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

impl Display for MapKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //TODO proper formatting
        f.write_fmt(format_args!("{:?}", self))
    }
}

#[derive(Debug, Clone)]
pub enum PathSegment {
    MapKey(MapKey),
    StructField(&'static str),
    SeqIndex(usize),
}

impl Display for PathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //TODO proper formatting
        f.write_fmt(format_args!("{:?}", self))
    }
}

impl From<MapKey> for PathSegment {
    fn from(map_key: MapKey) -> Self {
        PathSegment::MapKey(map_key)
    }
}
