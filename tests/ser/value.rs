use std::{borrow::Cow, cell::RefCell, cmp::Ordering, collections::HashSet};

use serde::Serialize;

use serde_hooks::ser;

#[derive(Serialize)]
struct UnitStruct;

#[derive(Serialize)]
enum Enum {
    UnitVariant,
    NewtypeVariant(()),
}

#[derive(Serialize)]
struct Newtype(());

#[derive(Serialize)]
struct Payload<'s, 'b> {
    val_bool: bool,
    val_i8: i8,
    val_i16: i16,
    val_i32: i32,
    val_i64: i64,
    val_u8: u8,
    val_u16: u16,
    val_u32: u32,
    val_u64: u64,
    val_f32: f32,
    val_f64: f64,
    val_char: char,

    val_str: &'s str,
    val_str_static: &'static str,
    val_str_owned: String,

    #[serde(with = "serde_bytes")]
    val_bytes: &'b [u8],

    #[serde(with = "serde_bytes")]
    val_bytes_static: &'static [u8],

    #[serde(with = "serde_bytes")]
    val_bytes_owned: Vec<u8>,

    val_unit: (),
    val_none: Option<()>,
    val_some: Option<()>,
    val_unit_struct: UnitStruct,
    val_unit_variant: Enum,
    val_newtype_variant: Enum,
    val_newtype: Newtype,
}

impl<'s, 'b> Payload<'s, 'b> {
    fn new(val_str: &'s str, val_bytes: &'b [u8]) -> Self {
        Payload {
            val_bool: true,
            val_i8: -8,
            val_i16: -16,
            val_i32: -32,
            val_i64: -64,
            val_u8: 8,
            val_u16: 16,
            val_u32: 32,
            val_u64: 64,
            val_f32: 32.0,
            val_f64: 64.0,
            val_char: 'x',
            val_str,
            val_str_static: "str_static",
            val_str_owned: "str_owned".into(),
            val_bytes,
            val_bytes_static: &[2, 3, 4],
            val_bytes_owned: [3, 4, 5, 6].into(),
            val_unit: (),
            val_none: None,
            val_some: Some(()),
            val_unit_struct: UnitStruct,
            val_unit_variant: Enum::UnitVariant,
            val_newtype_variant: Enum::NewtypeVariant(()),
            val_newtype: Newtype(()),
        }
    }

    fn fields() -> HashSet<String> {
        match serde_json::to_value(Self::new("", &[])).unwrap() {
            serde_json::Value::Object(o) => o.into_iter().map(|(k, _)| format!("$.{k}")).collect(),
            _ => unreachable!(),
        }
    }
}

#[test]
fn test_values() {
    let val_str = "str".to_string();
    let val_bytes: Vec<u8> = vec![1, 2];

    struct Hooks {
        fields_to_expect: RefCell<HashSet<String>>,
    }
    impl ser::Hooks for Hooks {
        fn on_value<S: serde::Serializer>(&self, value: &mut ser::ValueScope<S>) {
            let path = value.path().to_string();
            self.fields_to_expect.borrow_mut().remove(&path);
            use serde_hooks::{PrimitiveValue, Value};

            // Note, all owned values will be received here as borrowed, just
            // with their own lifetimes
            match (path.as_str(), value.value()) {
                ("$.val_bool", Value::Primitive(PrimitiveValue::Bool(true)))
                | ("$.val_i8", Value::Primitive(PrimitiveValue::I8(-8)))
                | ("$.val_i16", Value::Primitive(PrimitiveValue::I16(-16)))
                | ("$.val_i32", Value::Primitive(PrimitiveValue::I32(-32)))
                | ("$.val_i64", Value::Primitive(PrimitiveValue::I64(-64)))
                | ("$.val_u8", Value::Primitive(PrimitiveValue::U8(8)))
                | ("$.val_u16", Value::Primitive(PrimitiveValue::U16(16)))
                | ("$.val_u32", Value::Primitive(PrimitiveValue::U32(32)))
                | ("$.val_u64", Value::Primitive(PrimitiveValue::U64(64)))
                | ("$.val_char", Value::Primitive(PrimitiveValue::Char('x')))
                | ("$.val_str", Value::Primitive(PrimitiveValue::Str(Cow::Borrowed("str"))))
                | (
                    "$.val_str_static",
                    Value::Primitive(PrimitiveValue::Str(Cow::Borrowed("str_static"))),
                )
                | (
                    "$.val_str_owned",
                    Value::Primitive(PrimitiveValue::Str(Cow::Borrowed("str_owned"))),
                )
                | (
                    "$.val_bytes",
                    Value::Primitive(PrimitiveValue::Bytes(Cow::Borrowed(&[1, 2]))),
                )
                | (
                    "$.val_bytes_static",
                    Value::Primitive(PrimitiveValue::Bytes(Cow::Borrowed(&[2, 3, 4]))),
                )
                | (
                    "$.val_bytes_owned",
                    Value::Primitive(PrimitiveValue::Bytes(Cow::Borrowed(&[3, 4, 5, 6]))),
                )
                | ("$.val_unit", Value::Primitive(PrimitiveValue::Unit))
                | ("$.val_none", Value::Primitive(PrimitiveValue::None))
                | ("$.val_some", Value::Primitive(PrimitiveValue::Some))
                | (
                    "$.val_unit_struct",
                    Value::Primitive(PrimitiveValue::UnitStruct("UnitStruct")),
                )
                | (
                    "$.val_unit_variant",
                    Value::Primitive(PrimitiveValue::UnitVariant {
                        name: "Enum",
                        variant_index: 0,
                        variant: "UnitVariant",
                    }),
                )
                | (
                    "$.val_newtype_variant",
                    Value::Primitive(PrimitiveValue::NewtypeVariant {
                        name: "Enum",
                        variant_index: 1,
                        variant: "NewtypeVariant",
                    }),
                )
                | ("$.val_newtype", Value::Primitive(PrimitiveValue::NewtypeStruct("Newtype"))) => {
                }

                ("$.val_f32", Value::Primitive(PrimitiveValue::F32(v))) => {
                    assert_eq!(v.partial_cmp(&32.0f32), Some(Ordering::Equal));
                }
                ("$.val_f64", Value::Primitive(PrimitiveValue::F64(v))) => {
                    assert_eq!(v.partial_cmp(&64.0f64), Some(Ordering::Equal));
                }
                (path, value) => panic!("unexpected value {:?} at {}", value, path),
            }
        }
    }

    let hooks = Hooks {
        fields_to_expect: RefCell::new(Payload::fields()),
    };
    assert!(serde_json::to_string(&ser::hook(&Payload::new(&val_str, &val_bytes), &hooks)).is_ok());
    assert!(
        hooks.fields_to_expect.borrow().is_empty(),
        "following fields were expected, but not called back about {:?}",
        hooks.fields_to_expect.borrow()
    );
}
