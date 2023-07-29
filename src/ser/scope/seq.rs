use smallvec::SmallVec;

use crate::{Path, StaticValue};

#[derive(Debug)]
pub(crate) enum SeqElementAction {
    Retain(usize),
    Skip(usize),
    ReplaceValue(usize, StaticValue),
}

pub(crate) type OnSeqElementActions = SmallVec<[SeqElementAction; 8]>;

pub trait SeqManipulation {
    fn retain_element(&mut self, index: usize) -> &mut Self;

    fn skip_element(&mut self, index: usize) -> &mut Self;

    fn replace_value(&mut self, index: usize, new_value: impl Into<StaticValue>) -> &mut Self;

    //TODO insert before, insert after, push back
}

pub struct SeqScope<'p> {
    path: &'p Path,
    seq_len: Option<usize>,
    actions: OnSeqElementActions,
}

impl<'p> SeqScope<'p> {
    pub(crate) fn new(path: &'p Path, seq_len: Option<usize>) -> Self {
        Self {
            path,
            seq_len,
            actions: Default::default(),
        }
    }

    pub(crate) fn into_actions(self) -> OnSeqElementActions {
        self.actions
    }

    pub fn path(&self) -> &Path {
        self.path
    }

    pub fn seq_len(&self) -> Option<usize> {
        self.seq_len
    }
}

impl SeqManipulation for SeqScope<'_> {
    fn retain_element(&mut self, index: usize) -> &mut Self {
        self.actions.push(SeqElementAction::Retain(index));
        self
    }

    fn skip_element(&mut self, index: usize) -> &mut Self {
        self.actions.push(SeqElementAction::Skip(index));
        self
    }

    fn replace_value(&mut self, index: usize, new_value: impl Into<StaticValue>) -> &mut Self {
        self.actions
            .push(SeqElementAction::ReplaceValue(index, new_value.into()));
        self
    }
}
