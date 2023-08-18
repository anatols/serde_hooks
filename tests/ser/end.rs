use std::{cell::Cell, pin::Pin};

use serde::Serialize;
use serde_hooks::ser;

#[test]
fn test_is_called() {
    struct Hooks {
        is_called: Cell<bool>,
    }
    impl ser::Hooks for Hooks {
        fn on_end(&self, _end: &mut ser::EndScope) {
            self.is_called.set(true);
        }
    }
    let hooks = Hooks {
        is_called: Cell::new(false),
    };

    serde_json::to_string(&ser::hook(&(), &hooks)).unwrap();
    assert!(hooks.is_called.get());
}

#[test]
fn test_take_static_strs() {
    #[derive(Serialize)]
    struct Payload {
        field: (),
    }

    struct Hooks {
        static_strs: Cell<Vec<Pin<Box<str>>>>,
    }

    impl ser::Hooks for Hooks {
        fn on_struct(&self, _path: &serde_hooks::Path, st: &mut ser::StructScope) {
            st.rename_all_fields_case("UPPERCASE");
        }

        fn on_end(&self, end: &mut ser::EndScope) {
            unsafe { self.static_strs.set(end.take_static_strs()) }
        }
    }
    let hooks = Hooks {
        static_strs: Cell::new(vec![]),
    };

    let json = serde_json::to_string(&ser::hook(&Payload { field: () }, &hooks)).unwrap();
    assert_eq!(json, "{\"FIELD\":null}");

    let static_strs = hooks.static_strs.into_inner();
    assert_eq!(static_strs.len(), 1);
    assert_eq!(Pin::get_ref(static_strs[0].as_ref()), "FIELD");
}
