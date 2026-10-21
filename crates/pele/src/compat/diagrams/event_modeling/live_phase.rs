//! Live Eventmodeling values are job-owned; rendering never borrows document state.
use super::phase::PhaseInput;
use crate::compat::{
    public::{LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
pub(crate) struct EventModelingDetachedInput {
    limit: Option<PeleError>,
}
impl EventModelingDetachedInput {
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
    e.diagram_type = Some("eventmodeling".into());
    e
}
fn limit() -> PeleError {
    let mut e = invalid("Eventmodeling live rendering exceeds native work/text budget");
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
    fn get(&mut self, v: &R::Value, key: &'static str) -> Result<R::Value, PublicError> {
        self.read.get(v, key)
    }
    fn count(&mut self, v: &R::Value) -> Result<usize, PublicError> {
        let v = self.read.get(v, "length")?;
        let n = self
            .read
            .number(&v)
            .ok_or_else(|| invalid("Eventmodeling collections must be Arrays"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(invalid("Eventmodeling collections must be Arrays").into());
        }
        if n > 1_000_000. {
            return Err(limit().into());
        }
        Ok(n as usize)
    }
    fn index(&mut self, v: &R::Value, i: usize) -> Result<R::Value, PublicError> {
        self.read.index(v, i)
    }
    fn present(&mut self, v: &R::Value, i: usize) -> Result<bool, PublicError> {
        self.read.has_index(v, i)
    }
    fn identity(&mut self, v: &R::Value) -> Result<u64, PublicError> {
        self.read
            .object_identity(v)?
            .ok_or_else(|| invalid("Eventmodeling record must be an object").into())
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
        text.well_formed().map(Some).ok_or_else(|| {
            invalid("Eventmodeling live rendering requires well-formed UTF16").into()
        })
    }
    fn truthy(&self, v: &R::Value) -> bool {
        self.read.truthy(v)
    }
    fn join(&mut self, v: &R::Value) -> Result<String, PublicError> {
        let text = self.read.array_join(v, ", ")?;
        self.keep(text.0.len())?;
        text.well_formed().ok_or_else(|| {
            invalid("Eventmodeling live rendering requires well-formed UTF16").into()
        })
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
    seed: &EventModelingDetachedInput,
    _c: &Object,
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
        o,
        r,
    )
}
