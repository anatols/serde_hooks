use std::fmt::{Debug, Display};

use serde::{Serialize, Serializer};

use crate::ser::Path;

use super::{
    path::PathMapKey,
    wrapper::{OnMapEntryActions, OnValueAction},
};

#[derive(Debug)]
pub enum MapKeySelector {
    ByValue(PrimitiveValue),
    ByIndex(usize),
}

impl MapKeySelector {
    pub(crate) fn matches_path_key(&self, key: &PathMapKey) -> bool {
        match self {
            MapKeySelector::ByValue(v) => key.primitive_value().map(|kv| kv.eq(v)).unwrap_or(false),
            MapKeySelector::ByIndex(i) => key.index() == *i,
        }
    }
}

impl<T: Into<PrimitiveValue>> From<T> for MapKeySelector {
    fn from(value: T) -> Self {
        MapKeySelector::ByValue(value.into())
    }
}

impl From<usize> for MapKeySelector {
    fn from(value: usize) -> Self {
        MapKeySelector::ByIndex(value)
    }
}

#[derive(Debug, PartialEq)]
pub enum PrimitiveValue {
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
    //TODO can replace with str?
    Str(String),
}

impl Serialize for PrimitiveValue {
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

impl Display for PrimitiveValue {
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

//TODO implement other conversions
impl From<&str> for PrimitiveValue {
    fn from(value: &str) -> Self {
        PrimitiveValue::Str(value.to_string())
    }
}

#[derive(Debug)]
pub enum Value {
    Primitive(PrimitiveValue),
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

//TODO does it need to be pub?
#[derive(Debug)]
pub enum MapEntryAction {
    Retain(MapKeySelector),
    Skip(MapKeySelector),
    Insert(MapKeySelector, Option<PrimitiveValue>),
}

//TODO move to a submodule
pub struct MapScope<'p> {
    path: &'p Path,
    map_len: Option<usize>,
    actions: OnMapEntryActions,
}

impl<'p> MapScope<'p> {
    pub(crate) fn new(path: &'p Path, map_len: Option<usize>) -> Self {
        Self {
            path,
            map_len,
            actions: Default::default(),
        }
    }

    pub(crate) fn into_actions(self) -> OnMapEntryActions {
        self.actions
    }

    pub fn path(&self) -> &Path {
        self.path
    }

    pub fn map_len(&self) -> Option<usize> {
        self.map_len
    }

    pub fn retain_entry(&mut self, key: impl Into<MapKeySelector>) -> &mut Self {
        self.actions.push(MapEntryAction::Retain(key.into()));
        self
    }

    pub fn skip_entry(&mut self, key: impl Into<MapKeySelector>) -> &mut Self {
        self.actions.push(MapEntryAction::Skip(key.into()));
        self
    }

    //TODO 'insert after'?
    //TODO should we have a 'replace' that will only replace, but not insert?
    pub fn insert_entry(
        &mut self,
        key: impl Into<MapKeySelector>,
        value: impl Into<PrimitiveValue>,
    ) -> &mut Self {
        self.actions
            .push(MapEntryAction::Insert(key.into(), Some(value.into())));
        self
    }

    pub fn insert_key(&mut self, key: impl Into<MapKeySelector>) -> &mut Self {
        self.actions.push(MapEntryAction::Insert(key.into(), None));
        self
    }
}

pub struct ValueScope<'p, S: Serializer> {
    path: &'p Path,
    action: Option<OnValueAction<S>>,
    //TODO add MapValue
}

impl<'p, S: Serializer> ValueScope<'p, S> {
    pub(crate) fn new(path: &'p Path, serializer: S) -> Self {
        Self {
            path,
            action: Some(OnValueAction::ContinueSerialization(serializer)),
        }
    }

    pub(crate) fn into_action(self) -> OnValueAction<S> {
        self.action.unwrap()
    }

    pub fn path(&self) -> &Path {
        self.path
    }

    pub fn replace<T: Serialize + ?Sized>(&mut self, new_value: &T) -> &mut Self {
        let serializer = match self.action.take().unwrap() {
            OnValueAction::ContinueSerialization(s) => s,
            OnValueAction::ValueReplaced(_) => panic!("value already replaced"),
        };
        let res = new_value.serialize(serializer);
        self.action = Some(OnValueAction::ValueReplaced(res));
        self
    }
}

pub trait Hooks {
    fn start(&self) {}
    fn end(&self) {}

    fn on_map(&self, _map: &mut MapScope) {}

    fn on_value<S: Serializer>(&self, _value: &mut ValueScope<S>) {}
}

// skip field(s)
// retain field(s)
// replace value (in struct, map, array or leaf?)
// replace key?
// rename key free-form & cases
// flatten?
