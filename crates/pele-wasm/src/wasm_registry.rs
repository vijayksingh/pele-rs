//! Owned web registry. Native registrations and retained documents stay in Rust.
use pele::compat::{public::PublicEngine, ErrorCode, PeleError, MAX_PUBLIC_SOURCE_BYTES};
use pele::compat::{DiagramType, Engine};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = NativeRegistry)]
pub struct NativeRegistry {
    pub(super) inner: Option<PublicEngine>,
    foreign: HashMap<DiagramType, JsValue>,
    // Only raw caller adoption needs an origin token. Ordinary registries do
    // not allocate one, and replacement never changes an existing token.
    pub(super) raw_entry_origin: std::cell::OnceCell<std::rc::Rc<()>>,
}
fn closed() -> PeleError {
    PeleError::new("Registry is closed", ErrorCode::Semantic)
}
fn bounded(input: &str) -> Result<(), PeleError> {
    if input.len() > MAX_PUBLIC_SOURCE_BYTES {
        Err(PeleError::new(
            "Native transport: declared input exceeds budget",
            ErrorCode::Limit,
        ))
    } else {
        Ok(())
    }
}
fn failure(error: PeleError) -> JsValue {
    JsValue::from_str(&super::render_numbers::failure(&error))
}
#[wasm_bindgen]
impl NativeRegistry {
    #[wasm_bindgen(constructor)]
    pub fn new(all_builtins: bool) -> Self {
        Self {
            inner: Some(if all_builtins {
                PublicEngine::with_builtins()
            } else {
                PublicEngine::empty()
            }),
            foreign: HashMap::new(),
            raw_entry_origin: std::cell::OnceCell::new(),
        }
    }
    pub fn registered(&self, id: &str) -> Result<String, JsValue> {
        let engine = self.inner.as_ref().ok_or_else(|| failure(closed()))?;
        bounded(id).map_err(failure)?;
        let id = pele::compat::DIAGRAM_TYPES
            .into_iter()
            .find(|kind| kind.as_str() == id)
            .map(pele::compat::DiagramId::Builtin)
            .unwrap_or_else(|| pele::compat::DiagramId::extension(id));
        let foreign = pele::compat::DIAGRAM_TYPES
            .into_iter()
            .find(|kind| kind.as_str() == id.as_str())
            .is_some_and(|kind| self.foreign.contains_key(&kind));
        Ok(super::public_result(&(foreign || engine.registered(id))))
    }
    pub fn capabilities(&self) -> Result<String, JsValue> {
        let engine = self.inner.as_ref().ok_or_else(|| failure(closed()))?;
        let mut capabilities = engine.capabilities();
        for value in &mut capabilities {
            if pele::compat::DIAGRAM_TYPES
                .into_iter()
                .find(|kind| kind.as_str() == value.diagram_type.as_str())
                .is_some_and(|kind| self.foreign.contains_key(&kind))
            {
                value.registered = true;
                value.syntax_parity = false;
                value.semantic_parity = false;
                value.render_parity = false;
                value.host_parity = false;
            }
        }
        Ok(super::public_result(&capabilities))
    }
    pub fn register_builtin(&mut self, id: &str) -> Result<(), JsValue> {
        let engine = self.inner.as_mut().ok_or_else(|| failure(closed()))?;
        bounded(id).map_err(failure)?;
        let kind = pele::compat::DIAGRAM_TYPES
            .into_iter()
            .find(|kind| kind.as_str() == id)
            .ok_or_else(|| {
                failure(PeleError::new(
                    format!("Unknown built-in diagram type \"{id}\""),
                    ErrorCode::UnsupportedDiagram,
                ))
            })?;
        engine.register_builtin(kind);
        self.foreign.remove(&kind);
        Ok(())
    }
    pub fn detect_type(&self, source: &str) -> Result<String, JsValue> {
        let engine = self.inner.as_ref().ok_or_else(|| failure(closed()))?;
        bounded(source).map_err(failure)?;
        let kind = engine.detect_type(source).map_err(failure)?;
        Ok(super::public_result(&kind))
    }
    pub fn supports(&self, source: &str) -> Result<String, JsValue> {
        let engine = self.inner.as_ref().ok_or_else(|| failure(closed()))?;
        bounded(source).map_err(failure)?;
        let supported = engine
            .detect_type(source)
            .ok()
            .flatten()
            .is_some_and(|kind| self.foreign.contains_key(&kind) || engine.registered(kind));
        Ok(super::public_result(&supported))
    }
    pub fn register_foreign(&mut self, id: &str, provider: JsValue) -> Result<(), JsValue> {
        self.inner.as_ref().ok_or_else(|| failure(closed()))?;
        let kind = pele::compat::DIAGRAM_TYPES
            .into_iter()
            .find(|kind| kind.as_str() == id)
            .ok_or_else(|| {
                failure(PeleError::new(
                    "Unknown foreign provider diagram type",
                    ErrorCode::UnsupportedDiagram,
                ))
            })?;
        self.foreign.insert(kind, provider);
        Ok(())
    }
    pub fn has_foreign(&self) -> bool {
        !self.foreign.is_empty()
    }
    #[allow(clippy::too_many_arguments)]
    pub fn parse_foreign(
        &self,
        source: &str,
        json: &str,
        source_kind: u32,
        source_value: f64,
        edge_kind: u32,
        edge_value: f64,
    ) -> Result<Option<super::foreign_provider::ForeignDocument>, JsValue> {
        self.inner.as_ref().ok_or_else(|| failure(closed()))?;
        if self.foreign.is_empty() {
            return Ok(None);
        }
        let (options, limits) = super::parse_limits::decode_parse_options(
            source,
            json,
            source_kind,
            source_value,
            edge_kind,
            edge_value,
        )
        .map_err(failure)?;
        let inspection = Engine::empty()
            .inspect_with_limits(source, &options, limits)
            .map_err(failure)?;
        let Some(kind) = inspection.diagram_type else {
            return Ok(None);
        };
        let Some(provider) = self.foreign.get(&kind) else {
            return Ok(None);
        };
        super::foreign_provider::prepare(kind, provider.clone(), inspection, &options, limits)
            .map(Some)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn parse(
        &self,
        source: &str,
        json: &str,
        source_kind: u32,
        source_value: f64,
        edge_kind: u32,
        edge_value: f64,
    ) -> Result<super::MutableDocument, JsValue> {
        let engine = self.inner.as_ref().ok_or_else(|| failure(closed()))?;
        let (options, limits) = super::parse_limits::decode_parse_options(
            source,
            json,
            source_kind,
            source_value,
            edge_kind,
            edge_value,
        )
        .map_err(failure)?;
        Ok(super::MutableDocument {
            inner: engine
                .parse_with_limits(source, &options, limits)
                .map_err(failure)?,
        })
    }
    #[allow(clippy::too_many_arguments)]
    pub fn render_numbers_with_resources(
        &self,
        source: &str,
        json: &str,
        mask: u32,
        numbers: &[f64],
        output_kind: u32,
        output: f64,
        source_kind: u32,
        source_value: f64,
        edge_kind: u32,
        edge_value: f64,
        width: Option<js_sys::Function>,
        icons: Option<js_sys::Function>,
        clock: Option<js_sys::Function>,
    ) -> String {
        super::render_numbers::rendered((|| {
            let engine = self.inner.as_ref().ok_or_else(closed)?;
            let (options, limits, output) = super::render_numbers::options_with_parse_limits(
                source,
                json,
                mask,
                numbers,
                output_kind,
                output,
                source_kind,
                source_value,
                edge_kind,
                edge_value,
            )?;
            engine.render_source_bounded_with_limits(
                source,
                &options,
                limits,
                output,
                &mut super::render_numbers::NumericResources {
                    browser: super::BrowserResources { width, icons },
                    clock,
                },
            )
        })())
    }
    /// Retained PublicDocuments own their registration and survive registry close.
    pub fn close(&mut self) {
        self.inner.take();
        self.foreign.clear();
    }
}
