use std::{cell::Cell, rc::Rc};

use serde::Serialize;
use serde_hooks::ser;

use super::MockHooks;

//TODO
// error conditions: not matching fields

#[derive(Serialize)]
struct Payload {
    foo: i32,
    bar: Option<char>,
    baz: String,
}

impl Payload {
    fn new() -> Self {
        Payload {
            foo: 42,
            bar: Some('a'),
            baz: "sample".into(),
        }
    }
}

#[test]
fn test_struct_traversing() {
    #[derive(Serialize)]
    struct Outer {
        sample: u32,
        payload: Payload,
    }

    let outer = Outer {
        sample: 123,
        payload: Payload::new(),
    };

    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct(&self, st: &mut ser::StructScope) {
            //TODO use mock to ensure this is called
            match st.path().to_string().as_str() {
                "$" => {
                    assert_eq!(st.struct_name(), "Outer");
                    assert_eq!(st.struct_len(), 2);
                }
                "$.payload" => {
                    assert_eq!(st.struct_name(), "Payload");
                    assert_eq!(st.struct_len(), 3);
                }
                _ => unreachable!(),
            }
        }
    }

    serde_json::to_string(&ser::hook(&outer, Hooks)).unwrap();
}

#[test]
fn test_skip_field() {
    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct(&self, st: &mut ser::StructScope) {
            st.skip_field("foo").skip_field("baz");
        }
    }

    let json = serde_json::to_string(&ser::hook(&Payload::new(), Hooks)).unwrap();
    assert_eq!(json, r#"{"bar":"a"}"#);

    let yaml = serde_yaml::to_string(&ser::hook(&Payload::new(), Hooks)).unwrap();
    assert_eq!(yaml, "bar: 'a'\n");
}

#[test]
fn test_retain_field() {
    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct(&self, st: &mut ser::StructScope) {
            st.retain_field("foo").retain_field("bar");
        }
    }

    let json = serde_json::to_string(&ser::hook(&Payload::new(), Hooks)).unwrap();
    assert_eq!(json, r#"{"foo":42,"bar":"a"}"#);

    let yaml = serde_yaml::to_string(&ser::hook(&Payload::new(), Hooks)).unwrap();
    assert_eq!(yaml, "foo: 42\nbar: 'a'\n");
}

#[test]
fn test_rename_field() {
    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct(&self, st: &mut ser::StructScope) {
            st.rename_field("foo", "not_foo")
                .rename_field("bar", format!("bar_{}", 42))
                .rename_field("baz", "baz2")
                .rename_field("baz2", "baz3");
        }
    }

    let json = serde_json::to_string(&ser::hook(&Payload::new(), Hooks)).unwrap();
    assert_eq!(json, r#"{"not_foo":42,"bar_42":"a","baz3":"sample"}"#);

    let yaml = serde_yaml::to_string(&ser::hook(&Payload::new(), Hooks)).unwrap();
    assert_eq!(yaml, "not_foo: 42\nbar_42: 'a'\nbaz3: sample\n");
}

#[test]
fn test_replace_value() {
    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct(&self, st: &mut ser::StructScope) {
            st.replace_value("baz", -15i16);
        }
    }

    let json = serde_json::to_string(&ser::hook(&Payload::new(), Hooks)).unwrap();
    assert_eq!(json, r#"{"foo":42,"bar":"a","baz":-15}"#);

    let yaml = serde_yaml::to_string(&ser::hook(&Payload::new(), Hooks)).unwrap();
    assert_eq!(yaml, "foo: 42\nbar: 'a'\nbaz: -15\n");
}

#[test]
fn test_error() {
    // let mut hooks = MockHooks::new();
    // hooks.expect_start().return_const(());
    // hooks.expect_end().return_const(());
    // hooks
    //     .expect_on_struct()
    //     .return_once(|st: &mut ser::StructScope| {
    //         st.retain_field("invalid");
    //     });

    // hooks
    //     .expect_on_error()
    //     .return_once(|err: &mut ser::ErrorScope| {
    //         assert_eq!(err.path().to_string(), "$");
    //         assert_eq!(
    //             *err.error(),
    //             ser::HooksError::FieldNotFound("invalid".into())
    //         );
    //         err.propagate();
    //     });

    let on_error_called = Rc::new(Cell::new(false));
    struct Hooks {
        on_error_called: Rc<Cell<bool>>,
    }
    impl ser::Hooks for Hooks {
        fn on_struct(&self, st: &mut ser::StructScope) {
            //TODO test other functions
            st.retain_field("invalid");
        }

        fn on_error(&self, err: &mut ser::ErrorScope) {
            //TODO use mock to ensure this is called
            assert_eq!(err.path().to_string(), "$");
            assert_eq!(
                *err.error(),
                ser::HooksError::FieldNotFound("invalid".into())
            );
            err.propagate();
            self.on_error_called.set(true);
        }
    }
    let hooks = Hooks {
        on_error_called: on_error_called.clone(),
    };
    assert!(serde_json::to_string(&ser::hook(&Payload::new(), hooks)).is_err());
    assert!(on_error_called.get());
}
