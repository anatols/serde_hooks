use std::{cell::Cell, fmt::Display};

use serde::{ser::Impossible, Serialize, Serializer};
use thiserror::Error;

use super::{OnMapEntryActions, SerializableKind, SerializableWithHooks, SerializerWrapperHooks};
use crate::ser::{
    hooks::{MapEntryAction, MapKeySelector},
    path::PathMapKey,
    HooksError, PrimitiveValue,
};

pub struct SerializeMapWrapper<'h, S: Serializer, H: SerializerWrapperHooks> {
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
        let mut replacement_value: Option<PrimitiveValue> = None;
        let mut replacement_key: Option<PrimitiveValue> = None;
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
                        map_key =
                            PathMapKey::from_index_and_primitive_value(map_key.index(), v.clone());
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
                &SerializableWithHooks {
                    serializable: value,
                    hooks: self.hooks,
                    kind: SerializableKind::Value,
                },
            )
        } else {
            self.serialize_map.serialize_entry(
                &SerializableWithHooks {
                    serializable: key,
                    hooks: self.hooks,
                    kind: SerializableKind::MapKey,
                },
                &SerializableWithHooks {
                    serializable: value,
                    hooks: self.hooks,
                    kind: SerializableKind::Value,
                },
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
                    if let Some(primitive_value) = &v {
                        self.serialize_map.serialize_entry(&k, primitive_value)?
                    } else {
                        self.serialize_map.serialize_entry(
                            &k,
                            &SerializableWithHooks {
                                serializable: &v,
                                hooks: self.hooks,
                                kind: SerializableKind::Value,
                            },
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

#[derive(Debug, Error)]
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
        Ok(PathMapKey::Bool(self.entry_index, v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::I8(self.entry_index, v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::I16(self.entry_index, v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::I32(self.entry_index, v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::I64(self.entry_index, v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::U8(self.entry_index, v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::U16(self.entry_index, v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::U32(self.entry_index, v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::U64(self.entry_index, v))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::F32(self.entry_index, v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::F64(self.entry_index, v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::Char(self.entry_index, v))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::Str(self.entry_index, v.to_owned().into()))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::Bytes(self.entry_index))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::None(self.entry_index))
    }

    fn serialize_some<T: ?Sized>(self, v: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        v.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::Unit(self.entry_index))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::UnitStruct(self.entry_index))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(PathMapKey::UnitVariant(self.entry_index))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Ok(PathMapKey::NewtypeStruct(self.entry_index))
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Ok(PathMapKey::NewtypeVariant(self.entry_index))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(MapKeyCaptureError(PathMapKey::Seq(self.entry_index)))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(MapKeyCaptureError(PathMapKey::Tuple(self.entry_index)))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(MapKeyCaptureError(PathMapKey::TupleStruct(
            self.entry_index,
        )))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(MapKeyCaptureError(PathMapKey::TupleVariant(
            self.entry_index,
        )))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(MapKeyCaptureError(PathMapKey::Map(self.entry_index)))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(MapKeyCaptureError(PathMapKey::Struct(self.entry_index)))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(MapKeyCaptureError(PathMapKey::StructVariant(
            self.entry_index,
        )))
    }
}
