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
