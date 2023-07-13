use std::{borrow::Cow, sync::Mutex};

use serde::{
    ser::{Error, SerializeStruct},
    Serialize, Serializer,
};

use crate::ser::{hooks::StructFieldAction, PrimitiveValue};

use super::{
    OnStructFieldActions, PathSegment, SerializableKind, SerializableWithHooks,
    SerializerWrapperHooks,
};

pub struct SerializeStructWrapper<'h, S: Serializer, H: SerializerWrapperHooks> {
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
        println!("serialize_field {key}");

        let mut field_key: Cow<'static, str> = key.into();
        let mut retain_field = false;
        let mut skip_field = false;
        let mut replacement_value: Option<PrimitiveValue> = None;

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

        let res = if skip_field {
            self.serialize_struct.skip_field(key)
        } else {
            let s = SerializableWithHooks {
                serializable: value,
                hooks: self.hooks,
                kind: SerializableKind::Value,
            };
            self.serialize_maybe_renamed_field(field_key, &s)
        };

        self.hooks.path_pop();
        res
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        //TODO verify that no actions remain
        self.serialize_struct.end()
    }
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> SerializeStructWrapper<'h, S, H> {
    fn serialize_maybe_renamed_field<T: ?Sized>(
        &mut self,
        key: Cow<'static, str>,
        value: &T,
    ) -> Result<(), S::Error>
    where
        T: Serialize,
    {
        match key {
            Cow::Borrowed(static_key) => self.serialize_struct.serialize_field(static_key, value),
            Cow::Owned(string_key) => {
                static KEYS: Mutex<Vec<Box<str>>> = Mutex::new(vec![]);

                let mut keys = KEYS.lock().unwrap();
                let maybe_key = keys.iter().find(|k| ***k == string_key);

                // Boxes can move in the vector, but where they point remains in place until the end
                // of the program because we never delete. For all practical purposes it is 'static,
                // so safe to transmute here.
                let static_key: &'static str = match maybe_key {
                    Some(boxed_key) => unsafe {
                        std::mem::transmute::<&str, &'static str>(boxed_key.as_ref())
                    },
                    None => {
                        // This is obviously "leaking" memory on each new field, but hey, how many of those
                        // renamed fields are you planning to have?
                        keys.push(string_key.clone().into_boxed_str());
                        unsafe {
                            std::mem::transmute::<&str, &'static str>(keys.last().unwrap().as_ref())
                        }
                    }
                };

                self.serialize_struct.serialize_field(static_key, value)
            }
        }
    }
}
