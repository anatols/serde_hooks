use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};

use crate::ser::scope::{OnSeqElementActions, SeqElementAction};
use crate::ser::HooksError;
use crate::Value;

use super::{PathSegment, SerializableKind, SerializableWithHooks, SerializerWrapperHooks};

pub(crate) enum Wrap<S: Serializer> {
    SerializeSeq(S::SerializeSeq),
}

impl<S: Serializer> Wrap<S> {
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), S::Error>
    where
        T: Serialize,
    {
        match self {
            Wrap::SerializeSeq(s) => s.serialize_element(value),
        }
    }

    fn end(self) -> Result<S::Ok, S::Error> {
        match self {
            Wrap::SerializeSeq(s) => s.end(),
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub(crate) enum SerializeSeqWrapper<'h, S: Serializer, H: SerializerWrapperHooks> {
    Wrapped {
        wrap: Wrap<S>,
        hooks: &'h H,
        actions: OnSeqElementActions,
        have_retains: bool,
        current_index: usize,
    },
    Skipped {
        end_result: Result<S::Ok, S::Error>,
    },
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> SerializeSeqWrapper<'h, S, H> {
    pub(super) fn new_wrapped_seq(
        serialize_seq: S::SerializeSeq,
        hooks: &'h H,
        actions: OnSeqElementActions,
    ) -> Self {
        Self::Wrapped {
            wrap: Wrap::SerializeSeq(serialize_seq),
            hooks,
            have_retains: actions
                .iter()
                .any(|a| matches!(a, SeqElementAction::Retain(_))),
            actions,
            current_index: 0,
        }
    }

    pub(super) fn new_skipped(end_result: Result<S::Ok, S::Error>) -> Self {
        Self::Skipped { end_result }
    }

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), S::Error>
    where
        T: Serialize,
    {
        match self {
            SerializeSeqWrapper::Skipped { .. } => Ok(()),
            SerializeSeqWrapper::Wrapped {
                wrap,
                hooks,
                actions,
                have_retains,
                current_index,
            } => {
                let mut retain_field = false;
                let mut skip_field = false;
                let mut replacement_value: Option<Value> = None;

                actions.retain_mut(|a| match a {
                    SeqElementAction::Retain(index) => {
                        let matches = *current_index == *index;
                        if matches {
                            retain_field = true;
                        }
                        !matches
                    }
                    SeqElementAction::Skip(index) => {
                        let matches = *current_index == *index;
                        if matches {
                            skip_field = true;
                        }
                        !matches
                    }
                    SeqElementAction::ReplaceValue(index, v) => {
                        let matches = *current_index == *index;
                        if matches {
                            replacement_value = Some(v.clone());
                        }
                        !matches
                    }
                });

                if *have_retains && !retain_field {
                    skip_field = true;
                }

                hooks.path_push(PathSegment::SeqIndex(*current_index));

                if let Some(replacement_value) = &replacement_value {
                    replacement_value
                        .check_if_can_serialize()
                        .or_else(|err| hooks.on_error::<S>(err))?;
                }

                let res = if skip_field {
                    Ok(())
                } else if let Some(replacement_value) = replacement_value {
                    wrap.serialize_element(&replacement_value)
                } else {
                    let s = SerializableWithHooks::new(value, *hooks, SerializableKind::Value);
                    wrap.serialize_element(&s)
                };

                hooks.path_pop();
                *current_index += 1;

                res
            }
        }
    }

    fn end(self) -> Result<S::Ok, S::Error> {
        match self {
            SerializeSeqWrapper::Skipped { end_result } => end_result,
            SerializeSeqWrapper::Wrapped {
                wrap,
                hooks,
                actions,
                ..
            } => {
                if let Some(a) = actions.into_iter().next() {
                    match a {
                        SeqElementAction::Retain(index)
                        | SeqElementAction::Skip(index)
                        | SeqElementAction::ReplaceValue(index, _) => {
                            hooks.on_error::<S>(HooksError::IndexNotFound(index))?
                        }
                    }
                }

                wrap.end()
            }
        }
    }
}

impl<'h, S: Serializer, H: SerializerWrapperHooks> serde::ser::SerializeSeq
    for SerializeSeqWrapper<'h, S, H>
{
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.serialize_element(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end()
    }
}
