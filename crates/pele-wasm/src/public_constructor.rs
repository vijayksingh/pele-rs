//! Retained, native-authenticated public constructors. The host owns the newly
//! allocated JavaScript receiver; native construction owns all model defaults.
use super::{render_numbers, MutableDocument};
use pele::compat::public::{CapturedPublicConstructor, ConstructorArgument, PrototypeId};
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
    pub fn accepts_raw_document(&self, document: &MutableDocument) -> bool {
        self.inner.accepts_raw_document(&document.inner)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn render_raw_operation_packet_with_functions(
        &self,
        document: &mut MutableDocument,
        revision: &str,
        config_json: &str,
        json: &str,
        mask: u32,
        numbers: &[f64],
        width: Option<js_sys::Function>,
        icons: Option<js_sys::Function>,
        clock: Option<js_sys::Function>,
        inline_icons: Option<js_sys::Function>,
        dispatcher: Option<js_sys::Function>,
    ) -> Result<JsValue, JsValue> {
        super::prepared_entries::raw_operation_packet(
            document,
            Some(&self.inner),
            revision,
            config_json,
            json,
            mask,
            numbers,
            width,
            icons,
            clock,
            inline_icons,
            dispatcher,
        )
    }
    pub fn prototype_id(&self) -> u32 {
        self.inner.prototype().0
    }

    pub fn ignores_arguments(&self) -> bool {
        self.inner.ignores_arguments()
    }

    pub fn live_fields(&self) -> String {
        serde_json::to_string(&self.inner.live_fields().iter().map(|field|serde_json::json!({"key":field.key,"methods":field.methods.iter().map(|method|method.0).collect::<Vec<_>>()})).collect::<Vec<_>>()).expect("static field metadata")
    }
    pub fn construct_with_host_reference(
        &self,
        reference: Option<u32>,
    ) -> Result<MutableDocument, JsValue> {
        if reference.is_some_and(|id| id >= 1_000_000) {
            return Err(JsValue::from_str(&render_numbers::failure(
                &pele::compat::PeleError::new(
                    "Invalid owner host token",
                    pele::compat::ErrorCode::Semantic,
                ),
            )));
        }
        self.inner
            .construct_with(
                reference
                    .map(ConstructorArgument::Opaque)
                    .unwrap_or(ConstructorArgument::Undefined),
            )
            .map(|inner| MutableDocument { inner })
            .map_err(|error| JsValue::from_str(&render_numbers::failure(&error)))
    }
    pub fn construct(&self) -> Result<MutableDocument, JsValue> {
        self.inner
            .construct()
            .map(|inner| MutableDocument { inner })
            .map_err(|error| JsValue::from_str(&render_numbers::failure(&error)))
    }
}
