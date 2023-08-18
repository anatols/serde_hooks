use std::pin::Pin;

/// Inspect serialization state after serialization ends.
///
/// See [`Hooks::on_end`](crate::ser::Hooks::on_end).
pub struct EndScope {
    static_strs: Vec<Pin<Box<str>>>,
}

impl EndScope {
    pub(crate) fn new(static_strs: Vec<Pin<Box<str>>>) -> Self {
        Self { static_strs }
    }

    /// Forces all static strings that were captured during serialization to be leaked
    /// and therefore to become truly `&'static str`.
    ///
    /// By default static strings are not leaked and are deallocated after serialization
    /// ends.
    ///
    /// See [Static strings](crate#static-strings) for more info.
    pub fn leak_static_strs(&mut self) -> &mut Self {
        self.static_strs.drain(..).for_each(|pinned_str| {
            Box::leak(Pin::into_inner(pinned_str));
        });

        self
    }

    /// Extracts and returns all static strings that were captured during serialization.
    ///
    /// If the output of our your serializer holds on to static string references, but only
    /// until certain moment in time, you might want to deallocate the strings after
    /// that moment instead of leaking them.
    ///
    /// The returned collection contains pinned `str` slices allocated on heap (boxes).
    /// During serialization, your serializer was fed references to these slices, but
    /// transmuted to `&'static str`.
    ///
    /// See [Static strings](crate#static-strings) for more info.
    ///
    /// # Safety
    ///
    /// This method is unsafe since you are responsible for holding on to the returned
    /// boxes until the moment your serializer output is not referring to the strings
    /// anymore. Dropping the boxes earlier will mean the serializer output contains
    /// invalid references.
    pub unsafe fn take_static_strs(&mut self) -> Vec<Pin<Box<str>>> {
        std::mem::take(&mut self.static_strs)
    }
}
