mod error;
mod map;
mod seq;
mod r#struct;
mod value;

pub use error::ErrorScope;
pub use map::{MapKeySelector, MapScope};
pub use r#struct::{StructManipulation, StructScope, StructVariantScope};
pub use seq::{SeqManipulation, SeqScope};
pub use value::ValueScope;

pub type MapKeyScope<'p, 'v, S> = ValueScope<'p, 'v, S>;

pub(crate) use map::{MapEntryAction, OnMapEntryActions};
pub(crate) use r#struct::{OnStructFieldActions, StructFieldAction};
pub(crate) use seq::{OnSeqElementActions, SeqElementAction};
pub(crate) use value::OnValueAction;
