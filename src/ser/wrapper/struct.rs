use std::borrow::Cow;

use serde::ser::{SerializeStruct, SerializeStructVariant};
use serde::{Serialize, Serializer};

use crate::ser::scope::{OnStructFieldActions, StructFieldAction};
use crate::ser::HooksError;
use crate::static_str::into_static_str;
use crate::{Case, Value};

use super::{PathSegment, SerializableKind, SerializableWithHooks, SerializerWrapperHooks};

pub(crate) enum Wrap<S: Serializer> {
    SerializeStruct(S::SerializeStruct),
    SerializeStructVariant(S::SerializeStructVariant),
}

impl<S: Serializer> Wrap<S> {
    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), S::Error>
    where
        T: Serialize,
    {
        match self {
            Wrap::SerializeStruct(s) => s.serialize_field(key, value),
            Wrap::SerializeStructVariant(s) => s.serialize_field(key, value),
        }
    }

    fn skip_field(&mut self, key: &'static str) -> Result<(), S::Error> {
        match self {
            Wrap::SerializeStruct(s) => s.skip_field(key),
            Wrap::SerializeStructVariant(s) => s.skip_field(key),
        }
    }

    fn end(self) -> Result<S::Ok, S::Error> {
        match self {
            Wrap::SerializeStruct(s) => s.end(),
            Wrap::SerializeStructVariant(s) => s.end(),
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub(crate) enum SerializeStructWrapper<'h, S: Serializer, H: SerializerWrapperHooks> {
    Wrapped {
        wrap: Wrap<S>,
        hooks: &'h H,
        actions: OnStructFieldActions,
        have_retains: bool,
        rename_all: Option<Case>,
    },
    Skipped {
        end_result: Result<S::Ok, S::Error>,
    },
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> SerializeStructWrapper<'h, S, H> {
    pub(super) fn new_wrapped_struct(
        serialize_struct: S::SerializeStruct,
        hooks: &'h H,
        actions: OnStructFieldActions,
    ) -> Self {
        Self::Wrapped {
            wrap: Wrap::SerializeStruct(serialize_struct),
            hooks,
            have_retains: actions
                .iter()
                .any(|a| matches!(a, StructFieldAction::Retain(_))),
            rename_all: actions.iter().rev().find_map(|a| match a {
                StructFieldAction::RenameAll(case) => Some(*case),
                _ => None,
            }),
            actions,
        }
    }

    pub(super) fn new_wrapped_struct_variant(
        serialize_struct_variant: S::SerializeStructVariant,
        hooks: &'h H,
        actions: OnStructFieldActions,
    ) -> Self {
        Self::Wrapped {
            wrap: Wrap::SerializeStructVariant(serialize_struct_variant),
            hooks,
            have_retains: actions
                .iter()
                .any(|a| matches!(a, StructFieldAction::Retain(_))),
            rename_all: actions.iter().rev().find_map(|a| match a {
                StructFieldAction::RenameAll(case) => Some(*case),
                _ => None,
            }),
            actions,
        }
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
                actions,
                have_retains,
                rename_all,
            } => {
                let mut field_key: Cow<'static, str> = key.into();
                let mut renamed_field = false;
                let mut retain_field = false;
                let mut skip_field = false;
                let mut replacement_value: Option<Value> = None;

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
                    StructFieldAction::RenameAll(_) => false,
                });

                if *have_retains && !retain_field {
                    skip_field = true;
                }

                if !renamed_field {
                    if let Some(case) = rename_all {
                        field_key = Case::string_to_case(&field_key, *case).into();
                    }
                }

                hooks.path_push(PathSegment::StructField(key));

                if let Some(replacement_value) = &replacement_value {
                    replacement_value
                        .check_if_can_serialize()
                        .or_else(|err| hooks.on_error::<S>(err))?;
                }

                let res = if skip_field {
                    wrap.skip_field(key)
                } else if let Some(replacement_value) = replacement_value {
                    wrap.serialize_field(into_static_str(field_key), &replacement_value)
                } else {
                    let s = SerializableWithHooks::new(value, *hooks, SerializableKind::Value);
                    wrap.serialize_field(into_static_str(field_key), &s)
                };

                hooks.path_pop();
                res
            }
        }
    }

    fn end(self) -> Result<S::Ok, S::Error> {
        match self {
            SerializeStructWrapper::Skipped { end_result } => end_result,
            SerializeStructWrapper::Wrapped {
                wrap,
                hooks,
                actions,
                ..
            } => {
                if let Some(a) = actions.into_iter().next() {
                    match a {
                        StructFieldAction::Retain(f)
                        | StructFieldAction::Skip(f)
                        | StructFieldAction::Rename(f, _)
                        | StructFieldAction::ReplaceValue(f, _) => {
                            hooks.on_error::<S>(HooksError::FieldNotFound(f))?
                        }
                        StructFieldAction::RenameAll(_) => {}
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
