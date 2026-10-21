//! Owned Requirement descriptor handles; host reads occur at shared renderer phases.
use super::phase::{CapturedEdge, CapturedNode, PhaseInput, Scene};
use crate::compat::{
    public::{LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
use std::collections::HashSet;
pub(crate) struct RequirementDetachedInput {
    limit: Option<PeleError>,
}
impl RequirementDetachedInput {
    pub(super) fn new(error: Option<&PeleError>) -> Self {
        Self {
            limit: error.filter(|e| e.code == ErrorCode::Limit).cloned(),
        }
    }
}
#[derive(Clone)]
struct Record<V> {
    value: V,
    requirement: bool,
}
struct Host<'a, R: LiveRenderRead> {
    read: &'a mut R,
    units: usize,
    work: usize,
}
fn invalid(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("requirement".into());
    e
}
fn limit() -> PeleError {
    let mut e = invalid("Requirement live rendering exceeds native graph/work budget");
    e.code = ErrorCode::Limit;
    e
}
impl<R: LiveRenderRead> Host<'_, R> {
    fn charge(&mut self, n: usize) -> Result<(), PublicError> {
        self.work = self.work.checked_add(n).ok_or_else(limit)?;
        if self.work > 8_000_000 {
            return Err(limit().into());
        }
        Ok(())
    }
    fn keep(&mut self, n: usize) -> Result<(), PublicError> {
        self.units = self.units.checked_add(n).ok_or_else(limit)?;
        if self.units > 16_000_000 {
            return Err(limit().into());
        }
        Ok(())
    }
    fn root_field(&mut self, key: &str) -> Result<R::Value, PublicError> {
        let root = self.read.root();
        self.read.get(&root, key)
    }
    fn text(&mut self, value: &R::Value) -> Result<String, PublicError> {
        let s = self.read.require_string(
            value,
            LiveStringUse {
                method: "includes",
                expression_receiver: "t",
            },
        )?;
        self.keep(s.0.len())?;
        s.well_formed()
            .ok_or_else(|| invalid("Requirement rendering requires well-formed UTF16 text").into())
    }
    fn field_text(&mut self, value: &R::Value, key: &str) -> Result<String, PublicError> {
        let v = self.read.get(value, key)?;
        self.text(&v)
    }
    fn length(&mut self, value: &R::Value) -> Result<usize, PublicError> {
        let raw = self.read.get(value, "length")?;
        let n = self
            .read
            .number(&raw)
            .ok_or_else(|| invalid("Requirement collections must be Arrays"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(invalid("Requirement collections must be Arrays").into());
        }
        if n > 1_000_000. {
            return Err(limit().into());
        }
        let n = n as usize;
        self.charge(n)?;
        Ok(n)
    }
}
impl<R: LiveRenderRead> PhaseInput for Host<'_, R> {
    type Node = Record<R::Value>;
    type Styles = R::Value;
    type Error = PublicError;
    fn direction(&mut self) -> Result<String, PublicError> {
        let v = self.root_field("direction")?;
        self.text(&v)
    }
    fn capture(&mut self) -> Result<Scene<Self::Node, Self::Styles>, PublicError> {
        self.units = 0;
        self.work = 0;
        let mut nodes = Vec::new();
        let mut keys = HashSet::new();
        for (name, requirement) in [("requirements", true), ("elements", false)] {
            let map = self.root_field(name)?;
            let pairs = self.read.snapshot_iterable(&map, 1_000_000)?;
            self.charge(pairs.len().saturating_mul(2))?;
            if nodes.len().saturating_add(pairs.len()) > 1_000_000 {
                return Err(limit().into());
            }
            for pair in pairs {
                let key = self.read.index(&pair, 0)?;
                keys.insert(self.text(&key)?);
                let value = self.read.index(&pair, 1)?;
                let id = self.field_text(&value, "name")?;
                let styles = self.read.get(&value, "cssStyles")?;
                let classes = self.read.get(&value, "classes")?;
                let classes = self.read.array_join(&classes, " ")?;
                self.keep(classes.0.len())?;
                let classes = classes.well_formed().ok_or_else(|| {
                    invalid("Requirement rendering requires well-formed UTF16 classes")
                })?;
                nodes.push(CapturedNode {
                    id,
                    requirement,
                    record: Record { value, requirement },
                    styles,
                    classes,
                });
            }
        }
        let array = self.root_field("relations")?;
        let n = self.length(&array)?;
        let mut edges = Vec::new();
        for i in 0..n {
            let value = self.read.index(&array, i)?;
            let src = self.field_text(&value, "src")?;
            let dst = self.field_text(&value, "dst")?;
            let id = format!("{src}-{dst}-{i}");
            self.keep(id.encode_utf16().count())?;
            let kind = self.field_text(&value, "type")?;
            edges.push(CapturedEdge {
                id,
                start: keys.contains(&src).then_some(src),
                end: keys.contains(&dst).then_some(dst),
                contains: kind == "contains",
                kind,
            });
        }
        let direction = self.direction()?;
        Ok(Scene {
            nodes,
            edges,
            direction,
        })
    }
    fn styles(&mut self, value: &R::Value) -> Result<Vec<String>, PublicError> {
        let n = self.length(value)?;
        let mut out = Vec::new();
        for i in 0..n {
            let v = self.read.index(value, i)?;
            out.push(self.text(&v)?)
        }
        Ok(out)
    }
    fn kind(&mut self, node: &Self::Node) -> Result<String, PublicError> {
        if node.requirement {
            self.field_text(&node.value, "type")
        } else {
            Ok("Element".into())
        }
    }
    fn fields(&mut self, node: &Self::Node) -> Result<Vec<(&'static str, String)>, PublicError> {
        let fields: &[(&str, &str)] = if node.requirement {
            &[
                ("ID", "requirementId"),
                ("Text", "text"),
                ("Risk", "risk"),
                ("Verification", "verifyMethod"),
            ]
        } else {
            &[("Type", "type"), ("Doc Ref", "docRef")]
        };
        // Capture all values before measuring any row, exactly like the source array literal.
        let mut values = Vec::new();
        for &(label, key) in fields {
            values.push((label, self.read.get(&node.value, key)?))
        }
        let mut out = Vec::new();
        for (label, value) in values {
            if self.read.truthy(&value) {
                out.push((label, self.text(&value)?))
            }
        }
        Ok(out)
    }
    fn metadata(&mut self, key: &'static str) -> Result<Option<String>, PublicError> {
        let mut value = self.root_field(key)?;
        if key == "title" {
            if self.read.kind(&value) == LiveValueKind::Undefined {
                return Ok(None);
            }
        } else {
            if !self.read.truthy(&value) {
                return Ok(None);
            }
            value = self.root_field(key)?;
        }
        self.text(&value).map(Some)
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    seed: &RequirementDetachedInput,
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
            work: 0,
        },
        c,
        o,
        r,
    )
}
