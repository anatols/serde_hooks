use std::collections::BTreeMap;

use serde_hooks::{ser, Path};

#[test]
fn test_skip_entry() {
    let payload: BTreeMap<u32, u32> = [(1, 1), (2, 2), (3, 3), (4, 4)].into();

    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_map(&self, _path: &Path, map: &mut ser::MapScope) {
            map.skip_entry(1u32) // by key
                .skip_entry(2usize); // by index
        }
    }

    let with_hooks = ron::to_string(&ser::hook(&payload, &Hooks)).unwrap();
    assert_eq!(with_hooks, "{2:2,4:4}");
}

#[test]
fn test_retain_entry() {
    let payload: BTreeMap<u32, u32> = [(1, 1), (2, 2), (3, 3), (4, 4)].into();

    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_map(&self, _path: &Path, map: &mut ser::MapScope) {
            map.retain_entry(1u32) // by key
                .retain_entry(2usize); // by index
        }
    }

    let with_hooks = ron::to_string(&ser::hook(&payload, &Hooks)).unwrap();
    assert_eq!(with_hooks, "{1:1,3:3}");
}

#[test]
fn test_replace_value() {
    let payload: BTreeMap<u32, u32> = [(1, 1), (2, 2), (3, 3), (4, 4)].into();

    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_map(&self, _path: &Path, map: &mut ser::MapScope) {
            map.replace_value(1u32, "hello") // by key
                .replace_value(2usize, -1i32); // by index
        }
    }

    let with_hooks = ron::to_string(&ser::hook(&payload, &Hooks)).unwrap();
    assert_eq!(with_hooks, "{1:\"hello\",2:2,3:-1,4:4}");
}

#[test]
fn test_replace_key() {
    let payload: BTreeMap<u32, u32> = [(1, 1), (2, 2), (3, 3), (4, 4)].into();

    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_map(&self, _path: &Path, map: &mut ser::MapScope) {
            map.replace_key(1u32, "hello") // by key
                .replace_key(2usize, -1i32); // by index
        }
    }

    let with_hooks = ron::to_string(&ser::hook(&payload, &Hooks)).unwrap();
    assert_eq!(with_hooks, "{\"hello\":1,2:2,-1:3,4:4}");
}

#[test]
fn test_rename_key() {
    let payload: BTreeMap<&'static str, u32> =
        [("first_entry", 1), ("second_entry", 2), ("third_entry", 3)].into();

    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_map(&self, _path: &Path, map: &mut ser::MapScope) {
            map.rename_key("first_entry", "static") // by key
                .rename_key(2usize, "owned".to_string()); // by index
        }
    }

    let with_hooks = ron::to_string(&ser::hook(&payload, &Hooks)).unwrap();
    assert_eq!(with_hooks, "{\"static\":1,\"second_entry\":2,\"owned\":3}");
}

#[test]
fn test_rename_key_case() {
    let payload: BTreeMap<&'static str, u32> =
        [("first_entry", 1), ("second_entry", 2), ("third_entry", 3)].into();

    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_map(&self, _path: &Path, map: &mut ser::MapScope) {
            map.rename_key_case("first_entry", "SCREAMING_SNAKE_CASE") // by key
                .rename_key_case(2usize, "PascalCase"); // by index
        }
    }

    let with_hooks = ron::to_string(&ser::hook(&payload, &Hooks)).unwrap();
    assert_eq!(
        with_hooks,
        "{\"FIRST_ENTRY\":1,\"second_entry\":2,\"ThirdEntry\":3}"
    );
}

#[test]
fn test_rename_all_fields_case() {
    let payload: BTreeMap<&'static str, u32> =
        [("first_entry", 1), ("second_entry", 2), ("third_entry", 3)].into();

    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_map(&self, _path: &Path, map: &mut ser::MapScope) {
            map.rename_all_fields_case("SCREAMING-KEBAB-CASE");
        }
    }

    let with_hooks = ron::to_string(&ser::hook(&payload, &Hooks)).unwrap();
    assert_eq!(
        with_hooks,
        "{\"FIRST-ENTRY\":1,\"SECOND-ENTRY\":2,\"THIRD-ENTRY\":3}"
    );
}

#[test]
fn test_insert_entry() {
    let payload: BTreeMap<u32, u32> = [(1, 1), (2, 2), (3, 3), (4, 4)].into();

    struct Hooks;
    impl ser::Hooks for Hooks {
        fn on_map(&self, _path: &Path, map: &mut ser::MapScope) {
            map.insert_entry("a", 'a', ser::MapInsertLocation::Before(0usize.into())) // by index
                .insert_entry("b", 'b', ser::MapInsertLocation::After(0usize.into())) // by index
                .insert_entry("c", 'c', ser::MapInsertLocation::Before(3u32.into())) // by value
                .insert_entry("d", 'd', ser::MapInsertLocation::Before(3u32.into())) // by value
                .insert_entry("e", 'e', ser::MapInsertLocation::After(3u32.into())) // by value
                .insert_entry("f", 'f', ser::MapInsertLocation::After(3u32.into())) // by value
                .insert_entry("g", 'g', ser::MapInsertLocation::End);
        }
    }

    let with_hooks = ron::to_string(&ser::hook(&payload, &Hooks)).unwrap();
    assert_eq!(
        with_hooks,
        "{\"a\":'a',1:1,\"b\":'b',2:2,\"c\":'c',\"d\":'d',3:3,\"e\":'e',\"f\":'f',4:4,\"g\":'g'}"
    );
}
