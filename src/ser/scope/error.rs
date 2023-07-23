use serde::Serializer;

use crate::{ser::HooksError, Path};

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

    pub fn path(&self) -> &Path {
        self.path
    }

    pub fn error(&self) -> &HooksError {
        &self.error
    }

    pub fn ignore(&mut self) {
        self.ignore = true;
    }

    pub fn panic(&mut self) {
        panic!("{}", self.format_error_message());
    }

    pub fn propagate(&mut self) {
        self.ignore = false;
    }

    fn format_error_message(&self) -> String {
        format!(
            "Error at path '{path}': {err}",
            path = self.path.to_string(),
            err = self.error,
        )
    }
}
