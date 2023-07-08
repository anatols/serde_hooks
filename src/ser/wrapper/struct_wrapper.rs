use serde::{ser::Error, Serialize, Serializer};

use super::{PathSegment, SerializableWithHooks, SerializerWrapperHooks};

pub struct SerializeStructWrapper<'h, S: Serializer, H: SerializerWrapperHooks> {
    serialize_struct: S::SerializeStruct,
    hooks: &'h H,
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> SerializeStructWrapper<'h, S, H> {
    pub(super) fn new(serialize_struct: S::SerializeStruct, hooks: &'h H) -> Self {
        Self {
            serialize_struct,
            hooks,
        }
    }
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> serde::ser::SerializeStruct
    for SerializeStructWrapper<'h, S, H>
{
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        println!("serialize_field {key}");
        self.hooks.path_push(PathSegment::StructField(key));

        let s = SerializableWithHooks {
            serializable: value,
            hooks: self.hooks,
        };
        let res = self.serialize_struct.serialize_field(key, &s);

        self.hooks.path_pop();
        res
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_struct.end()
    }
}
