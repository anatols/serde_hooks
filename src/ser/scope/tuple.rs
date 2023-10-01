/// Inspect tuples.
///
/// See [`Hooks::on_tuple`](crate::ser::Hooks::on_tuple),
/// [`Hooks::on_tuple_variant`](crate::ser::Hooks::on_tuple_variant).
///
/// This scope does not allow modifying the tuple elements. However, hooks
/// for tuples would additionally receive a [`SeqScope`](crate::ser::SeqScope)
/// that would allow to do so.
pub struct TupleScope {
    tuple_len: usize,
}

impl TupleScope {
    pub(crate) fn new(tuple_len: usize) -> Self {
        Self { tuple_len }
    }

    /// Returns the tuple length.
    pub fn tuple_len(&self) -> usize {
        self.tuple_len
    }
}

/// Inspect tuple struct.
///
/// See [`Hooks::on_tuple_struct`](crate::ser::Hooks::on_tuple_struct).
///
/// This scope does not allow modifying the tuple elements. However, hooks
/// for tuples would additionally receive a [`SeqScope`](crate::ser::SeqScope)
/// that would allow to do so.
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

    /// Returns the tuple struct name.
    pub fn struct_name(&self) -> &'static str {
        self.struct_name
    }

    /// Returns the tuple length.
    pub fn tuple_len(&self) -> usize {
        self.tuple_len
    }
}
