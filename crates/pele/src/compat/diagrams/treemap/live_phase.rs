//! Geometry captures node handles; presentation reads remain live after resources.
use super::{layout, render::PhaseInput};
use crate::compat::{
    public::{LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
use std::collections::HashMap;
pub(crate) struct TreemapDetachedInput {
    limit: Option<PeleError>,
}
impl TreemapDetachedInput {
    pub(super) fn new(error: Option<PeleError>) -> Self {
        Self {
            limit: error.filter(|e| e.code == ErrorCode::Limit),
        }
    }
}
fn invalid(message: &str) -> PublicError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("treemap".into());
    e.into()
}
fn limit() -> PublicError {
    let mut e = PeleError::new(
        "Treemap hierarchy expansion exceeds native work budget",
        ErrorCode::Limit,
    );
    e.diagram_type = Some("treemap".into());
    e.into()
}
struct HostPhase<'a, R> {
    read: &'a mut R,
    text: HashMap<(usize, &'static str, usize), usize>,
    units: usize,
}
impl<R: LiveRenderRead> HostPhase<'_, R> {
    fn root_field(&mut self, key: &str) -> Result<R::Value, PublicError> {
        let root = self.read.root();
        self.read.get(&root, key)
    }
    fn length(&mut self, array: &R::Value) -> Result<usize, PublicError> {
        let raw = self.read.get(array, "length")?;
        let n = self
            .read
            .number(&raw)
            .ok_or_else(|| invalid("Treemap collections must be Arrays"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(invalid("Treemap collections must be Arrays"));
        }
        if n > 1_000_000. {
            return Err(limit());
        }
        Ok(n as usize)
    }
    fn array(&mut self, array: &R::Value) -> Result<Vec<R::Value>, PublicError> {
        let n = self.length(array)?;
        let mut result = Vec::with_capacity(n);
        for i in 0..n {
            result.push(self.read.index(array, i)?)
        }
        Ok(result)
    }
    fn string(
        &mut self,
        value: &R::Value,
        slot: (usize, &'static str, usize),
    ) -> Result<String, PublicError> {
        let value = self.read.require_string(
            value,
            LiveStringUse {
                method: "includes",
                expression_receiver: "e",
            },
        )?;
        let max = self.text.entry(slot).or_default();
        if value.0.len() > *max {
            self.units = self.units.saturating_add(value.0.len() - *max);
            *max = value.0.len();
        }
        if self.units > 16_000_000 {
            return Err(limit());
        }
        value.well_formed().ok_or_else(|| {
            invalid("Treemap rendering of lone UTF-16 surrogates is not implemented")
        })
    }
}
impl<R: LiveRenderRead> layout::Input for HostPhase<'_, R> {
    type Node = R::Value;
    type Error = PublicError;
    fn roots(&mut self) -> Result<Vec<R::Value>, PublicError> {
        let roots = self.root_field("roots")?;
        self.array(&roots)
    }
    fn children(&mut self, node: &R::Value) -> Result<Vec<R::Value>, PublicError> {
        let children = self.read.get(node, "children")?;
        if matches!(
            self.read.kind(&children),
            LiveValueKind::Undefined | LiveValueKind::Null
        ) {
            Ok(vec![])
        } else {
            self.array(&children)
        }
    }
    fn initial_value(&mut self, node: &R::Value) -> Result<f64, PublicError> {
        let children = self.read.get(node, "children")?;
        if self.read.kind(&children) != LiveValueKind::Undefined {
            return Ok(0.);
        }
        let value = self.read.get(node, "value")?;
        Ok(self
            .read
            .number(&value)
            .filter(|v| v.is_finite() && *v > 0.)
            .unwrap_or(0.))
    }
    fn reserve(&mut self, count: usize) -> Result<(), PublicError> {
        if count >= 1_000_000 {
            Err(limit())
        } else {
            Ok(())
        }
    }
}
impl<R: LiveRenderRead> PhaseInput for HostPhase<'_, R> {
    fn metadata(&mut self, key: &'static str) -> Result<Option<String>, PublicError> {
        let mut value = self.root_field(key)?;
        if self.read.kind(&value) == LiveValueKind::Undefined
            || (key != "title" && !self.read.truthy(&value))
        {
            return Ok(None);
        }
        if key != "title" {
            value = self.root_field(key)?
        }
        self.string(&value, (usize::MAX, key, 0)).map(Some)
    }
    fn styles(&mut self, node: &R::Value, slot: usize) -> Result<Vec<String>, PublicError> {
        let value = self.read.get(node, "cssCompiledStyles")?;
        if matches!(
            self.read.kind(&value),
            LiveValueKind::Undefined | LiveValueKind::Null
        ) {
            return Ok(vec![]);
        }
        let n = self.length(&value)?;
        let mut result = Vec::new();
        for i in 0..n {
            let row = self.read.index(&value, i)?;
            result.push(self.string(&row, (slot, "cssCompiledStyles", i))?)
        }
        Ok(result)
    }
    fn text(
        &mut self,
        node: &R::Value,
        key: &'static str,
        slot: usize,
    ) -> Result<Option<String>, PublicError> {
        let value = self.read.get(node, key)?;
        if key == "classSelector"
            && matches!(
                self.read.kind(&value),
                LiveValueKind::Undefined | LiveValueKind::Null
            )
        {
            return Ok(None);
        }
        self.string(&value, (slot, key, 0)).map(Some)
    }
    fn section(&mut self, node: &R::Value) -> Result<bool, PublicError> {
        let value = self.read.get(node, "children")?;
        Ok(self.read.kind(&value) != LiveValueKind::Undefined)
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    seed: &TreemapDetachedInput,
    c: &Object,
    o: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    if let Some(e) = &seed.limit {
        return Err(e.clone().into());
    }
    super::render::render_phases(
        &mut HostPhase {
            read,
            text: HashMap::new(),
            units: 0,
        },
        c,
        o,
        resources,
    )
}
