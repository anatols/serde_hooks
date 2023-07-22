use serde::{Serialize, Serializer};

use super::map::SerializeMapWrapper;
use super::r#struct::SerializeStructWrapper;
use super::{SerializableKind, SerializerWrapperHooks};
use crate::ser::scope::OnValueAction;

pub(crate) struct SerializerWrapper<'h, S, H: SerializerWrapperHooks> {
    serializer: S,
    hooks: &'h H,
    kind: SerializableKind,
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> SerializerWrapper<'h, S, H> {
    pub(crate) fn new(serializer: S, hooks: &'h H, kind: SerializableKind) -> Self {
        Self {
            serializer,
            hooks,
            kind,
        }
    }
}

macro_rules! value_ctor {
    ($variant:ident) => {
        crate::Value::Primitive(crate::PrimitiveValue::$variant)
    };
    ($variant:ident, $arg:ident) => {
        crate::Value::Primitive(crate::PrimitiveValue::$variant($arg.into()))
    };
    ($variant:ident, $arg0:ident $(, $arg:ident)+) => {
        crate::Value::Primitive(crate::PrimitiveValue::$variant{
            $arg0 : $arg0.into(),
            $($arg : $arg.into(),)*
        })
    };
}

macro_rules! value_serialize {
    ($fn:ident, $variant:ident $(, $arg:ident : $type:ty)* $(=> $v:ident : $vt:ident)?) => {
        fn $fn $(<$vt>)? (self, $($arg: $type,)* $($v: &$vt)?) -> Result<Self::Ok, Self::Error>
        $(where $vt: Serialize + ?Sized)?
        {
            let value = value_ctor!($variant $(, $arg)*);
            let value_action = match self.kind {
                SerializableKind::Value => self
                    .hooks
                    .on_value(self.serializer, value),
                SerializableKind::MapKey => self
                    .hooks
                    .on_map_key(self.serializer, value),
            };

            match value_action {
                OnValueAction::ContinueSerialization(s) => s.$fn($($arg,)* $($v)?),
                OnValueAction::ValueReplaced(r) => r,
            }
        }
    };
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

    value_serialize!(serialize_bool, Bool, v: bool);
    value_serialize!(serialize_i8, I8, v: i8);
    value_serialize!(serialize_i16, I16, v: i16);
    value_serialize!(serialize_i32, I32, v: i32);
    value_serialize!(serialize_i64, I64, v: i64);
    value_serialize!(serialize_u8, U8, v: u8);
    value_serialize!(serialize_u16, U16, v: u16);
    value_serialize!(serialize_u32, U32, v: u32);
    value_serialize!(serialize_u64, U64, v: u64);
    value_serialize!(serialize_f32, F32, v: f32);
    value_serialize!(serialize_f64, F64, v: f64);
    value_serialize!(serialize_char, Char, v: char);
    value_serialize!(serialize_str, Str, v: &str);
    value_serialize!(serialize_bytes, Bytes, v: &[u8]);
    value_serialize!(serialize_unit, Unit);
    value_serialize!(serialize_none, None);
    value_serialize!(serialize_unit_struct, UnitStruct, name: &'static str);
    value_serialize!(
        serialize_unit_variant,
        UnitVariant,
        name: &'static str,
        variant_index: u32,
        variant: &'static str
    );

    value_serialize!(
        serialize_newtype_struct,
        NewtypeStruct,
        name: &'static str
        =>
        value: T
    );

    fn serialize_some<T: ?Sized>(self, v: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.serializer.serialize_some(v)
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
        println!("serialize_map {len:?}");

        let actions = self.hooks.on_map(len);
        self.serializer
            .serialize_map(if actions.is_empty() { len } else { None })
            .map(|serialize_map| SerializeMapWrapper::new(serialize_map, self.hooks, actions))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        println!("serialize_struct {name} {len}");

        let actions = self.hooks.on_struct(len, name);
        self.serializer
            .serialize_struct(name, len)
            .map(|serialize_struct| {
                SerializeStructWrapper::new(serialize_struct, self.hooks, actions)
            })
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
