use serde::{Serialize, Serializer};

use super::Hooks;

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

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_map.end()
    }
}
