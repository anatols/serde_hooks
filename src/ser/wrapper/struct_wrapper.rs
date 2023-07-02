use serde::{ser::Error, Serialize, Serializer};

use super::{ValueAction, SerializerWrapperHooks, PathSegment, SerializableWithHooks};

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

impl<'h, S: Serializer, H: SerializerWrapperHooks> serde::ser::SerializeStruct for SerializeStructWrapper<'h, S, H> {
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

        let res = match self.hooks.before_value() {
            ValueAction::GoAhead => {
                let s = SerializableWithHooks {
                    serializable: value,
                    hooks: self.hooks,
                };
                self.serialize_struct.serialize_field(key, &s)
            }
            // ValueAction::Skip => self.skip_field(key),
            ValueAction::Replace => todo!(),
            ValueAction::Error(message) => Err(Self::Error::custom(message)),
        };
        self.hooks.path_pop();
        res
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_struct.end()
    }
}
