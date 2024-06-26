use crate::xkb::XKBTransformer;
use crate::*;
use std::collections::HashMap;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct TransformerParams {
    pub model: String,
    pub layout: String,
    pub variant: Option<String>,
    pub options: Option<String>,
}

impl TransformerParams {
    pub fn new(
        model: Option<String>,
        layout: Option<String>,
        variant: Option<String>,
        options: Option<String>,
    ) -> Self {
        let default = global::DEFAULT_TRANSFORMER_PARAMS.read().unwrap();
        let model = model.unwrap_or(default.model.clone());
        let layout = layout.unwrap_or(default.layout.clone());
        let variant = variant.or(default.variant.clone());
        let options = options.or(default.options.clone());
        Self { model, layout, variant, options }
    }
}

impl Default for TransformerParams {
    fn default() -> Self {
        Self { model: "pc105".to_string(), layout: "us".to_string(), variant: None, options: None }
    }
}

pub struct XKBTransformerRegistry {
    registry: Mutex<HashMap<TransformerParams, Weak<XKBTransformer>>>,
}

impl XKBTransformerRegistry {
    pub fn new() -> Self {
        Self { registry: Mutex::new(HashMap::new()) }
    }

    pub fn get(&self, params: &TransformerParams) -> Result<Arc<XKBTransformer>> {
        let mut registry = self.registry.lock().unwrap();
        let res = registry.get(&params);

        match res {
            Some(f) => match f.upgrade() {
                Some(transformer) => Ok(transformer),
                None => {
                    let transformer = Arc::new(XKBTransformer::new(
                        &params.model,
                        &params.layout,
                        params.variant.as_deref(),
                        params.options.clone(),
                    )?);
                    registry.insert(params.clone(), Arc::downgrade(&transformer));
                    Ok(transformer)
                }
            },
            None => {
                let transformer = Arc::new(XKBTransformer::new(
                    &params.model,
                    &params.layout,
                    params.variant.as_deref(),
                    params.options.clone(),
                )?);
                registry.insert(params.clone(), Arc::downgrade(&transformer));
                Ok(transformer)
            }
        }
    }
}

lazy_static! {
    pub static ref XKB_TRANSFORMER_REGISTRY: XKBTransformerRegistry = XKBTransformerRegistry::new();
}
