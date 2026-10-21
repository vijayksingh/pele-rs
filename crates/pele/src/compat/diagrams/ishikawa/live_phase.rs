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
struct HostPhase<'a, R>(&'a mut R);
fn invalid(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("ishikawa".into());
    e
}
impl<R: LiveRenderRead> HostPhase<'_, R> {
    fn field(&mut self, key: &str) -> Result<R::Value, PublicError> {
        let root = self.0.root();
        self.0.get(&root, key)
    }
    fn string(&mut self, value: &R::Value, receiver: &'static str) -> Result<String, PublicError> {
        self.0
            .require_string(
                value,
                LiveStringUse {
                    method: "includes",
                    expression_receiver: receiver,
                },
            )?
            .well_formed()
            .ok_or_else(|| {
                invalid("Ishikawa rendering of lone UTF-16 surrogates is not implemented").into()
            })
    }
}
impl<R: LiveRenderRead> PhaseInput for HostPhase<'_, R> {
    type Error = PublicError;
    type Node = R::Value;
    type Children = R::Value;
    fn title(&mut self) -> Result<Option<String>, PublicError> {
        let value = self.field("title")?;
        if self.0.kind(&value) == LiveValueKind::Undefined {
            Ok(None)
        } else {
            self.string(&value, "t").map(Some)
        }
    }
    fn root(&mut self, repeat: bool) -> Result<Option<R::Value>, PublicError> {
        let root = self.field("root")?;
        if !self.0.truthy(&root) {
            return Ok(None);
        }
        Ok(Some(if repeat { self.field("root")? } else { root }))
    }
    fn text(&mut self, node: &R::Value) -> Result<String, PublicError> {
        let value = self.0.get(node, "text")?;
        self.string(&value, "t")
    }
    fn children(&mut self, node: &R::Value) -> Result<R::Value, PublicError> {
        self.0.get(node, "children")
    }
    fn count(&mut self, children: &R::Value) -> Result<usize, PublicError> {
        let value = self.0.get(children, "length")?;
        let n = self
            .0
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
        self.0.has_index(children, index)
    }
    fn child(&mut self, children: &R::Value, index: usize) -> Result<R::Value, PublicError> {
        self.0.index(children, index)
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
    super::render::render_phases(&mut HostPhase(read), settings, resources)
}
