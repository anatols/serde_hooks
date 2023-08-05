use crate::{
    ser::wrapper::{SeqElementAction, SeqElementActions},
    StaticValue,
};

//TODO add support for insert before, insert after, push back
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

    pub fn seq_len(&self) -> Option<usize> {
        self.seq_len
    }

    pub fn retain_element(&mut self, index: usize) -> &mut Self {
        self.actions.push(SeqElementAction::Retain(index));
        self
    }

    pub fn skip_element(&mut self, index: usize) -> &mut Self {
        self.actions.push(SeqElementAction::Skip(index));
        self
    }

    pub fn replace_value(&mut self, index: usize, new_value: impl Into<StaticValue>) -> &mut Self {
        self.actions
            .push(SeqElementAction::ReplaceValue(index, new_value.into()));
        self
    }
}
