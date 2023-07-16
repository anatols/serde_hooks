use std::{
    borrow::Cow,
    fmt::{Debug, Display},
};

use serde::{de::Error, Serialize, Serializer};

use crate::ser::Path;

use super::{
    path::PathMapKey,
    wrapper::{OnMapEntryActions, OnStructFieldActions, OnValueAction},
    PrimitiveValue, Value,
};

//TODO add support for:
// skip field(s)
// retain field(s)
// replace value (in struct, map, array or leaf?)
// replace key?
// rename key free-form & cases
// flatten?
// convert struct to map

pub trait Hooks {
    fn start(&self) {}

    fn end(&self) {}

    fn on_error(&self, err: &mut ErrorScope) {}

    fn on_map(&self, _map: &mut MapScope) {}

    fn on_map_key<S: Serializer>(&self, _map_key: &mut MapKeyScope<S>) {}

    fn on_struct(&self, _st: &mut StructScope) {}

    fn on_value<S: Serializer>(&self, _value: &mut ValueScope<S>) {}
}

#[derive(Debug, Clone, Eq, PartialEq)]
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

impl Display for MapKeySelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapKeySelector::ByValue(value) => f.write_fmt(format_args!("[{value}]")),
            MapKeySelector::ByIndex(index) => f.write_fmt(format_args!("[{index}]")),
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

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum HooksError {
    #[error("cannot add key {0}, the key is already present in the map")]
    KeyAlreadyPresent(MapKeySelector),
    #[error("cannot add entry with an index {0}, please specify key value")]
    CannotAddEntryByIndex(usize),
    #[error("key {0} not found")]
    KeyNotFound(MapKeySelector),
    #[error("field \"{0}\" not found")]
    FieldNotFound(Cow<'static, str>),
}

//TODO does it need to be pub?
#[derive(Debug)]
pub enum MapEntryAction {
    Retain(MapKeySelector),
    Skip(MapKeySelector),
    Add(MapKeySelector, Option<PrimitiveValue>),
    Replace(MapKeySelector, Option<PrimitiveValue>),
    ReplaceOrAdd(MapKeySelector, Option<PrimitiveValue>),
    ReplaceKey(MapKeySelector, PrimitiveValue),
}

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

    pub fn add_entry(
        &mut self,
        key: impl Into<MapKeySelector>,
        value: impl Into<PrimitiveValue>,
    ) -> &mut Self {
        self.actions
            .push(MapEntryAction::Add(key.into(), Some(value.into())));
        self
    }

    pub fn add_entry_on_serialize(&mut self, key: impl Into<MapKeySelector>) -> &mut Self {
        self.actions.push(MapEntryAction::Add(key.into(), None));
        self
    }

    pub fn add_or_replace_entry(
        &mut self,
        key: impl Into<MapKeySelector>,
        new_value: impl Into<PrimitiveValue>,
    ) -> &mut Self {
        self.actions.push(MapEntryAction::ReplaceOrAdd(
            key.into(),
            Some(new_value.into()),
        ));
        self
    }

    pub fn add_or_replace_entry_on_serialize(
        &mut self,
        key: impl Into<MapKeySelector>,
    ) -> &mut Self {
        self.actions
            .push(MapEntryAction::ReplaceOrAdd(key.into(), None));
        self
    }

    pub fn replace_value(
        &mut self,
        key: impl Into<MapKeySelector>,
        new_value: impl Into<PrimitiveValue>,
    ) -> &mut Self {
        self.actions
            .push(MapEntryAction::Replace(key.into(), Some(new_value.into())));
        self
    }

    //TODO is this needed at all?
    pub fn replace_value_on_serialize(&mut self, key: impl Into<MapKeySelector>) -> &mut Self {
        self.actions.push(MapEntryAction::Replace(key.into(), None));
        self
    }

    pub fn replace_key(
        &mut self,
        key: impl Into<MapKeySelector>,
        new_key: impl Into<PrimitiveValue>,
    ) -> &mut Self {
        self.actions
            .push(MapEntryAction::ReplaceKey(key.into(), new_key.into()));
        self
    }

    //TODO play better with cows, no need to do to_owned all for &'static str
    pub fn rename_key(&mut self, key: &str, new_key: &str) -> &mut Self {
        self.actions.push(MapEntryAction::ReplaceKey(
            key.to_owned().into(),
            new_key.to_owned().into(),
        ));
        self
    }
}

#[derive(Debug)]
pub enum StructFieldAction {
    Retain(Cow<'static, str>),
    Skip(Cow<'static, str>),
    Rename(Cow<'static, str>, Cow<'static, str>),
    ReplaceValue(Cow<'static, str>, PrimitiveValue),
}

pub struct StructScope<'p> {
    path: &'p Path,
    struct_len: usize,
    struct_name: &'static str,
    actions: OnStructFieldActions,
}

impl<'p> StructScope<'p> {
    pub(crate) fn new(path: &'p Path, struct_len: usize, struct_name: &'static str) -> Self {
        Self {
            path,
            struct_len,
            struct_name,
            actions: Default::default(),
        }
    }

    pub(crate) fn into_actions(self) -> OnStructFieldActions {
        self.actions
    }

    pub fn path(&self) -> &Path {
        self.path
    }

    pub fn struct_len(&self) -> usize {
        self.struct_len
    }

    pub fn struct_name(&self) -> &'static str {
        self.struct_name
    }

    pub fn retain_field(&mut self, key: impl Into<Cow<'static, str>>) -> &mut Self {
        self.actions.push(StructFieldAction::Retain(key.into()));
        self
    }

    pub fn skip_field(&mut self, key: impl Into<Cow<'static, str>>) -> &mut Self {
        self.actions.push(StructFieldAction::Skip(key.into()));
        self
    }

    pub fn rename_field(
        &mut self,
        key: impl Into<Cow<'static, str>>,
        new_key: impl Into<Cow<'static, str>>,
    ) -> &mut Self {
        self.actions
            .push(StructFieldAction::Rename(key.into(), new_key.into()));
        self
    }

    pub fn replace_value(
        &mut self,
        key: impl Into<Cow<'static, str>>,
        new_value: impl Into<PrimitiveValue>,
    ) -> &mut Self {
        self.actions.push(StructFieldAction::ReplaceValue(
            key.into(),
            new_value.into(),
        ));
        self
    }
}

pub struct ValueScope<'p, S: Serializer> {
    path: &'p Path,
    action: Option<OnValueAction<S>>,
    value: Value,
}

impl<'p, S: Serializer> ValueScope<'p, S> {
    pub(crate) fn new(path: &'p Path, serializer: S, value: Value) -> Self {
        Self {
            path,
            action: Some(OnValueAction::ContinueSerialization(serializer)),
            value,
        }
    }

    pub(crate) fn into_action(self) -> OnValueAction<S> {
        self.action.unwrap()
    }

    pub fn path(&self) -> &Path {
        self.path
    }

    pub fn value(&self) -> &Value {
        &self.value
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

pub type MapKeyScope<'p, S> = ValueScope<'p, S>;

pub struct ErrorScope<'p> {
    path: &'p Path,
    error: HooksError,
    ignore: bool,
}

impl<'p> ErrorScope<'p> {
    pub(crate) fn new(path: &'p Path, error: HooksError) -> Self {
        Self {
            path,
            error,
            ignore: false,
        }
    }

    pub(crate) fn into_result<S: Serializer>(self) -> Result<(), S::Error> {
        if self.ignore {
            Ok(())
        } else {
            Err(serde::ser::Error::custom(self.format_error_message()))
        }
    }

    pub fn path(&self) -> &Path {
        self.path
    }

    pub fn error(&self) -> &HooksError {
        &self.error
    }

    pub fn ignore(&mut self) {
        self.ignore = true;
    }

    pub fn panic(&mut self) {
        panic!("{}", self.format_error_message());
    }

    pub fn propagate(&mut self) {
        self.ignore = false;
    }

    fn format_error_message(&self) -> String {
        format!(
            "Error at {path}: {err}",
            path = self.path.to_string(),
            err = self.error,
        )
    }
}
