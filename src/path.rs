use std::{
    cell::{Ref, RefCell},
    fmt::{Debug, Display, Write},
};

use smallvec::SmallVec;

use crate::{StaticValue, Value};

/// A path within the structure of serialized data.
///
/// A path is a list of segments, each segment representing an element of a
/// nested container (e.g. a field of a struct, a map entry or a sequence element).
///
/// The top level (root) path has no segments. If the top level path is a container,
/// the first segment on the path will be an element of that container.
pub struct Path {
    segments: SmallVec<[PathSegment; 8]>,
    str_cache: RefCell<PathStrCache>,
}

struct PathStrCache {
    written_lengths: SmallVec<[u16; 8]>,
    cache: String,
}

impl Path {
    pub(crate) fn new() -> Self {
        Self {
            segments: Default::default(),
            str_cache: RefCell::new(PathStrCache {
                written_lengths: Default::default(),
                cache: String::new(), // no allocation initially
            }),
        }
    }

    pub(crate) fn push_segment(&mut self, segment: PathSegment) {
        self.segments.push(segment);
    }

    pub(crate) fn pop_segment(&mut self) -> PathSegment {
        let res = self.segments.pop().expect("unbalanced pop_segment");

        let mut str_cache = self.str_cache.borrow_mut();

        while str_cache.written_lengths.len() > self.segments.len() {
            // Safety: we're appending valid UTF-8 strings, and removing exactly
            // the amount of bytes that we have appended, so we are guaranteed
            // to end up on a valid utf-8 char boundary.
            // We could have used String::truncate, but it has an unnecessary
            // safety check & a panic in our case.
            unsafe {
                let new_len =
                    str_cache.cache.len() - *str_cache.written_lengths.last().unwrap() as usize;
                str_cache.cache.as_mut_vec().truncate(new_len);
            }
            str_cache.written_lengths.pop();
        }

        res
    }

    /// Returns path segments.
    pub fn segments(&self) -> &[PathSegment] {
        &self.segments
    }

    /// Returns `true` if the path is pointing at the root element of the serialized
    /// data.
    pub fn is_root(&self) -> bool {
        self.segments.is_empty()
    }

    /// Returns a string representation of the path.
    ///
    /// The string representation resembles how you would access elements of your
    /// data in Rust, starting with the elements of the top-level container.
    ///
    /// For example, for the following data structure:
    /// ```
    /// struct Outer {
    ///     inner: Inner,
    /// }
    ///
    /// struct Inner {
    ///     field: u32,
    /// }
    /// ```
    /// the path to `field` will be `"inner.field"`.
    ///
    /// `Path` maintains an internal cache for the string representation of the
    /// segments that is updated lazily when this method is called. It is
    /// optimized to reduce allocations and string formatting for individual
    /// path segments.
    ///
    /// This method returns a borrowed `Ref` for the cached string representation.
    /// The borrowed `Ref` must be dropped at the end of the hook otherwise `Path`
    /// will panic later when serialization goes on.
    ///
    /// You will need to deref the returned `Ref` if you want to compare it against
    /// another string:
    ///
    /// ```
    /// # use serde_hooks::{ser, Path};
    /// struct Hooks;
    /// impl ser::Hooks for Hooks {
    ///     fn on_value<S: serde::Serializer>(&self, path: &Path, value: &mut ser::ValueScope<S>) {
    ///         if *path.borrow_str() == "somewhere.some_day" { // note the deref * in front of path
    ///              //...
    ///         }
    ///     }
    /// }
    /// ```
    pub fn borrow_str(&self) -> Ref<'_, String> {
        {
            let mut str_cache = self.str_cache.borrow_mut();
            while str_cache.written_lengths.len() < self.segments.len() {
                if str_cache.cache.capacity() == 0 {
                    str_cache.cache.reserve(256); // reserve a larger chunk at once, to reduce reallocations
                }

                let len_before = str_cache.cache.len();
                match &self.segments[str_cache.written_lengths.len()] {
                    item @ PathSegment::StructField(_) => {
                        if str_cache.cache.is_empty() {
                            write!(&mut str_cache.cache, "{item}").expect("path concat failed");
                        } else {
                            write!(&mut str_cache.cache, ".{item}").expect("path concat failed");
                        }
                    }
                    item => write!(&mut str_cache.cache, "{item}").expect("path concat failed"),
                }
                let written_length = (str_cache.cache.len() - len_before) as u16;
                str_cache.written_lengths.push(written_length);
            }
        }

        Ref::map(self.str_cache.borrow(), |c| &c.cache)
    }
}

impl ToString for Path {
    /// Returns a string representation of the path.
    ///
    /// This method allocates a new `String` with a copy of path's string representation.
    /// If you want to avoid allocations and copying, consider using [`borrow_str()`](crate::Path::borrow_str)
    /// instead.
    fn to_string(&self) -> String {
        self.borrow_str().clone()
    }
}

impl PartialEq<str> for Path {
    fn eq(&self, other: &str) -> bool {
        *self.borrow_str() == other
    }
}

impl PartialEq<Path> for str {
    fn eq(&self, other: &Path) -> bool {
        *other.borrow_str() == self
    }
}

/// A key of a serialized map entry.
#[derive(Debug, Clone)]
pub struct PathMapKey {
    /// Sequential index of the key in the map during serialization.
    ///
    /// Note that this is the index as serde "sees" it. If you're using unordered maps (e.g. a HashMap),
    /// this index will have little meaning and might actually point to different elements
    /// even if the data in the maps is the exactly the same, or for the same map in different
    /// application runs.
    ///
    /// For ordered maps this index might be useful if your map keys are not trivially serializable,
    /// like, for example, tuples. In which case you won't have the full key value captured, but would
    /// still be able to refer to a concrete map entry by its index.
    pub index: usize,

    /// Captured value of the map key.
    ///
    /// For trivial values, like numbers, the actual value is captured here.
    ///
    /// For non-trivial, compound values, only the metadata is captured.
    ///
    /// For map keys of type `Value::Bytes` the actual bytes are not captured to avoid
    /// allocation on every map key.
    pub value: StaticValue,
}

impl PathMapKey {
    pub(crate) fn new(index: usize, value: StaticValue) -> Self {
        Self { index, value }
    }
}

impl Display for PathMapKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            Value::Bool(_)
            | Value::I8(_)
            | Value::I16(_)
            | Value::I32(_)
            | Value::I64(_)
            | Value::U8(_)
            | Value::U16(_)
            | Value::U32(_)
            | Value::U64(_)
            | Value::F32(_)
            | Value::F64(_)
            | Value::Char(_)
            | Value::Str(_)
            | Value::Unit
            | Value::Some
            | Value::None
            | Value::UnitVariant { .. } => Display::fmt(&self.value, f),
            _ => Display::fmt(&self.index, f),
        }
    }
}

/// A segment in a path that represents an element of a container in the serialized data.
#[derive(Debug, Clone)]
pub enum PathSegment {
    /// The segment is an entry in a map, identified by a map key.
    MapEntry(PathMapKey),
    /// The segment is a field in a `struct`, identified by the field name.
    StructField(&'static str),
    /// The segment is an element in a sequence or a tuple, identified by its index.
    SeqElement(usize),
}

impl Display for PathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathSegment::MapEntry(key) => f.write_fmt(format_args!("[{key}]")),
            PathSegment::StructField(field_name) => f.write_str(field_name),
            PathSegment::SeqElement(index) => f.write_fmt(format_args!("[{index}]")),
        }
    }
}

impl From<PathMapKey> for PathSegment {
    fn from(map_key: PathMapKey) -> Self {
        PathSegment::MapEntry(map_key)
    }
}
