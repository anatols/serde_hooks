use serde::{Serialize, Serializer};

use super::{Hooks, MapKey, SerializerWrapper};

pub struct SerializeMapWrapper<'h, S: Serializer, H: Hooks> {
    serialize_map: S::SerializeMap,
    hooks: &'h H,
}

impl<'h, S: Serializer, H: Hooks> SerializeMapWrapper<'h, S, H> {
    pub(super) fn new(serialize_map: S::SerializeMap, hooks: &'h H) -> Self {
        Self {
            serialize_map,
            hooks,
        }
    }
}

impl<'h, S: Serializer, H: Hooks> serde::ser::SerializeMap for SerializeMapWrapper<'h, S, H> {
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
        self.serialize_map.serialize_entry(
            &MapKeyWrapper {
                key,
                hooks: self.hooks,
            },
            &MapValueWrapper {
                value,
                hooks: self.hooks,
            },
        )
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_map.end()
    }
}

pub struct MapKeyWrapper<'s, 'h, T: Serialize + ?Sized, H: Hooks> {
    key: &'s T,
    hooks: &'h H,
}

impl<T: Serialize + ?Sized, H: Hooks> Serialize for MapKeyWrapper<'_, '_, T, H> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.key
            .serialize(MapKeyCapture::new(serializer, self.hooks))
    }
}

pub struct MapValueWrapper<'s, 'h, T: Serialize + ?Sized, H: Hooks> {
    value: &'s T,
    hooks: &'h H,
}

impl<T: Serialize + ?Sized, H: Hooks> Serialize for MapValueWrapper<'_, '_, T, H> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let res = self
            .value
            .serialize(SerializerWrapper::new(serializer, self.hooks));
        self.hooks.path_pop();
        res
    }
}

struct MapKeyCapture<'h, S, H: Hooks> {
    serializer: S,
    hooks: &'h H,
}

impl<'h, S: Serializer, H: Hooks> MapKeyCapture<'h, S, H> {
    fn new(serializer: S, hooks: &'h H) -> Self {
        Self { serializer, hooks }
    }
}

impl<'h, S: Serializer, H: Hooks> Serializer for MapKeyCapture<'h, S, H> {
    type Ok = S::Ok;
    type Error = S::Error;
    type SerializeSeq = S::SerializeSeq;
    type SerializeTuple = S::SerializeTuple;
    type SerializeTupleStruct = S::SerializeTupleStruct;
    type SerializeTupleVariant = S::SerializeTupleVariant;
    type SerializeMap = S::SerializeMap;
    type SerializeStruct = S::SerializeStruct;
    type SerializeStructVariant = S::SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.hooks.path_push(MapKey::Bool(v).into());
        self.serializer.serialize_bool(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_i8(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_i16(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_i32(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_i64(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_u8(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_u16(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_u32(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_u64(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_f32(v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_f64(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_char(v)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.hooks.path_push(MapKey::Str(v.to_string()).into());
        self.serializer.serialize_str(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_bytes(v)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_none()
    }

    fn serialize_some<T: ?Sized>(self, v: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.serializer.serialize_some(v)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_unit()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_unit_struct(name)
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serializer
            .serialize_unit_variant(name, variant_index, variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.serializer.serialize_newtype_struct(name, value)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.serializer
            .serialize_newtype_variant(name, variant_index, variant, value)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.serializer.serialize_seq(len)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serializer.serialize_tuple(len)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serializer.serialize_tuple_struct(name, len)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serializer
            .serialize_tuple_variant(name, variant_index, variant, len)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.serializer.serialize_map(len)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serializer.serialize_struct(name, len)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serializer
            .serialize_struct_variant(name, variant_index, variant, len)
    }
}
