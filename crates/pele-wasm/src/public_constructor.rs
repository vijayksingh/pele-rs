//! Retained, native-authenticated public constructors. The host owns the newly
//! allocated JavaScript receiver; native construction owns all model defaults.
use super::{render_numbers, MutableDocument};
use pele::compat::public::{CapturedPublicConstructor, PrototypeId};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = PublicConstructor)]
pub struct PublicConstructor {
    inner: CapturedPublicConstructor,
}

impl PublicConstructor {
    pub(super) fn captured(inner: CapturedPublicConstructor) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
impl MutableDocument {
    pub fn capture_constructor(&self, prototype: u32) -> Option<PublicConstructor> {
        self.inner
            .capture_constructor(PrototypeId(prototype))
            .map(PublicConstructor::captured)
    }
}

#[wasm_bindgen]
impl PublicConstructor {
    pub fn prototype_id(&self) -> u32 {
        self.inner.prototype().0
    }

    pub fn ignores_arguments(&self) -> bool {
        self.inner.ignores_arguments()
    }

    pub fn construct(&self) -> Result<MutableDocument, JsValue> {
        self.inner
            .construct()
            .map(|inner| MutableDocument { inner })
            .map_err(|error| JsValue::from_str(&render_numbers::failure(&error)))
    }
}
