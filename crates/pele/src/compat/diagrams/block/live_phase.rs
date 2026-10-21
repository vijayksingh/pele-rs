//! Owned Block values, read only at the shared renderer's selected phases.
use super::phase::PhaseInput;
use crate::compat::{
    public::{LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
use std::collections::HashMap;
pub(crate) struct BlockDetachedInput {
    limit: Option<PeleError>,
}
impl BlockDetachedInput {
    pub(super) fn new(error: Option<&PeleError>) -> Self {
        Self {
            limit: match error {
                Some(e) if e.code == ErrorCode::Limit => Some(e.clone()),
                _ => None,
            },
        }
    }
}
#[derive(Clone, Hash, PartialEq, Eq)]
enum Slot {
    Field(u64, &'static str),
    Index(u64, usize),
}
#[derive(Clone)]
struct HostValue<V> {
    raw: V,
    slot: Option<Slot>,
}
struct Host<'a, R: LiveRenderRead> {
    read: &'a mut R,
    text: HashMap<Slot, usize>,
    arrays: HashMap<u64, usize>,
    units: usize,
    extents: usize,
    steps: usize,
    labels: usize,
}
fn invalid(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("block".into());
    e
}
fn limit() -> PeleError {
    let mut e = invalid("Block public model exceeds native graph/work budget");
    e.code = ErrorCode::Limit;
    e
}
impl<R: LiveRenderRead> Host<'_, R> {
    fn charge_text(&mut self, v: &HostValue<R::Value>, count: usize) -> Result<(), PublicError> {
        let old = v
            .slot
            .as_ref()
            .and_then(|s| self.text.get(s))
            .copied()
            .unwrap_or(0);
        if count > old {
            self.units = self.units.checked_add(count - old).ok_or_else(limit)?;
            if self.units > 16_000_000 {
                return Err(limit().into());
            }
            if let Some(s) = &v.slot {
                self.text.insert(s.clone(), count);
            }
        }
        Ok(())
    }
    fn id(&mut self, v: &R::Value) -> Result<u64, PublicError> {
        self.read
            .object_identity(v)?
            .ok_or_else(|| invalid("Block collection or record must be an object").into())
    }
}
impl<R: LiveRenderRead> PhaseInput for Host<'_, R> {
    type Value = HostValue<R::Value>;
    type Error = PublicError;
    fn begin_draw(&mut self) {
        self.text.clear();
        self.arrays.clear();
        self.units = 0;
        self.extents = 0;
        self.steps = 0;
        self.labels = 0;
    }
    fn root(&mut self) -> Self::Value {
        HostValue {
            raw: self.read.root(),
            slot: None,
        }
    }
    fn get(&mut self, v: &Self::Value, k: &'static str) -> Result<Self::Value, PublicError> {
        let raw = self.read.get(&v.raw, k)?;
        let id = self.id(&v.raw)?;
        Ok(HostValue {
            raw,
            slot: Some(Slot::Field(id, k)),
        })
    }
    fn count(&mut self, v: &Self::Value) -> Result<usize, PublicError> {
        let raw = self.read.get(&v.raw, "length")?;
        let n = self
            .read
            .number(&raw)
            .ok_or_else(|| invalid("Block collections must be Arrays"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(invalid("Block collections must be Arrays").into());
        }
        if n > 1_000_000. {
            return Err(limit().into());
        }
        let n = n as usize;
        let id = self.id(&v.raw)?;
        let old = self.arrays.get(&id).copied().unwrap_or(0);
        if n > old {
            self.extents = self.extents.checked_add(n - old).ok_or_else(limit)?;
            if self.extents > 8_000_000 {
                return Err(limit().into());
            }
            self.arrays.insert(id, n);
        }
        Ok(n)
    }
    fn index(&mut self, v: &Self::Value, i: usize) -> Result<Self::Value, PublicError> {
        let raw = self.read.index(&v.raw, i)?;
        let id = self.id(&v.raw)?;
        Ok(HostValue {
            raw,
            slot: Some(Slot::Index(id, i)),
        })
    }
    fn nullish(&self, v: &Self::Value) -> bool {
        matches!(
            self.read.kind(&v.raw),
            LiveValueKind::Null | LiveValueKind::Undefined
        )
    }
    fn truthy(&self, v: &Self::Value) -> bool {
        self.read.truthy(&v.raw)
    }
    fn map_get_text(&mut self, v: &Self::Value, key: &str) -> Result<Self::Value, PublicError> {
        let raw = self.read.map_get_text(&v.raw, key)?;
        Ok(HostValue { raw, slot: None })
    }
    fn join(&mut self, v: &Self::Value) -> Result<String, PublicError> {
        let value = self.read.array_join(&v.raw, " ")?;
        self.charge_text(v, value.0.len())?;
        value
            .well_formed()
            .ok_or_else(|| invalid("Block rendering requires well-formed UTF16 text").into())
    }
    fn number(&self, v: &Self::Value) -> Option<f64> {
        self.read.number(&v.raw)
    }
    fn text(&mut self, v: &Self::Value) -> Result<String, PublicError> {
        let text = self.read.require_string(
            &v.raw,
            LiveStringUse {
                method: "includes",
                expression_receiver: "t",
            },
        )?;
        self.charge_text(v, text.0.len())?;
        text.well_formed()
            .ok_or_else(|| invalid("Block rendering requires well-formed UTF16 text").into())
    }
    fn work(&mut self, n: usize) -> Result<(), PublicError> {
        self.steps = self.steps.checked_add(n).ok_or_else(limit)?;
        if self.steps > 8_000_000 {
            Err(limit().into())
        } else {
            Ok(())
        }
    }
    fn keep_text(&mut self, n: usize) -> Result<(), PublicError> {
        self.labels = self.labels.checked_add(n).ok_or_else(limit)?;
        if self.labels > 32_000_000 {
            Err(limit().into())
        } else {
            Ok(())
        }
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    seed: &BlockDetachedInput,
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
            text: HashMap::new(),
            arrays: HashMap::new(),
            units: 0,
            extents: 0,
            steps: 0,
            labels: 0,
        },
        c,
        o,
        r,
    )
}
