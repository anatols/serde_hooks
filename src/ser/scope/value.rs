use std::fmt::Display;

use serde::{Serialize, Serializer};

use crate::{ser::wrapper::ValueAction, Value};

/// Inspect and modify serialized values.
///
/// See [`Hooks::on_value`](crate::ser::Hooks::on_value).
pub struct ValueScope<'v, S: Serializer> {
    action: Option<ValueAction<S>>,
    value: Value<'v>,
    result: Result<(), String>,
}

impl<'v, S: Serializer> ValueScope<'v, S> {
    pub(crate) fn new(serializer: S, value: Value<'v>) -> Self {
        Self {
            action: Some(ValueAction::ContinueSerialization(serializer)),
            value,
            result: Ok(()),
        }
    }

    pub(crate) fn into_action(self) -> Result<ValueAction<S>, S::Error> {
        self.result
            .map(|_| self.action.unwrap())
            .map_err(serde::ser::Error::custom)
    }

    /// Returns the serialized value.
    ///
    /// Primitive values, like numbers, will have the actual value copied to the scope,
    /// whilst for compound values, like structs, only metadata is available.
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Fail serialization with a custom error.
    pub fn fail_serialization(&mut self, error: impl Display) {
        self.result = Err(error.to_string());
    }

    /// Replace the value with another serializable value.
    ///
    /// The new value is fed directly into the serializer.
    ///
    /// There is no requirement for the new value to be of the same type. However,
    /// the serializer you use can have restrictions on compatibility.
    ///
    /// Hooks will **not** be called for the serialization of the new value. If you want to
    /// attach hooks to the new value as well, you need to explicitly do it by calling
    /// [`ser::hook()`](crate::ser::hook) on it.
    ///
    /// # Panics
    ///
    /// A value can only be replaced once. This method will panic if the value has already been replaced.
    pub fn replace<T: Serialize + ?Sized>(&mut self, new_value: &T) -> &mut Self {
        let serializer = match self.action.take().unwrap() {
            ValueAction::ContinueSerialization(s) => s,
            ValueAction::ValueReplaced(_) => panic!("value already replaced"),
        };
        let res = new_value.serialize(serializer);
        self.action = Some(ValueAction::ValueReplaced(res));
        self
    }
}
