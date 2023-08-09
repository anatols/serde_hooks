use serde::Deserialize;

pub(crate) struct DeserializableWithContext {
    //TODO context
    //TODO wrapped
    //TODO hooks
}

impl<'de> Deserialize<'de> for DeserializableWithContext {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        todo!()
    }
}
