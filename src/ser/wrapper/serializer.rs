use std::borrow::Cow;

use serde::{Serialize, Serializer};

use super::map::SerializeMapWrapper;
use super::r#struct::SerializeStructWrapper;
use super::seq::SerializeSeqWrapper;
use super::{
    SeqElementAction, SeqElementActions, SerializableKind, SerializerWrapperHooks, ValueAction,
    VariantAction, VariantActions,
};
use crate::Case;

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
        crate::Value::$variant
    };
    ($variant:ident, $arg:ident) => {
        crate::Value::$variant($arg.into())
    };
    ($variant:ident, $arg0:ident $(, $arg:ident)+) => {
        crate::Value::$variant{
            $arg0 : $arg0.into(),
            $($arg : $arg.into(),)*
        }
    };
}

macro_rules! on_value_callback {
    ($self:ident $variant:ident $(, $arg:ident : $type:ty)*) => {
        match $self.kind {
            SerializableKind::Value => $self
                .hooks
                .on_value($self.serializer, value_ctor!($variant $(, $arg)*))?,
            SerializableKind::MapKey => $self
                .hooks
                .on_map_key($self.serializer, value_ctor!($variant $(, $arg)*))?,
        }
    }
}

macro_rules! value_serialize {
    ($fn:ident, $variant:ident $(, $arg:ident : $type:ty)* $(=> $v:ident : $vt:ident)?) => {
        fn $fn $(<$vt>)? (self, $($arg: $type,)* $($v: &$vt)?) -> Result<Self::Ok, Self::Error>
        $(where $vt: Serialize + ?Sized)?
        {
            let value_action = on_value_callback!(self $variant $(, $arg : $type)*);
            match value_action {
                ValueAction::ContinueSerialization(s) => s.$fn($($arg,)* $($v)?),
                ValueAction::ValueReplaced(r) => r,
            }
        }
    };
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> Serializer for SerializerWrapper<'h, S, H> {
    type Ok = S::Ok;
    type Error = S::Error;
    type SerializeSeq = SerializeSeqWrapper<'h, S, H>;
    type SerializeTuple = SerializeSeqWrapper<'h, S, H>;
    type SerializeTupleStruct = SerializeSeqWrapper<'h, S, H>;
    type SerializeTupleVariant = SerializeSeqWrapper<'h, S, H>;
    type SerializeMap = SerializeMapWrapper<'h, S, H>;
    type SerializeStruct = SerializeStructWrapper<'h, S, H>;
    type SerializeStructVariant = SerializeStructWrapper<'h, S, H>;

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

    value_serialize!(serialize_unit_struct, UnitStruct, name: &'static str);

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        let value_action = on_value_callback!(self UnitVariant,
            name: &'static str,
            variant_index: u32,
            variant: &'static str
        );

        match value_action {
            ValueAction::ValueReplaced(r) => r,
            ValueAction::ContinueSerialization(s) => {
                let variant_actions = self.hooks.on_unit_variant(name, variant, variant_index);
                let (name, variant_index, variant) = apply_variant_actions(
                    name,
                    variant_index,
                    variant,
                    variant_actions,
                    self.hooks,
                );
                s.serialize_unit_variant(name, variant_index, variant)
            }
        }
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
        let value_action = on_value_callback!(self NewtypeVariant,
            name: &'static str,
            variant_index: u32,
            variant: &'static str
        );

        match value_action {
            ValueAction::ValueReplaced(r) => r,
            ValueAction::ContinueSerialization(s) => {
                let variant_actions = self.hooks.on_newtype_variant(name, variant, variant_index);
                let (name, variant_index, variant) = apply_variant_actions(
                    name,
                    variant_index,
                    variant,
                    variant_actions,
                    self.hooks,
                );
                s.serialize_newtype_variant(name, variant_index, variant, value)
            }
        }
    }

    value_serialize!(
        serialize_newtype_struct,
        NewtypeStruct,
        name: &'static str
        =>
        value: T
    );

    value_serialize!(serialize_none, None);
    value_serialize!(
        serialize_some,
        Some
        =>
        value: T
    );

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let value_action = on_value_callback!(self Seq,
            len: Option<usize>
        );
        match value_action {
            ValueAction::ValueReplaced(r) => Ok(SerializeSeqWrapper::new_skipped(r)),
            ValueAction::ContinueSerialization(s) => {
                let actions = self.hooks.on_seq(len);
                s.serialize_seq(if actions.is_empty() { len } else { None })
                    .map(|serialize_seq| {
                        SerializeSeqWrapper::new_wrapped_seq(serialize_seq, self.hooks, actions)
                    })
            }
        }
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        let value_action = on_value_callback!(self Tuple,
            len: usize
        );
        match value_action {
            ValueAction::ValueReplaced(r) => Ok(SerializeSeqWrapper::new_skipped(r)),
            ValueAction::ContinueSerialization(s) => {
                let seq_actions = self.hooks.on_tuple(len);
                if seq_actions_may_change_length(&seq_actions) {
                    // If length may be changed, we force serialization of this tuple
                    // as seq.
                    s.serialize_seq(None).map(|serialize_seq| {
                        SerializeSeqWrapper::new_wrapped_seq(serialize_seq, self.hooks, seq_actions)
                    })
                } else {
                    s.serialize_tuple(len).map(|serialize_tuple| {
                        SerializeSeqWrapper::new_wrapped_tuple(
                            serialize_tuple,
                            self.hooks,
                            seq_actions,
                        )
                    })
                }
            }
        }
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        let value_action = on_value_callback!(self TupleStruct,
            name: &'static str,
            len: usize
        );
        match value_action {
            ValueAction::ValueReplaced(r) => Ok(SerializeSeqWrapper::new_skipped(r)),
            ValueAction::ContinueSerialization(s) => {
                let seq_actions = self.hooks.on_tuple_struct(name, len);
                if seq_actions_may_change_length(&seq_actions) {
                    // If length may be changed, we force serialization of this tuple
                    // as seq.
                    s.serialize_seq(None).map(|serialize_seq| {
                        SerializeSeqWrapper::new_wrapped_seq(serialize_seq, self.hooks, seq_actions)
                    })
                } else {
                    s.serialize_tuple_struct(name, len)
                        .map(|serialize_tuple_struct| {
                            SerializeSeqWrapper::new_wrapped_tuple_struct(
                                serialize_tuple_struct,
                                self.hooks,
                                seq_actions,
                            )
                        })
                }
            }
        }
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let value_action = on_value_callback!(self TupleVariant,
            name: &'static str,
            variant_index: u32,
            variant: &'static str,
            len: usize
        );

        match value_action {
            ValueAction::ValueReplaced(r) => Ok(SerializeSeqWrapper::new_skipped(r)),
            ValueAction::ContinueSerialization(s) => {
                let (variant_actions, seq_actions) =
                    self.hooks
                        .on_tuple_variant(name, variant_index, variant, len);
                if seq_actions_may_change_length(&seq_actions) {
                    // If length may be changed, we force serialization of this tuple
                    // as seq.
                    s.serialize_seq(None).map(|serialize_seq| {
                        SerializeSeqWrapper::new_wrapped_seq(serialize_seq, self.hooks, seq_actions)
                    })
                } else {
                    let (name, variant_index, variant) = apply_variant_actions(
                        name,
                        variant_index,
                        variant,
                        variant_actions,
                        self.hooks,
                    );

                    s.serialize_tuple_variant(name, variant_index, variant, len)
                        .map(|serialize_tuple_variant| {
                            SerializeSeqWrapper::new_wrapped_tuple_variant(
                                serialize_tuple_variant,
                                self.hooks,
                                seq_actions,
                            )
                        })
                }
            }
        }
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let value_action = on_value_callback!(self Map,
            len: Option<usize>
        );
        match value_action {
            ValueAction::ValueReplaced(r) => Ok(SerializeMapWrapper::new_skipped(r)),
            ValueAction::ContinueSerialization(s) => {
                let actions = self.hooks.on_map(len);
                //TODO analyze actions instead of relying on actions.is_empty()
                s.serialize_map(if actions.is_empty() { len } else { None })
                    .map(|serialize_map| {
                        SerializeMapWrapper::new_wrapped(serialize_map, self.hooks, actions)
                    })
            }
        }
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let value_action = on_value_callback!(self Struct,
            name: &'static str,
            len: usize
        );
        match value_action {
            ValueAction::ValueReplaced(r) => Ok(SerializeStructWrapper::new_skipped(r)),
            ValueAction::ContinueSerialization(s) => {
                let (struct_actions, field_actions) = self.hooks.on_struct(len, name);

                SerializeStructWrapper::serialize_struct(
                    s,
                    name,
                    len,
                    self.hooks,
                    struct_actions,
                    field_actions,
                )
            }
        }
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let value_action = on_value_callback!(self StructVariant,
            name: &'static str,
            variant_index: u32,
            variant: &'static str,
            len: usize
        );
        match value_action {
            ValueAction::ValueReplaced(r) => Ok(SerializeStructWrapper::new_skipped(r)),
            ValueAction::ContinueSerialization(s) => {
                let (variant_actions, struct_actions, field_actions) = self
                    .hooks
                    .on_struct_variant(len, name, variant, variant_index);

                let (name, variant_index, variant) = apply_variant_actions(
                    name,
                    variant_index,
                    variant,
                    variant_actions,
                    self.hooks,
                );

                SerializeStructWrapper::serialize_struct_variant(
                    s,
                    name,
                    variant_index,
                    variant,
                    len,
                    self.hooks,
                    struct_actions,
                    field_actions,
                )
            }
        }
    }
}

/// Applies variant actions and return (possibly) new enum name, variant index and variant name.
fn apply_variant_actions(
    name: &'static str,
    variant_index: u32,
    variant: &'static str,
    actions: VariantActions,
    hooks: &impl SerializerWrapperHooks,
) -> (&'static str, u32, &'static str) {
    let mut new_name: Option<Cow<'static, str>> = None;
    let mut enum_case: Option<Case> = None;
    let mut new_variant: Option<Cow<'static, str>> = None;
    let mut variant_case: Option<Case> = None;
    let mut new_variant_index: Option<u32> = None;

    actions.into_iter().rev().for_each(|a| match a {
        VariantAction::RenameEnumCase(c) => {
            enum_case.get_or_insert(c);
        }
        VariantAction::RenameEnum(n) => {
            new_name.get_or_insert(n);
        }
        VariantAction::RenameVariantCase(c) => {
            variant_case.get_or_insert(c);
        }
        VariantAction::RenameVariant(n) => {
            new_variant.get_or_insert(n);
        }
        VariantAction::ChangeVariantIndex(i) => {
            new_variant_index.get_or_insert(i);
        }
    });

    if new_name.is_none() {
        if let Some(c) = enum_case {
            new_name = Some(Case::string_to_case(name, c).into());
        }
    }

    if new_variant.is_none() {
        if let Some(c) = variant_case {
            new_variant = Some(Case::string_to_case(variant, c).into());
        }
    }

    (
        hooks.into_static_str(new_name.unwrap_or(name.into())),
        new_variant_index.unwrap_or(variant_index),
        hooks.into_static_str(new_variant.unwrap_or(variant.into())),
    )
}

fn seq_actions_may_change_length(actions: &SeqElementActions) -> bool {
    actions
        .iter()
        .any(|a| matches!(a, SeqElementAction::Retain(_) | SeqElementAction::Skip(_)))
}
