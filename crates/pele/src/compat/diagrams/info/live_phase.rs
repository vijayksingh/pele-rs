//! Info reads captured title before layout and live accessibility after callbacks.
use super::render::{render_phases, PhaseInput};
use crate::compat::{
    public::{LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};

// No projection is needed: all model fields are read at their source phases.
pub(crate) struct InfoDetachedInput;
struct HostPhase<'a, R>(&'a mut R);
impl<R: LiveRenderRead> HostPhase<'_, R> {
    fn field(&mut self, key: &str) -> Result<R::Value, PublicError> {
        let root = self.0.root();
        self.0.get(&root, key)
    }
    fn text(&mut self, value: &R::Value, receiver: &'static str) -> Result<String, PublicError> {
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
                let mut error = PeleError::new(
                    "UTF-16 Info text rendering is not implemented",
                    ErrorCode::Semantic,
                );
                error.diagram_type = Some("info".into());
                error.into()
            })
    }
}
impl<R: LiveRenderRead> PhaseInput for HostPhase<'_, R> {
    type Error = PublicError;
    fn title(&mut self) -> Result<Option<String>, PublicError> {
        let value = self.field("title")?;
        if self.0.kind(&value) == LiveValueKind::Undefined {
            return Ok(None);
        }
        self.text(&value, "t").map(Some)
    }
    fn accessibility(&mut self, description: bool) -> Result<Option<String>, PublicError> {
        let key = if description { "accDescr" } else { "accTitle" };
        let value = self.field(key)?;
        if !self.0.truthy(&value) {
            return Ok(None);
        }
        let value = self.field(key)?;
        self.text(&value, "e").map(Some)
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    _seed: &InfoDetachedInput,
    _config: &Object,
    settings: &RenderSettings,
    reader: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    render_phases(&mut HostPhase(reader), settings, resources)
}
