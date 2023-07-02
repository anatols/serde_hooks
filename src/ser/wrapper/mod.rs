use serde::{ser::Error, Serialize, Serializer};

mod map_wrapper;
mod struct_wrapper;

use crate::ser::path::PathSegment;
use map_wrapper::SerializeMapWrapper;
use struct_wrapper::SerializeStructWrapper;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ValueAction {
    GoAhead,
    Replace,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum StructAction<S>
where
    S: Serializer,
    // F: Fn(S) -> Result<S::Ok, S::Error>
{
    RetainFields(Vec<&'static str>),
    AddOrReplaceField {
        name: &'static str,
        with: fn(S) -> Result<S::Ok, S::Error>,
    },
    RemoveField {
        name: &'static str,
    },
}

pub trait SerializerWrapperHooks {
    fn path_push(&self, segment: PathSegment);
    fn path_pop(&self);

    fn before_value(&self) -> ValueAction;

    fn before_struct<S: Serializer>(&self) -> Vec<StructAction<S>>;

    fn before_map<S: Serializer>(&self) -> Vec<StructAction<S>>;
}

pub(super) struct SerializerWrapper<'h, S, H: SerializerWrapperHooks> {
    serializer: S,
    hooks: &'h H,
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> SerializerWrapper<'h, S, H> {
    pub(super) fn new(serializer: S, hooks: &'h H) -> Self {
        Self { serializer, hooks }
    }
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> Serializer for SerializerWrapper<'h, S, H> {
    type Ok = S::Ok;
    type Error = S::Error;
    type SerializeSeq = S::SerializeSeq;
    type SerializeTuple = S::SerializeTuple;
    type SerializeTupleStruct = S::SerializeTupleStruct;
    type SerializeTupleVariant = S::SerializeTupleVariant;
    type SerializeMap = SerializeMapWrapper<'h, S, H>;
    type SerializeStruct = SerializeStructWrapper<'h, S, H>;
    type SerializeStructVariant = S::SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        match self.hooks.before_value() {
            ValueAction::GoAhead => self.serializer.serialize_bool(v),
            // ValueAction::Skip => unreachable!(),
            ValueAction::Replace => todo!(),
            ValueAction::Error(message) => Err(Self::Error::custom(message)),
        }
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_i8(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_i16(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        println!("serialize_i32 {v}");
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
        self.serializer
            .serialize_map(len)
            .map(|serialize_map| SerializeMapWrapper::new(serialize_map, self.hooks))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        println!("serialize_struct {name} {len}");

        let actions = self.hooks.before_struct::<S>();

        self.serializer
            .serialize_struct(name, len)
            .map(|serialize_struct| SerializeStructWrapper::new(serialize_struct, self.hooks))
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

pub struct SerializableWithHooks<'s, 'h, T: Serialize + ?Sized, H: SerializerWrapperHooks> {
    serializable: &'s T,
    hooks: &'h H,
}

impl<T: Serialize + ?Sized, H: SerializerWrapperHooks> Serialize
    for SerializableWithHooks<'_, '_, T, H>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.serializable
            .serialize(SerializerWrapper::new(serializer, self.hooks))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use mockall::predicate::*;
    use mockall::*;

    mock! {
        Hooks {
            fn path_push(&self, segment: PathSegment);
            fn path_pop(&self);
            fn before_serialize(&self) -> ValueAction;
            // fn before_struct<S: Serializer + 'static>(&self) -> Vec<StructAction<S>>;
        }
    }

    impl SerializerWrapperHooks for MockHooks {
        fn path_push(&self, segment: PathSegment) {
            MockHooks::path_push(self, segment)
        }

        fn path_pop(&self) {
            MockHooks::path_pop(self)
        }

        fn before_struct<S: Serializer>(&self) -> Vec<StructAction<S>> {
            // MockHooks::before_struct(self)
            vec![StructAction::AddOrReplaceField {
                name: "field",
                with: |serializer| serializer.serialize_str("fake"),
            }]
        }

        fn before_value(&self) -> ValueAction {
            MockHooks::before_serialize(self)
        }

        fn before_map<S: Serializer>(&self) -> Vec<StructAction<S>> {
            todo!()
        }
    }

    #[test]
    fn test() {
        #[derive(Serialize)]
        struct Child {
            name: &'static str,
        }

        #[derive(Serialize)]
        struct S {
            field: bool,
            // #[serde(flatten)]
            child: Child,
            map: HashMap<String, i32>,
        }

        let mut hooks = MockHooks::new();

        hooks
            .expect_path_push()
            .returning(|segment| println!("### path_push {segment:?}"));

        hooks
            .expect_path_pop()
            .returning(|| println!("### path_pop"));

        hooks
            .expect_before_serialize()
            .return_const(ValueAction::GoAhead);

        let s = S {
            field: true,
            child: Child { name: "child" },
            map: [("foo".to_string(), 123), ("bar".to_string(), 234)]
                .into_iter()
                .collect(),
        };

        let wrapped = SerializableWithHooks {
            serializable: &s,
            hooks: &hooks,
        };

        print!("{}", serde_json::to_string_pretty(&wrapped).unwrap());
    }
}
