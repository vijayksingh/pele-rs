//! Job-local Kanban references keep arrays and cards alive across callbacks.
use super::render::PhaseInput;
use crate::compat::{
    public::{LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
use std::collections::HashMap;
pub(crate) struct KanbanDetachedInput {
    limit: Option<PeleError>,
}
impl KanbanDetachedInput {
    pub(super) fn new(error: Option<PeleError>) -> Self {
        Self {
            limit: error.filter(|e| e.code == ErrorCode::Limit),
        }
    }
}
fn invalid(message: &str) -> PublicError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("kanban".into());
    e.into()
}
fn limit() -> PublicError {
    let mut e = PeleError::new(
        "Kanban public operation exceeds native work budget",
        ErrorCode::Limit,
    );
    e.diagram_type = Some("kanban".into());
    e.into()
}
struct HostPhase<'a, R> {
    read: &'a mut R,
    text: HashMap<(usize, usize, &'static str), usize>,
    units: usize,
    extents: HashMap<usize, usize>,
    work: usize,
}
// A slot identifies one retained card occurrence, not an object identity. This
// bounds duplicated labels even when a public array repeats the same object.
#[derive(Clone)]
struct HostNode<V> {
    value: V,
    section: usize,
    item: usize,
}
struct HostCollection<V> {
    value: V,
    section: Option<usize>,
}
impl<R: LiveRenderRead> HostPhase<'_, R> {
    fn field(&mut self, key: &str) -> Result<R::Value, PublicError> {
        let root = self.read.root();
        self.read.get(&root, key)
    }
    fn string(
        &mut self,
        v: &R::Value,
        slot: (usize, usize, &'static str),
    ) -> Result<Option<String>, PublicError> {
        let kind = self.read.kind(v);
        if kind == LiveValueKind::Undefined
            || (kind == LiveValueKind::Null && matches!(slot.2, "priority" | "cssClasses"))
            || (slot.2 == "icon" && !self.read.truthy(v))
        {
            return Ok(None);
        }
        let units = self.read.require_string(
            v,
            LiveStringUse {
                method: "includes",
                expression_receiver: "e",
            },
        )?;
        let max = self.text.entry(slot).or_default();
        if units.0.len() > *max {
            self.units = self.units.saturating_add(units.0.len() - *max);
            *max = units.0.len();
        }
        if self.units > 16_000_000 {
            return Err(limit());
        }
        units
            .well_formed()
            .map(Some)
            .ok_or_else(|| invalid("Kanban rendering of lone UTF-16 surrogates is not implemented"))
    }
}
impl<R: LiveRenderRead> PhaseInput for HostPhase<'_, R> {
    type Error = PublicError;
    type Collection = HostCollection<R::Value>;
    type Node = HostNode<R::Value>;
    fn sections(&mut self) -> Result<Self::Collection, PublicError> {
        Ok(HostCollection {
            value: self.field("sections")?,
            section: None,
        })
    }
    fn count(&mut self, c: &Self::Collection) -> Result<usize, PublicError> {
        let raw = self.read.get(&c.value, "length")?;
        let n = self
            .read
            .number(&raw)
            .ok_or_else(|| invalid("Kanban active collections must be Arrays"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(invalid("Kanban active collections must be Arrays"));
        }
        if n > 1_000_000. {
            return Err(limit());
        }
        let n = n as usize;
        if let Some(section) = c.section {
            let max = self.extents.entry(section).or_default();
            if n > *max {
                self.work = self.work.saturating_add(n - *max);
                *max = n;
            }
            if self.work > 8_000_000 {
                return Err(limit());
            }
        }
        Ok(n)
    }
    fn node(&mut self, c: &Self::Collection, index: usize) -> Result<Self::Node, PublicError> {
        let value = self.read.index(&c.value, index)?;
        Ok(HostNode {
            value,
            section: c.section.unwrap_or(index),
            item: if c.section.is_some() { index + 1 } else { 0 },
        })
    }
    fn items(&mut self, node: &Self::Node) -> Result<Self::Collection, PublicError> {
        Ok(HostCollection {
            value: self.read.get(&node.value, "items")?,
            section: Some(node.section),
        })
    }
    fn text(
        &mut self,
        node: &Self::Node,
        key: &'static str,
    ) -> Result<Option<String>, PublicError> {
        let v = self.read.get(&node.value, key)?;
        self.string(&v, (node.section, node.item, key))
    }
    fn title(&mut self) -> Result<Option<String>, PublicError> {
        let v = self.field("title")?;
        self.string(&v, (usize::MAX, 0, "title"))
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    seed: &KanbanDetachedInput,
    c: &Object,
    o: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    if let Some(error) = &seed.limit {
        return Err(error.clone().into());
    }
    super::render::render_phases(
        &mut HostPhase {
            read,
            text: HashMap::new(),
            units: 0,
            extents: HashMap::new(),
            work: 0,
        },
        c,
        o,
        resources,
    )
}
