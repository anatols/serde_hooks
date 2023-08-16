/// Inspect serializer information before serialization begins.
pub struct StartScope {
    is_human_readable: bool,
}

impl StartScope {
    pub(crate) fn new(is_human_readable: bool) -> Self {
        Self { is_human_readable }
    }

    /// Returns `true` if used serializer is expected to produce a human-readable format.
    ///
    /// See [`serde::ser::Serializer::is_human_readable`] for more info.
    pub fn is_format_human_readable(&self) -> bool {
        self.is_human_readable
    }
}
