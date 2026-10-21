//! Fishbone phases retain native-selected host node identities across callbacks.
use super::render::PhaseInput;
use crate::compat::{
    public::{LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
pub(crate) struct IshikawaDetachedInput {
    limit: Option<PeleError>,
}
impl IshikawaDetachedInput {
    pub(super) fn new(error: Option<&PeleError>) -> Self {
        Self {
            limit: error
                .filter(|error| error.code == ErrorCode::Limit)
                .cloned(),
        }
    }
}
struct HostPhase<'a, R> {
    read: &'a mut R,
    text: std::collections::HashMap<(u64, &'static str), usize>,
    units: usize,
}
fn invalid(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("ishikawa".into());
    e
}
fn limit() -> PeleError {
    let mut e = invalid("Ishikawa public operation exceeds native work budget");
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
        field: &'static str,
        value: &R::Value,
        receiver: &'static str,
    ) -> Result<String, PublicError> {
        let text = self.read.require_string(
            value,
            LiveStringUse {
                method: "includes",
                expression_receiver: receiver,
            },
        )?;
        let identity = self
            .read
            .object_identity(owner)?
            .ok_or_else(|| invalid("Ishikawa active record requires an object"))?;
        let previous = self.text.get(&(identity, field)).copied().unwrap_or(0);
        if text.0.len() > previous {
            self.units = self
                .units
                .checked_add(text.0.len() - previous)
                .ok_or_else(limit)?;
            if self.units > 16_000_000 {
                return Err(limit().into());
            }
            self.text.insert((identity, field), text.0.len());
        }
        text.well_formed().ok_or_else(|| {
            invalid("Ishikawa rendering of lone UTF-16 surrogates is not implemented").into()
        })
    }
}
impl<R: LiveRenderRead> PhaseInput for HostPhase<'_, R> {
    type Error = PublicError;
    type Node = R::Value;
    type Children = R::Value;
    fn begin_draw(&mut self) {
        self.text.clear();
        self.units = 0;
    }
    fn title(&mut self) -> Result<Option<String>, PublicError> {
        let value = self.field("title")?;
        if self.read.kind(&value) == LiveValueKind::Undefined {
            Ok(None)
        } else {
            let owner = self.read.root();
            self.string(&owner, "title", &value, "t").map(Some)
        }
    }
    fn root(&mut self, repeat: bool) -> Result<Option<R::Value>, PublicError> {
        let root = self.field("root")?;
        if !self.read.truthy(&root) {
            return Ok(None);
        }
        Ok(Some(if repeat { self.field("root")? } else { root }))
    }
    fn text(&mut self, node: &R::Value) -> Result<String, PublicError> {
        let value = self.read.get(node, "text")?;
        self.string(node, "text", &value, "t")
    }
    fn children(&mut self, node: &R::Value) -> Result<R::Value, PublicError> {
        self.read.get(node, "children")
    }
    fn count(&mut self, children: &R::Value) -> Result<usize, PublicError> {
        let value = self.read.get(children, "length")?;
        let n = self
            .read
            .number(&value)
            .ok_or_else(|| invalid("Ishikawa children must be Arrays"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(invalid("Ishikawa children must be Arrays").into());
        }
        if n > 1_000_000. {
            let mut e = invalid("Ishikawa public operation exceeds native work budget");
            e.code = ErrorCode::Limit;
            return Err(e.into());
        }
        Ok(n as usize)
    }
    fn has_child(&mut self, children: &R::Value, index: usize) -> Result<bool, PublicError> {
        self.read.has_index(children, index)
    }
    fn child(&mut self, children: &R::Value, index: usize) -> Result<R::Value, PublicError> {
        self.read.index(children, index)
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    input: &IshikawaDetachedInput,
    _: &Object,
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
            text: std::collections::HashMap::new(),
            units: 0,
        },
        settings,
        resources,
    )
}
