//! Experimental native-owned canonical storage. No storage-query JS imports.
use super::wasm_registry::NativeRegistry;
use js_sys::{Array, Function};
use pele::compat::{
    host_sankey::{
        self, AdmissionKind, AdmissionRead, Document, FlatEpoch, FlatStringHistory, FlatWorking,
        NativeObservation, NativeSourceWriter, Parsed,
    },
    public::{HostSankeyEntrySelection, DEFAULT_BINARY_BUDGET},
    DiagramType, ErrorCode, PeleError, Rendered,
};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};
use wasm_bindgen::prelude::*;

fn invalid(message: &str) -> PeleError {
    PeleError::new(message, ErrorCode::Semantic)
}
fn protocol() -> PeleError {
    invalid("Invalid native canonical storage protocol")
}
fn failure(error: PeleError) -> JsValue {
    JsValue::from_str(&super::render_numbers::failure(&error))
}
fn revision(value: &str) -> Result<u64, PeleError> {
    value
        .parse()
        .map_err(|_| invalid("Invalid public model revision"))
}
fn word(value: usize) -> Result<u32, PeleError> {
    u32::try_from(value).map_err(|_| protocol())
}
fn pointer<T>(value: &[T]) -> Result<u32, PeleError> {
    if value.is_empty() {
        return Ok(0);
    }
    let ptr = value.as_ptr() as usize;
    let bytes = value
        .len()
        .checked_mul(std::mem::size_of::<T>())
        .ok_or_else(protocol)?;
    let end = ptr.checked_add(bytes).ok_or_else(protocol)?;
    if end > u32::MAX as usize {
        return Err(protocol());
    }
    word(ptr)
}
#[wasm_bindgen]
pub fn native_canonical_memory() -> JsValue {
    wasm_bindgen::memory()
}

/// Directory memory is owned by the lease, never borrowed from a temporary Vec.
struct Directory {
    top: Box<[u32; 256]>,
    _seconds: Vec<Box<[u32; 512]>>,
}
impl Directory {
    fn collect<T>(
        width: usize,
        visit: impl FnOnce(&mut dyn FnMut(usize, &[T])),
    ) -> Result<Self, PeleError> {
        let mut pages: BTreeMap<usize, Box<[u32; 512]>> = BTreeMap::new();
        let mut error = None;
        visit(&mut |index, data| {
            if error.is_some() {
                return;
            }
            let run = || -> Result<(usize, usize, u32, u32), PeleError> {
                if index >= 256 * 256
                    || data.is_empty()
                    || data.len() > 1024 * width
                    || data.len() % width != 0
                {
                    return Err(protocol());
                }
                Ok((index / 256, index % 256, pointer(data)?, word(data.len())?))
            };
            match run() {
                Ok((upper, lower, ptr, len)) => {
                    let page = pages.entry(upper).or_insert_with(|| Box::new([0; 512]));
                    if page[lower * 2] != 0 {
                        error = Some(protocol());
                        return;
                    }
                    page[lower * 2] = ptr;
                    page[lower * 2 + 1] = len;
                }
                Err(value) => error = Some(value),
            }
        });
        if let Some(error) = error {
            return Err(error);
        }
        let mut top = Box::new([0; 256]);
        let mut seconds = Vec::with_capacity(pages.len());
        for (index, page) in pages {
            top[index] = pointer(&page[..])?;
            seconds.push(page);
        }
        Ok(Self {
            top,
            _seconds: seconds,
        })
    }
    fn address(&self) -> Result<u32, PeleError> {
        pointer(&self.top[..])
    }
}
struct EpochValue {
    serial: u64,
    graph: FlatEpoch,
}
struct Lease {
    value: Rc<EpochValue>,
    directories: [Directory; 5],
    arguments: Option<Box<[[u32; 4]]>>,
    argument_length: u32,
}
#[wasm_bindgen]
pub struct NativeCanonicalEpoch {
    origin: Rc<()>,
    lease: RefCell<Option<Lease>>,
}
impl NativeCanonicalEpoch {
    fn new(
        origin: Rc<()>,
        value: Rc<EpochValue>,
        argument_length: u32,
        arguments: Vec<(u32, [u32; 3])>,
    ) -> Result<Self, PeleError> {
        let graph = &value.graph;
        let directories = [
            Directory::collect(7, |visit| graph.visit_header_pages(visit))?,
            Directory::collect(5, |visit| graph.visit_property_pages(visit))?,
            Directory::collect(3, |visit| graph.visit_item_pages(visit))?,
            Directory::collect(2, |visit| graph.visit_string_pages(visit))?,
            Directory::collect(1, |visit| graph.visit_unit_pages(visit))?,
        ];
        let mut rows = Vec::with_capacity(arguments.len());
        let mut previous = None;
        for (index, atom) in arguments {
            if index >= argument_length || previous.is_some_and(|old| old >= index) {
                return Err(protocol());
            }
            previous = Some(index);
            rows.push([index, atom[0], atom[1], atom[2]]);
        }
        Ok(Self {
            origin,
            lease: RefCell::new(Some(Lease {
                value,
                directories,
                arguments: Some(rows.into_boxed_slice()),
                argument_length,
            })),
        })
    }
    fn value(&self) -> Result<Rc<EpochValue>, PeleError> {
        self.lease
            .borrow()
            .as_ref()
            .map(|lease| lease.value.clone())
            .ok_or_else(protocol)
    }
}
#[wasm_bindgen]
impl NativeCanonicalEpoch {
    pub fn descriptor(&self) -> Result<Vec<u32>, JsValue> {
        let run = || -> Result<Vec<u32>, PeleError> {
            let lease = self.lease.borrow();
            let lease = lease.as_ref().ok_or_else(protocol)?;
            let root = lease.value.graph.root_words();
            let (nodes, strings) = lease.value.graph.logical_counts();
            let arguments = lease.arguments.as_deref().unwrap_or(&[]);
            Ok(vec![
                1,
                1024,
                lease.value.serial as u32,
                (lease.value.serial >> 32) as u32,
                root[0],
                root[1],
                root[2],
                word(nodes)?,
                word(strings)?,
                lease.directories[0].address()?,
                lease.directories[1].address()?,
                lease.directories[2].address()?,
                lease.directories[3].address()?,
                lease.directories[4].address()?,
                pointer(arguments)?,
                lease.argument_length,
                word(arguments.len())?,
            ])
        };
        run().map_err(failure)
    }
    pub fn exact_equals(&self, other: &NativeCanonicalEpoch) -> Result<bool, JsValue> {
        let a = self.value().map_err(failure)?;
        let b = other.value().map_err(failure)?;
        Ok(Rc::ptr_eq(&a, &b) || a.graph.exact_eq(&b.graph))
    }
    pub fn release_arguments(&self) -> Result<(), JsValue> {
        let mut lease = self.lease.borrow_mut();
        let lease = lease.as_mut().ok_or_else(|| failure(protocol()))?;
        lease.arguments = None;
        lease.argument_length = 0;
        Ok(())
    }
    pub fn close(&self) {
        self.lease.borrow_mut().take();
    }
    pub fn preflight_encoding(&self, metadata_bytes: u32) -> Result<Vec<u32>, JsValue> {
        let value = self.value().map_err(failure)?;
        let (nodes, _) = value.graph.logical_counts();
        let mut missing = false;
        value.graph.visit_header_pages(|page, words| {
            for (i, h) in words.chunks_exact(7).enumerate() {
                if page * 1024 + i < nodes && h[0] == u32::MAX {
                    missing = true;
                }
            }
        });
        if missing {
            return Ok(vec![1, 0]);
        }
        let mut graph = value.graph.clone();
        let count = match host_sankey::canonical_frame_bytes(
            &mut graph,
            metadata_bytes as usize,
            DEFAULT_BINARY_BUDGET,
        ) {
            Ok(count) => count,
            Err(error) if error.code == ErrorCode::Limit => return Ok(vec![3, 0]),
            Err(error) => return Err(failure(error)),
        };
        for id in 0..nodes {
            let header = AdmissionRead::header(&mut graph, id).map_err(failure)?;
            if matches!(
                header.kind,
                AdmissionKind::InvalidArray | AdmissionKind::InvalidError
            ) {
                return Ok(vec![2, 0]);
            }
        }
        Ok(vec![0, word(count).map_err(failure)?])
    }
}

struct State {
    document: Option<Document>,
    accepted: Rc<EpochValue>,
    observed: Rc<EpochValue>,
    history: FlatStringHistory,
    draft: Option<(HostSankeyEntrySelection, Rc<()>, Rc<()>)>,
}
struct Shared {
    state: RefCell<State>,
    origin: Rc<()>,
    serial: Cell<u64>,
    busy: Cell<bool>,
    observing: Cell<bool>,
    disposed: Cell<bool>,
    revision: Cell<u64>,
}
impl Shared {
    fn next(&self, graph: FlatEpoch) -> Result<Rc<EpochValue>, PeleError> {
        let serial = self.serial.get().checked_add(1).ok_or_else(protocol)?;
        self.serial.set(serial);
        Ok(Rc::new(EpochValue { serial, graph }))
    }
    fn check(&self) -> Result<(), JsValue> {
        if self.disposed.get() {
            return Err(js_sys::TypeError::new("Public model document is disposed").into());
        }
        if self.busy.get() || self.observing.get() {
            return Err(js_sys::TypeError::new("Public model document is already in use").into());
        }
        Ok(())
    }
    fn epoch(&self, value: Rc<EpochValue>) -> Result<NativeCanonicalEpoch, JsValue> {
        NativeCanonicalEpoch::new(self.origin.clone(), value, 0, Vec::new()).map_err(failure)
    }
    fn check_epoch(&self, epoch: &NativeCanonicalEpoch) -> Result<Rc<EpochValue>, JsValue> {
        if !Rc::ptr_eq(&self.origin, &epoch.origin) {
            return Err(failure(protocol()));
        }
        epoch.value().map_err(failure)
    }
}
#[wasm_bindgen]
pub struct NativeCanonicalObservation {
    owner: Rc<Shared>,
    inner: RefCell<Option<NativeObservation>>,
}
impl Drop for NativeCanonicalObservation {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.get_mut().take() {
            self.owner.state.borrow_mut().history = inner.abort();
            self.owner.observing.set(false);
        }
    }
}
#[wasm_bindgen]
impl NativeCanonicalObservation {
    pub fn history_count(&self) -> Result<u32, JsValue> {
        let inner = self.inner.borrow();
        word(
            inner
                .as_ref()
                .ok_or_else(|| failure(protocol()))?
                .history_count(),
        )
        .map_err(failure)
    }
    pub fn append(&self, bytes: &[u8]) -> Result<(), JsValue> {
        self.inner
            .borrow_mut()
            .as_mut()
            .ok_or_else(|| failure(protocol()))?
            .append_batch(bytes)
            .map_err(failure)
    }
    pub fn abort(&self) {
        if let Some(inner) = self.inner.borrow_mut().take() {
            self.owner.state.borrow_mut().history = inner.abort();
            self.owner.observing.set(false);
        }
    }
    pub fn finish(&self) -> Result<NativeCanonicalEpoch, JsValue> {
        let inner = self
            .inner
            .borrow_mut()
            .take()
            .ok_or_else(|| failure(protocol()))?;
        let history = inner.history();
        let result = inner.finish_stream();
        self.owner.observing.set(false);
        self.owner.state.borrow_mut().history = history;
        let captured = result.map_err(failure)?;
        let value = self.owner.next(captured.epoch).map_err(failure)?;
        let lease = NativeCanonicalEpoch::new(
            self.owner.origin.clone(),
            value.clone(),
            captured.argument_length,
            captured.arguments,
        )
        .map_err(failure)?;
        {
            let mut state = self.owner.state.borrow_mut();
            state.observed = value.clone();
            state.history = value.graph.string_history();
        }
        Ok(lease)
    }
}

/// The local operation owns Document and storage. No owner borrow spans resources.
struct Active {
    owner: Rc<Shared>,
    document: Option<Document>,
    accepted: Rc<EpochValue>,
}
impl Active {
    fn document(&mut self) -> Result<&mut Document, JsValue> {
        self.document.as_mut().ok_or_else(|| failure(protocol()))
    }
    fn commit_graph(&mut self, before: u64, graph: FlatEpoch) -> Result<(), JsValue> {
        if self.document()?.revision() != before {
            self.accepted = self.owner.next(graph).map_err(failure)?;
            self.owner.state.borrow_mut().accepted = self.accepted.clone();
        }
        Ok(())
    }
    fn commit_working(&mut self, before: u64, working: FlatWorking) -> Result<(), JsValue> {
        self.commit_graph(before, working.seal())
    }
}
impl Drop for Active {
    fn drop(&mut self) {
        if let Some(document) = self.document.take() {
            self.owner.revision.set(document.revision());
            if !self.owner.disposed.get() {
                self.owner.state.borrow_mut().document = Some(document);
            }
        }
        self.owner.busy.set(false);
    }
}
#[wasm_bindgen]
pub struct NativeCanonicalSankeyDocument {
    shared: Rc<Shared>,
}
impl Drop for NativeCanonicalSankeyDocument {
    fn drop(&mut self) {
        self.shared.disposed.set(true);
    }
}
impl NativeCanonicalSankeyDocument {
    fn new(
        document: Option<Document>,
        graph: FlatEpoch,
        draft: Option<(HostSankeyEntrySelection, Rc<()>, Rc<()>)>,
    ) -> Self {
        let revision = document.as_ref().map_or(0, |d| d.revision());
        let history = graph.string_history();
        let value = Rc::new(EpochValue { serial: 0, graph });
        Self {
            shared: Rc::new(Shared {
                state: RefCell::new(State {
                    document,
                    accepted: value.clone(),
                    observed: value,
                    history,
                    draft,
                }),
                origin: Rc::new(()),
                serial: Cell::new(0),
                busy: Cell::new(false),
                observing: Cell::new(false),
                disposed: Cell::new(false),
                revision: Cell::new(revision),
            }),
        }
    }
    fn take(&self) -> Result<Active, JsValue> {
        self.shared.check()?;
        let mut state = self.shared.state.borrow_mut();
        let document = state
            .document
            .take()
            .ok_or_else(|| failure(invalid("Captured document is unavailable")))?;
        self.shared.busy.set(true);
        Ok(Active {
            owner: self.shared.clone(),
            document: Some(document),
            accepted: state.accepted.clone(),
        })
    }
    fn schema_view(&self) -> Result<pele::compat::public::ModelSchemaView, JsValue> {
        self.shared.check()?;
        let state = self.shared.state.borrow();
        if let Some(document) = &state.document {
            return Ok(document.schema());
        }
        state
            .draft
            .as_ref()
            .map(|(selection, _, _)| selection.schema())
            .ok_or_else(|| failure(protocol()))
    }
}
#[wasm_bindgen]
impl NativeCanonicalSankeyDocument {
    pub fn revision(&self) -> String {
        self.shared.revision.get().to_string()
    }
    pub fn entry_type(&self) -> String {
        "sankey".into()
    }
    pub fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    pub fn schema(&self) -> Result<String, JsValue> {
        Ok(super::public_result(&self.schema_view()?))
    }
    pub fn schema_cache_key(&self) -> Result<String, JsValue> {
        let view = self.schema_view()?;
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
        let json = {
            let mut active = self.take()?;
            super::foreign_provider::config_json(active.document()?.config()).map_err(failure)?
        };
        let config = js_sys::JSON::parse(&json)
            .map_err(super::callback_error)
            .map_err(failure)?;
        let out = Array::new();
        out.push(&JsValue::from_str("sankey"));
        out.push(&config);
        out.push(&super::foreign_provider::retained_policy(DiagramType::Sankey).into());
        Ok(out)
    }
    pub fn initial_epoch(&self) -> Result<NativeCanonicalEpoch, JsValue> {
        self.shared.check()?;
        let value = self.shared.state.borrow().observed.clone();
        self.shared.epoch(value)
    }
    pub fn begin_observation(
        &self,
        epoch: &NativeCanonicalEpoch,
    ) -> Result<NativeCanonicalObservation, JsValue> {
        self.shared.check()?;
        let value = self.shared.check_epoch(epoch)?;
        let state = self.shared.state.borrow();
        if !Rc::ptr_eq(&value, &state.observed) {
            return Err(failure(protocol()));
        }
        let inner = NativeObservation::new(&value.graph, &state.history);
        self.shared.observing.set(true);
        Ok(NativeCanonicalObservation {
            owner: self.shared.clone(),
            inner: RefCell::new(Some(inner)),
        })
    }
    pub fn publication_started(&self, epoch: &NativeCanonicalEpoch) -> Result<u32, JsValue> {
        self.shared.check()?;
        let value = self.shared.check_epoch(epoch)?;
        let mut state = self.shared.state.borrow_mut();
        let history = value.graph.string_history();
        let prefix = word(state.history.common_prefix_len(&history)).map_err(failure)?;
        state.observed = value;
        state.history = history;
        Ok(prefix)
    }
    pub fn apply_native(
        &self,
        epoch: &NativeCanonicalEpoch,
        expected: &str,
    ) -> Result<String, JsValue> {
        let mut active = self.take()?;
        let value = self.shared.check_epoch(epoch)?;
        let mut graph = value.graph.clone();
        host_sankey::admit_observed(&mut graph, 0, DEFAULT_BINARY_BUDGET).map_err(failure)?;
        let expected = revision(expected).map_err(failure)?;
        let before = active.document()?.revision();
        let result = active.document()?.apply(expected, &mut graph);
        let committed = active.commit_graph(before, graph);
        result.map_err(failure)?;
        committed?;
        Ok(super::public_result(
            &active.document()?.revision().to_string(),
        ))
    }
    pub fn invoke_native(&self, expected: &str, call_json: &str) -> Result<JsValue, JsValue> {
        let mut active = self.take()?;
        if call_json.len() > 64 * 1024 * 1024 {
            return Err(failure(invalid(
                "Public method JSON exceeds 64 MiB transport budget",
            )));
        }
        let expected = revision(expected).map_err(failure)?;
        let call = serde_json::from_str(call_json)
            .map_err(|_| failure(invalid("Invalid public model method call")))?;
        let mut working = active.accepted.graph.fork();
        let before = active.document()?.revision();
        let result = active.document()?.invoke(expected, &mut working, call);
        let committed = active.commit_working(before, working);
        // Preserve partial accepted state even when post-dispatch validation
        // fails; run bounded maintenance without replacing the original error.
        let maintained = if committed.is_ok() {
            compact_accepted(&mut active)
        } else {
            Ok(())
        };
        let outcome = result.map_err(failure)?;
        committed?;
        maintained?;
        let bytes = super::render_numbers::encode(&outcome.result).map_err(failure)?;
        let length = bytes.len();
        admit_result(&active, length)?;
        packet(&active, outcome.revision, length, 0, metadata_text(bytes)?)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn render_native(
        &self,
        expected: &str,
        json: &str,
        mask: u32,
        numbers: &[f64],
        output_kind: u32,
        output: f64,
        width: Option<Function>,
        icons: Option<Function>,
        clock: Option<Function>,
    ) -> Result<JsValue, JsValue> {
        let mut active = self.take()?;
        let (options, limit) =
            super::render_numbers::options(json, mask, numbers, output_kind, output)
                .map_err(failure)?;
        let expected = revision(expected).map_err(failure)?;
        let mut graph = active.accepted.graph.clone();
        let before = active.document()?.revision();
        let mut resources = super::render_numbers::NumericResources {
            browser: super::BrowserResources { width, icons },
            clock,
        };
        let result =
            active
                .document()?
                .render(expected, &mut graph, &options, &mut resources, limit, true);
        let committed = active.commit_graph(before, graph);
        let outcome = result.map_err(failure)?;
        committed?;
        let transport = outcome
            .result
            .as_ref()
            .map(pele::compat::render_result_transport);
        render_packet(
            &active,
            outcome.revision,
            false,
            outcome
                .result
                .as_ref()
                .map_or("", |v| v.diagram_type.as_str()),
            outcome.result.as_ref().ok().map(|v| &v.rendered),
            &transport,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub fn render_raw_native(
        &self,
        expected: &str,
        config_json: &str,
        json: &str,
        mask: u32,
        numbers: &[f64],
        width: Option<Function>,
        icons: Option<Function>,
        clock: Option<Function>,
        inline_icons: Option<Function>,
    ) -> Result<JsValue, JsValue> {
        let mut active = self.take()?;
        let config = super::prepared_entries::config(config_json).map_err(failure)?;
        let (options, _) =
            super::render_numbers::options(json, mask, numbers, 3, 0.).map_err(failure)?;
        let expected = revision(expected).map_err(failure)?;
        let mut graph = active.accepted.graph.clone();
        let before = active.document()?.revision();
        let mut resources =
            super::prepared_entries::host_er_raw_resources(width, icons, clock, inline_icons);
        let result = active.document()?.render_raw(
            expected,
            &mut graph,
            &config,
            &options.render,
            &mut resources,
        );
        let committed = active.commit_graph(before, graph);
        let outcome = result.map_err(failure)?;
        committed?;
        let transport = outcome
            .result
            .as_ref()
            .map(super::prepared_entries::transport);
        render_packet(
            &active,
            outcome.revision,
            true,
            "",
            outcome.result.as_ref().ok(),
            &transport,
        )
    }
}
fn compact_accepted(active: &mut Active) -> Result<(), JsValue> {
    // Sealed N is already committed. Maintenance failure cannot roll it back.
    let mut graph = active.accepted.graph.clone();
    graph.compact().map_err(failure)?;
    active.accepted = active.owner.next(graph).map_err(failure)?;
    active.owner.state.borrow_mut().accepted = active.accepted.clone();
    Ok(())
}
fn admit_result(active: &Active, bytes: usize) -> Result<(), JsValue> {
    let mut graph = active.accepted.graph.clone();
    host_sankey::admit_outbound(&mut graph, bytes, DEFAULT_BINARY_BUDGET).map_err(failure)
}
fn metadata_text(bytes: Vec<u8>) -> Result<JsValue, JsValue> {
    String::from_utf8(bytes)
        .map(|v| JsValue::from_str(&v))
        .map_err(|_| failure(protocol()))
}
fn packet(
    active: &Active,
    revision: u64,
    bytes: usize,
    status: u32,
    payload: JsValue,
) -> Result<JsValue, JsValue> {
    let lease = active.owner.epoch(active.accepted.clone())?;
    let out = Array::new();
    out.push(&JsValue::from_f64(1.));
    out.push(&JsValue::from_str(&revision.to_string()));
    out.push(&JsValue::from_f64(bytes as f64));
    out.push(&JsValue::from_f64(status as f64));
    out.push(&payload);
    out.push(&lease.into());
    Ok(out.into())
}
fn render_packet(
    active: &Active,
    revision: u64,
    raw: bool,
    diagram_type: &str,
    rendered: Option<&Rendered>,
    transport: &impl serde::Serialize,
) -> Result<JsValue, JsValue> {
    let Some(rendered) = rendered else {
        let bytes = super::render_numbers::encode(transport).map_err(failure)?;
        let length = bytes.len();
        admit_result(active, length)?;
        return packet(active, revision, length, 0, metadata_text(bytes)?);
    };
    let prepared = super::render_packet::prepare(rendered, transport).map_err(failure)?;
    let length = prepared.metadata_bytes();
    admit_result(active, length)?;
    match super::render_packet::emit(raw, diagram_type, rendered, prepared) {
        Ok(value) => packet(active, revision, length, 0, value),
        Err(error) => packet(active, revision, length, 1, error),
    }
}
fn initial_admission(document: &Document, graph: &mut FlatEpoch) -> Result<(), JsValue> {
    document.snapshot_admission(graph).map_err(failure)?;
    let bytes = serde_json::to_vec(&serde_json::json!({"type":document.diagram_type()}))
        .map_err(|_| failure(protocol()))?;
    host_sankey::admit_outbound(graph, bytes.len(), DEFAULT_BINARY_BUDGET).map_err(failure)
}
fn parsed(parsed: Parsed<FlatEpoch>) -> Result<Array, JsValue> {
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
            initial_admission(&document, &mut storage)?;
            let handle = NativeCanonicalSankeyDocument::new(Some(document), storage, None);
            let epoch = handle.initial_epoch()?;
            out.push(&JsValue::from_f64(1.));
            out.push(&handle.into());
            out.push(&epoch.into());
        }
    }
    Ok(out)
}
#[wasm_bindgen]
pub struct NativeCanonicalSankeyEntrySelection {
    inner: HostSankeyEntrySelection,
    origin: Rc<()>,
    identity: Rc<()>,
}
#[wasm_bindgen]
impl NativeCanonicalSankeyEntrySelection {
    pub fn schema(&self) -> String {
        super::public_result(&self.inner.schema())
    }
}
#[wasm_bindgen]
impl NativeRegistry {
    pub fn select_prepared_native_sankey(
        &self,
        id: &str,
    ) -> Result<Option<NativeCanonicalSankeyEntrySelection>, JsValue> {
        let engine = self
            .inner
            .as_ref()
            .ok_or_else(|| failure(invalid("Registry is closed")))?;
        let id = super::prepared_entries::builtin_id(id).map_err(failure)?;
        Ok(engine
            .select_host_sankey_entry(&id)
            .map_err(failure)?
            .map(|inner| NativeCanonicalSankeyEntrySelection {
                inner,
                origin: self.raw_entry_origin.get_or_init(|| Rc::new(())).clone(),
                identity: Rc::new(()),
            }))
    }
    pub fn create_prepared_native_sankey(
        &self,
        selection: &NativeCanonicalSankeyEntrySelection,
    ) -> Result<NativeCanonicalSankeyDocument, JsValue> {
        self.check_native_selection(selection)?;
        Ok(NativeCanonicalSankeyDocument::new(
            None,
            FlatEpoch::empty(),
            Some((
                selection.inner.clone(),
                selection.origin.clone(),
                selection.identity.clone(),
            )),
        ))
    }
    pub fn bind_prepared_native_sankey(
        &self,
        selection: &NativeCanonicalSankeyEntrySelection,
        draft: &NativeCanonicalSankeyDocument,
        config_json: &str,
    ) -> Result<NativeCanonicalEpoch, JsValue> {
        self.check_native_selection(selection)?;
        draft.shared.check()?;
        let value = {
            let state = draft.shared.state.borrow();
            let (_, origin, identity) = state.draft.as_ref().ok_or_else(|| failure(protocol()))?;
            if state.document.is_some()
                || !Rc::ptr_eq(origin, &selection.origin)
                || !Rc::ptr_eq(identity, &selection.identity)
            {
                return Err(failure(protocol()));
            }
            state.observed.clone()
        };
        let config = super::prepared_entries::config(config_json).map_err(failure)?;
        let mut graph = value.graph.clone();
        host_sankey::admit_observed(&mut graph, 0, DEFAULT_BINARY_BUDGET).map_err(failure)?;
        let document = selection.inner.bind(config, &mut graph).map_err(failure)?;
        initial_admission(&document, &mut graph)?;
        let revision = document.revision();
        let accepted = draft.shared.next(graph).map_err(failure)?;
        let lease = draft.shared.epoch(accepted.clone())?;
        {
            let mut state = draft.shared.state.borrow_mut();
            state.document = Some(document);
            state.accepted = accepted;
            state.draft = None;
        }
        draft.shared.revision.set(revision);
        Ok(lease)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn parse_with_native_storage(
        &self,
        source: &str,
        json: &str,
        source_kind: u32,
        source_value: f64,
        edge_kind: u32,
        edge_value: f64,
        make_er: Function,
        capture_er: Function,
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
        let result = engine
            .parse_host_storage_with_limits(
                source,
                &options,
                limits,
                |schema| super::host_er::host_construction(schema, &make_er, &capture_er),
                |_| Ok(NativeSourceWriter::new()),
            )
            .map_err(failure)?;
        match result {
            host_sankey::CombinedParsed::Er { policy, output } => Ok(super::host_er::host_parsed(
                pele::compat::host_er::Parsed::Host { policy, output },
            )),
            host_sankey::CombinedParsed::Sankey { document, storage } => {
                let out = parsed(Parsed::Captured { document, storage })?;
                out.set(0, JsValue::from_f64(2.));
                Ok(out)
            }
            host_sankey::CombinedParsed::Arena(inner) => parsed(Parsed::Arena(inner)),
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn parse_prepared_with_native_sankey(
        &self,
        id: &str,
        source: &str,
        json: &str,
        title: Option<String>,
        line_offset: f64,
        edge_kind: u32,
        edge_value: f64,
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
        parsed(
            input
                .engine
                .parse_prepared_host_sankey_as(
                    &input.id,
                    pele::compat::ParseContext {
                        source,
                        config: &input.config,
                        title: title.as_deref(),
                        line_offset: input.line_offset,
                        max_edges: pele::compat::DEFAULT_MAX_EDGES,
                    },
                    input.limit,
                    |_| Ok(NativeSourceWriter::new()),
                )
                .map_err(failure)?,
        )
    }
}
impl NativeRegistry {
    fn check_native_selection(
        &self,
        selection: &NativeCanonicalSankeyEntrySelection,
    ) -> Result<(), JsValue> {
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
        Ok(())
    }
}
