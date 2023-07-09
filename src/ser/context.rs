use std::{cell::RefCell, rc::Rc};

use serde::{Serialize, Serializer};

use super::hooks::{Hooks, MapScope, ValueScope};
use super::path::{Path, PathSegment};
use super::wrapper;
use super::Value;

pub struct SerializableWithContext<'s, T: Serialize + ?Sized, H: Hooks> {
    serializable: &'s T,
    context: Context<H>,
}

impl<'s, T: Serialize + ?Sized, H: Hooks> SerializableWithContext<'s, T, H> {
    pub(super) fn new(serializable: &'s T, hooks: H) -> Self {
        Self {
            serializable,
            context: Context::new(hooks),
        }
    }
}

impl<T: Serialize + ?Sized, H: Hooks> Serialize for SerializableWithContext<'_, T, H> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.context.start();
        let res = self
            .serializable
            .serialize(wrapper::SerializerWrapper::new(serializer, &self.context));
        self.context.end();
        res
    }
}

#[derive(Debug, Clone)]
pub struct Context<H: Hooks> {
    inner: Rc<RefCell<ContextInner<H>>>,
}

impl<H: Hooks> wrapper::SerializerWrapperHooks for Context<H> {
    fn path_push(&self, segment: PathSegment) {
        self.inner.borrow_mut().path.push_segment(segment);
    }

    fn path_pop(&self) {
        self.inner.borrow_mut().path.pop_segment();
    }

    fn on_map(&self, len: Option<usize>) -> wrapper::OnMapEntryActions {
        let path = &self.inner.borrow().path;
        let mut scope = MapScope::new(path, len);
        self.inner.borrow().hooks.on_map(&mut scope);
        scope.into_actions()
    }

    fn on_value<S: Serializer>(&self, serializer: S, value: Value) -> wrapper::OnValueAction<S> {
        let path = &self.inner.borrow().path;

        let mut scope = ValueScope::new(path, serializer, value);
        self.inner.borrow().hooks.on_value(&mut scope);
        scope.into_action()
    }
}

impl<H: Hooks> Context<H> {
    pub(super) fn new(hooks: H) -> Self {
        Self {
            inner: Rc::new(RefCell::new(ContextInner {
                path: Default::default(),
                hooks,
            })),
        }
    }

    pub(super) fn start(&self) {
        self.inner.borrow().hooks.start();
    }

    pub(super) fn end(&self) {
        self.inner.borrow().hooks.end();
    }
}

#[derive(Debug)]
struct ContextInner<H: Hooks> {
    path: Path,
    hooks: H,
}
