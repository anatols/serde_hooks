use std::{cell::RefCell, rc::Rc};

use serde::{Serialize, Serializer};

use super::scope::{
    self, EnumVariantScope, ErrorScope, MapKeyScope, MapScope, SeqScope, StructScope, TupleScope,
    TupleStructScope, ValueScope,
};
use super::wrapper;
use crate::path::{Path, PathSegment};
use crate::ser::{Hooks, HooksError};
use crate::Value;

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

    fn on_map(&self, map_len: Option<usize>) -> scope::OnMapEntryActions {
        let path = &self.inner.borrow().path;
        let mut scope = MapScope::new(path, map_len);
        self.inner.borrow().hooks.on_map(&mut scope);
        scope.into_actions()
    }

    fn on_struct(
        &self,
        struct_len: usize,
        struct_name: &'static str,
    ) -> scope::OnStructFieldActions {
        let path = &self.inner.borrow().path;
        let mut scope = StructScope::new(path, struct_len, struct_name);
        self.inner.borrow().hooks.on_struct(&mut scope);
        scope.into_actions()
    }

    fn on_struct_variant(
        &self,
        struct_len: usize,
        enum_name: &'static str,
        variant_name: &'static str,
        variant_index: u32,
    ) -> (scope::OnVariantActions, scope::OnStructFieldActions) {
        let path = &self.inner.borrow().path;

        let mut variant_scope = EnumVariantScope::new(path, enum_name, variant_name, variant_index);
        let mut struct_scope = StructScope::new(path, struct_len, variant_name);

        let hooks = self.inner.borrow().hooks;

        hooks.on_enum_variant(&mut variant_scope);
        hooks.on_struct(&mut struct_scope);
        hooks.on_struct_variant(&mut variant_scope, &mut struct_scope);

        (variant_scope.into_actions(), struct_scope.into_actions())
    }

    fn on_map_key<S: Serializer>(&self, serializer: S, value: Value) -> scope::OnValueAction<S> {
        let path = &self.inner.borrow().path;

        let mut scope = MapKeyScope::new(path, serializer, value);
        self.inner.borrow().hooks.on_map_key(&mut scope);
        scope.into_action()
    }

    fn on_value<S: Serializer>(&self, serializer: S, value: Value) -> scope::OnValueAction<S> {
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

    fn on_seq(&self, len: Option<usize>) -> scope::OnSeqElementActions {
        let path = &self.inner.borrow().path;

        let mut scope = SeqScope::new(path, len);
        self.inner.borrow().hooks.on_seq(&mut scope);
        scope.into_actions()
    }

    fn on_unit_variant(
        &self,
        enum_name: &'static str,
        variant_name: &'static str,
        variant_index: u32,
    ) -> scope::OnVariantActions {
        let path = &self.inner.borrow().path;

        let mut variant_scope = EnumVariantScope::new(path, enum_name, variant_name, variant_index);

        let hooks = self.inner.borrow().hooks;
        hooks.on_enum_variant(&mut variant_scope);

        variant_scope.into_actions()
    }

    fn on_newtype_variant(
        &self,
        enum_name: &'static str,
        variant_name: &'static str,
        variant_index: u32,
    ) -> scope::OnVariantActions {
        let path = &self.inner.borrow().path;

        let mut variant_scope = EnumVariantScope::new(path, enum_name, variant_name, variant_index);

        let hooks = self.inner.borrow().hooks;
        hooks.on_enum_variant(&mut variant_scope);

        variant_scope.into_actions()
    }

    fn on_tuple(&self, len: usize) -> scope::OnSeqElementActions {
        let path = &self.inner.borrow().path;

        let mut tuple_scope = TupleScope::new(path, len);
        let mut seq_scope = SeqScope::new(path, Some(len));

        let hooks = self.inner.borrow().hooks;

        hooks.on_tuple(&mut tuple_scope, &mut seq_scope);

        seq_scope.into_actions()
    }

    fn on_tuple_struct(&self, name: &'static str, len: usize) -> scope::OnSeqElementActions {
        let path = &self.inner.borrow().path;

        let mut tuple_scope = TupleScope::new(path, len);
        let mut tuple_struct_scope = TupleStructScope::new(path, name, len);
        let mut seq_scope = SeqScope::new(path, Some(len));

        let hooks = self.inner.borrow().hooks;

        hooks.on_tuple(&mut tuple_scope, &mut seq_scope);
        hooks.on_tuple_struct(&mut tuple_struct_scope, &mut seq_scope);

        seq_scope.into_actions()
    }

    fn on_tuple_variant(
        &self,
        enum_name: &'static str,
        variant_index: u32,
        variant_name: &'static str,
        len: usize,
    ) -> (scope::OnVariantActions, scope::OnSeqElementActions) {
        let path = &self.inner.borrow().path;

        let mut variant_scope = EnumVariantScope::new(path, enum_name, variant_name, variant_index);
        let mut tuple_scope = TupleScope::new(path, len);
        let mut seq_scope = SeqScope::new(path, Some(len));

        let hooks = self.inner.borrow().hooks;

        hooks.on_enum_variant(&mut variant_scope);
        hooks.on_tuple(&mut tuple_scope, &mut seq_scope);
        hooks.on_tuple_variant(&mut variant_scope, &mut tuple_scope, &mut seq_scope);

        (variant_scope.into_actions(), seq_scope.into_actions())
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
