use std::{cell::RefCell, collections::HashSet};

use indoc::indoc;
use serde::Serialize;

use serde_hooks::ser;

#[derive(Serialize)]
struct UnitStruct;

//TODO add tuple variant
#[allow(clippy::enum_variant_names)]
#[derive(Serialize)]
enum Enum {
    UnitVariant,
    NewtypeVariant(()),
    StructVariant { struct_variant_val: () },
}

#[derive(Serialize)]
struct Payload {
    unit_variant: Enum,
    newtype_variant: Enum,
    struct_variant: Enum,
}

impl Payload {
    fn new() -> Self {
        Self {
            unit_variant: Enum::UnitVariant,
            newtype_variant: Enum::NewtypeVariant(()),
            struct_variant: Enum::StructVariant {
                struct_variant_val: (),
            },
        }
    }
}

#[test]
fn test_variant_traversing() {
    struct Hooks {
        variants_to_expect: RefCell<HashSet<String>>,
        structs_to_expect: RefCell<HashSet<String>>,
        structs_variants_to_expect: RefCell<HashSet<String>>,
    }
    impl ser::Hooks for Hooks {
        fn on_enum_variant(&self, ev: &mut ser::EnumVariantScope) {
            let path = ev.path().to_string();
            assert!(self.variants_to_expect.borrow_mut().remove(&path));

            assert_eq!(ev.enum_name(), "Enum");

            match path.as_str() {
                "unit_variant" => {
                    assert_eq!(ev.variant_name(), "UnitVariant");
                    assert_eq!(ev.variant_index(), 0);
                }
                "newtype_variant" => {
                    assert_eq!(ev.variant_name(), "NewtypeVariant");
                    assert_eq!(ev.variant_index(), 1);
                }
                "struct_variant" => {
                    assert_eq!(ev.variant_name(), "StructVariant");
                    assert_eq!(ev.variant_index(), 2);
                }
                _ => unreachable!("{path}"),
            }
        }

        fn on_struct(&self, st: &mut ser::StructScope) {
            let path = st.path().to_string();
            self.structs_to_expect.borrow_mut().remove(&path);

            match path.as_str() {
                "" => {}
                "struct_variant" => {
                    assert_eq!(st.struct_name(), "StructVariant");
                    assert_eq!(st.struct_len(), 1);
                }
                _ => unreachable!("{path}"),
            }
        }

        fn on_struct_variant(&self, ev: &mut ser::EnumVariantScope, st: &mut ser::StructScope) {
            let path = st.path().to_string();
            self.structs_variants_to_expect.borrow_mut().remove(&path);

            match path.as_str() {
                "struct_variant" => {
                    assert_eq!(ev.enum_name(), "Enum");
                    assert_eq!(ev.variant_name(), "StructVariant");
                    assert_eq!(ev.variant_index(), 2);
                    assert_eq!(st.struct_name(), "StructVariant");
                    assert_eq!(st.struct_len(), 1);
                }
                _ => unreachable!("{path}"),
            }
        }
    }
    let hooks = Hooks {
        variants_to_expect: RefCell::new(
            ["unit_variant", "newtype_variant", "struct_variant"]
                .into_iter()
                .map(Into::into)
                .collect(),
        ),
        structs_to_expect: RefCell::new(
            ["", "struct_variant"].into_iter().map(Into::into).collect(),
        ),
        structs_variants_to_expect: RefCell::new(
            ["struct_variant"].into_iter().map(Into::into).collect(),
        ),
    };

    serde_json::to_string(&ser::hook(&Payload::new(), &hooks)).unwrap();
    assert!(
        hooks.variants_to_expect.borrow().is_empty(),
        "following variants were expected, but not called back about {:?}",
        hooks.variants_to_expect.borrow()
    );
    assert!(
        hooks.structs_to_expect.borrow().is_empty(),
        "following structs were expected, but not called back about {:?}",
        hooks.structs_to_expect.borrow()
    );
    assert!(
        hooks.structs_variants_to_expect.borrow().is_empty(),
        "following struct variants were expected, but not called back about {:?}",
        hooks.structs_variants_to_expect.borrow()
    );
}

#[test]
fn test_enum_rename() {
    struct Hooks;

    impl ser::Hooks for Hooks {
        fn on_enum_variant(&self, ev: &mut ser::EnumVariantScope) {
            let path = ev.path().to_string();
            match path.as_str() {
                "unit_variant" => {
                    ev.rename_enum("new_enum_name");
                }
                "newtype_variant" => {
                    ev.rename_enum(format!("NEW_{path}"));
                }
                "struct_variant" => {
                    ev.rename_enum_case(serde_hooks::Case::Upper);
                }
                _ => unreachable!(),
            }
        }
    }

    use serde_reflection::{Samples, Tracer, TracerConfig};

    let mut tracer = Tracer::new(TracerConfig::default());
    let mut samples = Samples::new();
    tracer
        .trace_value(&mut samples, &ser::hook(&Payload::new(), &Hooks))
        .unwrap();
    let registry = tracer.registry().unwrap();

    let actual = serde_yaml::to_string(&registry).unwrap();
    let expected = indoc! {"
        ENUM: !ENUM
          2:
            StructVariant: !STRUCT
            - struct_variant_val: UNIT
        NEW_newtype_variant: !ENUM
          1:
            NewtypeVariant: !NEWTYPE UNIT
        Payload: !STRUCT
        - unit_variant: !TYPENAME new_enum_name
        - newtype_variant: !TYPENAME NEW_newtype_variant
        - struct_variant: !TYPENAME ENUM
        new_enum_name: !ENUM
          0:
            UnitVariant: UNIT
    "};
    assert_eq!(
        actual, expected,
        "\n\nExpected YAML:\n\n{expected}\n\nActual YAML:\n\n{actual}\n\n"
    );
}

#[test]
fn test_variant_index_change() {
    struct Hooks;

    impl ser::Hooks for Hooks {
        fn on_enum_variant(&self, ev: &mut ser::EnumVariantScope) {
            let path = ev.path().to_string();
            if path == "unit_variant" {
                ev.change_variant_index(10);
            }
        }
    }

    use serde_reflection::{Samples, Tracer, TracerConfig};

    let mut tracer = Tracer::new(TracerConfig::default());
    let mut samples = Samples::new();
    tracer
        .trace_value(&mut samples, &ser::hook(&Payload::new(), &Hooks))
        .unwrap();
    let registry = tracer.registry().unwrap();

    let actual = serde_yaml::to_string(&registry.get("Enum").unwrap()).unwrap();
    let expected = indoc! {"
        !ENUM
        1:
          NewtypeVariant: !NEWTYPE UNIT
        2:
          StructVariant: !STRUCT
          - struct_variant_val: UNIT
        10:
          UnitVariant: UNIT
    "};
    assert_eq!(
        actual, expected,
        "\n\nExpected YAML:\n\n{expected}\n\nActual YAML:\n\n{actual}\n\n"
    );
}

#[test]
fn test_variant_rename() {
    struct Hooks;

    impl ser::Hooks for Hooks {
        fn on_enum_variant(&self, ev: &mut ser::EnumVariantScope) {
            let path = ev.path().to_string();
            match path.as_str() {
                "unit_variant" => {
                    ev.rename_variant("new_variant_name");
                }
                "newtype_variant" => {
                    ev.rename_variant(format!("NEW_{path}"));
                }
                "struct_variant" => {
                    ev.rename_variant_case(serde_hooks::Case::ScreamingKebab);
                }
                _ => unreachable!(),
            }
        }
    }

    let json = serde_json::to_string(&ser::hook(&Payload::new(), &Hooks)).unwrap();
    assert_eq!(
        json,
        "{\"unit_variant\":\"new_variant_name\",\"newtype_variant\":{\"NEW_newtype_variant\":null},\"struct_variant\":{\"STRUCT-VARIANT\":{\"struct_variant_val\":null}}}"
    );
}
