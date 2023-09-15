use std::{borrow::Cow, fmt::Display};

use crate::{
    ser::wrapper::{MapEntryAction, MapEntryActions},
    StaticValue,
};

//TODO tests for everything
//TODO add support for rename_key_case and rename_all_keys_case

/// Inspect maps and modify their contents.
///
/// See [`Hooks::on_map`](crate::ser::Hooks::on_map).
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

    /// Returns the original number of entries in this map, if known.
    ///
    /// This is a hint that the serializer gets from the map's `Serialize` implementation
    /// if the map knows the number of items in it at runtime.
    ///
    /// The returned value is not affected by any retain or skip actions.
    pub fn map_len(&self) -> Option<usize> {
        self.map_len
    }

    /// Skips an entry during serialization.
    ///
    /// This is similar to `#[serde(skip)]` or `#[serde(skip_serializing)]`, but
    /// works for maps.
    ///
    /// At the moment the hook is called it is impossible to predict which
    /// fields will actually be fed to the serializer afterwards. Therefore, it's not
    /// possible to correctly adjust the length hint. The underlying serializer will thus
    /// be given `None` as the map length hint if you call this method. Some serializers
    /// might not support this.
    ///
    /// Returns `self` to allow chaining calls.
    pub fn skip_entry(&mut self, key: impl Into<MapKeySelector>) -> &mut Self {
        self.actions.push(MapEntryAction::Skip(key.into()));
        self
    }

    /// Retains an entry.
    ///
    /// Calling this method switches processing to a 'retain' mode, in which
    /// all not retained entries are skipped. You can retain multiple entries by
    /// calling this method multiple times.
    ///
    /// You can see this as a 'whitelist' counterpart of [`skip_entry`](Self::skip_entry).
    ///
    /// At the moment the hook is called it is impossible to predict which
    /// entries will actually be fed to the serializer afterwards. Therefore, it's not
    /// possible to correctly adjust the length hint. The underlying serializer will thus
    /// be given `None` as the map length hint if you call this method. Some serializers
    /// might not support this.
    ///
    /// Returns `self` to allow chaining calls.
    pub fn retain_entry(&mut self, key: impl Into<MapKeySelector>) -> &mut Self {
        self.actions.push(MapEntryAction::Retain(key.into()));
        self
    }

    /// Insert a new entry at a given location during serialization.
    ///
    /// There is no check for key uniqueness. If you insert an entry with a key that already
    /// exists, it will still be passed on to the serializer. Some serializers might not
    /// support this.
    ///
    /// Primitive keys and values are copied, and are later fed to the serializer.
    /// For compound values, only metadata is stored, therefore it's not possible to
    /// serialize the actual values from the contents of [`StaticValue`]. Passing in a
    /// compound value in `key` or `value` here would result in an
    /// [`HooksError::ValueNotSerializable`](crate::ser::HooksError::ValueNotSerializable) error.
    /// The trick to insert a compound value is to first insert a primitive one
    /// (e.g. a unit), subscribe to `on_value` hook, and replace the value there again with the
    /// compound one.
    ///
    /// At the moment the hook is called it is impossible to predict which
    /// fields will actually be fed to the serializer afterwards. Therefore, it's not
    /// possible to correctly adjust the length hint. The underlying serializer will thus
    /// be given `None` as the map length hint if you call this method. Some serializers
    /// might not support this.
    ///
    /// Will produce [`HooksError::KeyNotFound`](crate::ser::HooksError::KeyNotFound) error
    /// if the insertion location refers to a key that does not occur during serialization.
    ///
    /// Returns `self` to allow chaining calls.
    pub fn insert_entry(
        &mut self,
        key: impl Into<StaticValue>,
        value: impl Into<StaticValue>,
        location: MapInsertLocation,
    ) -> &mut Self {
        self.actions
            .push(MapEntryAction::Insert(key.into(), value.into(), location));
        self
    }

    /// Replace value of an existing entry.
    ///
    /// Primitive values are copied, and are later fed to the serializer.
    /// For compound values, only metadata is stored, therefore it's not possible to
    /// serialize the actual values from the contents of [`StaticValue`]. Passing in a
    /// compound value here would result in an
    /// [`HooksError::ValueNotSerializable`](crate::ser::HooksError::ValueNotSerializable) error.
    /// If you want to use a compound value as a replacement, subscribe to `on_value` hook, and
    /// replace the value there.
    ///
    /// Will produce [`HooksError::KeyNotFound`](crate::ser::HooksError::KeyNotFound) error
    /// if the key does not occur during serialization.
    ///
    /// Returns `self` to allow chaining calls.
    pub fn replace_value(
        &mut self,
        key: impl Into<MapKeySelector>,
        new_value: impl Into<StaticValue>,
    ) -> &mut Self {
        self.actions
            .push(MapEntryAction::ReplaceValue(key.into(), new_value.into()));
        self
    }

    /// Replace key of an existing entry.
    ///
    /// There is no check for key uniqueness. If you use a key that is used for some other entry,
    /// it will still be passed on to the serializer. Some serializers might not
    /// support this.
    ///
    /// Primitive keys are copied, and are later fed to the serializer.
    /// For compound keys, only metadata is stored, therefore it's not possible to
    /// serialize the actual values from the contents of [`StaticValue`]. Passing in a
    /// compound key here would result in an
    /// [`HooksError::ValueNotSerializable`](crate::ser::HooksError::ValueNotSerializable) error.
    /// If you want to use a compound value as a replacement, subscribe to `on_value` hook, and
    /// replace the value there.
    ///
    /// Will produce [`HooksError::KeyNotFound`](crate::ser::HooksError::KeyNotFound) error
    /// if the key does not occur during serialization.
    ///
    /// Returns `self` to allow chaining calls.
    pub fn replace_key(
        &mut self,
        key: impl Into<MapKeySelector>,
        new_key: impl Into<StaticValue>,
    ) -> &mut Self {
        self.actions
            .push(MapEntryAction::ReplaceKey(key.into(), new_key.into()));
        self
    }

    /// Rename key of an existing entry.
    ///
    /// Same effect as replacing a key with a new string key. See [`replace_key`](Self::replace_key).
    ///
    /// Returns `self` to allow chaining calls.
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

/// Selector for map entries.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MapKeySelector {
    /// Select entry by matching key.
    ///
    /// For compound values only metadata is stored in [`StaticValue`] and then matched
    /// against the serialized keys. This means that if your map is keyed by compound keys
    /// (tuples, structs etc.), there is no way to reliably select a concrete key.
    ByValue(StaticValue),

    /// Select entry by it's sequential index during serialization.
    ///
    /// This is the position in the order in which the original map entries are fed into the
    /// serializer. Selecting by index obviously only makes sense for ordered maps.
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

/// Location in the map where an entry is inserted.
pub enum MapInsertLocation {
    /// Insert the entry before another entry specified by the selector.
    Before(MapKeySelector),

    /// Insert the entry after another entry specified by the selector.
    After(MapKeySelector),

    /// Insert the entry to the very end of the map.
    End,
}
