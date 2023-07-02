use std::{fmt::Display, marker::PhantomData};

use serde::{ser::Error, Serialize, Serializer};

mod map_wrapper;
mod struct_wrapper;

use map_wrapper::SerializeMapWrapper;
use struct_wrapper::SerializeStructWrapper;

#[derive(Debug, Clone)]
pub enum MapKey {
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Char(char),
    Str(String),
    Bytes(usize),
    None(usize),
    Some(usize),
    Unit(usize),
    UnitStruct(usize),
    UnitVariant(usize),
    NewtypeStruct(usize),
    NewtypeVariant(usize),
    Seq(usize),
    Tuple(usize),
    TupleStruct(usize),
    TupleVariant(usize),
    Map(usize),
    Struct(usize),
    StructVariant(usize),
}

#[derive(Debug, Clone)]
pub enum PathSegment {
    MapKey(MapKey),
    StructField(&'static str),
    SeqIndex(usize),
}

impl From<MapKey> for PathSegment {
    fn from(map_key: MapKey) -> Self {
        PathSegment::MapKey(map_key)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Action {
    GoAhead,
    Skip,
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

pub trait Hooks {
    fn path_push(&self, segment: PathSegment);
    fn path_pop(&self);

    fn before_struct<S: Serializer>(&self) -> Vec<StructAction<S>>;

    fn before_serialize(&self) -> Action;
}

pub(super) struct SerializerWrapper<'h, S, H: Hooks> {
    serializer: S,
    hooks: &'h H,
}

impl<'h, S: Serializer, H: Hooks> SerializerWrapper<'h, S, H> {
    pub(super) fn new(serializer: S, hooks: &'h H) -> Self {
        Self { serializer, hooks }
    }
}

impl<'h, S: Serializer, H: Hooks> Serializer for SerializerWrapper<'h, S, H> {
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
        match self.hooks.before_serialize() {
            Action::GoAhead => self.serializer.serialize_bool(v),
            Action::Skip => unreachable!(),
            Action::Replace => todo!(),
            Action::Error(message) => Err(Self::Error::custom(message)),
        }
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

pub struct SerializableWithHooks<'s, T: Serialize + ?Sized, H: Hooks> {
    serializable: &'s T,
    hooks: H,
}

impl<'s, T: Serialize + ?Sized, H: Hooks> SerializableWithHooks<'s, T, H> {
    pub(super) fn new(serializable: &'s T, hooks: H) -> Self {
        Self {
            serializable,
            hooks,
        }
    }
}

impl<T: Serialize + ?Sized, H: Hooks> Serialize for SerializableWithHooks<'_, T, H> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.serializable
            .serialize(SerializerWrapper::new(serializer, &self.hooks))
    }
}

pub struct SerializableWithHooksRef<'s, 'h, T: Serialize + ?Sized, H: Hooks> {
    serializable: &'s T,
    hooks: &'h H,
}

impl<T: Serialize + ?Sized, H: Hooks> Serialize for SerializableWithHooksRef<'_, '_, T, H> {
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
            fn before_serialize(&self) -> Action;
            // fn before_struct<S: Serializer + 'static>(&self) -> Vec<StructAction<S>>;
        }
    }

    impl Hooks for MockHooks {
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

        fn before_serialize(&self) -> Action {
            MockHooks::before_serialize(self)
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
            .return_const(Action::GoAhead);

        let s = S {
            field: true,
            child: Child { name: "child" },
            map: [("foo".to_string(), 123), ("bar".to_string(), 234)]
                .into_iter()
                .collect(),
        };

        let wrapped = SerializableWithHooks::new(&s, hooks);

        print!("{}", serde_json::to_string_pretty(&wrapped).unwrap());
    }
}
