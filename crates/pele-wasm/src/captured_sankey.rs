//! Opt-in captured Sankey facade boundary. No flat graph is retained in Rust.
//! The production JS selector remains disabled until complete owner proof.
use super::{
    captured_store::{CapturedStore, ErrorLane, InitialTransport},
    wasm_registry::NativeRegistry,
};
use js_sys::{Array, Function};
use pele::compat::{
    host_sankey::{admit_observed, admit_outbound, Document, Parsed},
    public::{HostSankeyEntrySelection, ModelSchemaView, PrototypeId, DEFAULT_BINARY_BUDGET},
    DiagramType, ErrorCode, PeleError, Rendered,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::prelude::*;

fn invalid(message: &str) -> PeleError {
    PeleError::new(message, ErrorCode::Semantic)
}
fn failure(error: PeleError) -> JsValue {
    JsValue::from_str(&super::render_numbers::failure(&error))
}
fn original(error: JsValue) -> JsValue {
    let out = Array::new();
    out.push(&error);
    out.into()
}
fn finish<T>(result: Result<T, PeleError>, errors: &ErrorLane) -> Result<T, JsValue> {
    if let Some(error) = errors.take() {
        Err(original(error))
    } else {
        result.map_err(failure)
    }
}
fn revision(text: &str) -> Result<u64, PeleError> {
    text.parse()
        .map_err(|_| invalid("Invalid public model revision"))
}
fn prototype(schema: ModelSchemaView) -> Result<PrototypeId, PeleError> {
    schema
        .model
        .root
        .and_then(|root| schema.model.records.iter().find(|record| record.id == root))
        .and_then(|record| record.prototype)
        .ok_or_else(|| invalid("Captured model has no root prototype"))
}
fn metadata(value: &impl serde::Serialize) -> Result<Vec<u8>, PeleError> {
    // Initial snapshot type metadata retains the existing serde JSON encoding.
    serde_json::to_vec(value).map_err(|_| invalid("Invalid captured operation metadata"))
}
fn metadata_text(bytes: Vec<u8>) -> Result<JsValue, PeleError> {
    String::from_utf8(bytes)
        .map(|value| JsValue::from_str(&value))
        .map_err(|_| invalid("Invalid captured operation metadata"))
}
fn packet(revision: u64, bytes: usize, status: u32, payload: JsValue) -> JsValue {
    let out = Array::new();
    out.push(&JsValue::from_f64(1.));
    out.push(&JsValue::from_str(&revision.to_string()));
    out.push(&JsValue::from_f64(bytes as f64));
    out.push(&JsValue::from_f64(status as f64));
    out.push(&payload);
    out.into()
}

/// take/restore prevents a RefCell borrow from spanning imports or callbacks.
/// Native/private revision commits separately from JS facade acknowledgement.
struct Active<'a> {
    owner: &'a CapturedSankeyDocument,
    document: Option<Document>,
}
impl Active<'_> {
    fn document(&mut self) -> Result<&mut Document, PeleError> {
        self.document
            .as_mut()
            .ok_or_else(|| invalid("Captured document is unavailable"))
    }
    fn commit_if_changed(&mut self, before: u64, commit: &Function) -> Result<(), JsValue> {
        let current = self.document().map_err(failure)?.revision();
        if current != before {
            commit
                .call1(
                    &JsValue::UNDEFINED,
                    &JsValue::from_str(&current.to_string()),
                )
                .map_err(original)?;
        }
        Ok(())
    }
}
impl Drop for Active<'_> {
    fn drop(&mut self) {
        if let Some(document) = self.document.take() {
            self.owner.revision.set(document.revision());
            *self.owner.document.borrow_mut() = Some(document);
        }
        self.owner.busy.set(false);
    }
}
#[wasm_bindgen(js_name = CapturedSankeyDocument)]
pub struct CapturedSankeyDocument {
    document: RefCell<Option<Document>>,
    revision: Cell<u64>,
    busy: Cell<bool>,
}
impl CapturedSankeyDocument {
    fn new(document: Document) -> Self {
        Self {
            revision: Cell::new(document.revision()),
            document: RefCell::new(Some(document)),
            busy: Cell::new(false),
        }
    }
    fn take(&self) -> Result<Active<'_>, JsValue> {
        if self.busy.replace(true) {
            return Err(js_sys::TypeError::new("Public model document is already in use").into());
        }
        let document = self.document.borrow_mut().take();
        if document.is_none() {
            self.busy.set(false);
            return Err(failure(invalid("Captured document is unavailable")));
        }
        Ok(Active {
            owner: self,
            document,
        })
    }
    fn store<'a>(
        &self,
        context: JsValue,
        active: &mut Active<'_>,
        errors: &'a ErrorLane,
    ) -> Result<CapturedStore<'a>, PeleError> {
        CapturedStore::new(context, prototype(active.document()?.schema())?, errors)
    }
}
#[wasm_bindgen]
impl CapturedSankeyDocument {
    pub fn revision(&self) -> String {
        self.revision.get().to_string()
    }
    pub fn entry_type(&self) -> String {
        "sankey".into()
    }
    pub fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    pub fn schema(&self) -> Result<String, JsValue> {
        let mut active = self.take()?;
        Ok(super::public_result(
            &active.document().map_err(failure)?.schema(),
        ))
    }
    #[cfg(target_arch = "wasm32")]
    pub fn schema_cache_key(&self) -> Result<String, JsValue> {
        let mut active = self.take()?;
        let view = active.document().map_err(failure)?.schema();
        Ok(format!(
            "1:{:x}:{:x}:{}:{:x}:{}",
            std::ptr::from_ref(view.model) as usize,
            view.shared_functions.as_ptr() as usize,
            view.shared_functions.len(),
            view.function_shapes.as_ptr() as usize,
            view.function_shapes.len()
        ))
    }
    pub fn live_provider_view(&self) -> Result<Array, JsValue> {
        let config_json = {
            let mut active = self.take()?;
            super::foreign_provider::config_json(active.document().map_err(failure)?.config())
                .map_err(failure)?
        };
        let config = js_sys::JSON::parse(&config_json)
            .map_err(super::callback_error)
            .map_err(failure)?;
        let out = Array::new();
        out.push(&JsValue::from_str("sankey"));
        out.push(&config);
        out.push(&super::foreign_provider::retained_policy(DiagramType::Sankey).into());
        Ok(out)
    }
    pub fn apply_captured(
        &self,
        context: JsValue,
        expected: &str,
        commit: Function,
    ) -> Result<String, JsValue> {
        let errors = ErrorLane::default();
        let mut active = self.take()?;
        let mut store = finish(self.store(context, &mut active, &errors), &errors)?;
        // Decoder-equivalent ingress precedes the core's stale-revision check.
        finish(
            admit_observed(&mut store, 0, DEFAULT_BINARY_BUDGET),
            &errors,
        )?;
        let expected = revision(expected).map_err(failure)?;
        let before = active.document().map_err(failure)?.revision();
        let result = active
            .document()
            .map_err(failure)?
            .apply(expected, &mut store);
        let committed = active.commit_if_changed(before, &commit);
        finish(result, &errors)?;
        committed?;
        Ok(super::public_result(
            &active.document().map_err(failure)?.revision().to_string(),
        ))
    }
    pub fn invoke_captured(
        &self,
        context: JsValue,
        expected: &str,
        call_json: &str,
        commit: Function,
    ) -> Result<JsValue, JsValue> {
        let errors = ErrorLane::default();
        let mut active = self.take()?;
        // Keep old call JSON byte/parse validation before native invocation.
        if call_json.len() > 64 * 1024 * 1024 {
            return Err(failure(invalid(
                "Public method JSON exceeds 64 MiB transport budget",
            )));
        }
        let expected = revision(expected).map_err(failure)?;
        let call = serde_json::from_str(call_json)
            .map_err(|_| failure(invalid("Invalid public model method call")))?;
        let mut store = finish(self.store(context, &mut active, &errors), &errors)?;
        let before = active.document().map_err(failure)?.revision();
        let result = active
            .document()
            .map_err(failure)?
            .invoke(expected, &mut store, call);
        let committed = active.commit_if_changed(before, &commit);
        let outcome = finish(result, &errors)?;
        committed?;
        let bytes = super::render_numbers::encode(&outcome.result).map_err(failure)?;
        let length = bytes.len();
        finish(
            admit_outbound(&mut store, length, DEFAULT_BINARY_BUDGET),
            &errors,
        )?;
        Ok(packet(
            outcome.revision,
            length,
            0,
            metadata_text(bytes).map_err(failure)?,
        ))
    }
    #[allow(clippy::too_many_arguments)]
    pub fn render_captured(
        &self,
        context: JsValue,
        expected: &str,
        json: &str,
        mask: u32,
        numbers: &[f64],
        output_kind: u32,
        output: f64,
        width: Option<Function>,
        icons: Option<Function>,
        clock: Option<Function>,
        commit: Function,
    ) -> Result<JsValue, JsValue> {
        let errors = ErrorLane::default();
        let mut active = self.take()?;
        let (options, limit) =
            super::render_numbers::options(json, mask, numbers, output_kind, output)
                .map_err(failure)?;
        let expected = revision(expected).map_err(failure)?;
        let mut store = finish(self.store(context, &mut active, &errors), &errors)?;
        let before = active.document().map_err(failure)?.revision();
        let mut resources = super::render_numbers::NumericResources {
            browser: super::BrowserResources { width, icons },
            clock,
        };
        let result = active.document().map_err(failure)?.render(
            expected,
            &mut store,
            &options,
            &mut resources,
            limit,
            true,
        );
        let committed = active.commit_if_changed(before, &commit);
        let outcome = finish(result, &errors)?;
        committed?;
        let transport = outcome
            .result
            .as_ref()
            .map(pele::compat::render_result_transport);
        render_packet(
            &mut store,
            &errors,
            outcome.revision,
            false,
            outcome
                .result
                .as_ref()
                .map_or("", |value| value.diagram_type.as_str()),
            outcome.result.as_ref().ok().map(|value| &value.rendered),
            &transport,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub fn render_raw_captured(
        &self,
        context: JsValue,
        expected: &str,
        config_json: &str,
        json: &str,
        mask: u32,
        numbers: &[f64],
        width: Option<Function>,
        icons: Option<Function>,
        clock: Option<Function>,
        inline_icons: Option<Function>,
        commit: Function,
    ) -> Result<JsValue, JsValue> {
        let errors = ErrorLane::default();
        let mut active = self.take()?;
        let config = super::prepared_entries::config(config_json).map_err(failure)?;
        let (options, _) =
            super::render_numbers::options(json, mask, numbers, 3, 0.).map_err(failure)?;
        let expected = revision(expected).map_err(failure)?;
        let mut store = finish(self.store(context, &mut active, &errors), &errors)?;
        let before = active.document().map_err(failure)?.revision();
        let mut resources =
            super::prepared_entries::host_er_raw_resources(width, icons, clock, inline_icons);
        let result = active.document().map_err(failure)?.render_raw(
            expected,
            &mut store,
            &config,
            &options.render,
            &mut resources,
        );
        let committed = active.commit_if_changed(before, &commit);
        let outcome = finish(result, &errors)?;
        committed?;
        let transport = outcome
            .result
            .as_ref()
            .map(super::prepared_entries::transport);
        render_packet(
            &mut store,
            &errors,
            outcome.revision,
            true,
            "",
            outcome.result.as_ref().ok(),
            &transport,
        )
    }
}
fn render_packet(
    store: &mut CapturedStore<'_>,
    errors: &ErrorLane,
    revision: u64,
    raw: bool,
    diagram_type: &str,
    rendered: Option<&Rendered>,
    transport: &impl serde::Serialize,
) -> Result<JsValue, JsValue> {
    let Some(rendered) = rendered else {
        let bytes = super::render_numbers::encode(transport).map_err(failure)?;
        let length = bytes.len();
        finish(admit_outbound(store, length, DEFAULT_BINARY_BUDGET), errors)?;
        return Ok(packet(
            revision,
            length,
            0,
            metadata_text(bytes).map_err(failure)?,
        ));
    };
    let prepared = super::render_packet::prepare(rendered, transport).map_err(failure)?;
    let length = prepared.metadata_bytes();
    finish(admit_outbound(store, length, DEFAULT_BINARY_BUDGET), errors)?;
    Ok(
        match super::render_packet::emit(raw, diagram_type, rendered, prepared) {
            Ok(value) => packet(revision, length, 0, value),
            Err(error) => packet(revision, length, 1, error),
        },
    )
}

#[wasm_bindgen(js_name = CapturedSankeyEntrySelection)]
pub struct CapturedSankeyEntrySelection {
    inner: HostSankeyEntrySelection,
    origin: Rc<()>,
}
#[wasm_bindgen]
impl CapturedSankeyEntrySelection {
    pub fn schema(&self) -> String {
        super::public_result(&self.inner.schema())
    }
}
fn parsed(parsed: Parsed<CapturedStore<'_>>, errors: &ErrorLane) -> Result<Array, JsValue> {
    let out = Array::new();
    match parsed {
        Parsed::Arena(inner) => {
            out.push(&JsValue::from_f64(0.));
            out.push(&super::MutableDocument { inner }.into());
        }
        Parsed::Captured {
            document,
            mut storage,
        } => {
            // Original snapshot_view schema validation precedes its type metadata
            // and encode_binary_exchange structural/combined frame admission.
            finish(document.snapshot_admission(&mut storage), errors)?;
            let type_bytes =
                metadata(&serde_json::json!({"type":document.diagram_type()})).map_err(failure)?;
            finish(
                admit_outbound(&mut storage, type_bytes.len(), DEFAULT_BINARY_BUDGET),
                errors,
            )?;
            let context = storage.into_context();
            out.push(&JsValue::from_f64(1.));
            out.push(&CapturedSankeyDocument::new(document).into());
            out.push(&context);
        }
    }
    Ok(out)
}
#[wasm_bindgen]
impl NativeRegistry {
    pub fn select_prepared_captured_sankey(
        &self,
        id: &str,
    ) -> Result<Option<CapturedSankeyEntrySelection>, JsValue> {
        let engine = self
            .inner
            .as_ref()
            .ok_or_else(|| failure(invalid("Registry is closed")))?;
        let id = super::prepared_entries::builtin_id(id).map_err(failure)?;
        let selected = engine.select_host_sankey_entry(&id).map_err(failure)?;
        Ok(selected.map(|inner| CapturedSankeyEntrySelection {
            inner,
            origin: self.raw_entry_origin.get_or_init(|| Rc::new(())).clone(),
        }))
    }
    pub fn bind_prepared_captured_sankey(
        &self,
        selection: &CapturedSankeyEntrySelection,
        context: JsValue,
        config_json: &str,
    ) -> Result<CapturedSankeyDocument, JsValue> {
        self.inner
            .as_ref()
            .ok_or_else(|| failure(invalid("Registry is closed")))?;
        if !self
            .raw_entry_origin
            .get()
            .is_some_and(|origin| Rc::ptr_eq(origin, &selection.origin))
        {
            return Err(failure(invalid(
                "Raw entry selection belongs to another registry",
            )));
        }
        let config = super::prepared_entries::config(config_json).map_err(failure)?;
        let errors = ErrorLane::default();
        let mut store = finish(
            CapturedStore::new(
                context,
                prototype(selection.inner.schema()).map_err(failure)?,
                &errors,
            ),
            &errors,
        )?;
        finish(
            admit_observed(&mut store, 0, DEFAULT_BINARY_BUDGET),
            &errors,
        )?;
        let document = finish(selection.inner.bind(config, &mut store), &errors)?;
        finish(document.snapshot_admission(&mut store), &errors)?;
        let type_bytes =
            metadata(&serde_json::json!({"type":document.diagram_type()})).map_err(failure)?;
        finish(
            admit_outbound(&mut store, type_bytes.len(), DEFAULT_BINARY_BUDGET),
            &errors,
        )?;
        Ok(CapturedSankeyDocument::new(document))
    }
    #[allow(clippy::too_many_arguments)]
    pub fn parse_with_host_storage(
        &self,
        source: &str,
        json: &str,
        source_kind: u32,
        source_value: f64,
        edge_kind: u32,
        edge_value: f64,
        make_er: Function,
        capture_er: Function,
        owner: JsValue,
    ) -> Result<Array, JsValue> {
        let engine = self
            .inner
            .as_ref()
            .ok_or_else(|| failure(invalid("Registry is closed")))?;
        let (options, limits) = super::parse_limits::decode_parse_options(
            source,
            json,
            source_kind,
            source_value,
            edge_kind,
            edge_value,
        )
        .map_err(failure)?;
        let errors = ErrorLane::default();
        let result = engine.parse_host_storage_with_limits(
            source,
            &options,
            limits,
            |schema| super::host_er::host_construction(schema, &make_er, &capture_er),
            |schema| InitialTransport::writer(&owner, prototype(schema)?, &errors),
        );
        match finish(result, &errors)? {
            pele::compat::host_sankey::CombinedParsed::Er { policy, output } => Ok(
                super::host_er::host_parsed(pele::compat::host_er::Parsed::Host { policy, output }),
            ),
            pele::compat::host_sankey::CombinedParsed::Sankey { document, storage } => {
                let value = parsed(Parsed::Captured { document, storage }, &errors)?;
                value.set(0, JsValue::from_f64(2.));
                Ok(value)
            }
            pele::compat::host_sankey::CombinedParsed::Arena(inner) => {
                parsed(Parsed::Arena(inner), &errors)
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn parse_prepared_with_captured_sankey(
        &self,
        id: &str,
        source: &str,
        json: &str,
        title: Option<String>,
        line_offset: f64,
        edge_kind: u32,
        edge_value: f64,
        owner: JsValue,
    ) -> Result<Array, JsValue> {
        let input = super::prepared_entries::prepared_input(
            self,
            id,
            source,
            json,
            title.as_deref(),
            line_offset,
            edge_kind,
            edge_value,
        )
        .map_err(failure)?;
        let errors = ErrorLane::default();
        let result = input.engine.parse_prepared_host_sankey_as(
            &input.id,
            pele::compat::ParseContext {
                source,
                config: &input.config,
                title: title.as_deref(),
                line_offset: input.line_offset,
                max_edges: pele::compat::DEFAULT_MAX_EDGES,
            },
            input.limit,
            |schema| InitialTransport::writer(&owner, prototype(schema)?, &errors),
        );
        parsed(finish(result, &errors)?, &errors)
    }
}
