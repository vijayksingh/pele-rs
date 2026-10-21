//! Opt-in native presentation data. Rendering and URL policy remain owned by modules.
use super::{CalendarFields, CivilDateTime, PeleError, RenderResources, TextRequest, TrustedSvg};
use serde::{Deserialize, Serialize};

pub const PRESENTATION_VERSION: u32 = 1;
pub const MAX_PRESENTATION_ELEMENTS: usize = 16_384;
pub const MAX_PRESENTATION_TEXT_BYTES: usize = 4 * 1024 * 1024;

/// Per candidate frame; hard ceilings apply even when larger values are requested.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PresentationBudget {
    pub max_elements: usize,
    pub max_text_bytes: usize,
}
impl Default for PresentationBudget {
    fn default() -> Self {
        Self {
            max_elements: 4096,
            max_text_bytes: 1024 * 1024,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PresentationRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
impl PresentationRect {
    pub(crate) fn centered(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x: x - width / 2.,
            y: y - height / 2.,
            width,
            height,
        }
    }
    fn valid(self) -> bool {
        [
            self.x,
            self.y,
            self.width,
            self.height,
            self.x + self.width,
            self.y + self.height,
        ]
        .iter()
        .all(|n| n.is_finite())
            && self.width >= 0.
            && self.height >= 0.
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PresentationText {
    pub id: String,
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PresentationLink {
    pub id: String,
    pub href: String,
    pub internal: bool,
    /// Logical layout rectangle in SVG user units, before SVG decimal serialization.
    /// This is not an exact painted-path/stroke hit contour.
    pub bounds: PresentationRect,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PresentationArtifact {
    pub version: u32,
    pub width: f64,
    pub height: f64,
    pub title: Option<String>,
    pub description: Option<String>,
    /// Plain rendered node/group/edge labels in renderer traversal order.
    pub text: Vec<PresentationText>,
    pub links: Vec<PresentationLink>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PresentationUnavailable {
    UnsupportedDiagram,
    BudgetExceeded,
    InvalidGeometry,
    InvalidText,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", content = "value", rename_all = "camelCase")]
pub enum PresentationOutcome {
    Available(PresentationArtifact),
    Unavailable(PresentationUnavailable),
}

/// Opt in around any typed or mutable engine render. Only successful final renders
/// publish an outcome. A failed render retains the last good outcome; `take()` can
/// clear it before a new attempt when the caller wants attempt-local ownership.
pub struct PresentationCapture<'a> {
    host: &'a mut dyn RenderResources,
    budget: Option<PresentationBudget>,
    outcome: Option<PresentationOutcome>,
}
impl<'a> PresentationCapture<'a> {
    pub fn new(host: &'a mut dyn RenderResources, budget: PresentationBudget) -> Self {
        Self {
            host,
            budget: Some(budget),
            outcome: None,
        }
    }
    pub(crate) fn deferred(host: &'a mut dyn RenderResources) -> Self {
        let budget = host.presentation_budget();
        Self {
            host,
            budget,
            outcome: None,
        }
    }
    pub(crate) fn publish(self) {
        if let Some(outcome) = self.outcome {
            self.host.capture_presentation(outcome);
        }
    }
    pub fn outcome(&self) -> Option<&PresentationOutcome> {
        self.outcome.as_ref()
    }
    pub fn take(&mut self) -> Option<PresentationOutcome> {
        self.outcome.take()
    }
}
impl RenderResources for PresentationCapture<'_> {
    fn javascript_failure(
        &mut self,
        operation: crate::compat::JavaScriptFailure,
    ) -> crate::compat::public::PublicError {
        self.host.javascript_failure(operation)
    }

    fn host_function(
        &mut self,
        reference: u32,
    ) -> Result<std::rc::Rc<dyn super::public::HostFunction>, super::public::PublicError> {
        self.host.host_function(reference)
    }

    fn presentation_budget(&self) -> Option<PresentationBudget> {
        self.budget
    }
    fn capture_presentation(&mut self, outcome: PresentationOutcome) {
        self.outcome = Some(outcome);
    }
    fn image(&mut self, uri: &str) -> Result<super::ImageResource, PeleError> {
        self.host.image(uri)
    }
    fn raw_inline_icon(&mut self, name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        self.host.raw_inline_icon(name)
    }
    fn text_width(&mut self, request: TextRequest<'_>) -> Result<f64, PeleError> {
        self.host.text_width(request)
    }
    fn icon(&mut self, name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        self.host.icon(name)
    }
    fn direct_icon(&mut self, name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        self.host.direct_icon(name)
    }
    fn parse_date(&mut self, input: &str) -> Result<Option<f64>, PeleError> {
        self.host.parse_date(input)
    }
    fn calendar_fields(&mut self, instant_ms: f64) -> Result<Option<CalendarFields>, PeleError> {
        self.host.calendar_fields(instant_ms)
    }
    fn calendar_instant(&mut self, civil: CivilDateTime) -> Result<Option<f64>, PeleError> {
        self.host.calendar_instant(civil)
    }
    fn direct_now_millis(&mut self) -> Result<f64, PeleError> {
        self.host.direct_now_millis()
    }
    fn now_millis(&mut self) -> Result<f64, PeleError> {
        self.host.now_millis()
    }
}

pub(crate) struct FrameBuilder {
    artifact: Option<PresentationArtifact>,
    unavailable: PresentationUnavailable,
    remaining_elements: usize,
    remaining_bytes: usize,
}
impl FrameBuilder {
    pub(crate) fn new(
        budget: PresentationBudget,
        width: f64,
        height: f64,
        title: Option<&str>,
        description: Option<&str>,
    ) -> Self {
        let mut builder = Self {
            artifact: Some(PresentationArtifact {
                version: PRESENTATION_VERSION,
                width,
                height,
                title: None,
                description: None,
                text: vec![],
                links: vec![],
            }),
            unavailable: PresentationUnavailable::BudgetExceeded,
            remaining_elements: budget.max_elements.min(MAX_PRESENTATION_ELEMENTS),
            remaining_bytes: budget.max_text_bytes.min(MAX_PRESENTATION_TEXT_BYTES),
        };
        if !width.is_finite() || !height.is_finite() || width < 0. || height < 0. {
            builder.fail(PresentationUnavailable::InvalidGeometry);
        }
        for (value, is_title) in [(title, true), (description, false)] {
            if let Some(value) = value {
                if let Some(decoded) = builder.decoded(value) {
                    if let Some(artifact) = &mut builder.artifact {
                        if is_title {
                            artifact.title = Some(decoded);
                        } else {
                            artifact.description = Some(decoded);
                        }
                    }
                }
            }
        }
        builder
    }
    fn fail(&mut self, reason: PresentationUnavailable) {
        if self.artifact.take().is_some() {
            self.unavailable = reason;
        }
    }
    fn reserve(&mut self, elements: usize, bytes: usize) -> bool {
        if self.artifact.is_none() {
            return false;
        }
        if elements > self.remaining_elements || bytes > self.remaining_bytes {
            self.fail(PresentationUnavailable::BudgetExceeded);
            return false;
        }
        self.remaining_elements -= elements;
        self.remaining_bytes -= bytes;
        true
    }
    fn decoded(&mut self, text: &str) -> Option<String> {
        // Check before decoding/copying as well as after: no oversized temporary metadata.
        if !self.reserve(0, text.len()) {
            return None;
        }
        match super::source::decode_entities(text) {
            Ok(value) if value.len() <= text.len() => Some(value),
            Ok(value) => {
                if self.reserve(0, value.len() - text.len()) {
                    Some(value)
                } else {
                    None
                }
            }
            Err(_) => {
                self.fail(PresentationUnavailable::InvalidText);
                None
            }
        }
    }
    pub(crate) fn label(&mut self, id: &str, label: &super::text::Label) {
        if self.artifact.is_none() {
            return;
        }
        let Some(bytes) = label
            .lines
            .iter()
            .flatten()
            .try_fold(label.lines.len().saturating_sub(1), |sum, span| {
                sum.checked_add(span.text.len())
            })
        else {
            self.fail(PresentationUnavailable::BudgetExceeded);
            return;
        };
        if bytes == 0 || !self.reserve(1, bytes) {
            return;
        }
        let Some(id) = self.decoded(id) else {
            return;
        };
        let mut text = String::with_capacity(bytes);
        for (i, line) in label.lines.iter().enumerate() {
            if i > 0 {
                text.push('\n');
            }
            for span in line {
                text.push_str(&span.text);
            }
        }
        if let Some(artifact) = &mut self.artifact {
            artifact.text.push(PresentationText { id, text });
        }
    }
    pub(crate) fn link(&mut self, id: &str, href: &str, internal: bool, bounds: PresentationRect) {
        if !bounds.valid() {
            self.fail(PresentationUnavailable::InvalidGeometry);
            return;
        }
        if !self.reserve(1, href.len()) {
            return;
        }
        let Some(id) = self.decoded(id) else {
            return;
        };
        if let Some(artifact) = &mut self.artifact {
            artifact.links.push(PresentationLink {
                id,
                href: href.into(),
                internal,
                bounds,
            });
        }
    }
    pub(crate) fn finish(self) -> PresentationOutcome {
        match self.artifact {
            Some(artifact) => PresentationOutcome::Available(artifact),
            None => PresentationOutcome::Unavailable(self.unavailable),
        }
    }
}
