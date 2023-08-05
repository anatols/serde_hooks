pub struct TupleScope {
    tuple_len: usize,
}

impl TupleScope {
    pub(crate) fn new(tuple_len: usize) -> Self {
        Self { tuple_len }
    }

    pub fn tuple_len(&self) -> usize {
        self.tuple_len
    }
}

pub struct TupleStructScope {
    struct_name: &'static str,
    tuple_len: usize,
}

impl TupleStructScope {
    pub(crate) fn new(struct_name: &'static str, tuple_len: usize) -> Self {
        Self {
            struct_name,
            tuple_len,
        }
    }

    pub fn struct_name(&self) -> &'static str {
        self.struct_name
    }

    pub fn tuple_len(&self) -> usize {
        self.tuple_len
    }
}
