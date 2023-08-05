use serde::{Serialize, Serializer};

use crate::{ser::wrapper::ValueAction, Value};

pub struct ValueScope<'v, S: Serializer> {
    action: Option<ValueAction<S>>,
    value: Value<'v>,
}

impl<'v, S: Serializer> ValueScope<'v, S> {
    pub(crate) fn new(serializer: S, value: Value<'v>) -> Self {
        Self {
            action: Some(ValueAction::ContinueSerialization(serializer)),
            value,
        }
    }

    pub(crate) fn into_action(self) -> ValueAction<S> {
        self.action.unwrap()
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

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
