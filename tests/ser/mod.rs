mod r#struct;

use mockall::predicate::*;
use mockall::*;
use serde::Serializer;

use serde_hooks::ser::{ErrorScope, Hooks, MapKeyScope, MapScope, StructScope, ValueScope};
use serde_hooks::{Path, Value};

mock! {
    Hooks {
        fn start(&self);
        fn end(&self);
        fn on_error<'p>(&self, err: &mut serde_hooks::ser::ErrorScope<'p>);
        fn on_map<'p>(&self, map: &mut serde_hooks::ser::MapScope<'p>);
        fn on_map_key(&self, path: &Path, value: &Value);
        fn on_struct<'p>(&self, st: &mut serde_hooks::ser::StructScope<'p>);
        fn on_value(&self, path: &Path, value: &Value);
    }
}

impl Hooks for MockHooks {
    fn start(&self) {
        MockHooks::start(self)
    }

    fn end(&self) {
        MockHooks::end(self)
    }

    fn on_error(&self, err: &mut ErrorScope) {
        MockHooks::on_error(self, err)
    }

    fn on_map(&self, map: &mut MapScope) {
        MockHooks::on_map(self, map)
    }

    fn on_map_key<S: Serializer>(&self, map_key: &mut MapKeyScope<S>) {
        MockHooks::on_map_key(self, map_key.path(), map_key.value())
    }

    fn on_struct(&self, st: &mut StructScope) {
        MockHooks::on_struct(self, st)
    }

    fn on_value<S: Serializer>(&self, value: &mut ValueScope<S>) {
        MockHooks::on_value(self, value.path(), value.value())
    }
}
