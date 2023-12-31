use std::borrow::Cow;
use std::marker::PhantomData;
use std::{cell::Cell, fmt::Display};

use serde::{ser::Impossible, Serialize, Serializer};
use smallvec::SmallVec;

use crate::path::PathMapKey;
use crate::ser::{HooksError, MapInsertLocation};
use crate::{Case, PathSegment, StaticValue, Value};

use super::{
    MapEntryAction, MapEntryActions, SerializableKind, SerializableWithHooks,
    SerializerWrapperHooks,
};

#[allow(clippy::large_enum_variant)]
pub(crate) enum SerializeMapWrapper<'h, S: Serializer, H: SerializerWrapperHooks> {
    Wrapped {
        serialize_map: S::SerializeMap,
        hooks: &'h H,
        actions: MapEntryActions,
        have_retains: bool,
        entry_index: Cell<usize>,
        str_key_buffer: String, // reusable String for &str type keys to reduce allocations
        rename_all: Option<Case>,
    },
    Skipped {
        end_result: Result<S::Ok, S::Error>,
    },
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> SerializeMapWrapper<'h, S, H> {
    pub(super) fn serialize_map(
        serializer: S,
        len: Option<usize>,
        hooks: &'h H,
        actions: MapEntryActions,
    ) -> Result<Self, S::Error> {
        // If there's any potential of entries being skipped or added, don't feed map length hint
        // to the serializer.
        let len = len.and_then(|len| {
            if actions.iter().any(|a| {
                matches!(
                    a,
                    MapEntryAction::Retain(_)
                        | MapEntryAction::Skip(_)
                        | MapEntryAction::Insert(_, _, _)
                )
            }) {
                None
            } else {
                Some(len)
            }
        });

        Ok(Self::Wrapped {
            serialize_map: serializer.serialize_map(len)?,
            hooks,
            have_retains: have_retains(&actions),
            rename_all: rename_all(&actions),
            actions,
            entry_index: Cell::new(0),
            str_key_buffer: String::default(),
        })
    }

    pub(super) fn new_skipped(end_result: Result<S::Ok, S::Error>) -> Self {
        Self::Skipped { end_result }
    }

    fn insert_entry(
        serialize_map: &mut S::SerializeMap,
        hooks: &'h H,
        entry_index: usize,
        key: StaticValue,
        value: StaticValue,
    ) -> Result<(), S::Error> {
        use serde::ser::SerializeMap;

        key.check_if_can_serialize()
            .or_else(|err| hooks.on_error::<S>(err))?;

        value
            .check_if_can_serialize()
            .or_else(|err| hooks.on_error::<S>(err))?;

        let path_map_key = PathMapKey::new(entry_index, key.clone());
        hooks.path_push(path_map_key.into());
        let res = serialize_map.serialize_entry(
            &key,
            &SerializableWithHooks::new(&value, hooks, SerializableKind::Value),
        );
        hooks.path_pop();
        res?;

        Ok(())
    }
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> serde::ser::SerializeMap
    for SerializeMapWrapper<'h, S, H>
{
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        match self {
            SerializeMapWrapper::Skipped { .. } => Ok(()),
            SerializeMapWrapper::Wrapped { serialize_map, .. } => serialize_map.serialize_key(key),
        }
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        match self {
            SerializeMapWrapper::Skipped { .. } => Ok(()),
            SerializeMapWrapper::Wrapped { serialize_map, .. } => {
                serialize_map.serialize_value(value)
            }
        }
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: Serialize,
        V: Serialize,
    {
        match self {
            SerializeMapWrapper::Skipped { .. } => Ok(()),
            SerializeMapWrapper::Wrapped {
                serialize_map,
                hooks,
                actions,
                have_retains,
                entry_index,
                str_key_buffer,
                rename_all,
            } => {
                let mut map_key_value = MapKeyCapture::capture(key, std::mem::take(str_key_buffer));

                let mut retain_entry = false;
                let mut skip_entry = false;
                let mut replacement_value: Option<Value> = None;
                let mut replacement_key: Option<Value> = None;
                let mut insert_before: SmallVec<[(StaticValue, StaticValue); 2]> =
                    Default::default();
                let mut insert_after: SmallVec<[(StaticValue, StaticValue); 2]> =
                    Default::default();

                actions.retain_mut(|a| match a {
                    MapEntryAction::Retain(k) => {
                        let matches = k.matches_path_key(&map_key_value, entry_index.get());
                        if matches {
                            retain_entry = true;
                        }
                        !matches
                    }
                    MapEntryAction::Skip(k) => {
                        let matches = k.matches_path_key(&map_key_value, entry_index.get());
                        if matches {
                            skip_entry = true;
                        }
                        !matches
                    }
                    MapEntryAction::Insert(k, v, location) => match location {
                        MapInsertLocation::Before(before) => {
                            let matches =
                                before.matches_path_key(&map_key_value, entry_index.get());
                            if matches {
                                insert_before.push((k.clone(), v.clone()));
                            }
                            !matches
                        }
                        MapInsertLocation::After(after) => {
                            let matches = after.matches_path_key(&map_key_value, entry_index.get());
                            if matches {
                                insert_after.push((k.clone(), v.clone()));
                            }
                            !matches
                        }
                        MapInsertLocation::End => true,
                    },
                    MapEntryAction::ReplaceValue(k, v) => {
                        let matches = k.matches_path_key(&map_key_value, entry_index.get());
                        if matches {
                            replacement_value = Some(v.clone());
                        }
                        !matches
                    }
                    MapEntryAction::ReplaceKey(k, v) => {
                        let matches = k.matches_path_key(&map_key_value, entry_index.get());
                        if matches {
                            map_key_value = v.clone();
                            replacement_key = Some(v.clone());
                        }
                        !matches
                    }
                    MapEntryAction::RenameAllCase(_) => false,
                    MapEntryAction::RenameCase(k, case) => {
                        let matches = k.matches_path_key(&map_key_value, entry_index.get());
                        if matches {
                            if let Value::Str(s) = &map_key_value {
                                replacement_key = Some(Case::string_to_case(s, *case).into());
                            }
                        }
                        !matches
                    }
                });

                if *have_retains && !retain_entry {
                    skip_entry = true;
                }

                // Insert entries before
                for (k, v) in insert_before {
                    Self::insert_entry(serialize_map, hooks, entry_index.get(), k, v)?;
                }

                let res = if skip_entry {
                    Ok(())
                } else {
                    if replacement_key.is_none() {
                        if let Some(case) = rename_all {
                            if let Value::Str(s) = &map_key_value {
                                replacement_key = Some(Case::string_to_case(s, *case).into());
                            }
                        }
                    }

                    let path_map_key = PathMapKey::new(entry_index.get(), map_key_value);
                    hooks.path_push(path_map_key.into());

                    if let Some(replacement_value) = &replacement_value {
                        replacement_value
                            .check_if_can_serialize()
                            .or_else(|err| hooks.on_error::<S>(err))?;
                    }

                    if let Some(replacement_key) = &replacement_key {
                        replacement_key
                            .check_if_can_serialize()
                            .or_else(|err| hooks.on_error::<S>(err))?;
                    }

                    let res = match (&replacement_key, &replacement_value) {
                        (None, None) => serialize_map.serialize_entry(
                            &SerializableWithHooks::new(key, *hooks, SerializableKind::MapKey),
                            &SerializableWithHooks::new(value, *hooks, SerializableKind::Value),
                        ),
                        (None, Some(v)) => serialize_map.serialize_entry(
                            &SerializableWithHooks::new(key, *hooks, SerializableKind::MapKey),
                            v,
                        ),
                        (Some(k), None) => serialize_map.serialize_entry(
                            k,
                            &SerializableWithHooks::new(value, *hooks, SerializableKind::Value),
                        ),
                        (Some(k), Some(v)) => serialize_map.serialize_entry(k, v),
                    };

                    let segment = hooks.path_pop();

                    // Trying to reclaim the reusable string buffer from the popped path segment
                    if let PathSegment::MapEntry(PathMapKey {
                        value: Value::Str(Cow::Owned(mut s)),
                        ..
                    }) = segment
                    {
                        std::mem::swap(str_key_buffer, &mut s)
                    }

                    res
                };

                // Insert entries after
                for (k, v) in insert_after {
                    Self::insert_entry(serialize_map, hooks, entry_index.get(), k, v)?;
                }

                entry_index.replace(entry_index.get() + 1);

                res
            }
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            SerializeMapWrapper::Skipped { end_result } => end_result,
            SerializeMapWrapper::Wrapped {
                mut serialize_map,
                hooks,
                actions,
                entry_index,
                ..
            } => {
                for a in actions {
                    match a {
                        MapEntryAction::Insert(k, v, l) => match l {
                            MapInsertLocation::Before(location_selector)
                            | MapInsertLocation::After(location_selector) => {
                                hooks.on_error::<S>(HooksError::KeyNotFound(location_selector))?
                            }
                            MapInsertLocation::End => Self::insert_entry(
                                &mut serialize_map,
                                hooks,
                                entry_index.get(),
                                k,
                                v,
                            )?,
                        },
                        MapEntryAction::ReplaceValue(k, _)
                        | MapEntryAction::Retain(k)
                        | MapEntryAction::Skip(k)
                        | MapEntryAction::ReplaceKey(k, _)
                        | MapEntryAction::RenameCase(k, _) => {
                            hooks.on_error::<S>(HooksError::KeyNotFound(k))?
                        }
                        MapEntryAction::RenameAllCase(_) => {}
                    }
                }

                serialize_map.end()
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("")]
struct MapKeyCaptureError<'b>(Value<'b>);

impl serde::ser::Error for MapKeyCaptureError<'_> {
    fn custom<T>(_msg: T) -> Self
    where
        T: Display,
    {
        unimplemented!()
    }
}

struct MapKeyCapture<'b> {
    str_buffer: String,
    marker: PhantomData<&'b ()>,
}

impl MapKeyCapture<'_> {
    fn capture<'b, K>(key: &K, str_buffer: String) -> Value<'b>
    where
        K: Serialize + ?Sized,
    {
        match key.serialize(MapKeyCapture {
            str_buffer,
            marker: PhantomData,
        }) {
            Ok(v) => v,                      // primitive keys via Ok
            Err(MapKeyCaptureError(v)) => v, // complex keys via Err
        }
    }
}

impl<'b> Serializer for MapKeyCapture<'b> {
    type Ok = Value<'b>;
    type Error = MapKeyCaptureError<'b>;
    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::I8(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::I16(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::I32(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::I64(v))
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        Ok(Value::I128(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::U8(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::U16(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::U32(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::U64(v))
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        Ok(Value::U128(v))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::F32(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::F64(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Char(v))
    }

    fn serialize_str(mut self, v: &str) -> Result<Self::Ok, Self::Error> {
        // Putting the value in a reusable buffer to avoid certain allocation per key
        self.str_buffer.clear();
        self.str_buffer.push_str(v);
        Ok(Value::Str(Cow::Owned(self.str_buffer)))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bytes(Cow::Borrowed(&[])))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::None)
    }

    fn serialize_some<T: ?Sized>(self, v: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        v.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Unit)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::UnitStruct(name))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(Value::UnitVariant {
            name,
            variant_index,
            variant,
        })
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Ok(Value::NewtypeStruct(name))
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Ok(Value::NewtypeVariant {
            name,
            variant_index,
            variant,
        })
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(MapKeyCaptureError(Value::Seq(len)))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(MapKeyCaptureError(Value::Tuple(len)))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(MapKeyCaptureError(Value::TupleStruct { name, len }))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(MapKeyCaptureError(Value::TupleVariant {
            name,
            variant_index,
            variant,
            len,
        }))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(MapKeyCaptureError(Value::Map(len)))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(MapKeyCaptureError(Value::Struct { name, len }))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(MapKeyCaptureError(Value::StructVariant {
            name,
            variant_index,
            variant,
            len,
        }))
    }
}

fn have_retains(entry_actions: &MapEntryActions) -> bool {
    entry_actions
        .iter()
        .any(|a| matches!(a, MapEntryAction::Retain(_)))
}

fn rename_all(entry_actions: &MapEntryActions) -> Option<Case> {
    entry_actions.iter().rev().find_map(|a| match a {
        MapEntryAction::RenameAllCase(case) => Some(*case),
        _ => None,
    })
}
