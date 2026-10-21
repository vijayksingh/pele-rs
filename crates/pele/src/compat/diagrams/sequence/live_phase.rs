//! Live Sequence values are job-owned; rendering never borrows document sequence.
use super::phase::PhaseInput;
use crate::compat::{
    public::{LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
pub(crate) struct SequenceDetachedInput {
    limit: Option<PeleError>,
}
impl SequenceDetachedInput {
    pub(super) fn new(error: Option<&PeleError>) -> Self {
        Self {
            limit: error.filter(|e| e.code == ErrorCode::Limit).cloned(),
        }
    }
}
#[derive(Clone, Hash, PartialEq, Eq)]
enum Slot {
    Field(u64, &'static str),
    Index(u64, usize),
    Map(u64, String),
}
#[derive(Clone)]
struct HostValue<V> {
    raw: V,
    slot: Option<Slot>,
}
struct Host<'a, R: LiveRenderRead> {
    read: &'a mut R,
    slots: std::collections::HashMap<Slot, usize>,
    units: usize,
    steps: usize,
    labels: usize,
    actors: usize,
}
fn invalid(s: &str) -> PeleError {
    let mut e = PeleError::new(s, ErrorCode::Semantic);
    e.diagram_type = Some("sequence".into());
    e
}
fn limit() -> PeleError {
    let mut e = invalid("Sequence live rendering exceeds native work/text budget");
    e.code = ErrorCode::Limit;
    e
}
impl<R: LiveRenderRead> Host<'_, R> {
    fn keep(&mut self, v: &HostValue<R::Value>, n: usize) -> Result<(), PublicError> {
        let old = v
            .slot
            .as_ref()
            .and_then(|s| self.slots.get(s))
            .copied()
            .unwrap_or(0);
        if n > old {
            self.units = self.units.checked_add(n - old).ok_or_else(limit)?;
            if self.units > 16_000_000 {
                return Err(limit().into());
            }
            if let Some(slot) = &v.slot {
                self.slots.insert(slot.clone(), n);
            }
        }
        Ok(())
    }
}
impl<R: LiveRenderRead> PhaseInput for Host<'_, R> {
    type Value = HostValue<R::Value>;
    type Iterator = R::Value;
    type Error = PublicError;
    fn begin_draw(&mut self) {
        self.slots.clear();
        self.units = 0;
        self.steps = 0;
        self.labels = 0;
        self.actors = 0;
    }
    fn root(&self) -> Self::Value {
        HostValue {
            raw: self.read.root(),
            slot: None,
        }
    }
    fn entries(&mut self, v: &Self::Value) -> Result<R::Value, PublicError> {
        self.read.iterator_begin(&v.raw)
    }
    fn next(
        &mut self,
        v: &mut R::Value,
    ) -> Result<Option<(Self::Value, Self::Value)>, PublicError> {
        let Some(pair) = self.read.iterator_next(v)? else {
            return Ok(None);
        };
        let body = (|| {
            self.step()?;
            self.actors = self.actors.checked_add(1).ok_or_else(limit)?;
            if self.actors > 1_000_000 {
                return Err(limit().into());
            }
            let key = self.read.index(&pair, 0)?;
            let value = self.read.index(&pair, 1)?;
            Ok(Some((
                HostValue {
                    raw: key,
                    slot: None,
                },
                HostValue {
                    raw: value,
                    slot: None,
                },
            )))
        })();
        if body.is_err() {
            let _ = self.read.iterator_close(v);
        }
        body
    }
    fn close(&mut self, v: &mut R::Value) -> Result<(), PublicError> {
        self.read.iterator_close(v)
    }
    fn lookup(&mut self, v: &Self::Value, key: &str) -> Result<Self::Value, PublicError> {
        let raw = self.read.map_get_text(&v.raw, key)?;
        let slot = self
            .read
            .object_identity(&v.raw)?
            .map(|id| Slot::Map(id, key.into()));
        Ok(HostValue { raw, slot })
    }
    fn identity(&mut self, v: &Self::Value) -> Result<Option<u64>, PublicError> {
        self.read.object_identity(&v.raw)
    }
    fn number(&self, v: &Self::Value) -> Option<f64> {
        self.read.number(&v.raw)
    }
    fn get(&mut self, v: &Self::Value, key: &'static str) -> Result<Self::Value, PublicError> {
        let raw = self.read.get(&v.raw, key)?;
        let slot = self
            .read
            .object_identity(&v.raw)?
            .map(|id| Slot::Field(id, key));
        Ok(HostValue { raw, slot })
    }
    fn count(&mut self, v: &Self::Value) -> Result<usize, PublicError> {
        let value = self.read.get(&v.raw, "length")?;
        let n = self
            .read
            .number(&value)
            .ok_or_else(|| invalid("Sequence collections must be Arrays"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(invalid("Sequence collections must be Arrays").into());
        }
        if n > 1_000_000. {
            return Err(limit().into());
        }
        Ok(n as usize)
    }
    fn index(&mut self, v: &Self::Value, i: usize) -> Result<Self::Value, PublicError> {
        let raw = self.read.index(&v.raw, i)?;
        let slot = self
            .read
            .object_identity(&v.raw)?
            .map(|id| Slot::Index(id, i));
        Ok(HostValue { raw, slot })
    }
    fn text(&mut self, v: &Self::Value) -> Result<Option<String>, PublicError> {
        if self.read.kind(&v.raw) != LiveValueKind::String {
            return Ok(None);
        }
        let text = self.read.require_string(
            &v.raw,
            LiveStringUse {
                method: "includes",
                expression_receiver: "t",
            },
        )?;
        self.keep(v, text.0.len())?;
        text.well_formed()
            .map(Some)
            .ok_or_else(|| invalid("Sequence live rendering requires well-formed UTF16").into())
    }
    fn label_text(&mut self, v: &Self::Value) -> Result<Option<String>, PublicError> {
        if matches!(
            self.read.kind(&v.raw),
            LiveValueKind::Undefined | LiveValueKind::Null
        ) {
            return Ok(None);
        }
        let text = self.read.require_string(
            &v.raw,
            LiveStringUse {
                method: "includes",
                expression_receiver: "t",
            },
        )?;
        self.keep(v, text.0.len())?;
        text.well_formed()
            .map(Some)
            .ok_or_else(|| invalid("Sequence live rendering requires well-formed UTF16").into())
    }
    fn truthy(&self, v: &Self::Value) -> bool {
        self.read.truthy(&v.raw)
    }
    fn step(&mut self) -> Result<(), PublicError> {
        self.steps = self.steps.checked_add(1).ok_or_else(limit)?;
        if self.steps > 8_000_000 {
            return Err(limit().into());
        }
        Ok(())
    }
    fn keep_text(&mut self, n: usize) -> Result<(), PublicError> {
        self.labels = self.labels.checked_add(n).ok_or_else(limit)?;
        if self.labels > 16_000_000 {
            return Err(limit().into());
        }
        Ok(())
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    seed: &SequenceDetachedInput,
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
            slots: std::collections::HashMap::new(),
            units: 0,
            steps: 0,
            labels: 0,
            actors: 0,
        },
        c,
        o,
        r,
    )
}
