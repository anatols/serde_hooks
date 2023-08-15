use std::{cell::RefCell, collections::HashSet};

use serde::Serialize;
use serde_hooks::{ser, Path, StaticValue};

#[test]
fn test_seq_traversing() {
    #[derive(Serialize)]
    struct Outer {
        vec: Vec<i32>,
        nested: Vec<Vec<i32>>,
    }

    let outer = Outer {
        vec: vec![1, 2, 3],
        nested: vec![vec![4, 5], vec![6]],
    };

    struct Hooks {
        fields_to_expect: RefCell<HashSet<String>>,
    }
    impl ser::Hooks for Hooks {
        fn on_seq(&self, path: &Path, seq: &mut ser::SeqScope) {
            let path = path.borrow_str();
            self.fields_to_expect.borrow_mut().remove(&*path);

            match path.as_ref() {
                "vec" => {
                    assert_eq!(seq.seq_len(), Some(3));
                }
                "nested" => {
                    assert_eq!(seq.seq_len(), Some(2));
                }
                "nested[0]" => {
                    assert_eq!(seq.seq_len(), Some(2));
                }
                "nested[1]" => {
                    assert_eq!(seq.seq_len(), Some(1));
                }
                _ => unreachable!("{path}"),
            }
        }
    }
    let hooks = Hooks {
        fields_to_expect: RefCell::new(
            ["vec", "nested", "nested[0]", "nested[1]"]
                .into_iter()
                .map(Into::into)
                .collect(),
        ),
    };

    serde_json::to_string(&ser::hook(&outer, &hooks)).unwrap();
    assert!(
        hooks.fields_to_expect.borrow().is_empty(),
        "following fields were expected, but not called back about {:?}",
        hooks.fields_to_expect.borrow()
    );
}

#[test]
fn test_seq_skip_element() {
    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_seq(&self, _path: &Path, seq: &mut ser::SeqScope) {
            seq.skip_element(1).skip_element(12345);
        }

        fn on_scope_error(&self, path: &Path, err: &mut ser::ErrorScope) {
            assert_eq!(&*path.borrow_str(), "");
            assert_eq!(*err.error(), ser::HooksError::IndexNotFound(12345));
            err.ignore();
        }
    }

    let json = serde_json::to_string(&ser::hook(&vec![0i32, 1, 2], &Hooks)).unwrap();
    assert_eq!(json, "[0,2]");
}

#[test]
fn test_seq_retain_element() {
    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_seq(&self, _path: &Path, seq: &mut ser::SeqScope) {
            seq.retain_element(1)
                .retain_element(2)
                .retain_element(12345);
        }

        fn on_scope_error(&self, path: &Path, err: &mut ser::ErrorScope) {
            assert_eq!(&*path.borrow_str(), "");
            assert_eq!(*err.error(), ser::HooksError::IndexNotFound(12345));
            err.ignore();
        }
    }

    let json = serde_json::to_string(&ser::hook(&vec![0i32, 1, 2, 3], &Hooks)).unwrap();
    assert_eq!(json, "[1,2]");
}

#[test]
fn test_seq_replace_value() {
    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_seq(&self, _path: &Path, seq: &mut ser::SeqScope) {
            seq.replace_value(1, -10i32)
                .replace_value(2, 'a')
                .replace_value(12345, "error");
        }

        fn on_scope_error(&self, path: &Path, err: &mut ser::ErrorScope) {
            assert_eq!(&*path.borrow_str(), "");
            assert_eq!(*err.error(), ser::HooksError::IndexNotFound(12345));
            err.ignore();
        }
    }

    let json = serde_json::to_string(&ser::hook(&vec![0i32, 1, 2, 3], &Hooks)).unwrap();
    assert_eq!(json, "[0,-10,\"a\",3]");
}

#[test]
fn test_seq_replace_value_unserializable() {
    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_seq(&self, _path: &Path, seq: &mut ser::SeqScope) {
            seq.replace_value(1, StaticValue::NewtypeStruct("STRUCT"));
        }
    }

    let err = serde_json::to_string(&ser::hook(&vec![0i32, 1, 2, 3], &Hooks)).unwrap_err();
    assert_eq!(err.to_string(), "Error at path '[1]': value is not serializable: newtype STRUCT cannot be represented fully in Value");
}
