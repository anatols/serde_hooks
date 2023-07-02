pub trait Hooks {
    fn start(&self) {}
    fn end(&self) {}
}

// skip field(s)
// retain field(s)
// replace value (in struct, map, array or leaf?)
// replace key?
