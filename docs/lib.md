# Serde Hooks

This crate allows you to hook into [`serde`] serialization. You can get callbacks for each piece of data that is being serialized, and modify serialization behavior at runtime.

For example, you can, *at runtime*:

- Rename or skip struct fields.
- Rename enum variants.
- Replace struct field values, map keys and values, sequence and tuple items.
- Add new entries to maps.
- Introspect serialized data structures.

# Why do I need this crate?

You likely don't.

Seriously, in the vast majority of cases, what [`serde`] offers out of the box is enough, and more optimal to use. Before you consider using this crate, please check [serde.rs](https://serde.rs), including the [examples](https://serde.rs/examples.html) section. After that, consider changing your data model in a way so you can use bare `serde`. Only then, if none of the above helps, come back for the hooks.

Some examples where having runtime control over serialization can be handy:

- You need to exclude certain fields from an API response based on permissions level. E.g. exclude `salary` field from `Employee` struct for everyone but their manager.
- You need to support different case conventions for different formats. E.g. camelCase in JSON and kebab-case in YAML.
- Your data type comes from a library you cannot change, and already has a `#[derive(Serialize)]`. Just not exactly with the `#[serde]` attributes you want there.

Obviously, all of these cases can be handled by either [implementing custom serialization](https://serde.rs/impl-serialize.html), or using something like [`#[serde(serialize_with = ...)]`](https://serde.rs/field-attrs.html#serialize_with). But let's face it, it's not fun: you'd need to type a ton of boilerplate and the default `serde` derive is so much nicer to use.

In fact, this crate actually is that ton of boilerplate that just calls back to you at the right moments.

# Serialization API

The main API concepts are:

- **Hooks**: callback functions that get called when serialization process reaches certain parts in the serialized data.
- **Scopes**: objects that represent the reached parts in hooks and allow introspection and modification of the data.

Below is a more detailed description for each.

The API design is in many cases dictated by the quite rigid internal structure of [serde APIs](serde::ser) and [serde data model](https://serde.rs/data-model.html). It might not be the most ergonomic one ever for this reason. The intention though is to support (at runtime) most of what serde derive allows you to do with `#[serde]` attributes, and more.

## Hooks

It all starts with the [ser::Hooks] trait. This trait defines a bunch of callback functions (hooks) that have empty default implementations. You implement this trait for some type of yours (it can even be a unit struct) and overload the hooks you want to receive (but not the others):

```rust
use serde_hooks::ser;

struct MyHooks;

impl ser::Hooks for MyHooks {
    // This hook is called on every serialized struct.
    fn on_struct(&self, st: &mut ser::StructScope) {
        // Here you can modify the struct by calling methods on `st`
    }
}
```

You then "attach" an instance of your hooks type to the serializable data by calling [ser::hook()]. This creates a serialization wrapper that will call back to your hooks. The serialization wrapper itself is serializable, so you just pass it on to the serialization format library you use:

```rust
# use serde_hooks::ser;
#
# struct MyHooks;
# impl ser::Hooks for MyHooks {}
#
# let data = 42i32;
#
let json = serde_json::to_string(&ser::hook(&data, &MyHooks)).unwrap();
```

## Scopes

Each of the hooks you implement will receive one or more "scopes" as the arguments. Scopes represent different parts of serde data model. You can query metadata from scope objects, such as struct names, sequence lengths etc. You can also perform actions on the serialized data by calling action methods on the scope objects.

For example, [`ser::StructScope`] represents a serialized `struct`. You can query the [struct name](ser::StructScope::struct_name()) or [the number of fields in the struct](ser::StructScope::struct_len()) from it. You can also, for example, [skip fields](ser::StructScope::skip_field()):

```rust
use serde::Serialize;
use serde_hooks::ser;

#[derive(Serialize)]
struct Employee {
    name: String,
    salary: f64,
}

struct EmployeeHooks;

impl ser::Hooks for EmployeeHooks {
    fn on_struct(&self, st: &mut ser::StructScope) {
        assert_eq!(st.struct_name(), "Employee");
        assert_eq!(st.struct_len(), 2);
        st.skip_field("salary");
    }
}

let poor_guy = Employee {
    name: "Richie".into(),
    salary: 1_000_000.99,
};

let json = serde_json::to_string(&ser::hook(&poor_guy, &EmployeeHooks)).unwrap();
assert_eq!(json, r#"{"name":"Richie"}"#);
```

Some hooks receive multiple scope objects. This is because in the serde data model types can have multiple "natures". A struct variant, for example, is an enum variant that is also a struct. A tuple is a tuple, but also a sequence. And so on.

For such data model types multiple hooks will be called in sequence, starting from the least specialized down to the most specialized. For example, for a struct variant (i.e. a variant in an enum that is a struct), the hooks will be called in this sequence:

0. [on_value](ser::Hooks::on_value)
1. [on_enum_variant](ser::Hooks::on_enum_variant)
2. [on_struct](ser::Hooks::on_struct)
3. [on_struct_variant](ser::Hooks::on_struct_variant)

Concrete sequences are documented on each hook in [ser::Hooks].

## Passing data to hooks

You might have noticed that each hook functions gets passed a reference to `self`. This is a reference to the value you pass to [ser::hook()]. You can use it to pass data into your hooks:

```rust
use serde::Serialize;
use serde_hooks::ser;

#[derive(Serialize)]
struct Employee {
    name: String,
    salary: f64,
}

struct EmployeeHooks {
    boss_is_asking: bool,
}

impl ser::Hooks for EmployeeHooks {
    fn on_struct(&self, st: &mut ser::StructScope) {
        if !self.boss_is_asking {
            st.skip_field("salary");
        }
    }
}

let poor_guy = Employee {
    name: "Richie".into(),
    salary: 1_000_000.99,
};

let json = serde_json::to_string(&ser::hook(
    &poor_guy, 
    &EmployeeHooks { boss_is_asking: false } 
)).unwrap();
assert_eq!(json, r#"{"name":"Richie"}"#);

let json = serde_json::to_string(&ser::hook(
    &poor_guy, 
    &EmployeeHooks { boss_is_asking: true }
)).unwrap();
assert_eq!(json, r#"{"name":"Richie","salary":1000000.99}"#);
```

Hooks will get an immutable reference to your hooks value. If you want to mutate some state, reach out for your favorite interior mutability construct.

You can reuse the hooks value for multiple serializations. There are two special hooks that can help managing its state when reused: [on_start](ser::Hooks::on_start) (called before serialization begins) and [on_end](ser::Hooks::on_end) (called after it ends).

## Path

TODO

# Performance considerations

This crate will put a wrapper layer between your data and the serializer. It is not zero cost. Although the implementation strives to add as little overhead as possible, for example, by relying heavily on generics and compile-time polymorphism, and reducing allocations to the bare minimum, it is still a layer of logic with calls and branches and so on.

On the global level, the fewer hooks you implement, the lower the performance impact. The compiler can throw away large chunks of code this way.

Next thing to look after is the amount of actions you want performed on your data. The `#[serde]` derive attributes are processed at compile time. This means that, for example, your struct fields will get renamed at compile time to the case you want, and it will be zero cost at runtime. This is obviously not zero cost when the renaming needs to happen as a hook action.

Generally speaking, if your serialization is performance-critical, you should probably not use hooks. Or at least benchmark before you do.