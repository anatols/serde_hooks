use std::borrow::Cow;

use serde::ser::{SerializeMap, SerializeStruct, SerializeStructVariant};
use serde::{Serialize, Serializer};

use crate::ser::HooksError;
use crate::{Case, Value};

use super::flatten::{FlattenError, FlattenSerializer};
use super::map::SerializeMapWrapper;
use super::{
    PathSegment, SerializableKind, SerializableWithHooks, SerializerWrapperHooks, StructActions,
    StructFieldAction, StructFieldActions,
};

#[allow(clippy::enum_variant_names)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum Wrap<'h, S: Serializer, H: SerializerWrapperHooks> {
    SerializeStruct(S::SerializeStruct),
    SerializeStructVariant(S::SerializeStructVariant),
    SerializeAsMap(SerializeMapWrapper<'h, S, H>),
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> Wrap<'h, S, H> {
    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), S::Error>
    where
        T: Serialize,
    {
        match self {
            Wrap::SerializeStruct(s) => s.serialize_field(key, value),
            Wrap::SerializeStructVariant(s) => s.serialize_field(key, value),
            Wrap::SerializeAsMap(s) => s.serialize_entry(key, value),
        }
    }

    fn skip_field(&mut self, key: &'static str) -> Result<(), S::Error> {
        match self {
            Wrap::SerializeStruct(s) => s.skip_field(key),
            Wrap::SerializeStructVariant(s) => s.skip_field(key),
            Wrap::SerializeAsMap(_) => Ok(()),
        }
    }

    fn end(self) -> Result<S::Ok, S::Error> {
        match self {
            Wrap::SerializeStruct(s) => s.end(),
            Wrap::SerializeStructVariant(s) => s.end(),
            Wrap::SerializeAsMap(s) => s.end(),
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub(crate) enum SerializeStructWrapper<'h, S: Serializer, H: SerializerWrapperHooks> {
    Wrapped {
        wrap: Wrap<'h, S, H>,
        hooks: &'h H,
        field_actions: StructFieldActions,
        have_retains: bool,
        rename_all: Option<Case>,
    },
    Skipped {
        end_result: Result<S::Ok, S::Error>,
    },
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> SerializeStructWrapper<'h, S, H> {
    pub(super) fn serialize_struct(
        serializer: S,
        name: &'static str,
        len: usize,
        hooks: &'h H,
        struct_actions: StructActions,
        field_actions: StructFieldActions,
    ) -> Result<Self, S::Error> {
        if should_serialize_as_map(&struct_actions, &field_actions) {
            return Self::serialize_struct_as_map(
                serializer,
                len,
                hooks,
                struct_actions,
                field_actions,
            );
        }

        Ok(Self::Wrapped {
            wrap: Wrap::SerializeStruct(serializer.serialize_struct(name, len)?),
            hooks,
            have_retains: have_retains(&field_actions),
            rename_all: rename_all(&field_actions),
            field_actions,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn serialize_struct_variant(
        serializer: S,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
        hooks: &'h H,
        struct_actions: StructActions,
        field_actions: StructFieldActions,
    ) -> Result<Self, S::Error> {
        if should_serialize_as_map(&struct_actions, &field_actions) {
            return Self::serialize_struct_as_map(
                serializer,
                len,
                hooks,
                struct_actions,
                field_actions,
            );
        }

        Ok(Self::Wrapped {
            wrap: Wrap::SerializeStructVariant(serializer.serialize_struct_variant(
                name,
                variant_index,
                variant,
                len,
            )?),
            hooks,
            have_retains: have_retains(&field_actions),
            rename_all: rename_all(&field_actions),
            field_actions,
        })
    }

    fn serialize_struct_as_map(
        serializer: S,
        len: usize,
        hooks: &'h H,
        _struct_actions: StructActions,
        field_actions: StructFieldActions,
    ) -> Result<Self, S::Error> {
        // If there's any potential of fields being skipped or added, don't feed map length hint
        // to the serializer.
        let len = if can_change_in_length(&field_actions) {
            None
        } else {
            Some(len)
        };

        let map_entry_actions = hooks.on_map(len);
        Ok(Self::Wrapped {
            wrap: Wrap::SerializeAsMap(SerializeMapWrapper::serialize_map(
                serializer,
                len,
                hooks,
                map_entry_actions,
            )?),
            hooks,
            have_retains: have_retains(&field_actions),
            rename_all: rename_all(&field_actions),
            field_actions,
        })
    }

    pub(super) fn new_skipped(end_result: Result<S::Ok, S::Error>) -> Self {
        Self::Skipped { end_result }
    }

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), S::Error>
    where
        T: Serialize,
    {
        match self {
            SerializeStructWrapper::Skipped { .. } => Ok(()),
            SerializeStructWrapper::Wrapped {
                wrap,
                hooks,
                field_actions: actions,
                have_retains,
                rename_all,
            } => {
                let mut field_key: Cow<'static, str> = key.into();
                let mut renamed_field = false;
                let mut retain_field = false;
                let mut skip_field = false;
                let mut replacement_value: Option<Value> = None;
                let mut flatten = false;

                actions.retain_mut(|a| match a {
                    StructFieldAction::Retain(n) => {
                        let matches = field_key == *n;
                        if matches {
                            retain_field = true;
                        }
                        !matches
                    }
                    StructFieldAction::Skip(n) => {
                        let matches = field_key == *n;
                        if matches {
                            skip_field = true;
                        }
                        !matches
                    }
                    StructFieldAction::Rename(n, r) => {
                        let matches = field_key == *n;
                        if matches {
                            renamed_field = true;
                            field_key = r.clone();
                        }
                        !matches
                    }
                    StructFieldAction::ReplaceValue(n, v) => {
                        let matches = field_key == *n;
                        if matches {
                            replacement_value = Some(v.clone());
                        }
                        !matches
                    }
                    StructFieldAction::RenameAllCase(_) => false,
                    StructFieldAction::Flatten(n) => {
                        let matches = field_key == *n;
                        if matches {
                            flatten = true;
                        }
                        !matches
                    }
                });

                if *have_retains && !retain_field {
                    skip_field = true;
                }

                if skip_field {
                    wrap.skip_field(key)
                } else {
                    hooks.path_push(PathSegment::StructField(key));

                    if !renamed_field {
                        if let Some(case) = rename_all {
                            field_key = Case::cow_to_case(&field_key, *case);
                        }
                    }

                    if let Some(replacement_value) = &replacement_value {
                        replacement_value
                            .check_if_can_serialize()
                            .or_else(|err| hooks.on_error::<S>(err))?;
                    }

                    let res = if let Some(replacement_value) = replacement_value {
                        wrap.serialize_field(hooks.make_static_str(field_key), &replacement_value)
                    } else {
                        let s = SerializableWithHooks::new(value, *hooks, SerializableKind::Value);

                        if flatten {
                            let serialize_map = match wrap {
                                Wrap::SerializeAsMap(m) => m,
                                _ => unreachable!(),
                            };

                            let flatten_serializer = FlattenSerializer::new(serialize_map);
                            match s.serialize(flatten_serializer) {
                                Ok(r) => Ok(r),
                                Err(FlattenError::SerializerError(e)) => Err(e),
                                Err(FlattenError::UnsupportedDataType(data_type)) => hooks
                                    .on_error::<S>(HooksError::CannotFlattenUnsupportedDataType(
                                        data_type,
                                    )),
                            }
                        } else {
                            wrap.serialize_field(hooks.make_static_str(field_key), &s)
                        }
                    };
                    hooks.path_pop();
                    res
                }
            }
        }
    }

    fn end(self) -> Result<S::Ok, S::Error> {
        match self {
            SerializeStructWrapper::Skipped { end_result } => end_result,
            SerializeStructWrapper::Wrapped {
                wrap,
                hooks,
                field_actions: actions,
                ..
            } => {
                if let Some(a) = actions.into_iter().next() {
                    match a {
                        StructFieldAction::Retain(f)
                        | StructFieldAction::Skip(f)
                        | StructFieldAction::Rename(f, _)
                        | StructFieldAction::ReplaceValue(f, _)
                        | StructFieldAction::Flatten(f) => {
                            hooks.on_error::<S>(HooksError::FieldNotFound(f))?
                        }
                        StructFieldAction::RenameAllCase(_) => {}
                    }
                }

                wrap.end()
            }
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
        self.serialize_field(key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end()
    }
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> serde::ser::SerializeStructVariant
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
        self.serialize_field(key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end()
    }
}

fn should_serialize_as_map(
    struct_actions: &StructActions,
    field_actions: &StructFieldActions,
) -> bool {
    struct_actions.serialize_as_map
        || field_actions
            .iter()
            .any(|a| matches!(a, StructFieldAction::Flatten(_)))
}

fn can_change_in_length(field_actions: &StructFieldActions) -> bool {
    field_actions.iter().any(|a| {
        matches!(
            a,
            StructFieldAction::Retain(_)
                | StructFieldAction::Skip(_)
                | StructFieldAction::Flatten(_)
        )
    })
}

fn have_retains(field_actions: &StructFieldActions) -> bool {
    field_actions
        .iter()
        .any(|a| matches!(a, StructFieldAction::Retain(_)))
}

fn rename_all(field_actions: &StructFieldActions) -> Option<Case> {
    field_actions.iter().rev().find_map(|a| match a {
        StructFieldAction::RenameAllCase(case) => Some(*case),
        _ => None,
    })
}
