//! Native-selected Mindmap node reads around resource callbacks.
use super::render::PhaseInput;
use crate::compat::{
    public::{LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
use std::collections::HashMap;
pub(crate) struct MindmapDetachedInput {
    limit: Option<PeleError>,
}
impl MindmapDetachedInput {
    pub(super) fn new(error: Option<&PeleError>) -> Self {
        Self {
            limit: error.filter(|e| e.code == ErrorCode::Limit).cloned(),
        }
    }
}
struct HostPhase<'a, R> {
    read: &'a mut R,
    text: HashMap<(u64, &'static str), usize>,
    units: usize,
    retained_units: usize,
    steps: usize,
}
fn invalid(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("mindmap".into());
    e
}
fn limit() -> PeleError {
    let mut e = invalid("Mindmap public operation exceeds native work budget");
    e.code = ErrorCode::Limit;
    e
}
impl<R: LiveRenderRead> HostPhase<'_, R> {
    fn field(&mut self, key: &str) -> Result<R::Value, PublicError> {
        let root = self.read.root();
        self.read.get(&root, key)
    }
    fn string(
        &mut self,
        owner: &R::Value,
        key: &'static str,
        value: &R::Value,
    ) -> Result<String, PublicError> {
        let text = self.read.require_string(
            value,
            LiveStringUse {
                method: "includes",
                expression_receiver: "t",
            },
        )?;
        let identity = self
            .read
            .object_identity(owner)?
            .ok_or_else(|| invalid("Mindmap node must be an object"))?;
        let old = self.text.get(&(identity, key)).copied().unwrap_or(0);
        if text.0.len() > old {
            self.units = self
                .units
                .checked_add(text.0.len() - old)
                .ok_or_else(limit)?;
            if self.units > 16_000_000 {
                return Err(limit().into());
            }
            self.text.insert((identity, key), text.0.len());
        }
        text.well_formed().ok_or_else(|| {
            invalid("Mindmap rendering of lone UTF-16 surrogates is not implemented").into()
        })
    }
}
impl<R: LiveRenderRead> PhaseInput for HostPhase<'_, R> {
    type Error = PublicError;
    type Node = R::Value;
    type List = R::Value;
    fn begin_draw(&mut self) {
        self.text.clear();
        self.units = 0;
        self.retained_units = 0;
        self.steps = 0;
    }
    fn retain_label(&mut self, text: Option<&str>) -> Result<(), PublicError> {
        self.retained_units = self
            .retained_units
            .checked_add(text.map_or(0, |s| s.encode_utf16().count()))
            .ok_or_else(limit)?;
        if self.retained_units > 16_000_000 {
            return Err(limit().into());
        }
        Ok(())
    }
    fn step(&mut self) -> Result<(), PublicError> {
        // At most four complete adjacency passes for an admitted eight-million
        // edge graph. Callback expansion cannot make live loops unbounded.
        self.steps = self.steps.checked_add(1).ok_or_else(limit)?;
        if self.steps > 32_000_000 {
            return Err(limit().into());
        }
        Ok(())
    }
    fn nodes(&mut self) -> Result<R::Value, PublicError> {
        self.field("nodes")
    }
    fn count(&mut self, list: &R::Value) -> Result<usize, PublicError> {
        let v = self.read.get(list, "length")?;
        let n = self
            .read
            .number(&v)
            .ok_or_else(|| invalid("Mindmap collections must be Arrays"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(invalid("Mindmap collections must be Arrays").into());
        }
        if n > 1_000_000. {
            return Err(limit().into());
        }
        Ok(n as usize)
    }
    fn node(&mut self, list: &R::Value, index: usize) -> Result<R::Value, PublicError> {
        self.read.index(list, index)
    }
    fn children(&mut self, node: &R::Value) -> Result<R::Value, PublicError> {
        self.read.get(node, "children")
    }
    fn index(&mut self, node: &R::Value, bound: usize) -> Result<usize, PublicError> {
        let v = self.read.get(node, "id")?;
        let n = self
            .read
            .number(&v)
            .ok_or_else(|| invalid("Mindmap child ID must be a native node index"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n >= bound as f64 {
            return Err(invalid("Mindmap child ID must be a native node index").into());
        }
        Ok(n as usize)
    }
    fn text(
        &mut self,
        node: &R::Value,
        key: &'static str,
        optional: bool,
    ) -> Result<Option<String>, PublicError> {
        let v = self.read.get(node, key)?;
        if optional
            && (self.read.kind(&v) == LiveValueKind::Undefined
                || (key == "class" && self.read.kind(&v) == LiveValueKind::Null))
        {
            Ok(None)
        } else {
            self.string(node, key, &v).map(Some)
        }
    }
    fn node_type(&mut self, node: &R::Value) -> Result<u8, PublicError> {
        let v = self.read.get(node, "type")?;
        if self.read.kind(&v) == LiveValueKind::String {
            let text = self.read.string_units(&v)?.unwrap();
            return match text.well_formed().as_deref() {
                Some("0") => Ok(0),
                Some("1") => Ok(1),
                Some("2") => Ok(2),
                Some("3") => Ok(3),
                Some("4") => Ok(4),
                Some("5") => Ok(5),
                Some("6") => Ok(6),
                Some("length" | "constructor" | "__proto__" | "map" | "toString") => {
                    Err(invalid("Mindmap shape-array prototype lookup is not implemented").into())
                }
                _ => Err(invalid("Mindmap non-number shape coercion is not implemented").into()),
            };
        }
        Ok(self
            .read
            .number(&v)
            .filter(|n| n.fract() == 0. && (1.0..=6.0).contains(n))
            .map_or(0, |n| n as u8))
    }

    fn section(&mut self, node: &R::Value) -> Result<Option<usize>, PublicError> {
        let v = self.read.get(node, "section")?;
        if matches!(
            self.read.kind(&v),
            LiveValueKind::Null | LiveValueKind::Undefined
        ) {
            return Ok(None);
        }
        let n = self
            .read
            .number(&v)
            .ok_or_else(|| invalid("Mindmap section coercion is not implemented"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n >= usize::MAX as f64 {
            return Err(invalid("Mindmap section must be a native palette index").into());
        }
        Ok(Some(n as usize))
    }
    fn title(&mut self) -> Result<Option<String>, PublicError> {
        let root = self.read.root();
        let v = self.read.get(&root, "title")?;
        if self.read.kind(&v) == LiveValueKind::Undefined {
            Ok(None)
        } else {
            self.string(&root, "title", &v).map(Some)
        }
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    input: &MindmapDetachedInput,
    config: &Object,
    settings: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    if let Some(e) = &input.limit {
        return Err(e.clone().into());
    }
    super::render::render_phases(
        &mut HostPhase {
            read,
            text: HashMap::new(),
            units: 0,
            retained_units: 0,
            steps: 0,
        },
        config,
        settings,
        resources,
    )
}
