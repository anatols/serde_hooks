# Must have before the initial release

- Publish on crates.io / docs.rs.

- Set up github pipeline to generate docs.

# Ideas & wishlist

- Make struct field-related errors ("field not found" when doing a rename/skip/flatten) not propagate by default.
  This should make the scope API more ergonomic by avoiding the necessity to check path or implement on_error.

- Flatten for maps.

- Flatten for sequences.

- Support for insert before, insert after, push back in sequences.

- Examples for each function of each scope

- Add shields to readme.
