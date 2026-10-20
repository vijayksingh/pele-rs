//! Additive bounded browser numeric lane. Existing WASM exports remain unchanged.
use super::{BrowserResources, MutableDocument, ParsedDocument};
use pele::compat::{ErrorCode, OutputLimit, PeleError, RenderOptions, RenderResources};
use serde::{
    de::{
        self,
        value::{MapAccessDeserializer, StringDeserializer},
        DeserializeSeed, IgnoredAny, MapAccess, Visitor,
    },
    Deserialize, Serialize,
};
use std::io::Write;
use wasm_bindgen::prelude::*;
const JSON_BYTES: usize = 64 * 1024 * 1024;
const OPTIONS_BYTES: usize = 1024 * 1024;
fn invalid(message: &str) -> PeleError {
    PeleError::new(message, ErrorCode::Semantic)
}
fn budget() -> PeleError {
    PeleError::new(
        "Browser numeric transport exceeds byte budget",
        ErrorCode::Limit,
    )
}
fn selected(key: &str) -> u64 {
    match key {
        "fontSize" => 1,
        "maxWidth" => 2,
        "directionBreakpoint" => 4,
        "padding" => 8,
        "now" => 16,
        "outputLimit" => 1 << 5,
        "limit" => 1 << 6,
        "maxEdges" => 1 << 7,
        _ => 0,
    }
}
struct FilteredMap<A> {
    inner: A,
    mask: u64,
    seen: u64,
}
impl<'de, A: MapAccess<'de>> MapAccess<'de> for FilteredMap<A> {
    type Error = A::Error;
    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        loop {
            let Some(key) = self.inner.next_key::<String>()? else {
                return Ok(None);
            };
            let bit = selected(&key);
            if self.mask & bit != 0 {
                if self.seen & bit != 0 {
                    return Err(de::Error::custom(format!("duplicate field `{key}`")));
                }
                self.seen |= bit;
                self.inner.next_value::<IgnoredAny>()?;
            } else {
                return seed
                    .deserialize(StringDeserializer::<A::Error>::new(key))
                    .map(Some);
            }
        }
    }
    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        self.inner.next_value_seed(seed)
    }
}
struct FilterOptions {
    mask: u64,
}
impl<'de> DeserializeSeed<'de> for FilterOptions {
    type Value = RenderOptions;
    fn deserialize<D: de::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<RenderOptions, D::Error> {
        struct OptionsVisitor {
            mask: u64,
        }
        impl<'de> Visitor<'de> for OptionsVisitor {
            type Value = RenderOptions;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("render options object")
            }
            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<RenderOptions, A::Error> {
                RenderOptions::deserialize(MapAccessDeserializer::new(FilteredMap {
                    inner: map,
                    mask: self.mask,
                    seen: 0,
                }))
            }
        }
        deserializer.deserialize_map(OptionsVisitor { mask: self.mask })
    }
}
fn render_number_options(
    json: &str,
    present: u64,
    values: &[f64],
    output: Option<OutputLimit>,
    parsed: bool,
) -> Result<RenderOptions, PeleError> {
    if present == 0 && output.is_none() && !parsed {
        return pele::compat::render_options(json);
    }
    let mask = present
        | if output.is_some() { 1 << 5 } else { 0 }
        | if parsed { (1 << 6) | (1 << 7) } else { 0 };
    let mut options = if json.trim().is_empty() {
        RenderOptions::default()
    } else {
        let mut decoder = serde_json::Deserializer::from_str(json);
        let value = FilterOptions { mask }
            .deserialize(&mut decoder)
            .and_then(|value| decoder.end().map(|_| value));
        value.map_err(|error| {
            PeleError::new(
                format!("Invalid render options: {error}"),
                ErrorCode::Semantic,
            )
        })?
    };
    let fields = [
        &mut options.render.font_size,
        &mut options.render.max_width,
        &mut options.render.direction_breakpoint,
        &mut options.render.padding,
        &mut options.render.now,
    ];
    for (index, field) in fields.into_iter().enumerate() {
        if present & (1 << index) != 0 {
            *field = Some(values[index]);
        }
    }
    Ok(options)
}

fn validate_options(
    json: &str,
    mask: u32,
    numbers: &[f64],
    output_kind: u32,
    output: f64,
) -> Result<Option<OutputLimit>, PeleError> {
    if json.len() > OPTIONS_BYTES {
        return Err(budget());
    }
    if mask & !31 != 0 || numbers.len() != 5 {
        return Err(invalid("Invalid browser render number mask or values"));
    }
    let limit = match output_kind {
        0 => Some(OutputLimit::Undefined),
        1 => Some(OutputLimit::Null),
        2 => Some(OutputLimit::Number(output)),
        3 => None,
        _ => return Err(invalid("Invalid browser output-limit kind")),
    };
    Ok(limit)
}
pub(super) fn options(
    json: &str,
    mask: u32,
    numbers: &[f64],
    output_kind: u32,
    output: f64,
) -> Result<(RenderOptions, Option<OutputLimit>), PeleError> {
    let limit = validate_options(json, mask, numbers, output_kind, output)?;
    Ok((
        render_number_options(json, u64::from(mask), numbers, limit, false)?,
        limit,
    ))
}
/// Exact source/edge policies share the public parser and do not traverse JS
/// ModelBridge for source rendering. Parse fields were validated separately.
pub(super) fn options_with_parse_limits(
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
) -> Result<
    (
        RenderOptions,
        pele::compat::ParseLimits,
        Option<OutputLimit>,
    ),
    PeleError,
> {
    if json.len() > OPTIONS_BYTES || source.len() > pele::compat::MAX_PUBLIC_SOURCE_BYTES {
        return Err(budget());
    }
    let output_limit = validate_options(json, mask, numbers, output_kind, output)?;
    let (parse, limits) = super::parse_limits::decode_parse_options(
        source,
        json,
        source_kind,
        source_value,
        edge_kind,
        edge_value,
    )?;
    let mut options = render_number_options(json, u64::from(mask), numbers, output_limit, true)?;
    options.parse = parse;
    Ok((options, limits, output_limit))
}
struct Bounded(Vec<u8>);
impl Write for Bounded {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > JSON_BYTES - self.0.len() {
            return Err(std::io::Error::other("browser numeric output budget"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub(super) fn encode(value: &impl Serialize) -> Result<Vec<u8>, PeleError> {
    let mut output = Bounded(Vec::new());
    serde_json::to_writer(&mut output, value).map_err(|_| budget())?;
    Ok(output.0)
}
// Same serializer and write boundary as encode, without retaining output bytes.
struct Counted(usize);
impl Write for Counted {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > JSON_BYTES - self.0 {
            return Err(std::io::Error::other("browser numeric output budget"));
        }
        self.0 += bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub(super) fn encoded_length(value: &impl Serialize) -> Result<usize, PeleError> {
    let mut output = Counted(0);
    serde_json::to_writer(&mut output, value).map_err(|_| budget())?;
    Ok(output.0)
}
pub(super) fn failure(error: &PeleError) -> String {
    #[derive(Serialize)]
    struct Envelope<'a> {
        version: u8,
        ok: bool,
        error: &'a PeleError,
    }
    let encoded = encode(&Envelope {
        version: 2,
        ok: false,
        error,
    });
    match encoded {
        Ok(bytes) => String::from_utf8(bytes).expect("JSON UTF8"),
        Err(_) => r#"{"version":2,"ok":false,"error":{"name":"PeleError","message":"Browser numeric transport exceeds byte budget","code":"limit","type":null,"line":0,"column":0,"snippet":""}}"#.into(),
    }
}
pub(super) fn rendered(result: Result<pele::compat::RenderResult, PeleError>) -> String {
    #[derive(Serialize)]
    struct Envelope<T> {
        version: u8,
        ok: bool,
        result: T,
    }
    match result.and_then(|value| {
        encode(&Envelope {
            version: 2,
            ok: true,
            result: pele::compat::render_result_transport(&value),
        })
    }) {
        Ok(bytes) => String::from_utf8(bytes).expect("JSON UTF8"),
        Err(error) => failure(&error),
    }
}
pub(super) struct NumericResources {
    pub(super) browser: BrowserResources,
    pub(super) clock: Option<js_sys::Function>,
}
impl RenderResources for NumericResources {
    fn text_width(&mut self, request: pele::compat::TextRequest<'_>) -> Result<f64, PeleError> {
        self.browser.text_width(request)
    }
    fn icon(&mut self, name: &str) -> Result<Option<pele::compat::TrustedSvg>, PeleError> {
        self.browser.icon(name)
    }
    fn direct_icon(&mut self, name: &str) -> Result<Option<pele::compat::TrustedSvg>, PeleError> {
        self.browser.direct_icon(name)
    }
    fn parse_date(&mut self, input: &str) -> Result<Option<f64>, PeleError> {
        self.browser.parse_date(input)
    }
    fn calendar_fields(
        &mut self,
        instant: f64,
    ) -> Result<Option<pele::compat::CalendarFields>, PeleError> {
        self.browser.calendar_fields(instant)
    }
    fn calendar_instant(
        &mut self,
        civil: pele::compat::CivilDateTime,
    ) -> Result<Option<f64>, PeleError> {
        self.browser.calendar_instant(civil)
    }
    fn now_millis(&mut self) -> Result<f64, PeleError> {
        match &self.clock {
            Some(clock) => clock
                .call0(&JsValue::UNDEFINED)
                .map_err(super::callback_error)?
                .as_f64()
                .ok_or_else(|| invalid("Clock must return a number")),
            None => self.browser.now_millis(),
        }
    }
}
#[wasm_bindgen]
pub fn render_numbers_with_resources(
    source: &str,
    json: &str,
    mask: u32,
    numbers: &[f64],
    output_kind: u32,
    output: f64,
    width: Option<js_sys::Function>,
    icons: Option<js_sys::Function>,
    clock: Option<js_sys::Function>,
) -> String {
    rendered((|| {
        if json.len() > OPTIONS_BYTES || source.len() > pele::compat::MAX_PUBLIC_SOURCE_BYTES {
            return Err(budget());
        }
        let (options, limit) = options(json, mask, numbers, output_kind, output)?;
        pele::compat::Engine::with_builtins().render_source_bounded(
            source,
            &options,
            limit,
            &mut NumericResources {
                browser: BrowserResources { width, icons },
                clock,
            },
        )
    })())
}
#[wasm_bindgen]
impl ParsedDocument {
    pub fn render_numbers_with_resources(
        &self,
        json: &str,
        mask: u32,
        numbers: &[f64],
        output_kind: u32,
        output: f64,
        width: Option<js_sys::Function>,
        icons: Option<js_sys::Function>,
        clock: Option<js_sys::Function>,
    ) -> String {
        rendered((|| {
            let (options, limit) = options(json, mask, numbers, output_kind, output)?;
            pele::compat::Engine::empty().render_document_bounded(
                &self.inner,
                &options,
                limit,
                &mut NumericResources {
                    browser: BrowserResources { width, icons },
                    clock,
                },
            )
        })())
    }
}
#[wasm_bindgen]
impl MutableDocument {
    pub fn render_numbers_operation_binary(
        &mut self,
        revision: &str,
        json: &str,
        mask: u32,
        numbers: &[f64],
        output_kind: u32,
        output: f64,
        width: Option<js_sys::Function>,
        icons: Option<js_sys::Function>,
        clock: Option<js_sys::Function>,
    ) -> Result<Vec<u8>, JsValue> {
        let run = || -> Result<Vec<u8>, PeleError> {
            let (options, limit) = options(json, mask, numbers, output_kind, output)?;
            let revision = revision
                .parse::<u64>()
                .map_err(|_| invalid("Invalid public model revision"))?;
            let outcome = self.inner.render_operation_bounded(
                revision,
                &options,
                limit,
                &mut NumericResources {
                    browser: BrowserResources { width, icons },
                    clock,
                },
            )?;
            let metadata = encode(
                &outcome
                    .result
                    .as_ref()
                    .map(pele::compat::render_result_transport),
            )?;
            pele::compat::public::encode_binary_exchange(
                outcome.revision,
                &outcome.graph,
                &metadata,
                pele::compat::public::DEFAULT_BINARY_BUDGET,
            )
        };
        run().map_err(|error| JsValue::from_str(&failure(&error)))
    }
}
