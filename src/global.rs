use crate::*;
use crate::xkb_transformer_registry::TransformerParams;

lazy_static! {
    pub static ref DEFAULT_TRANSFORMER_PARAMS: RwLock<TransformerParams> = RwLock::new(TransformerParams::default());
}