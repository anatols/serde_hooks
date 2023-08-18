use std::borrow::Cow;
use std::cell::{RefCell, RefMut};
use std::pin::Pin;
use std::rc::Rc;

use serde::{Serialize, Serializer};

use super::scope::{
    EnumVariantScope, ErrorScope, MapKeyScope, MapScope, SeqScope, StartScope, StructScope,
    TupleScope, TupleStructScope, ValueScope,
};
use super::wrapper::{
    MapEntryActions, SeqElementActions, SerializableKind, SerializerWrapper,
    SerializerWrapperHooks, StructFieldActions, ValueAction, VariantActions,
};
use super::EndScope;
use crate::path::{Path, PathSegment};
use crate::ser::{Hooks, HooksError};
use crate::Value;

pub(crate) struct SerializableWithContext<'s, 'h, T: Serialize + ?Sized, H: Hooks> {
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
        self.context.on_start(serializer.is_human_readable());
        let res = self.serializable.serialize(SerializerWrapper::new(
            serializer,
            &self.context,
            SerializableKind::Value,
        ));
        self.context.on_end();
        res
    }
}

#[derive(Clone)]
pub(crate) struct Context<'h, H: Hooks> {
    inner: Rc<RefCell<ContextInner<'h, H>>>,
}

impl<H: Hooks> SerializerWrapperHooks for Context<'_, H> {
    fn path_push(&self, segment: PathSegment) {
        self.inner.borrow_mut().path.push_segment(segment);
    }

    fn path_pop(&self) {
        self.inner.borrow_mut().path.pop_segment();
    }

    fn on_map(&self, map_len: Option<usize>) -> MapEntryActions {
        let path = &self.inner.borrow().path;
        let mut scope = MapScope::new(map_len);
        self.inner.borrow().hooks.on_map(path, &mut scope);
        scope.into_actions()
    }

    fn on_struct(&self, struct_len: usize, struct_name: &'static str) -> StructFieldActions {
        let path = &self.inner.borrow().path;
        let mut scope = StructScope::new(struct_len, struct_name);
        self.inner.borrow().hooks.on_struct(path, &mut scope);
        scope.into_actions()
    }

    fn on_struct_variant(
        &self,
        struct_len: usize,
        enum_name: &'static str,
        variant_name: &'static str,
        variant_index: u32,
    ) -> (VariantActions, StructFieldActions) {
        let path = &self.inner.borrow().path;

        let mut variant_scope = EnumVariantScope::new(enum_name, variant_name, variant_index);
        let mut struct_scope = StructScope::new(struct_len, variant_name);

        let hooks = self.inner.borrow().hooks;

        hooks.on_enum_variant(path, &mut variant_scope);
        hooks.on_struct(path, &mut struct_scope);
        hooks.on_struct_variant(path, &mut variant_scope, &mut struct_scope);

        (variant_scope.into_actions(), struct_scope.into_actions())
    }

    fn on_map_key<S: Serializer>(&self, serializer: S, value: Value) -> ValueAction<S> {
        let path = &self.inner.borrow().path;

        let mut scope = MapKeyScope::new(serializer, value);
        self.inner.borrow().hooks.on_map_key(path, &mut scope);
        scope.into_action()
    }

    fn on_value<S: Serializer>(&self, serializer: S, value: Value) -> ValueAction<S> {
        let path = &self.inner.borrow().path;

        let mut scope = ValueScope::new(serializer, value);
        self.inner.borrow().hooks.on_value(path, &mut scope);
        scope.into_action()
    }

    fn on_error<S: Serializer>(&self, error: HooksError) -> Result<(), S::Error> {
        let path = &self.inner.borrow().path;

        let mut scope = ErrorScope::new(path, error);
        self.inner.borrow().hooks.on_scope_error(path, &mut scope);
        scope.into_result::<S>()
    }

    fn on_seq(&self, len: Option<usize>) -> SeqElementActions {
        let path = &self.inner.borrow().path;

        let mut scope = SeqScope::new(len);
        self.inner.borrow().hooks.on_seq(path, &mut scope);
        scope.into_actions()
    }

    fn on_unit_variant(
        &self,
        enum_name: &'static str,
        variant_name: &'static str,
        variant_index: u32,
    ) -> VariantActions {
        let path = &self.inner.borrow().path;

        let mut variant_scope = EnumVariantScope::new(enum_name, variant_name, variant_index);

        let hooks = self.inner.borrow().hooks;
        hooks.on_enum_variant(path, &mut variant_scope);

        variant_scope.into_actions()
    }

    fn on_newtype_variant(
        &self,
        enum_name: &'static str,
        variant_name: &'static str,
        variant_index: u32,
    ) -> VariantActions {
        let path = &self.inner.borrow().path;

        let mut variant_scope = EnumVariantScope::new(enum_name, variant_name, variant_index);

        let hooks = self.inner.borrow().hooks;
        hooks.on_enum_variant(path, &mut variant_scope);

        variant_scope.into_actions()
    }

    fn on_tuple(&self, len: usize) -> SeqElementActions {
        let path = &self.inner.borrow().path;

        let mut tuple_scope = TupleScope::new(len);
        let mut seq_scope = SeqScope::new(Some(len));

        let hooks = self.inner.borrow().hooks;

        hooks.on_seq(path, &mut seq_scope);
        hooks.on_tuple(path, &mut tuple_scope, &mut seq_scope);

        seq_scope.into_actions()
    }

    fn on_tuple_struct(&self, name: &'static str, len: usize) -> SeqElementActions {
        let path = &self.inner.borrow().path;

        let mut tuple_scope = TupleScope::new(len);
        let mut tuple_struct_scope = TupleStructScope::new(name, len);
        let mut seq_scope = SeqScope::new(Some(len));

        let hooks = self.inner.borrow().hooks;

        hooks.on_seq(path, &mut seq_scope);
        hooks.on_tuple(path, &mut tuple_scope, &mut seq_scope);
        hooks.on_tuple_struct(path, &mut tuple_struct_scope, &mut seq_scope);

        seq_scope.into_actions()
    }

    fn on_tuple_variant(
        &self,
        enum_name: &'static str,
        variant_index: u32,
        variant_name: &'static str,
        len: usize,
    ) -> (VariantActions, SeqElementActions) {
        let path = &self.inner.borrow().path;

        let mut variant_scope = EnumVariantScope::new(enum_name, variant_name, variant_index);
        let mut tuple_scope = TupleScope::new(len);
        let mut seq_scope = SeqScope::new(Some(len));

        let hooks = self.inner.borrow().hooks;

        hooks.on_enum_variant(path, &mut variant_scope);
        hooks.on_seq(path, &mut seq_scope);
        hooks.on_tuple(path, &mut tuple_scope, &mut seq_scope);
        hooks.on_tuple_variant(path, &mut variant_scope, &mut tuple_scope, &mut seq_scope);

        (variant_scope.into_actions(), seq_scope.into_actions())
    }

    fn into_static_str(&self, key: std::borrow::Cow<'static, str>) -> &'static str {
        match key {
            Cow::Borrowed(static_key) => static_key,
            Cow::Owned(string_key) => {
                let mut static_strs = RefMut::map(self.inner.borrow_mut(), |r| &mut r.static_strs);
                let boxed_key = Pin::new(string_key.into_boxed_str());
                let static_key = unsafe { std::mem::transmute::<&str, &'static str>(&boxed_key) };
                static_strs.push(boxed_key);
                static_key
            }
        }
    }
}

impl<'h, H: Hooks> Context<'h, H> {
    pub(super) fn new(hooks: &'h H) -> Self {
        Self {
            inner: Rc::new(RefCell::new(ContextInner {
                path: Path::new(),
                hooks,
                static_strs: Vec::new(),
            })),
        }
    }

    pub(super) fn on_start(&self, is_human_readable: bool) {
        self.inner
            .borrow()
            .hooks
            .on_start(&mut StartScope::new(is_human_readable));
    }

    pub(super) fn on_end(&self) {
        let static_strs = std::mem::take(&mut self.inner.borrow_mut().static_strs);
        self.inner
            .borrow()
            .hooks
            .on_end(&mut EndScope::new(static_strs));
    }
}

struct ContextInner<'h, H: Hooks> {
    path: Path,
    hooks: &'h H,
    static_strs: Vec<Pin<Box<str>>>,
}

#[test]
fn test_into_static_str() {
    // Comparing references here, not content
    fn assert_refs_eq(left: &str, right: &str) {
        assert_eq!(left as *const _, right as *const _);
    }

    fn assert_refs_ne(left: &str, right: &str) {
        assert_ne!(left as *const _, right as *const _);
    }

    struct FauxHooks;
    impl Hooks for FauxHooks {}
    let context = Context::new(&FauxHooks);

    // Static strings are just pass-through
    let foo_str: &'static str = "foo";
    assert_refs_eq(context.into_static_str(Cow::Borrowed(foo_str)), foo_str);
    let bar_str: &'static str = "bar";
    assert_refs_eq(context.into_static_str(Cow::Borrowed(bar_str)), bar_str);

    // Pass-through, even if the value is repeating
    let bar_str_again: &'static str = &"_bar"[1..]; // slice shenanigans, to stop compiler from reusing strs.
    assert_refs_ne(bar_str, bar_str_again);
    assert_refs_eq(
        context.into_static_str(Cow::Borrowed(bar_str_again)),
        bar_str_again,
    );

    // Owned values are cached
    let baz_str = "baz";
    let first_instance: &'static str = context.into_static_str(Cow::Owned(baz_str.to_string()));
    assert_refs_ne(baz_str, first_instance);

    // Static strings are still pass-through, even if we have cached the exact same
    // owned one
    assert_refs_eq(context.into_static_str(Cow::Borrowed(baz_str)), baz_str);
}
