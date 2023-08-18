use serde::Serializer;

use crate::{ser::HooksError, Path};

/// Inspect errors and choose recovery actions.
///
/// See [`Hooks::on_scope_error`](crate::ser::Hooks::on_scope_error).
///
/// By default the errors are propagated.
pub struct ErrorScope<'p> {
    path: &'p Path,
    error: HooksError,
    ignore: bool,
}

impl<'p> ErrorScope<'p> {
    pub(crate) fn new(path: &'p Path, error: HooksError) -> Self {
        Self {
            path,
            error,
            ignore: false,
        }
    }

    pub(crate) fn into_result<S: Serializer>(self) -> Result<(), S::Error> {
        if self.ignore {
            Ok(())
        } else {
            Err(serde::ser::Error::custom(self.format_error_message()))
        }
    }

    /// Returns the error the hook was called about.
    pub fn error(&self) -> &HooksError {
        &self.error
    }

    /// Ignore this error and continue serialization.
    pub fn ignore(&mut self) -> &mut Self {
        self.ignore = true;
        self
    }

    /// Immediately panic.
    ///
    /// The panic message will contain the error message.
    pub fn panic(&mut self) {
        panic!("{}", self.format_error_message());
    }

    /// Propagate this error as a custom serialization error.
    pub fn propagate(&mut self) -> &mut Self {
        self.ignore = false;
        self
    }

    fn format_error_message(&self) -> String {
        format!(
            "Error at path '{path}': {err}",
            path = self.path.borrow_str(),
            err = self.error,
        )
    }
}
