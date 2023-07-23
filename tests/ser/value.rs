use std::{borrow::Cow, cell::RefCell, cmp::Ordering, collections::HashSet};

use serde::Serialize;

use serde_hooks::ser;

#[derive(Serialize)]
struct UnitStruct;

#[derive(Serialize)]
struct Struct {
    struct_val: (),
}

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
    val_struct: Struct,

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
            val_struct: Struct { struct_val: () },
            val_unit_variant: Enum::UnitVariant,
            val_newtype_variant: Enum::NewtypeVariant(()),
            val_newtype: Newtype(()),
        }
    }

    fn fields() -> HashSet<String> {
        match serde_json::to_value(Self::new("", &[])).unwrap() {
            serde_json::Value::Object(o) => o.into_iter().map(|(k, _)| k).collect(),
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
            use serde_hooks::Value;

            // Note, all owned values will be received here as borrowed, just
            // with their own lifetimes
            match (path.as_str(), value.value()) {
                (
                    "",
                    Value::Struct {
                        name: "Payload", ..
                    },
                )
                | ("val_bool", Value::Bool(true))
                | ("val_i8", Value::I8(-8))
                | ("val_i16", Value::I16(-16))
                | ("val_i32", Value::I32(-32))
                | ("val_i64", Value::I64(-64))
                | ("val_u8", Value::U8(8))
                | ("val_u16", Value::U16(16))
                | ("val_u32", Value::U32(32))
                | ("val_u64", Value::U64(64))
                | ("val_char", Value::Char('x'))
                | ("val_str", Value::Str(Cow::Borrowed("str")))
                | ("val_str_static", Value::Str(Cow::Borrowed("str_static")))
                | ("val_str_owned", Value::Str(Cow::Borrowed("str_owned")))
                | ("val_bytes", Value::Bytes(Cow::Borrowed(&[1, 2])))
                | ("val_bytes_static", Value::Bytes(Cow::Borrowed(&[2, 3, 4])))
                | ("val_bytes_owned", Value::Bytes(Cow::Borrowed(&[3, 4, 5, 6])))
                | ("val_unit", Value::Unit)
                | ("val_none", Value::None)
                | ("val_some", Value::Some)
                | ("val_unit_struct", Value::UnitStruct("UnitStruct"))
                | (
                    "val_struct",
                    Value::Struct {
                        name: "Struct",
                        len: 1,
                    },
                )
                | ("val_struct.struct_val", _)
                | (
                    "val_unit_variant",
                    Value::UnitVariant {
                        name: "Enum",
                        variant_index: 0,
                        variant: "UnitVariant",
                    },
                )
                | (
                    "val_newtype_variant",
                    Value::NewtypeVariant {
                        name: "Enum",
                        variant_index: 1,
                        variant: "NewtypeVariant",
                    },
                )
                | ("val_newtype", Value::NewtypeStruct("Newtype")) => {}

                ("val_f32", Value::F32(v)) => {
                    assert_eq!(v.partial_cmp(&32.0f32), Some(Ordering::Equal));
                }
                ("val_f64", Value::F64(v)) => {
                    assert_eq!(v.partial_cmp(&64.0f64), Some(Ordering::Equal));
                }
                (path, value) => panic!("unexpected value {:?} at path '{}'", value, path),
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

#[test]
fn test_replace() {
    let val_str = "str".to_string();
    let val_bytes: Vec<u8> = vec![1, 2];

    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_value<S: serde::Serializer>(&self, value: &mut ser::ValueScope<S>) {
            if !value.path().segments().is_empty() {
                value.replace(&format!("R {}", value.path().to_string()));
            }
        }
    }

    let json =
        serde_json::to_string(&ser::hook(&Payload::new(&val_str, &val_bytes), &Hooks)).unwrap();
    assert_eq!(json, "{\"val_bool\":\"R val_bool\",\"val_i8\":\"R val_i8\",\"val_i16\":\"R val_i16\",\"val_i32\":\"R val_i32\",\"val_i64\":\"R val_i64\",\"val_u8\":\"R val_u8\",\"val_u16\":\"R val_u16\",\"val_u32\":\"R val_u32\",\"val_u64\":\"R val_u64\",\"val_f32\":\"R val_f32\",\"val_f64\":\"R val_f64\",\"val_char\":\"R val_char\",\"val_str\":\"R val_str\",\"val_str_static\":\"R val_str_static\",\"val_str_owned\":\"R val_str_owned\",\"val_bytes\":\"R val_bytes\",\"val_bytes_static\":\"R val_bytes_static\",\"val_bytes_owned\":\"R val_bytes_owned\",\"val_unit\":\"R val_unit\",\"val_none\":\"R val_none\",\"val_some\":\"R val_some\",\"val_unit_struct\":\"R val_unit_struct\",\"val_struct\":\"R val_struct\",\"val_unit_variant\":\"R val_unit_variant\",\"val_newtype_variant\":\"R val_newtype_variant\",\"val_newtype\":\"R val_newtype\"}");
}
