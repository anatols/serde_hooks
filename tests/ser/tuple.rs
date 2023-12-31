use std::{cell::RefCell, collections::HashSet};

use serde::Serialize;
use serde_hooks::{ser, Path};

#[test]
fn test_tuple_traversing() {
    #[allow(clippy::enum_variant_names)]
    #[derive(Serialize)]
    enum Enum {
        TupleVariant((), (), (), ()),
    }

    #[derive(Serialize)]
    struct TupleStruct((), (), (), (), ());

    #[derive(Serialize)]
    struct Outer {
        tuple: (i32, i32),
        nested: ((i8, i16), (u32, u64, f64)),
        tuple_variant: Enum,
        tuple_struct: TupleStruct,
    }

    let outer = Outer {
        tuple: (1, 2),
        nested: ((3, 4), (5, 6, 7.0)),
        tuple_variant: Enum::TupleVariant((), (), (), ()),
        tuple_struct: TupleStruct((), (), (), (), ()),
    };

    struct Hooks {
        tuples_to_expect: RefCell<HashSet<&'static str>>,
        tuple_variants_to_expect: RefCell<HashSet<&'static str>>,
        tuple_structs_to_expect: RefCell<HashSet<&'static str>>,
    }
    impl ser::Hooks for Hooks {
        fn on_tuple(&self, path: &Path, tpl: &mut ser::TupleScope, seq: &mut ser::SeqScope) {
            let path = path.borrow_str();
            self.tuples_to_expect.borrow_mut().remove(path.as_str());

            assert_eq!(Some(tpl.tuple_len()), seq.seq_len());

            match path.as_str() {
                "tuple" => {
                    assert_eq!(tpl.tuple_len(), 2);
                }
                "nested" => {
                    assert_eq!(tpl.tuple_len(), 2);
                }
                "nested[0]" => {
                    assert_eq!(tpl.tuple_len(), 2);
                }
                "nested[1]" => {
                    assert_eq!(tpl.tuple_len(), 3);
                }
                "tuple_variant" => {
                    assert_eq!(tpl.tuple_len(), 4);
                }
                "tuple_struct" => {
                    assert_eq!(tpl.tuple_len(), 5);
                }
                _ => unreachable!("{path}"),
            }
        }

        fn on_tuple_variant(
            &self,
            path: &Path,
            ev: &mut ser::EnumVariantScope,
            tpl: &mut ser::TupleScope,
            seq: &mut ser::SeqScope,
        ) {
            let path = path.borrow_str();
            self.tuple_variants_to_expect
                .borrow_mut()
                .remove(path.as_str());

            assert_eq!(Some(tpl.tuple_len()), seq.seq_len());

            match path.as_str() {
                "tuple_variant" => {
                    assert_eq!(ev.enum_name(), "Enum");
                    assert_eq!(ev.variant_name(), "TupleVariant");
                    assert_eq!(ev.variant_index(), 0);
                    assert_eq!(tpl.tuple_len(), 4);
                }
                _ => unreachable!("{path}"),
            }
        }

        fn on_tuple_struct(
            &self,
            path: &Path,
            tpl: &mut ser::TupleStructScope,
            seq: &mut ser::SeqScope,
        ) {
            let path = path.borrow_str();
            self.tuple_structs_to_expect
                .borrow_mut()
                .remove(path.as_str());

            assert_eq!(Some(tpl.tuple_len()), seq.seq_len());

            match path.as_str() {
                "tuple_struct" => {
                    assert_eq!(tpl.tuple_len(), 5);
                    assert_eq!(tpl.struct_name(), "TupleStruct");
                }
                _ => unreachable!("{path}"),
            }
        }
    }
    let hooks = Hooks {
        tuples_to_expect: RefCell::new(
            [
                "tuple",
                "nested",
                "nested[0]",
                "nested[1]",
                "tuple_variant",
                "tuple_struct",
            ]
            .into(),
        ),
        tuple_variants_to_expect: RefCell::new(["tuple_variant"].into()),
        tuple_structs_to_expect: RefCell::new(["tuple_struct"].into()),
    };

    serde_json::to_string(&ser::hook(&outer, &hooks)).unwrap();
    assert!(
        hooks.tuples_to_expect.borrow().is_empty(),
        "following tuples were expected, but not called back about {:?}",
        hooks.tuples_to_expect.borrow()
    );

    assert!(
        hooks.tuple_variants_to_expect.borrow().is_empty(),
        "following tuple variants were expected, but not called back about {:?}",
        hooks.tuple_variants_to_expect.borrow()
    );

    assert!(
        hooks.tuple_structs_to_expect.borrow().is_empty(),
        "following tuple structs were expected, but not called back about {:?}",
        hooks.tuple_structs_to_expect.borrow()
    );
}

#[test]
fn test_modified_tuple_turns_into_seq() {
    #[derive(Serialize)]
    enum Enum {
        TupleVariant(u32, u32),
    }

    #[derive(Serialize)]
    struct TupleStruct(u32, u32);

    #[derive(Serialize)]
    struct Payload {
        tuple: (u32, u32),
        tuple_variant: Enum,
        tuple_struct: TupleStruct,
    }

    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_tuple(&self, path: &Path, _tpl: &mut ser::TupleScope, seq: &mut ser::SeqScope) {
            if path == "tuple" {
                seq.skip_element(0);
            }
        }

        fn on_tuple_variant(
            &self,
            path: &Path,
            _ev: &mut ser::EnumVariantScope,
            _tpl: &mut ser::TupleScope,
            seq: &mut ser::SeqScope,
        ) {
            if path == "tuple_variant" {
                seq.retain_element(0);
            }
        }

        fn on_tuple_struct(
            &self,
            path: &Path,
            _tpl: &mut ser::TupleStructScope,
            seq: &mut ser::SeqScope,
        ) {
            if path == "tuple_struct" {
                seq.skip_element(0);
            }
        }
    }

    let payload = Payload {
        tuple: (1, 2),
        tuple_variant: Enum::TupleVariant(3, 4),
        tuple_struct: TupleStruct(5, 6),
    };

    let without_hooks = ron::to_string(&payload).unwrap();
    assert_eq!(
        without_hooks,
        "(tuple:(1,2),tuple_variant:TupleVariant(3,4),tuple_struct:(5,6))"
    );

    let with_hooks = ron::to_string(&ser::hook(&payload, &Hooks)).unwrap();
    assert_eq!(with_hooks, "(tuple:[2],tuple_variant:[3],tuple_struct:[6])");
}
