use crate::{
    ser::wrapper::{SeqElementAction, SeqElementActions},
    StaticValue,
};

//TODO add support for insert before, insert after, push back

/// Inspect and modify sequences and tuple elements.
///
/// See [`Hooks::on_seq`](crate::ser::Hooks::on_seq),
/// [`Hooks::on_tuple`](crate::ser::Hooks::on_tuple),
/// [`Hooks::on_tuple_variant`](crate::ser::Hooks::on_tuple_variant),
/// [`Hooks::on_tuple_struct`](crate::ser::Hooks::on_tuple_struct).
///
/// When this scope is used for tuples, specifying any actions that may change
/// the number of elements in the sequence (e.g. retaining or skipping elements)
/// will force the tuple to be serialized as a sequence.
/// Depending on the serializer you use, this might be totally unsupported or
/// lead to unexpected serialization results.
///
/// For sequences, specifying any actions that may change
/// the number of elements in the sequence (e.g. retaining or skipping elements)
/// will make the sequence serialize as one of an unknown length. Some
/// serializers do not support this.
pub struct SeqScope {
    seq_len: Option<usize>,
    actions: SeqElementActions,
}

impl SeqScope {
    pub(crate) fn new(seq_len: Option<usize>) -> Self {
        Self {
            seq_len,
            actions: Default::default(),
        }
    }

    pub(crate) fn into_actions(self) -> SeqElementActions {
        self.actions
    }

    /// Returns the original sequence length if known during serialization.
    ///
    /// The returned value is not affected by any retain or skip actions.
    ///
    /// Sequence length is always known for tuples.
    pub fn seq_len(&self) -> Option<usize> {
        self.seq_len
    }

    /// Skips an element at the given index.
    ///
    /// The index passed is the index in the original sequence.
    ///
    /// Returns `self` to allow chaining calls.
    pub fn skip_element(&mut self, index: usize) -> &mut Self {
        self.actions.push(SeqElementAction::Skip(index));
        self
    }

    /// Retains an element at the given index.
    ///
    /// Calling this method switches processing to a 'retain' mode, in which
    /// all not retained elements are skipped. You can retain multiple elements by
    /// calling this method multiple times.
    ///
    /// The index passed is the index in the original sequence.
    ///
    /// Returns `self` to allow chaining calls.
    pub fn retain_element(&mut self, index: usize) -> &mut Self {
        self.actions.push(SeqElementAction::Retain(index));
        self
    }

    /// Replace a value at the given index.
    ///
    /// The index passed is the index in the original sequence.
    ///
    /// The passed in [`StaticValue`] can represent both primitive and compound value types.
    ///
    /// Primitive values are copied, and are later fed to the serializer instead of the original
    /// sequence elements.
    ///
    /// For compound values, only metadata is stored, therefore it's not possible to
    /// serialize the actual values from the contents of [`StaticValue`]. Passing in a
    /// compound value here would result in an
    /// [`HooksError::ValueNotSerializable`](crate::ser::HooksError::ValueNotSerializable) error.
    ///
    /// The trick to replace a compound value is to replace it in this scope with a primitive one
    /// (e.g. a unit), subscribe to `on_value` hook, and replace the value there again with the
    /// compound one.
    ///
    /// The replacement value does not necessarily need to be of the same type as the
    /// original value in the sequence. E.g., you can replace an element in an integer sequence
    /// with a string. Although some serializers might not be happy about a mish-mash of types.
    ///
    /// Returns `self` to allow chaining calls.
    pub fn replace_value(&mut self, index: usize, new_value: impl Into<StaticValue>) -> &mut Self {
        self.actions
            .push(SeqElementAction::ReplaceValue(index, new_value.into()));
        self
    }
}
