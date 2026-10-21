//! Live Pie input phases over captured iterable values, never document borrows.
use super::render::PhaseInput;
use crate::compat::{
    public::{JsString, LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};

pub(crate) struct PieDetachedInput;
pub(crate) fn render_detached<R: LiveRenderRead>(
    _: &PieDetachedInput,
    config: &Object,
    settings: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    super::render::render_phases(&mut HostPhase(read), config, settings, resources)
}
struct HostPhase<'a, R>(&'a mut R);
fn invalid(message: &str) -> PublicError {
    let mut error = PeleError::new(message, ErrorCode::Semantic);
    error.diagram_type = Some("pie".into());
    error.into()
}
fn text(value: JsString) -> Result<String, PublicError> {
    value
        .well_formed()
        .ok_or_else(|| invalid("UTF-16 Pie text rendering is not implemented"))
}
impl<R: LiveRenderRead> HostPhase<'_, R> {
    fn field(&mut self, key: &str) -> Result<R::Value, PublicError> {
        let root = self.0.root();
        self.0.get(&root, key)
    }
    fn strict_text(
        &mut self,
        value: &R::Value,
        receiver: &'static str,
    ) -> Result<String, PublicError> {
        text(self.0.require_string(
            value,
            LiveStringUse {
                method: "includes",
                expression_receiver: receiver,
            },
        )?)
    }
}
impl<R: LiveRenderRead> PhaseInput for HostPhase<'_, R> {
    type Error = PublicError;
    type Entry = R::Value;
    type Entries = Vec<R::Value>;
    type Text = String;
    fn entries(&mut self) -> Result<Self::Entries, PublicError> {
        let sections = self.field("sections")?;
        // Every ordinary Map entry accounts for two atoms in the existing work
        // budget. Bound capture before retaining a count-proportional vector.
        self.0.snapshot_iterable(&sections, 4_000_000)
    }
    fn value(&mut self, entry: &R::Value) -> Result<f64, PublicError> {
        let value = self.0.index(entry, 1)?;
        self.0
            .number(&value)
            .ok_or_else(|| invalid("Pie section values must be numbers"))
    }
    fn legend_label(&mut self, entry: &R::Value) -> Result<String, PublicError> {
        let key = self.0.index(entry, 0)?;
        let value = self.0.index(entry, 1)?;
        let show = self.field("showData")?;
        if self.0.truthy(&show) {
            let key = text(self.0.to_string(&key)?)?;
            let value = text(self.0.to_string(&value)?)?;
            Ok(format!("{key} [{value}]"))
        } else {
            self.strict_text(&key, "t")
        }
    }
    fn data_id(&mut self, entry: &R::Value) -> Result<String, PublicError> {
        let key = self.0.index(entry, 0)?;
        self.strict_text(&key, "t")
    }
    fn title(&mut self) -> Result<Option<String>, PublicError> {
        let value = self.field("title")?;
        if self.0.kind(&value) == LiveValueKind::Undefined {
            return Ok(None);
        }
        self.strict_text(&value, "t").map(Some)
    }
    fn accessibility(&mut self, description: bool) -> Result<Option<String>, PublicError> {
        let key = if description { "accDescr" } else { "accTitle" };
        let value = self.field(key)?;
        if !self.0.truthy(&value) {
            return Ok(None);
        }
        let value = self.field(key)?;
        self.strict_text(&value, "e").map(Some)
    }
}
