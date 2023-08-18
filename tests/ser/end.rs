use std::{cell::Cell, pin::Pin};

use serde::Serialize;
use serde_hooks::ser;

#[test]
fn test_is_called() {
    struct Hooks {
        is_called: Cell<bool>,
    }
    impl ser::Hooks for Hooks {
        fn on_end<Error: serde::ser::Error>(&self, _end: &mut ser::EndScope<Error>) {
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

        fn on_end<Error: serde::ser::Error>(&self, end: &mut ser::EndScope<Error>) {
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

#[test]
fn test_ok_result() {
    struct Hooks {
        result: Cell<Option<Result<(), String>>>,
    }
    impl ser::Hooks for Hooks {
        fn on_end<Error: serde::ser::Error>(&self, end: &mut ser::EndScope<Error>) {
            self.result
                .set(Some(end.result().map_err(|err| err.to_string())));
        }
    }
    let hooks = Hooks {
        result: Cell::new(None),
    };

    serde_json::to_string(&ser::hook(&(), &hooks)).unwrap();
    assert!(hooks.result.into_inner().unwrap().is_ok());
}

#[test]
fn test_error_result_bincode() {
    #[derive(Serialize)]
    struct Payload {
        vec: Vec<u8>,
    }
    struct Hooks {
        result: Cell<Option<Result<(), String>>>,
    }
    impl ser::Hooks for Hooks {
        fn on_seq(&self, _path: &serde_hooks::Path, seq: &mut ser::SeqScope) {
            // We rely here on bincode not being capable of serializing
            // sequences of unknown length. Skipping a sequence element makes it
            // unknown length in current implementation of SeqScope.
            seq.skip_element(0);
        }

        fn on_end<Error: serde::ser::Error>(&self, end: &mut ser::EndScope<Error>) {
            self.result
                .set(Some(end.result().map_err(|err| err.to_string())));
        }
    }
    let hooks = Hooks {
        result: Cell::new(None),
    };

    let err = bincode::serialize(&ser::hook(&Payload { vec: vec![1, 2, 3] }, &hooks)).unwrap_err();
    let hooks_err = hooks.result.into_inner().unwrap().unwrap_err();
    assert_eq!(err.to_string(), hooks_err);
}

#[test]
fn test_error_result_failed_from_hook() {
    struct Hooks {
        result: Cell<Option<Result<(), String>>>,
    }
    impl ser::Hooks for Hooks {
        fn on_value<S: serde::Serializer>(
            &self,
            _path: &serde_hooks::Path,
            value: &mut ser::ValueScope<S>,
        ) {
            value.fail_serialization("FAUX ERROR");
        }

        fn on_end<Error: serde::ser::Error>(&self, end: &mut ser::EndScope<Error>) {
            self.result
                .set(Some(end.result().map_err(|err| err.to_string())));
        }
    }
    let hooks = Hooks {
        result: Cell::new(None),
    };

    let err = bincode::serialize(&ser::hook(&(), &hooks)).unwrap_err();
    let hooks_err = hooks.result.into_inner().unwrap().unwrap_err();
    assert_eq!(err.to_string(), hooks_err);
    assert!(hooks_err.contains("FAUX ERROR"));
}
