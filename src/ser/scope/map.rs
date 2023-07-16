use std::fmt::Display;

use smallvec::SmallVec;

use crate::{Path, PrimitiveValue, path::PathMapKey};

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

pub type OnMapEntryActions = SmallVec<[MapEntryAction; 8]>;

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
