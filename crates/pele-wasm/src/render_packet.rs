//! Successful render transport shared by source and retained host lanes.
//! The caller supplies its exact legacy envelope; this module owns no model.
use pele::compat::{PeleError, Rendered};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(raw_module = "../host-er-store.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = hostErRenderResult)]
    fn render_result_packet(
        raw: bool,
        diagram_type: &str,
        svg: &str,
        width_low: u32,
        width_high: u32,
        height_low: u32,
        height_high: u32,
        links: &str,
        metadata_bytes: u32,
    ) -> Result<JsValue, JsValue>;
}

pub(super) struct Prepared {
    metadata_bytes: usize,
    links: Vec<u8>,
}

#[cfg(feature = "captured-sankey")]
impl Prepared {
    pub(super) fn metadata_bytes(&self) -> usize {
        self.metadata_bytes
    }
}

pub(super) fn prepare(
    rendered: &Rendered,
    transport: &impl Serialize,
) -> Result<Prepared, PeleError> {
    // Reference serde traversal, with the same per-write 64 MiB gate, runs
    // before links encoding or any JS allocation. No SVG or LinkInfo clone.
    let metadata_bytes = super::render_numbers::encoded_length(transport)?;
    let links = super::render_numbers::encode(&rendered.links)?;
    Ok(Prepared {
        metadata_bytes,
        links,
    })
}

pub(super) fn emit(
    raw: bool,
    diagram_type: &str,
    rendered: &Rendered,
    prepared: Prepared,
) -> Result<JsValue, JsValue> {
    let links = std::str::from_utf8(&prepared.links).expect("JSON UTF8");
    let width = rendered.width.to_bits();
    let height = rendered.height.to_bits();
    render_result_packet(
        raw,
        diagram_type,
        &rendered.svg,
        width as u32,
        (width >> 32) as u32,
        height as u32,
        (height >> 32) as u32,
        links,
        prepared.metadata_bytes as u32,
    )
}

/// Full arena publication with a direct success result. Error outcomes retain
/// the exact legacy binary metadata format. No operation is replayed.
pub(super) fn arena(
    revision: u64,
    graph: &pele::compat::public::ModelGraph,
    raw: bool,
    diagram_type: &str,
    rendered: Option<&Rendered>,
    transport: &impl Serialize,
) -> Result<JsValue, JsValue> {
    use pele::compat::public::{encode_binary_exchange, DEFAULT_BINARY_BUDGET};
    let failure = |error: PeleError| JsValue::from_str(&super::render_numbers::failure(&error));
    let Some(rendered) = rendered else {
        let metadata = super::render_numbers::encode(transport).map_err(failure)?;
        let bytes = encode_binary_exchange(revision, graph, &metadata, DEFAULT_BINARY_BUDGET)
            .map_err(failure)?;
        return Ok(js_sys::Uint8Array::from(bytes.as_slice()).into());
    };
    let prepared = prepare(rendered, transport).map_err(failure)?;
    let metadata_bytes = prepared.metadata_bytes;
    // The existing metadata count prefix is always u32: changing its value to
    // zero leaves the frame header width unchanged. Subtracting the counted
    // legacy metadata admits exactly the old combined graph+metadata size.
    // encode_binary_exchange still validates the graph before any byte writes.
    let bytes =
        encode_binary_exchange(revision, graph, &[], DEFAULT_BINARY_BUDGET - metadata_bytes)
            .map_err(failure)?;
    let graph_bytes = js_sys::Uint8Array::from(bytes.as_slice());
    let packet = js_sys::Array::new();
    packet.push(&JsValue::from_f64(1.));
    packet.push(&JsValue::from_f64(metadata_bytes as f64));
    packet.push(&graph_bytes);
    match emit(raw, diagram_type, rendered, prepared) {
        Ok(value) => {
            packet.push(&JsValue::from_f64(0.));
            packet.push(&value);
        }
        Err(error) => {
            // The render committed and the complete graph was admitted. Carry
            // a factory throw without losing publication or its JS identity.
            packet.push(&JsValue::from_f64(1.));
            packet.push(&error);
        }
    }
    Ok(packet.into())
}
