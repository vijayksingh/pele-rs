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
    pub fn arity(&self) -> u32 {
        self.inner.arity()
    }
    /// Captured builtin identity only; arbitrary schemas cannot supply it.
    pub fn reflection(&self) -> Option<String> {
        #[derive(serde::Serialize)]
        struct ReflectionProof {
            reflection: pele::compat::public::ConstructorReflection,
            schema: pele::compat::public::ModelSchemaView,
        }
        self.inner.reflection().map(|reflection| {
            // Serialize the schema through its authoritative struct visitor.
            // A serde_json::Value would sort object keys and invalidate the
            // host's exact captured-schema binding before publication.
            super::public_result(&ReflectionProof {
                reflection,
                schema: self.inner.schema_view(),
            })
        })
    }

    pub fn argument_transfers(&self) -> Vec<u32> {
        self.inner
            .argument_transfers()
            .iter()
            .map(|mode| match mode {
                pele::compat::public::ConstructorArgumentTransfer::Retain => 0,
                pele::compat::public::ConstructorArgumentTransfer::Value => 1,
            })
            .collect()
    }
    pub fn begin_construct(
        &self,
        first_tag: u32,
        first: JsValue,
        second_tag: u32,
        second: JsValue,
    ) -> Result<super::constructor_body::PublicConstructorJob, JsValue> {
        let run = || -> Result<_, pele::compat::PeleError> {
            let schema = super::public_result(&self.inner.schema_view());
            let diagram_type = self.inner.diagram_type().as_str().to_owned();
            let inner = self.inner.begin_construct_with_arguments([
                super::detached_invocation::host_value(first_tag, &first)?,
                super::detached_invocation::host_value(second_tag, &second)?,
            ])?;
            Ok(super::constructor_body::PublicConstructorJob::new(
                inner,
                schema,
                diagram_type,
            ))
        };
        run().map_err(|error| JsValue::from_str(&render_numbers::failure(&error)))
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
