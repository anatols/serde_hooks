# Must have before the initial release

- Write a readme.
- Publish on crates.io / docs.rs.
- Address remaining TODOs in code. None must remain.


# Ideas & wishlist

- Make struct field-related errors ("field not found" when doing a rename/skip/flatten) not propagate by default.
  This should make the scope API more ergonomic by avoiding the necessity to check path or implement on_error.

- Flatten for maps.

- Flatten for sequences.

- Support for insert before, insert after, push back in sequences.

- Make it possible somehow to compare path to string literals w/o the weird deref-ref syntax (`&*path.borrow_str()`).
