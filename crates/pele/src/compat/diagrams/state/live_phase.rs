//! Live State values are job-owned; rendering never borrows document state.
use super::phase::PhaseInput;
use crate::compat::{
    public::{LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
pub(crate) struct StateDetachedInput {
    limit: Option<PeleError>,
}
impl StateDetachedInput {
    pub(super) fn new(error: Option<&PeleError>) -> Self {
        Self {
            limit: error.filter(|e| e.code == ErrorCode::Limit).cloned(),
        }
    }
}
struct Host<'a, R> {
    read: &'a mut R,
    units: usize,
    steps: usize,
}
fn invalid(s: &str) -> PeleError {
    let mut e = PeleError::new(s, ErrorCode::Semantic);
    e.diagram_type = Some("state".into());
    e
}
fn limit() -> PeleError {
    let mut e = invalid("State live rendering exceeds native work/text budget");
    e.code = ErrorCode::Limit;
    e
}
impl<R: LiveRenderRead> Host<'_, R> {
    fn keep(&mut self, n: usize) -> Result<(), PublicError> {
        self.units = self.units.checked_add(n).ok_or_else(limit)?;
        if self.units > 16_000_000 {
            return Err(limit().into());
        }
        Ok(())
    }
}
impl<R: LiveRenderRead> PhaseInput for Host<'_, R> {
    type Value = R::Value;
    type Error = PublicError;
    fn begin_draw(&mut self) {
        self.units = 0;
        self.steps = 0;
    }
    fn root(&self) -> R::Value {
        self.read.root()
    }
    fn direction(&mut self) -> Result<String, PublicError> {
        let root = self.read.root();
        let direct = self.read.get(&root, "direction")?;
        if self.read.kind(&direct) != LiveValueKind::Undefined {
            return Ok(self.text(&direct)?.unwrap_or_default());
        }
        // C15 graph coordinates cannot execute StateDb's inherited getter. Preserve its
        // existing rootDoc derivation; own Undefined versus inherited is a scoped gap.
        let doc = self.read.get(&root, "rootDoc")?;
        let mut value = "TB".to_string();
        let mut i = 0;
        while i < self.count(&doc)? {
            self.step()?;
            let item = self.read.index(&doc, i)?;
            i += 1;
            if self.read.kind(&item) != LiveValueKind::Object {
                continue;
            }
            let kind = self.read.get(&item, "stmt")?;
            if self.text(&kind)?.as_deref() == Some("dir") {
                let v = self.read.get(&item, "value")?;
                value = self.text(&v)?.unwrap_or_default();
            }
        }
        Ok(value)
    }
    fn link(&mut self, id: &str) -> Result<Option<(String, String)>, PublicError> {
        let root = self.read.root();
        let map = self.read.get(&root, "links")?;
        let link = self.read.map_get_text(&map, id)?;
        if !self.read.truthy(&link) {
            return Ok(None);
        }
        let tip = self.read.get(&link, "tooltip")?;
        let tip = self.text(&tip)?.unwrap_or_default();
        let url = self.read.get(&link, "url")?;
        let url = self.text(&url)?.unwrap_or_default();
        Ok(Some((url, tip)))
    }
    fn get(&mut self, v: &R::Value, key: &'static str) -> Result<R::Value, PublicError> {
        self.read.get(v, key)
    }
    fn count(&mut self, v: &R::Value) -> Result<usize, PublicError> {
        let v = self.read.get(v, "length")?;
        let n = self
            .read
            .number(&v)
            .ok_or_else(|| invalid("State collections must be Arrays"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(invalid("State collections must be Arrays").into());
        }
        if n > 1_000_000. {
            return Err(limit().into());
        }
        Ok(n as usize)
    }
    fn index(&mut self, v: &R::Value, i: usize) -> Result<R::Value, PublicError> {
        self.read.index(v, i)
    }
    fn text(&mut self, v: &R::Value) -> Result<Option<String>, PublicError> {
        if matches!(
            self.read.kind(v),
            LiveValueKind::Undefined | LiveValueKind::Null
        ) {
            return Ok(None);
        }
        let text = self.read.require_string(
            v,
            LiveStringUse {
                method: "includes",
                expression_receiver: "t",
            },
        )?;
        self.keep(text.0.len())?;
        text.well_formed()
            .map(Some)
            .ok_or_else(|| invalid("State live rendering requires well-formed UTF16").into())
    }
    fn truthy(&self, v: &R::Value) -> bool {
        self.read.truthy(v)
    }
    fn step(&mut self) -> Result<(), PublicError> {
        self.steps = self.steps.checked_add(1).ok_or_else(limit)?;
        if self.steps > 8_000_000 {
            return Err(limit().into());
        }
        Ok(())
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    seed: &StateDetachedInput,
    c: &Object,
    o: &RenderSettings,
    read: &mut R,
    r: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    if let Some(e) = &seed.limit {
        return Err(e.clone().into());
    }
    super::render::render_phases(
        &mut Host {
            read,
            units: 0,
            steps: 0,
        },
        c,
        o,
        r,
    )
}
