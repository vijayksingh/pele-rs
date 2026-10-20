//! Owned source-layout transport over the ordinary mutable document lifecycle.
//! Canonical frame admission precedes compact encoding and all JS publication.
use pele::compat::{
    public::{GraphReadView, PublicDocument, DEFAULT_BINARY_BUDGET},
    ErrorCode, PeleError, Rendered,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

fn failure(error: PeleError) -> JsValue {
    JsValue::from_str(&super::render_numbers::failure(&error))
}

struct GraphPacket {
    canonical_bytes: usize,
    bytes: Vec<u8>,
}

fn prepare_graph(
    revision: u64,
    graph: &GraphReadView<'_>,
    metadata_bytes: usize,
) -> Result<GraphPacket, PeleError> {
    let canonical_bytes = graph.canonical_binary_size(metadata_bytes, DEFAULT_BINARY_BUDGET)?;
    let bytes = match graph.source_layout(revision)? {
        Some(bytes) => bytes,
        None => {
            // A compact recipe can outgrow the canonical frame after arbitrary
            // edits. Stream the canonical frame instead; the host keeps its
            // existing layout-backed baseline and derives sparse replacements.
            let budget = DEFAULT_BINARY_BUDGET
                .checked_sub(metadata_bytes)
                .ok_or_else(|| {
                    PeleError::new(
                        "Public binary graph exceeds transport budget",
                        ErrorCode::Limit,
                    )
                })?;
            graph.encode_binary(revision, &[], budget)?
        }
    };
    Ok(GraphPacket {
        canonical_bytes,
        bytes,
    })
}

fn prefix(graph: GraphPacket) -> js_sys::Array {
    let packet = js_sys::Array::new();
    packet.push(&JsValue::from_f64(1.));
    packet.push(&JsValue::from_f64(graph.canonical_bytes as f64));
    packet.push(&js_sys::Uint8Array::from(graph.bytes.as_slice()));
    packet
}

/// Probe a real native capability. Legacy providers still take their original
/// snapshot route, and no source parse or mutable operation is replayed.
pub(super) fn snapshot(document: &PublicDocument) -> Result<JsValue, JsValue> {
    if !document.has_source_layout() {
        return Ok(JsValue::NULL);
    }
    let graph = document.snapshot_read().map_err(failure)?;
    let metadata = serde_json::to_vec(&serde_json::json!({"type":document.diagram_type()}))
        .expect("snapshot metadata is JSON");
    let canonical_bytes = graph
        .canonical_binary_size(metadata.len(), DEFAULT_BINARY_BUDGET)
        .map_err(failure)?;
    let Some(bytes) = graph.source_layout(document.revision()).map_err(failure)? else {
        return Ok(JsValue::NULL);
    };
    let packet = prefix(GraphPacket {
        canonical_bytes,
        bytes,
    });
    packet.push(&JsValue::from_str(
        std::str::from_utf8(&metadata).expect("JSON is UTF-8"),
    ));
    Ok(packet.into())
}

pub(super) fn outcome(
    revision: u64,
    graph: &GraphReadView<'_>,
    metadata: &[u8],
) -> Result<JsValue, PeleError> {
    let graph = prepare_graph(revision, graph, metadata.len())?;
    let packet = prefix(graph);
    packet.push(&JsValue::from_str(
        std::str::from_utf8(metadata).expect("JSON is UTF-8"),
    ));
    Ok(packet.into())
}

pub(super) fn render(
    revision: u64,
    graph: &GraphReadView<'_>,
    raw: bool,
    diagram_type: &str,
    rendered: Option<&Rendered>,
    transport: &impl Serialize,
) -> Result<JsValue, JsValue> {
    let Some(rendered) = rendered else {
        let metadata = super::render_numbers::encode(transport).map_err(failure)?;
        return outcome(revision, graph, &metadata).map_err(failure);
    };
    let prepared = super::render_packet::prepare(rendered, transport).map_err(failure)?;
    let metadata_bytes = prepared.metadata_bytes();
    let graph = prepare_graph(revision, graph, metadata_bytes).map_err(failure)?;
    let packet = prefix(graph);
    packet.push(&JsValue::from_f64(metadata_bytes as f64));
    match super::render_packet::emit(raw, diagram_type, rendered, prepared) {
        Ok(value) => {
            packet.push(&JsValue::from_f64(0.));
            packet.push(&value);
        }
        Err(error) => {
            // Accepted state must publish before the original factory throw.
            packet.push(&JsValue::from_f64(1.));
            packet.push(&error);
        }
    }
    Ok(packet.into())
}
