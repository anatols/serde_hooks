use crate::ser::Path;

pub trait Hooks {
    fn start(&self) {}
    fn end(&self) {}

    fn before_container(&self, path: &Path) {}
}

// skip field(s)
// retain field(s)
// replace value (in struct, map, array or leaf?)
// replace key?
