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
        p1: i32,
        p2: Option<char>,
        p3: String,
    },
}

#[derive(Serialize)]
struct Payload {
    p1: i32,
    p2: Option<char>,
    p3: String,
    e: Enum,
}

impl Payload {
    fn new() -> Self {
        Payload {
            p1: 42,
            p2: Some('a'),
            p3: "sample".into(),
            e: Enum::StructVariant {
                p1: 21,
                p2: Some('b'),
                p3: "example".into(),
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
            self.fields_to_expect.borrow_mut().remove(path.as_str());

            match path.as_str() {
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
            self.fields_to_expect.borrow_mut().remove(path.as_str());

            match path.as_str() {
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
            st.skip_field("p1").skip_field("p3");
        }

        fn on_struct_variant(
            &self,
            _path: &Path,
            _ev: &mut ser::EnumVariantScope,
            st: &mut ser::StructScope,
        ) {
            st.skip_field("p1").skip_field("p3");
        }
    }

    let json = serde_json::to_string(&ser::hook(&Payload::new(), &Hooks)).unwrap();
    assert_eq!(
        json,
        "{\"p2\":\"a\",\"e\":{\"StructVariant\":{\"p2\":\"b\"}}}"
    );
}

#[test]
fn test_retain_field() {
    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct(&self, _path: &Path, st: &mut ser::StructScope) {
            st.retain_field("p1").retain_field("p2");
        }
    }

    let json = serde_json::to_string(&ser::hook(&Payload::new(), &Hooks)).unwrap();
    assert_eq!(json, r#"{"p1":42,"p2":"a"}"#);
}

#[test]
fn test_retain_field_in_struct_variant() {
    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct(&self, path: &Path, st: &mut ser::StructScope) {
            if path.is_root() {
                st.retain_field("e");
            }
        }

        fn on_struct_variant(
            &self,
            _path: &Path,
            _ev: &mut ser::EnumVariantScope,
            st: &mut ser::StructScope,
        ) {
            st.retain_field("p1").retain_field("p2");
        }
    }

    let json = serde_json::to_string(&ser::hook(&Payload::new(), &Hooks)).unwrap();
    assert_eq!(json, "{\"e\":{\"StructVariant\":{\"p1\":21,\"p2\":\"b\"}}}");
}

#[test]
fn test_rename_field() {
    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct(&self, path: &Path, st: &mut ser::StructScope) {
            if path.is_root() {
                st.rename_field("p1", "not_foo")
                    .rename_field("p2", format!("bar_{}", 42))
                    .rename_field("p3", "baz2")
                    .rename_field("baz2", "baz3");
            }
        }

        fn on_struct_variant(
            &self,
            _path: &Path,
            _ev: &mut ser::EnumVariantScope,
            st: &mut ser::StructScope,
        ) {
            st.rename_field("p1", "not_foo_either")
                .rename_field("p2", format!("bar_{}", 21))
                .rename_field("p3", "baz4")
                .rename_field("baz4", "baz5");
        }
    }

    let json = serde_json::to_string(&ser::hook(&Payload::new(), &Hooks)).unwrap();
    assert_eq!(json, "{\"not_foo\":42,\"bar_42\":\"a\",\"baz3\":\"sample\",\"e\":{\"StructVariant\":{\"not_foo_either\":21,\"bar_21\":\"b\",\"baz5\":\"example\"}}}");
}

#[test]
fn test_rename_field_case() {
    #[derive(Serialize)]
    struct Payload {
        some_field: (),
    }

    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct(&self, _path: &Path, st: &mut ser::StructScope) {
            st.rename_field_case("some_field", "SCREAMING-KEBAB-CASE");
        }
    }

    let json = serde_json::to_string(&ser::hook(&Payload { some_field: () }, &Hooks)).unwrap();
    assert_eq!(json, "{\"SOME-FIELD\":null}");
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
            if path.is_root() {
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
            if path.is_root() {
                st.replace_value("p3", -15i16);
            }
        }

        fn on_struct_variant(
            &self,
            _path: &Path,
            _ev: &mut ser::EnumVariantScope,
            st: &mut ser::StructScope,
        ) {
            st.replace_value("p3", 'x');
        }
    }

    let json = serde_json::to_string(&ser::hook(&Payload::new(), &Hooks)).unwrap();
    assert_eq!(json, "{\"p1\":42,\"p2\":\"a\",\"p3\":-15,\"e\":{\"StructVariant\":{\"p1\":21,\"p2\":\"b\",\"p3\":\"x\"}}}");
}

#[test]
fn test_struct_replace_value_unserializable() {
    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct(&self, _path: &Path, st: &mut ser::StructScope) {
            st.replace_value("p3", StaticValue::NewtypeStruct("STRUCT"));
        }
    }

    let err = serde_json::to_string(&ser::hook(&Payload::new(), &Hooks)).unwrap_err();
    assert_eq!(err.to_string(), "Error at path 'p3': value is not serializable: newtype STRUCT cannot be represented fully in Value");
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
            st.replace_value("p3", StaticValue::NewtypeStruct("STRUCT"));
        }
    }

    let err = serde_json::to_string(&ser::hook(&Payload::new(), &Hooks)).unwrap_err();
    assert_eq!(err.to_string(), "Error at path 'e.p3': value is not serializable: newtype STRUCT cannot be represented fully in Value");
}

#[test]
fn test_error() {
    struct Hooks {
        on_error_called: Cell<bool>,
    }
    impl ser::Hooks for Hooks {
        fn on_struct(&self, _path: &Path, st: &mut ser::StructScope) {
            st.retain_field("invalid");
        }

        fn on_scope_error(&self, path: &Path, err: &mut ser::ErrorScope) {
            assert_eq!(path, "");
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
    struct Hooks {
        on_map_called: Cell<bool>,
    }

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

        fn on_map(&self, _path: &Path, _map: &mut ser::MapScope) {
            self.on_map_called.set(true);
        }
    }

    let payload = Payload::new();
    let hooks = Hooks {
        on_map_called: Cell::new(false),
    };

    // using RON in this test because it distinguishes between structs and maps
    let ron_original = ron::to_string(&payload).unwrap();
    assert_eq!(
        ron_original,
        "(p1:42,p2:Some('a'),p3:\"sample\",e:StructVariant(p1:21,p2:Some('b'),p3:\"example\"))"
    );

    let ron = ron::to_string(&ser::hook(&payload, &hooks)).unwrap();
    assert_eq!(ron, "{\"p1\":42,\"p2\":Some('a'),\"p3\":\"sample\",\"e\":{\"p1\":21,\"p2\":Some('b'),\"p3\":\"example\"}}");

    assert!(hooks.on_map_called.get());
}

#[test]
fn test_flatten() {
    #[derive(Serialize, Default)]
    struct Outer {
        outer_field: (),
        inner: Inner,
    }

    #[derive(Serialize, Default)]
    struct Inner {
        inner_field: (),
    }

    struct Hooks {
        struct_hook_called_for_inner_struct: Cell<bool>,
    }

    impl ser::Hooks for Hooks {
        fn on_struct(&self, path: &Path, st: &mut ser::StructScope) {
            if path.borrow_str().is_empty() {
                st.flatten_field("inner");
            }

            if st.struct_name() == "Inner" {
                self.struct_hook_called_for_inner_struct.set(true);
            }
        }
    }

    let payload = Outer::default();
    let hooks = Hooks {
        struct_hook_called_for_inner_struct: Cell::new(false),
    };

    // using RON in this test because it distinguishes between structs and maps
    let ron_original = ron::to_string(&payload).unwrap();
    assert_eq!(ron_original, "(outer_field:(),inner:(inner_field:()))");

    let ron = ron::to_string(&ser::hook(&payload, &hooks)).unwrap();
    assert_eq!(ron, "{\"outer_field\":(),\"inner_field\":()}");
    assert!(hooks.struct_hook_called_for_inner_struct.get());
}

#[test]
fn test_flatten_unsupported_data_type() {
    #[derive(Serialize, Default)]
    struct Outer {
        outer_field: (),
        inner: i8,
    }

    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_struct(&self, _path: &Path, st: &mut ser::StructScope) {
            st.flatten_field("inner");
        }
    }

    let err = ron::to_string(&ser::hook(&Outer::default(), &Hooks)).unwrap_err();
    assert_eq!(
        err.to_string(),
        "Error at path 'inner': cannot flatten unsupported data type \"i8\""
    );
}
