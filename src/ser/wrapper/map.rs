use std::borrow::Cow;
use std::{cell::Cell, fmt::Display};

use serde::{ser::Impossible, Serialize, Serializer};

use crate::path::PathMapKey;
use crate::ser::scope::{MapEntryAction, OnMapEntryActions};
use crate::ser::{HooksError, MapKeySelector};
use crate::Value;

use super::{SerializableKind, SerializableWithHooks, SerializerWrapperHooks};

pub(crate) struct SerializeMapWrapper<'h, S: Serializer, H: SerializerWrapperHooks> {
    serialize_map: S::SerializeMap,
    hooks: &'h H,
    actions: OnMapEntryActions,
    have_retains: bool,
    entry_index: Cell<usize>,
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> SerializeMapWrapper<'h, S, H> {
    pub(super) fn new(
        serialize_map: S::SerializeMap,
        hooks: &'h H,
        actions: OnMapEntryActions,
    ) -> Self {
        Self {
            serialize_map,
            hooks,
            have_retains: actions
                .iter()
                .any(|a| matches!(a, MapEntryAction::Retain(_))),
            actions,
            entry_index: Cell::new(0),
        }
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
        println!("serialize_key");
        self.serialize_map.serialize_key(key)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        println!("serialize_value");
        self.serialize_map.serialize_value(value)
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
        println!("serialize_entry");
        let mut map_key = MapKeyCapture::capture(self.entry_index.get(), key);

        let mut retain_entry = false;
        let mut skip_entry = false;
        let mut replace_entry = false;
        let mut replacement_value: Option<Value> = None;
        let mut replacement_key: Option<Value> = None;
        let mut error = None;

        self.actions.retain_mut(|a| {
            if error.is_some() {
                return true;
            }
            match a {
                MapEntryAction::Retain(k) => {
                    let matches = k.matches_path_key(&map_key);
                    if matches {
                        retain_entry = true;
                    }
                    !matches
                }
                MapEntryAction::Skip(k) => {
                    let matches = k.matches_path_key(&map_key);
                    if matches {
                        skip_entry = true;
                    }
                    !matches
                }
                MapEntryAction::Add(k, _) => {
                    let matches = k.matches_path_key(&map_key);
                    if matches {
                        error = Some(HooksError::KeyAlreadyPresent(k.clone()));
                    }
                    true
                }
                MapEntryAction::Replace(k, v) | MapEntryAction::ReplaceOrAdd(k, v) => {
                    let matches = k.matches_path_key(&map_key);
                    if matches {
                        replace_entry = true;
                        replacement_value = v.take();
                    }
                    !matches
                }
                MapEntryAction::ReplaceKey(k, v) => {
                    let matches = k.matches_path_key(&map_key);
                    if matches {
                        map_key = PathMapKey::new(map_key.index(), v.clone());
                        replacement_key = Some(v.clone());
                    }
                    !matches
                }
            }
        });

        if let Some(err) = error {
            self.hooks.on_error::<S>(err)?;
        }

        if self.have_retains && !retain_entry {
            skip_entry = true;
        }

        self.hooks.path_push(map_key.into());

        if let Some(replacement_value) = &replacement_value {
            replacement_value
                .check_if_can_serialize()
                .or_else(|err| self.hooks.on_error::<S>(err))?;
        }

        if let Some(replacement_key) = &replacement_key {
            replacement_key
                .check_if_can_serialize()
                .or_else(|err| self.hooks.on_error::<S>(err))?;
        }

        let res = if replace_entry {
            if let Some(replacement_key) = &replacement_key {
                self.serialize_map
                    .serialize_entry(replacement_key, &replacement_value)
            } else {
                self.serialize_map.serialize_entry(key, &replacement_value)
            }
        } else if skip_entry {
            Ok(())
        } else if let Some(replacement_key) = &replacement_key {
            self.serialize_map.serialize_entry(
                replacement_key,
                &SerializableWithHooks::new(value, self.hooks, SerializableKind::Value),
            )
        } else {
            self.serialize_map.serialize_entry(
                &SerializableWithHooks::new(key, self.hooks, SerializableKind::MapKey),
                &SerializableWithHooks::new(value, self.hooks, SerializableKind::Value),
            )
        };

        self.hooks.path_pop();
        self.entry_index.replace(self.entry_index.get() + 1);
        res
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        for a in self.actions {
            match a {
                MapEntryAction::Add(MapKeySelector::ByValue(k), v)
                | MapEntryAction::ReplaceOrAdd(MapKeySelector::ByValue(k), v) => {
                    if let Some(value) = &v {
                        self.serialize_map.serialize_entry(&k, value)?
                    } else {
                        self.serialize_map.serialize_entry(
                            &k,
                            &SerializableWithHooks::new(&v, self.hooks, SerializableKind::Value),
                        )?
                    }
                }
                MapEntryAction::Add(MapKeySelector::ByIndex(index), _)
                | MapEntryAction::ReplaceOrAdd(MapKeySelector::ByIndex(index), _) => self
                    .hooks
                    .on_error::<S>(HooksError::CannotAddEntryByIndex(index))?,
                MapEntryAction::Replace(k, _)
                | MapEntryAction::Retain(k)
                | MapEntryAction::Skip(k)
                | MapEntryAction::ReplaceKey(k, _) => {
                    self.hooks.on_error::<S>(HooksError::KeyNotFound(k))?
                }
            }
        }

        self.serialize_map.end()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("")]
struct MapKeyCaptureError(PathMapKey);

impl serde::ser::Error for MapKeyCaptureError {
    fn custom<T>(_msg: T) -> Self
    where
        T: Display,
    {
        unimplemented!()
    }
}

struct MapKeyCapture {
    entry_index: usize,
}

impl MapKeyCapture {
    fn capture<K>(entry_index: usize, key: &K) -> PathMapKey
    where
        K: Serialize + ?Sized,
    {
        match key.serialize(MapKeyCapture { entry_index }) {
            Ok(map_key) => map_key,                      // primitive keys via Ok
            Err(MapKeyCaptureError(map_key)) => map_key, // complex keys via Err
        }
    }
}

impl Serializer for MapKeyCapture {
    type Ok = PathMapKey;
    type Error = MapKeyCaptureError;
    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::new(self.entry_index, Value::Bool(v)))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::new(self.entry_index, Value::I8(v)))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::new(self.entry_index, Value::I16(v)))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::new(self.entry_index, Value::I32(v)))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::new(self.entry_index, Value::I64(v)))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::new(self.entry_index, Value::U8(v)))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::new(self.entry_index, Value::U16(v)))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::new(self.entry_index, Value::U32(v)))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::new(self.entry_index, Value::U64(v)))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::new(self.entry_index, Value::F32(v)))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::new(self.entry_index, Value::F64(v)))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::new(self.entry_index, Value::Char(v)))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::new(
            self.entry_index,
            //TODO cloning the string here for each map key is suboptimal
            Value::Str(v.to_owned().into()),
        ))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::new(
            self.entry_index,
            Value::Bytes(Cow::Borrowed(&[])),
        ))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::new(self.entry_index, Value::None))
    }

    fn serialize_some<T: ?Sized>(self, v: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        v.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::new(self.entry_index, Value::Unit))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::new(self.entry_index, Value::UnitStruct(name)))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::new(
            self.entry_index,
            Value::UnitVariant {
                name,
                variant_index,
                variant,
            },
        ))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Ok(PathMapKey::new(
            self.entry_index,
            Value::NewtypeStruct(name),
        ))
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
        Ok(PathMapKey::new(
            self.entry_index,
            Value::NewtypeVariant {
                name,
                variant_index,
                variant,
            },
        ))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(MapKeyCaptureError(PathMapKey::new(
            self.entry_index,
            Value::Seq(len),
        )))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(MapKeyCaptureError(PathMapKey::new(
            self.entry_index,
            Value::Tuple(len),
        )))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(MapKeyCaptureError(PathMapKey::new(
            self.entry_index,
            Value::TupleStruct { name, len },
        )))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(MapKeyCaptureError(PathMapKey::new(
            self.entry_index,
            Value::TupleVariant {
                name,
                variant_index,
                variant,
                len,
            },
        )))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(MapKeyCaptureError(PathMapKey::new(
            self.entry_index,
            Value::Map(len),
        )))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(MapKeyCaptureError(PathMapKey::new(
            self.entry_index,
            Value::Struct { name, len },
        )))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(MapKeyCaptureError(PathMapKey::new(
            self.entry_index,
            Value::StructVariant {
                name,
                variant_index,
                variant,
                len,
            },
        )))
    }
}
