//! Job-owned Venn reads; layout and SVG remain in the common renderer.
use super::{
    model::{VennStyle, VennSubset, VennText},
    render::PhaseInput,
};
use crate::compat::{
    public::{JsString, LiveRenderRead, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
use std::collections::{HashMap, HashSet};
pub(crate) struct VennDetachedInput {
    limit: Option<PeleError>,
}
impl VennDetachedInput {
    pub(super) fn new(error: Option<PeleError>) -> Self {
        Self {
            limit: error.filter(|e| e.code == ErrorCode::Limit),
        }
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    seed: &VennDetachedInput,
    config: &Object,
    settings: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    if let Some(error) = &seed.limit {
        return Err(error.clone().into());
    }
    super::render::render_phases(
        &mut HostPhase {
            read,
            text_slots: HashMap::new(),
            text_units: 0,
            lengths: HashMap::new(),
            values: 0,
        },
        config,
        settings,
        resources,
    )
}
fn invalid(message: &str) -> PublicError {
    super::model::semantic(message).into()
}
fn limit() -> PublicError {
    let mut error = PeleError::new(
        "Venn public operation exceeds native work budget",
        ErrorCode::Limit,
    );
    error.diagram_type = Some("venn".into());
    error.into()
}
fn key(ids: &[String]) -> String {
    if ids.len() == 1 {
        ids[0].clone()
    } else {
        serde_json::to_string(ids).unwrap()
    }
}
struct HostPhase<'a, R> {
    read: &'a mut R,
    text_slots: HashMap<(u8, usize, usize), usize>,
    text_units: usize,
    lengths: HashMap<(u8, usize), usize>,
    values: usize,
}
impl<R: LiveRenderRead> HostPhase<'_, R> {
    fn field(&mut self, key: &str) -> Result<R::Value, PublicError> {
        let root = self.read.root();
        self.read.get(&root, key)
    }
    fn nullish(&self, v: &R::Value) -> bool {
        matches!(
            self.read.kind(v),
            LiveValueKind::Undefined | LiveValueKind::Null
        )
    }
    fn length(&mut self, v: &R::Value, slot: (u8, usize)) -> Result<usize, PublicError> {
        let raw = self.read.get(v, "length")?;
        let n = self
            .read
            .number(&raw)
            .ok_or_else(|| invalid("Venn active collection requires an Array"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(invalid("Venn active collection requires an Array"));
        }
        let n = n as usize;
        let old = self.lengths.entry(slot).or_default();
        if n > *old {
            self.values = self.values.saturating_add(n - *old);
            *old = n;
        }
        if self.values > 1_000_000 {
            return Err(limit());
        }
        Ok(n)
    }
    fn text(&mut self, v: &R::Value, slot: (u8, usize, usize)) -> Result<String, PublicError> {
        let s = self
            .read
            .string_units(v)?
            .ok_or_else(|| invalid("Venn active text coercion is not implemented"))?;
        self.charge_text(s, slot)
    }
    fn charge_text(
        &mut self,
        s: JsString,
        slot: (u8, usize, usize),
    ) -> Result<String, PublicError> {
        let old = self.text_slots.entry(slot).or_default();
        if s.0.len() > *old {
            self.text_units = self.text_units.saturating_add(s.0.len() - *old);
            *old = s.0.len();
        }
        if self.text_units > 16_000_000 {
            return Err(limit());
        }
        s.well_formed()
            .ok_or_else(|| invalid("Venn rendering of lone UTF-16 text is not implemented"))
    }
    fn style_text(
        &mut self,
        v: &R::Value,
        slot: (u8, usize, usize),
    ) -> Result<String, PublicError> {
        if matches!(
            self.read.kind(v),
            LiveValueKind::Object | LiveValueKind::Function
        ) {
            return Err(invalid("Venn style object coercion is not implemented"));
        }
        let s = self.read.to_string(v)?;
        self.charge_text(s, slot)
    }
    fn strings(&mut self, v: &R::Value, kind: u8, row: usize) -> Result<Vec<String>, PublicError> {
        let n = self.length(v, (kind, row))?;
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let value = self.read.index(v, i)?;
            out.push(self.text(&value, (kind, row, i))?);
        }
        Ok(out)
    }
    fn sets(
        &mut self,
        record: &R::Value,
        kind: u8,
        row: usize,
    ) -> Result<Vec<String>, PublicError> {
        let v = self.read.get(record, "sets")?;
        self.strings(&v, kind, row)
    }
    fn label(
        &mut self,
        v: &R::Value,
        slot: (u8, usize, usize),
    ) -> Result<Option<String>, PublicError> {
        if self.nullish(v) {
            Ok(None)
        } else if !self.read.truthy(v) {
            Ok(Some(String::new()))
        } else {
            self.text(v, slot).map(Some)
        }
    }
}
impl<R: LiveRenderRead> PhaseInput for HostPhase<'_, R> {
    type Error = PublicError;
    type Subset = (usize, R::Value);
    type Texts = R::Value;
    fn geometry(&mut self) -> Result<Vec<VennSubset>, PublicError> {
        let values = self.field("subsets")?;
        let n = self.length(&values, (0, 0))?;
        let mut out = Vec::with_capacity(n);
        let mut nonnumber_singleton = false;
        for i in 0..n {
            let item = self.read.index(&values, i)?;
            let sets = self.sets(&item, 1, i)?;
            let value = self.read.get(&item, "size")?;
            let number_size = self.read.kind(&value) == LiveValueKind::Number;
            nonnumber_singleton |= sets.len() == 1 && !number_size;
            let size = match self.read.kind(&value) {
                LiveValueKind::Number => self.read.number(&value).unwrap(),
                LiveValueKind::Undefined => f64::NAN,
                LiveValueKind::Null => 0.,
                LiveValueKind::Boolean => {
                    if self.read.truthy(&value) {
                        1.
                    } else {
                        0.
                    }
                }
                LiveValueKind::String => self
                    .read
                    .string_units(&value)?
                    .and_then(|v| v.well_formed())
                    .map_or(f64::NAN, |v| super::public_model::js_number(&v)),
                _ => {
                    return Err(invalid(
                        "Venn active object size coercion is not implemented",
                    ))
                }
            };
            let size = if sets.len() == 1 && !number_size {
                f64::NAN
            } else {
                size
            };
            out.push(VennSubset {
                sets,
                size,
                label: None,
            });
        }
        if nonnumber_singleton && out.iter().any(|s| s.sets.len() >= 3) {
            return Err(invalid(
                "Venn triple inference with nonnumeric singleton sizes is not implemented",
            ));
        }
        let identifiers: HashSet<_> = out.iter().flat_map(|s| s.sets.iter()).collect();
        let circles_work = identifiers.len().saturating_mul(identifiers.len());
        if circles_work > 128_000_000 {
            return Err(limit());
        }
        let pairs = out
            .iter()
            .filter(|s| s.sets.len() >= 3)
            .fold(0usize, |n, s| {
                n.saturating_add(s.sets.len().saturating_mul(s.sets.len() - 1) / 2)
            });
        if pairs <= 3_000_000
            && out.iter().fold(circles_work, |n, s| {
                n.saturating_add(s.sets.len().saturating_pow(3))
            }) > 128_000_000
        {
            return Err(limit());
        }
        Ok(out)
    }
    fn styles(&mut self) -> Result<Vec<VennStyle>, PublicError> {
        let values = self.field("styles")?;
        let n = self.length(&values, (2, 0))?;
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let item = self.read.index(&values, i)?;
            let target = self.read.get(&item, "targets")?;
            let targets = self.strings(&target, 3, i)?;
            let map = self.read.get(&item, "styles")?;
            let pairs = self
                .read
                .snapshot_iterable(&map, 1_000_000usize.saturating_sub(self.values))?;
            self.values = self.values.saturating_add(pairs.len());
            if self.values > 1_000_000 {
                return Err(limit());
            }
            let mut styles = Vec::with_capacity(pairs.len());
            for (j, pair) in pairs.iter().enumerate() {
                let k = self.read.index(pair, 0)?;
                let v = self.read.index(pair, 1)?;
                styles.push((
                    self.style_text(&k, (4, i, j))?,
                    self.style_text(&v, (5, i, j))?,
                ));
            }
            out.push(VennStyle { targets, styles });
        }
        Ok(out)
    }
    fn stated(&mut self) -> Result<Vec<(String, Self::Subset)>, PublicError> {
        let values = self.field("subsets")?;
        let n = self.length(&values, (0, 0))?;
        let mut out = Vec::new();
        let mut positions = HashMap::new();
        for i in 0..n {
            let item = self.read.index(&values, i)?;
            let k = key(&self.sets(&item, 1, i)?);
            if let Some(&slot) = positions.get(&k) {
                out[slot] = (k, (i, item));
            } else {
                positions.insert(k.clone(), out.len());
                out.push((k, (i, item)));
            }
        }
        Ok(out)
    }
    fn subset(&mut self, (i, item): &Self::Subset) -> Result<VennSubset, PublicError> {
        let sets = self.sets(item, 1, *i)?;
        let label = self.read.get(item, "label")?;
        let label = self.label(&label, (6, *i, 0))?;
        Ok(VennSubset {
            sets,
            size: 0.,
            label,
        })
    }
    fn texts(&mut self) -> Result<Self::Texts, PublicError> {
        self.field("textNodes")
    }
    fn next_text(
        &mut self,
        values: &Self::Texts,
        i: usize,
        regions: &HashMap<String, usize>,
    ) -> Result<Option<Option<VennText>>, PublicError> {
        if i >= self.length(values, (7, 0))? {
            return Ok(None);
        }
        let item = self.read.index(values, i)?;
        // Upstream captures label (or id) before checking the current sets.
        let label = self.read.get(&item, "label")?;
        let fallback = self.nullish(&label);
        let raw = if fallback {
            self.read.get(&item, "id")?
        } else {
            label
        };
        let sets = self.sets(&item, 8, i)?;
        if !regions.contains_key(&key(&sets)) {
            return Ok(Some(None));
        }
        let id = self.read.get(&item, "id")?;
        let id = self.text(&id, (9, i, 0))?;
        let label = self.text(&raw, (if fallback { 9 } else { 10 }, i, 0))?;
        Ok(Some(Some(VennText {
            sets,
            id,
            label: Some(label),
        })))
    }
    fn title(&mut self) -> Result<Option<String>, PublicError> {
        let first = self.field("title")?;
        if !self.read.truthy(&first) {
            return Ok(None);
        }
        let title = self.field("title")?;
        // normalize() performs the pinned trim/unquote operation; supported text
        // remains well-formed UTF16, matching the existing native render gate.
        self.text(&title, (11, 0, 0)).map(Some)
    }
}
