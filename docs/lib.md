# Serde Hooks

This crate allows you to hook into [`serde`] serialization. You can get callbacks for each piece of data that is being serialized, and modify serialization behavior at runtime.

For example, you can, _at runtime_:

- Rename or skip struct fields.
- Rename enum variants.
- Replace struct field values, map keys and values, sequence and tuple items.
- Add new entries to maps.
- Introspect serialized data structures.

# "Why do I need this crate?"

You likely don't.

Seriously, in the vast majority of cases, what [`serde`] offers out of the box is enough, and more optimal to use. Before you consider using this crate, please check [serde.rs](https://serde.rs), including the [examples](https://serde.rs/examples.html) section. After that, consider changing your data model in a way that you can use bare `serde`. Only then, if none of the above helps, come back for the hooks.

Some examples where having runtime control over serialization can be handy:

- You need to exclude certain fields from an API response based on permissions level. E.g. exclude `salary` field from `Employee` struct for everyone but their manager.
- You need to support different case conventions for different formats. E.g. camelCase in JSON and kebab-case in YAML.
- Your data type comes from a library you cannot change, and already has a `#[derive(Serialize)]`. Just not exactly with the `#[serde]` attributes you want there.

Obviously, all of these cases can be handled by either [implementing custom serialization](https://serde.rs/impl-serialize.html), or using something like [`#[serde(serialize_with = ...)]`](https://serde.rs/field-attrs.html#serialize_with). But let's face it, it's not fun: you'd need to type a ton of boilerplate and the default `serde` derive is so much nicer to use.

In fact, this crate actually is that ton of boilerplate that just calls back to you at the right moments.

# Serialization API

See [documentation for module `ser`](crate::ser).

# Examples

There are more examples available in the repository, in the `examples` directory.
