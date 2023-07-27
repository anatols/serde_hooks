use std::borrow::Cow;

use serde::{Serialize, Serializer};

mod context;
mod scope;
mod value;
mod wrapper;

pub use scope::{
    ErrorScope, MapKeyScope, MapKeySelector, MapScope, StructManipulation, StructScope,
    StructVariantScope, ValueScope,
};

use context::SerializableWithContext;

//TODO add support for:
// rename key free-form & cases
// flatten?
// convert struct to map
// sequences & tuples

pub trait Hooks {
    fn start(&self) {}

    fn end(&self) {}

    #[allow(unused_variables)]
    fn on_error(&self, err: &mut ErrorScope) {}

    #[allow(unused_variables)]
    fn on_map(&self, map: &mut MapScope) {}

    #[allow(unused_variables)]
    fn on_map_key<S: Serializer>(&self, map_key: &mut MapKeyScope<S>) {}

    #[allow(unused_variables)]
    fn on_struct(&self, st: &mut StructScope) {}

    #[allow(unused_variables)]
    fn on_struct_variant(&self, stv: &mut StructVariantScope) {}

    #[allow(unused_variables)]
    fn on_value<S: Serializer>(&self, value: &mut ValueScope<S>) {}
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum HooksError {
    #[error("cannot add key {0}, the key is already present in the map")]
    KeyAlreadyPresent(MapKeySelector),
    #[error("cannot add entry with an index {0}, please specify key value")]
    CannotAddEntryByIndex(usize),
    #[error("key {0} not found")]
    KeyNotFound(MapKeySelector),
    #[error("field \"{0}\" not found")]
    FieldNotFound(Cow<'static, str>),
    #[error("value is not serializable: {0}")]
    ValueNotSerializable(String),
}

pub fn hook<'s, 'h, T: Serialize + ?Sized, H: Hooks>(
    serializable: &'s T,
    hooks: &'h H,
) -> SerializableWithContext<'s, 'h, T, H> {
    SerializableWithContext::new(serializable, hooks)
}
