use std::{borrow::Cow, fmt::Display};

use crate::{
    ser::wrapper::{MapEntryAction, MapEntryActions},
    StaticValue,
};

//TODO document everything
//TODO tests for everything
//TODO add support for rename_key_case and rename_all_keys_case
//TODO add support for add_entry_before, add_entry_after, push_entry
pub struct MapScope {
    map_len: Option<usize>,
    actions: MapEntryActions,
}

impl MapScope {
    pub(crate) fn new(map_len: Option<usize>) -> Self {
        Self {
            map_len,
            actions: Default::default(),
        }
    }

    pub(crate) fn into_actions(self) -> MapEntryActions {
        self.actions
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
        value: impl Into<StaticValue>,
    ) -> &mut Self {
        self.actions
            .push(MapEntryAction::Add(key.into(), Some(value.into())));
        self
    }

    //TODO is this needed at all?
    pub fn add_entry_on_serialize(&mut self, key: impl Into<MapKeySelector>) -> &mut Self {
        self.actions.push(MapEntryAction::Add(key.into(), None));
        self
    }

    pub fn add_or_replace_entry(
        &mut self,
        key: impl Into<MapKeySelector>,
        new_value: impl Into<StaticValue>,
    ) -> &mut Self {
        self.actions.push(MapEntryAction::ReplaceOrAdd(
            key.into(),
            Some(new_value.into()),
        ));
        self
    }

    //TODO is this needed at all?
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
        new_value: impl Into<StaticValue>,
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
        new_key: impl Into<StaticValue>,
    ) -> &mut Self {
        self.actions
            .push(MapEntryAction::ReplaceKey(key.into(), new_key.into()));
        self
    }

    pub fn rename_key(
        &mut self,
        key: impl Into<MapKeySelector>,
        new_key: impl Into<Cow<'static, str>>,
    ) -> &mut Self {
        self.actions.push(MapEntryAction::ReplaceKey(
            key.into(),
            new_key.into().into(),
        ));
        self
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MapKeySelector {
    ByValue(StaticValue),
    ByIndex(usize),
}

impl MapKeySelector {
    pub(crate) fn matches_path_key(&self, value: &crate::Value, index: usize) -> bool {
        match self {
            MapKeySelector::ByValue(v) => value == v,
            MapKeySelector::ByIndex(i) => index == *i,
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

impl<T: Into<StaticValue>> From<T> for MapKeySelector {
    fn from(value: T) -> Self {
        MapKeySelector::ByValue(value.into())
    }
}

impl From<usize> for MapKeySelector {
    fn from(value: usize) -> Self {
        MapKeySelector::ByIndex(value)
    }
}
