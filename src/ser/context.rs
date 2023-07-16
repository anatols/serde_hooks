use std::{cell::RefCell, rc::Rc};

use serde::{Serialize, Serializer};

use super::hooks::{ErrorScope, Hooks, HooksError, MapKeyScope, MapScope, StructScope, ValueScope};
use super::path::{Path, PathSegment};
use super::wrapper;
use super::Value;

pub struct SerializableWithContext<'s, 'h, T: Serialize + ?Sized, H: Hooks> {
    serializable: &'s T,
    context: Context<'h, H>,
}

impl<'s, 'h, T: Serialize + ?Sized, H: Hooks> SerializableWithContext<'s, 'h, T, H> {
    pub(super) fn new(serializable: &'s T, hooks: &'h H) -> Self {
        Self {
            serializable,
            context: Context::new(hooks),
        }
    }
}

impl<T: Serialize + ?Sized, H: Hooks> Serialize for SerializableWithContext<'_, '_, T, H> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.context.start();
        let res = self.serializable.serialize(wrapper::SerializerWrapper::new(
            serializer,
            &self.context,
            wrapper::SerializableKind::Value,
        ));
        self.context.end();
        res
    }
}

#[derive(Debug, Clone)]
pub struct Context<'h, H: Hooks> {
    inner: Rc<RefCell<ContextInner<'h, H>>>,
}

impl<H: Hooks> wrapper::SerializerWrapperHooks for Context<'_, H> {
    fn path_push(&self, segment: PathSegment) {
        self.inner.borrow_mut().path.push_segment(segment);
    }

    fn path_pop(&self) {
        self.inner.borrow_mut().path.pop_segment();
    }

    fn on_map(&self, map_len: Option<usize>) -> wrapper::OnMapEntryActions {
        let path = &self.inner.borrow().path;
        let mut scope = MapScope::new(path, map_len);
        self.inner.borrow().hooks.on_map(&mut scope);
        scope.into_actions()
    }

    fn on_struct(
        &self,
        struct_len: usize,
        struct_name: &'static str,
    ) -> wrapper::OnStructFieldActions {
        let path = &self.inner.borrow().path;
        let mut scope = StructScope::new(path, struct_len, struct_name);
        self.inner.borrow().hooks.on_struct(&mut scope);
        scope.into_actions()
    }

    fn on_map_key<S: Serializer>(&self, serializer: S, value: Value) -> wrapper::OnValueAction<S> {
        let path = &self.inner.borrow().path;

        let mut scope = MapKeyScope::new(path, serializer, value);
        self.inner.borrow().hooks.on_map_key(&mut scope);
        scope.into_action()
    }

    fn on_value<S: Serializer>(&self, serializer: S, value: Value) -> wrapper::OnValueAction<S> {
        let path = &self.inner.borrow().path;

        let mut scope = ValueScope::new(path, serializer, value);
        self.inner.borrow().hooks.on_value(&mut scope);
        scope.into_action()
    }

    fn on_error<S: Serializer>(&self, error: HooksError) -> Result<(), S::Error> {
        let path = &self.inner.borrow().path;

        let mut scope = ErrorScope::new(path, error);
        self.inner.borrow().hooks.on_error(&mut scope);
        scope.into_result::<S>()
    }
}

impl<'h, H: Hooks> Context<'h, H> {
    pub(super) fn new(hooks: &'h H) -> Self {
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
struct ContextInner<'h, H: Hooks> {
    path: Path,
    hooks: &'h H,
}
