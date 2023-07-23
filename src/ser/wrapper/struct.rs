use std::borrow::Cow;

use serde::{Serialize, Serializer};

use crate::ser::scope::{OnStructFieldActions, StructFieldAction};
use crate::ser::HooksError;
use crate::static_str::into_static_str;
use crate::Value;

use super::{PathSegment, SerializableKind, SerializableWithHooks, SerializerWrapperHooks};

pub(crate) struct SerializeStructWrapper<'h, S: Serializer, H: SerializerWrapperHooks> {
    serialize_struct: S::SerializeStruct,
    hooks: &'h H,
    actions: OnStructFieldActions,
    have_retains: bool,
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> SerializeStructWrapper<'h, S, H> {
    pub(super) fn new(
        serialize_struct: S::SerializeStruct,
        hooks: &'h H,
        actions: OnStructFieldActions,
    ) -> Self {
        Self {
            serialize_struct,
            hooks,
            have_retains: actions
                .iter()
                .any(|a| matches!(a, StructFieldAction::Retain(_))),
            actions,
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
        let mut field_key: Cow<'static, str> = key.into();
        let mut retain_field = false;
        let mut skip_field = false;
        let mut replacement_value: Option<Value> = None;

        self.actions.retain_mut(|a| match a {
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
        });

        if self.have_retains && !retain_field {
            skip_field = true;
        }

        self.hooks.path_push(PathSegment::StructField(key));

        if let Some(replacement_value) = &replacement_value {
            replacement_value
                .check_if_can_serialize()
                .or_else(|err| self.hooks.on_error::<S>(err))?;
        }

        let res = if skip_field {
            self.serialize_struct.skip_field(key)
        } else if let Some(replacement_value) = replacement_value {
            self.serialize_struct
                .serialize_field(into_static_str(field_key), &replacement_value)
        } else {
            let s = SerializableWithHooks::new(value, self.hooks, SerializableKind::Value);
            self.serialize_struct
                .serialize_field(into_static_str(field_key), &s)
        };

        self.hooks.path_pop();
        res
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if let Some(a) = self.actions.into_iter().next() {
            match a {
                StructFieldAction::Retain(f)
                | StructFieldAction::Skip(f)
                | StructFieldAction::Rename(f, _)
                | StructFieldAction::ReplaceValue(f, _) => {
                    self.hooks.on_error::<S>(HooksError::FieldNotFound(f))?
                }
            }
        }

        self.serialize_struct.end()
    }
}
