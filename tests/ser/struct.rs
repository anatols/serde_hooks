use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
};

use serde::Serialize;
use serde_hooks::{ser, Case, Path, StaticValue};

#[derive(Serialize)]
enum Enum {
    #[allow(dead_code)]
    Faux,
    StructVariant {
        foo: i32,
        bar: Option<char>,
        baz: String,
    },
}

#[derive(Serialize)]
struct Payload {
    foo: i32,
    bar: Option<char>,
    baz: String,
    e: Enum,
}

impl Payload {
    fn new() -> Self {
        Payload {
            foo: 42,
            bar: Some('a'),
            baz: "sample".into(),
            e: Enum::StructVariant {
                foo: 21,
                bar: Some('b'),
                baz: "example".into(),
            },
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

    struct Hooks {
        fields_to_expect: RefCell<HashSet<String>>,
    }
    impl ser::Hooks for Hooks {
        fn on_struct(&self, path: &Path, st: &mut ser::StructScope) {
            let path = path.borrow_str();
            self.fields_to_expect.borrow_mut().remove(&*path);

            match path.as_ref() {
                "" => {
                    assert_eq!(st.struct_name(), "Outer");
                    assert_eq!(st.struct_len(), 2);
                }
                "payload" => {
                    assert_eq!(st.struct_name(), "Payload");
                    assert_eq!(st.struct_len(), 4);
                }
                "payload.e" => {
                    assert_eq!(st.struct_name(), "StructVariant");
                    assert_eq!(st.struct_len(), 3);
                }
                _ => unreachable!("{path}"),
            }
        }

        fn on_struct_variant(
            &self,
            path: &Path,
            ev: &mut ser::EnumVariantScope,
            st: &mut ser::StructScope,
        ) {
            let path = path.borrow_str();
            self.fields_to_expect.borrow_mut().remove(&*path);

            match path.as_ref() {
                "payload.e" => {
                    assert_eq!(ev.enum_name(), "Enum");
                    assert_eq!(ev.variant_index(), 1);
                    assert_eq!(ev.variant_name(), "StructVariant");
                    assert_eq!(st.struct_name(), "StructVariant");
                    assert_eq!(st.struct_len(), 3);
                }
                _ => unreachable!("{path}"),
            }
        }
    }
    let hooks = Hooks {
        fields_to_expect: RefCell::new(
            ["", "payload", "payload.e"]
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
fn test_skip_field() {
    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct(&self, _path: &Path, st: &mut ser::StructScope) {
            st.skip_field("foo").skip_field("baz");
        }

        fn on_struct_variant(
            &self,
            _path: &Path,
            _ev: &mut ser::EnumVariantScope,
            st: &mut ser::StructScope,
        ) {
            st.skip_field("foo").skip_field("baz");
        }
    }

    let json = serde_json::to_string(&ser::hook(&Payload::new(), &Hooks)).unwrap();
    assert_eq!(
        json,
        "{\"bar\":\"a\",\"e\":{\"StructVariant\":{\"bar\":\"b\"}}}"
    );
}

#[test]
fn test_retain_field() {
    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct(&self, _path: &Path, st: &mut ser::StructScope) {
            st.retain_field("foo").retain_field("bar");
        }
    }

    let json = serde_json::to_string(&ser::hook(&Payload::new(), &Hooks)).unwrap();
    assert_eq!(json, r#"{"foo":42,"bar":"a"}"#);
}

#[test]
fn test_retain_field_in_struct_variant() {
    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct(&self, path: &Path, st: &mut ser::StructScope) {
            if path.segments().is_empty() {
                st.retain_field("e");
            }
        }

        fn on_struct_variant(
            &self,
            _path: &Path,
            _ev: &mut ser::EnumVariantScope,
            st: &mut ser::StructScope,
        ) {
            st.retain_field("foo").retain_field("bar");
        }
    }

    let json = serde_json::to_string(&ser::hook(&Payload::new(), &Hooks)).unwrap();
    assert_eq!(
        json,
        "{\"e\":{\"StructVariant\":{\"foo\":21,\"bar\":\"b\"}}}"
    );
}

#[test]
fn test_rename_field() {
    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct(&self, path: &Path, st: &mut ser::StructScope) {
            if path.segments().is_empty() {
                st.rename_field("foo", "not_foo")
                    .rename_field("bar", format!("bar_{}", 42))
                    .rename_field("baz", "baz2")
                    .rename_field("baz2", "baz3");
            }
        }

        fn on_struct_variant(
            &self,
            _path: &Path,
            _ev: &mut ser::EnumVariantScope,
            st: &mut ser::StructScope,
        ) {
            st.rename_field("foo", "not_foo_either")
                .rename_field("bar", format!("bar_{}", 21))
                .rename_field("baz", "baz4")
                .rename_field("baz4", "baz5");
        }
    }

    let json = serde_json::to_string(&ser::hook(&Payload::new(), &Hooks)).unwrap();
    assert_eq!(json, "{\"not_foo\":42,\"bar_42\":\"a\",\"baz3\":\"sample\",\"e\":{\"StructVariant\":{\"not_foo_either\":21,\"bar_21\":\"b\",\"baz5\":\"example\"}}}");
}

#[test]
fn test_rename_all_fields() {
    #[derive(Serialize)]
    enum Enum {
        #[serde(rename_all = "kebab-case")]
        StructVariant { baz_foo: () },
    }

    #[derive(Serialize)]
    #[serde(rename_all = "SCREAMING-KEBAB-CASE")]
    struct Cases {
        foo_bar: (),
        bar_baz: (),
        e: Enum,
    }

    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct(&self, path: &Path, st: &mut ser::StructScope) {
            if path.segments().is_empty() {
                st.rename_all_fields_case("PascalCase")
                    .rename_field("BAR-BAZ", "bbz");
            }
        }

        fn on_struct_variant(
            &self,
            _path: &Path,
            _ev: &mut ser::EnumVariantScope,
            st: &mut ser::StructScope,
        ) {
            st.rename_all_fields_case(Case::ScreamingSnake);
        }
    }

    let json = serde_json::to_string(&ser::hook(
        &Cases {
            foo_bar: (),
            bar_baz: (),
            e: Enum::StructVariant { baz_foo: () },
        },
        &Hooks,
    ))
    .unwrap();
    assert_eq!(
        json,
        "{\"FooBar\":null,\"bbz\":null,\"E\":{\"StructVariant\":{\"BAZ_FOO\":null}}}"
    );
}

#[test]
fn test_replace_value() {
    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct(&self, path: &Path, st: &mut ser::StructScope) {
            if path.segments().is_empty() {
                st.replace_value("baz", -15i16);
            }
        }

        fn on_struct_variant(
            &self,
            _path: &Path,
            _ev: &mut ser::EnumVariantScope,
            st: &mut ser::StructScope,
        ) {
            st.replace_value("baz", 'x');
        }
    }

    let json = serde_json::to_string(&ser::hook(&Payload::new(), &Hooks)).unwrap();
    assert_eq!(json, "{\"foo\":42,\"bar\":\"a\",\"baz\":-15,\"e\":{\"StructVariant\":{\"foo\":21,\"bar\":\"b\",\"baz\":\"x\"}}}");
}

#[test]
fn test_struct_replace_value_unserializable() {
    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct(&self, _path: &Path, st: &mut ser::StructScope) {
            st.replace_value("baz", StaticValue::NewtypeStruct("STRUCT"));
        }
    }

    let err = serde_json::to_string(&ser::hook(&Payload::new(), &Hooks)).unwrap_err();
    assert_eq!(err.to_string(), "Error at path 'baz': value is not serializable: newtype STRUCT cannot be represented fully in Value");
}

#[test]
fn test_struct_variant_replace_value_unserializable() {
    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct_variant(
            &self,
            _path: &Path,
            _ev: &mut ser::EnumVariantScope,
            st: &mut ser::StructScope,
        ) {
            st.replace_value("baz", StaticValue::NewtypeStruct("STRUCT"));
        }
    }

    let err = serde_json::to_string(&ser::hook(&Payload::new(), &Hooks)).unwrap_err();
    assert_eq!(err.to_string(), "Error at path 'e.baz': value is not serializable: newtype STRUCT cannot be represented fully in Value");
}

#[test]
fn test_error() {
    struct Hooks {
        on_error_called: Cell<bool>,
    }
    impl ser::Hooks for Hooks {
        fn on_struct(&self, _path: &Path, st: &mut ser::StructScope) {
            //TODO test other functions
            st.retain_field("invalid");
        }

        fn on_scope_error(&self, path: &Path, err: &mut ser::ErrorScope) {
            //TODO use mock to ensure this is called
            assert_eq!(&*path.borrow_str(), "");
            assert_eq!(
                *err.error(),
                ser::HooksError::FieldNotFound("invalid".into())
            );
            err.propagate();
            self.on_error_called.set(true);
        }
    }
    let hooks = Hooks {
        on_error_called: Cell::new(false),
    };
    assert!(serde_json::to_string(&ser::hook(&Payload::new(), &hooks)).is_err());
    assert!(hooks.on_error_called.get());
}

#[test]
fn test_serialize_as_map() {
    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct(&self, _path: &Path, st: &mut ser::StructScope) {
            st.serialize_as_map();
        }

        fn on_struct_variant(
            &self,
            _path: &Path,
            _ev: &mut ser::EnumVariantScope,
            st: &mut ser::StructScope,
        ) {
            st.serialize_as_map();
        }
    }

    let payload = Payload::new();

    // using RON in this test because it distinguishes between structs and maps
    let ron_original = ron::to_string(&payload).unwrap();
    assert_eq!(ron_original, "(foo:42,bar:Some('a'),baz:\"sample\",e:StructVariant(foo:21,bar:Some('b'),baz:\"example\"))");

    let ron = ron::to_string(&ser::hook(&payload, &Hooks)).unwrap();
    assert_eq!(ron, "{\"foo\":42,\"bar\":Some('a'),\"baz\":\"sample\",\"e\":{\"foo\":21,\"bar\":Some('b'),\"baz\":\"example\"}}");
}
