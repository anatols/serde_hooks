use serde::{Serialize, Serializer};

use crate::{Path, Value};

pub(crate) enum OnValueAction<S: Serializer> {
    ContinueSerialization(S),
    ValueReplaced(Result<S::Ok, S::Error>),
}

pub struct ValueScope<'p, S: Serializer> {
    path: &'p Path,
    action: Option<OnValueAction<S>>,
    value: Value,
}

impl<'p, S: Serializer> ValueScope<'p, S> {
    pub(crate) fn new(path: &'p Path, serializer: S, value: Value) -> Self {
        Self {
            path,
            action: Some(OnValueAction::ContinueSerialization(serializer)),
            value,
        }
    }

    pub(crate) fn into_action(self) -> OnValueAction<S> {
        self.action.unwrap()
    }

    pub fn path(&self) -> &Path {
        self.path
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn replace<T: Serialize + ?Sized>(&mut self, new_value: &T) -> &mut Self {
        let serializer = match self.action.take().unwrap() {
            OnValueAction::ContinueSerialization(s) => s,
            OnValueAction::ValueReplaced(_) => panic!("value already replaced"),
        };
        let res = new_value.serialize(serializer);
        self.action = Some(OnValueAction::ValueReplaced(res));
        self
    }
}
