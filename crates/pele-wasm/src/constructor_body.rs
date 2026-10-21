//! Provisional owned constructor state; no native borrow crosses a host request.
use super::{
    detached_invocation::{host_reply, step_metadata},
    render_numbers, MutableDocument,
};
use pele::compat::{public::*, ErrorCode, PeleError};
use wasm_bindgen::prelude::*;
fn invalid(message: &str) -> PeleError {
    PeleError::new(message, ErrorCode::Semantic)
}
fn failure(error: PeleError) -> JsValue {
    JsValue::from_str(&render_numbers::failure(&error))
}
#[wasm_bindgen(js_name=PublicConstructorJob)]
pub struct PublicConstructorJob {
    inner: PublicConstructionJob,
    schema: String,
    diagram_type: String,
}
impl PublicConstructorJob {
    pub(super) fn new(inner: PublicConstructionJob, schema: String, diagram_type: String) -> Self {
        Self {
            inner,
            schema,
            diagram_type,
        }
    }
    fn frame(&self, metadata: &impl serde::Serialize) -> Result<Vec<u8>, PeleError> {
        let metadata = render_numbers::encode(metadata)?;
        encode_binary_exchange(
            self.inner.revision()?,
            &self.inner.snapshot()?,
            &metadata,
            DEFAULT_BINARY_BUDGET,
        )
    }
}
#[wasm_bindgen]
impl PublicConstructorJob {
    pub fn schema(&self) -> String {
        self.schema.clone()
    }
    pub fn entry_type(&self) -> String {
        self.diagram_type.clone()
    }
    pub fn snapshot_binary(&self) -> Result<Vec<u8>, JsValue> {
        self.frame(&serde_json::json!({"type":self.diagram_type}))
            .map_err(failure)
    }
    pub fn apply_binary(&mut self, bytes: &[u8]) -> String {
        let mut run = || -> Result<String, PeleError> {
            let exchange = decode_binary_exchange(bytes, DEFAULT_BINARY_BUDGET)?;
            if !exchange.metadata.is_empty() {
                return Err(invalid("Apply binary metadata must be empty"));
            }
            self.inner.apply(exchange.revision, exchange.graph)?;
            Ok(self.inner.revision()?.to_string())
        };
        match run() {
            Ok(revision) => super::public_result(&revision),
            Err(error) => pele::compat::error_envelope(&error),
        }
    }
    pub fn invoke_binary_with_functions(
        &mut self,
        revision: &str,
        call_json: &str,
        dispatcher: js_sys::Function,
    ) -> Result<Vec<u8>, JsValue> {
        let run = || -> Result<Vec<u8>, PeleError> {
            if call_json.len() > DEFAULT_BINARY_BUDGET {
                return Err(invalid(
                    "Public method JSON exceeds 64 MiB transport budget",
                ));
            }
            let revision = revision
                .parse()
                .map_err(|_| invalid("Invalid public model revision"))?;
            let call = serde_json::from_str(call_json)
                .map_err(|_| invalid("Invalid public model method call"))?;
            let mut host = super::BrowserResources {
                width: None,
                icons: None,
            };
            let outcome = self.inner.invoke(
                revision,
                call,
                &mut super::host_functions::Resources {
                    host: &mut host,
                    dispatcher,
                },
            )?;
            let metadata = render_numbers::encode(&outcome.result)?;
            encode_binary_exchange(
                outcome.revision,
                &outcome.graph,
                &metadata,
                DEFAULT_BINARY_BUDGET,
            )
        };
        run().map_err(failure)
    }
    pub fn resume(
        &mut self,
        reply_state: u32,
        value_tag: u32,
        value: JsValue,
    ) -> Result<Vec<u8>, JsValue> {
        let mut run = || -> Result<Vec<u8>, PeleError> {
            let step = self
                .inner
                .resume(host_reply(reply_state, value_tag, &value)?)?;
            self.frame(&step_metadata(step))
        };
        run().map_err(failure)
    }
    pub fn finish(&mut self) -> Result<js_sys::Array, JsValue> {
        let mut run = || -> Result<js_sys::Array, PeleError> {
            let outcome = self.inner.finish_retaining_receiver()?;
            let metadata = render_numbers::encode(&outcome.result)?;
            let graph = outcome.document.snapshot_read()?.into_model()?;
            let bytes = encode_binary_exchange(
                outcome.document.revision(),
                &graph,
                &metadata,
                DEFAULT_BINARY_BUDGET,
            )?;
            drop(graph);
            let packet = js_sys::Array::new();
            packet.push(
                &MutableDocument {
                    inner: outcome.document,
                }
                .into(),
            );
            packet.push(&js_sys::Uint8Array::from(bytes.as_slice()));
            Ok(packet)
        };
        run().map_err(failure)
    }
}
