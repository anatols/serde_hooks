use crate::Path;

pub struct TupleScope<'p> {
    path: &'p Path,
    tuple_len: usize,
}

impl<'p> TupleScope<'p> {
    pub(crate) fn new(path: &'p Path, tuple_len: usize) -> Self {
        Self { path, tuple_len }
    }

    pub fn path(&self) -> &Path {
        self.path
    }

    pub fn tuple_len(&self) -> usize {
        self.tuple_len
    }
}

pub struct TupleStructScope<'p> {
    path: &'p Path,
    struct_name: &'static str,
    tuple_len: usize,
}

impl<'p> TupleStructScope<'p> {
    pub(crate) fn new(path: &'p Path, struct_name: &'static str, tuple_len: usize) -> Self {
        Self {
            path,
            struct_name,
            tuple_len,
        }
    }

    pub fn path(&self) -> &Path {
        self.path
    }

    pub fn struct_name(&self) -> &'static str {
        self.struct_name
    }

    pub fn tuple_len(&self) -> usize {
        self.tuple_len
    }
}
