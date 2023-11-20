use crate::*;
use crate::xkb_transformer_registry::TransformerParams;

lazy_static! {
    pub static ref DEFAULT_TRANSFORMER_PARAMS: RwLock<TransformerParams> = RwLock::new(TransformerParams::default());
}

#[cfg(feature = "integration")]
lazy_static! {
    pub static ref TEST_PIPE: Mutex<Vec<testing::TestEvent>> = Mutex::new(vec![]);
}