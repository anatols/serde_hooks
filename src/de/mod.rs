mod wrapper;
mod context;

pub trait Hooks {
    fn on_start(&self) {}

    fn on_end(&self) {}

    //on hint
    //on value
}

pub enum Hint {
    Any,
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Char,
    Str,
    String,

    Bytes,
    ByteBuf,

    Option,

    Unit,

    UnitStruct(&'static str),
    NewtypeStruct(&'static str),
    TupleStruct {
        name: &'static str,
        len: usize,
    },
    Struct {
        name: &'static str,
        fields: &'static [&'static str],
    },

    Seq,

    Tuple(usize),

    Map,

    Enum {
        name: &'static str,
        variants: &'static [&'static str],
    },

    Identifier,

    IgnoredAny,
}