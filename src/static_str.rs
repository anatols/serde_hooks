use std::{borrow::Cow, collections::HashSet, pin::Pin, sync::Mutex};

/// Serde expects many things to be `&'static str`.
///
/// For example, struct fields, because for structs the field names are known
/// at compile time. A serializer can theoretically
/// hold on to those field name references forever and expect them to be valid.
/// To be able to rename a field, we thus need to somehow generate a string at
/// runtime that will have a 'static lifetime.
///
/// The 'static lifetime is defined as 'will live till the program ends'. Here
/// we keep a static set of pinned Box<str>, and store unique names in it.
/// The static set will not get destroyed until the program ends. Although
/// boxes can move in the set, where they point remains pinned in place until the end
/// of the program because we never delete. For all practical purposes those boxed
/// strs are static, so safe to transmute to 'static lifetime here.
///
/// This is obviously "leaking" memory on each new field, but hey, how many of those
/// unique renamed fields are you planning to have?
pub(crate) fn into_static_str(key: Cow<'static, str>) -> &'static str {
    match key {
        Cow::Borrowed(static_key) => static_key,
        Cow::Owned(string_key) => {
            lazy_static::lazy_static! {
                static ref KEYS: Mutex<HashSet<Pin<Box<str>>>> = Mutex::new(HashSet::new());
            }

            let mut keys = KEYS.lock().unwrap();
            let boxed_key = Pin::new(string_key.into_boxed_str());

            let static_key: &'static str = match keys.get(&boxed_key) {
                Some(existing_boxed_key) => unsafe {
                    std::mem::transmute::<&str, &'static str>(existing_boxed_key)
                },
                None => {
                    let static_key =
                        unsafe { std::mem::transmute::<&str, &'static str>(&boxed_key) };
                    keys.insert(boxed_key);
                    static_key
                }
            };

            static_key
        }
    }
}

#[test]
fn test_into_static_str() {
    // Comparing references here, not content
    fn assert_refs_eq(left: &str, right: &str) {
        assert_eq!(left as *const _, right as *const _);
    }

    fn assert_refs_ne(left: &str, right: &str) {
        assert_ne!(left as *const _, right as *const _);
    }

    // Static strings are just pass-through
    let foo_str: &'static str = "foo";
    assert_refs_eq(into_static_str(Cow::Borrowed(foo_str)), foo_str);
    let bar_str: &'static str = "bar";
    assert_refs_eq(into_static_str(Cow::Borrowed(bar_str)), bar_str);

    // Pass-through, even if the value is repeating
    let bar_str_again: &'static str = &"_bar"[1..]; // slice shenanigans, to stop compiler from reusing strs.
    assert_refs_ne(bar_str, bar_str_again);
    assert_refs_eq(into_static_str(Cow::Borrowed(bar_str_again)), bar_str_again);

    // Owned values are cached
    let baz_str = "baz";
    let first_instance: &'static str = into_static_str(Cow::Owned(baz_str.to_string()));
    assert_refs_ne(baz_str, first_instance);

    // For a repeated owned string a ref to the previous instance is returned
    assert_refs_eq(
        into_static_str(Cow::Owned(baz_str.to_string())),
        first_instance,
    );

    // Static strings are still pass-through, even if we have cached the exact same
    // owned one (i.e., we don't want hash lookups)
    assert_refs_eq(into_static_str(Cow::Borrowed(baz_str)), baz_str);
}
