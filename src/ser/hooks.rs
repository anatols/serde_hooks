use crate::ser::Path;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MapKey {
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Char(char),
    Str(String),
    AtIndex(usize),
}

//TODO implement other conversions
impl From<&str> for MapKey {
    fn from(value: &str) -> Self {
        MapKey::Str(value.to_string())
    }
}

#[derive(Debug)]
pub enum MapAction {
    SkipKey(MapKey),
}

//TODO move to a submodule
pub struct MapScope<'p> {
    path: &'p Path,
    map_len: Option<usize>,
    actions: Vec<MapAction>,
}

impl<'h> MapScope<'h> {
    pub(crate) fn new(path: &'h Path, map_len: Option<usize>) -> Self {
        Self {
            path,
            map_len,
            actions: vec![],
        }
    }

    pub(crate) fn into_actions(self) -> Vec<MapAction> {
        //TODO validate if actions are compatible
        self.actions
    }

    pub fn path(&self) -> &Path {
        self.path
    }

    pub fn map_len(&self) -> Option<usize> {
        self.map_len
    }

    pub fn skip_key(&mut self, key: impl Into<MapKey>) {
        self.actions.push(MapAction::SkipKey(key.into()));
    }
}

pub trait Hooks {
    fn start(&self) {}
    fn end(&self) {}

    fn on_map(&self, _map: &mut MapScope) {}
}

// skip field(s)
// retain field(s)
// replace value (in struct, map, array or leaf?)
// replace key?
