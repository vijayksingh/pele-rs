//! Quadrant phase reads over job-owned host identities. No document borrow is
//! retained across a resource callback; layout and paint stay in render.rs.
use super::{
    render::{Axis, PhaseInput, PointGeometry},
    PointStyle, QuadrantText,
};
use crate::compat::{
    public::{JsString, LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
pub(crate) struct QuadrantDetachedInput {
    limit: Option<PeleError>,
}
impl QuadrantDetachedInput {
    pub(super) fn new(error: Option<PeleError>) -> Self {
        Self {
            limit: error.filter(|e| e.code == ErrorCode::Limit),
        }
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    seed: &QuadrantDetachedInput,
    c: &Object,
    o: &RenderSettings,
    read: &mut R,
    r: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    if let Some(error) = &seed.limit {
        return Err(error.clone().into());
    }
    super::render::render_phases(&mut HostPhase(read), c, o, r)
}
struct HostPhase<'a, R>(&'a mut R);
fn invalid(message: &str) -> PublicError {
    let mut error = PeleError::new(message, ErrorCode::Semantic);
    error.diagram_type = Some("quadrantChart".into());
    error.into()
}
fn text(value: JsString) -> Result<String, PublicError> {
    value
        .well_formed()
        .ok_or_else(|| invalid("UTF-16 Quadrant text rendering is not implemented"))
}
impl<R: LiveRenderRead> HostPhase<'_, R> {
    fn field(&mut self, key: &str) -> Result<R::Value, PublicError> {
        let root = self.0.root();
        self.0.get(&root, key)
    }
    fn nullish(&self, value: &R::Value) -> bool {
        matches!(
            self.0.kind(value),
            LiveValueKind::Null | LiveValueKind::Undefined
        )
    }
    fn strict_text(
        &mut self,
        value: &R::Value,
        method: &'static str,
        receiver: &'static str,
    ) -> Result<String, PublicError> {
        text(self.0.require_string(
            value,
            LiveStringUse {
                method,
                expression_receiver: receiver,
            },
        )?)
    }
    fn length(&mut self, value: &R::Value) -> Result<usize, PublicError> {
        let length = self.0.get(value, "length")?;
        let number = self
            .0
            .number(&length)
            .ok_or_else(|| invalid("Invalid array length"))?;
        if !number.is_finite() || number < 0. || number.fract() != 0. || number > u32::MAX as f64 {
            return Err(invalid("Invalid array length"));
        }
        if number > 1_000_000. {
            return Err(PeleError::new(
                "Quadrant collection exceeds native work budget",
                ErrorCode::Limit,
            )
            .into());
        }
        Ok(number as usize)
    }
    fn number_field(&mut self, point: &R::Value, key: &str) -> Result<f64, PublicError> {
        let value = self.0.get(point, key)?;
        self.0
            .number(&value)
            .ok_or_else(|| invalid("Quadrant point coordinates must be numbers"))
    }
    fn style(
        &mut self,
        point: &R::Value,
        shared: &R::Value,
        key: &str,
    ) -> Result<Option<R::Value>, PublicError> {
        let own = self.0.get(point, key)?;
        if !self.nullish(&own) {
            return Ok(Some(own));
        }
        if self.nullish(shared) {
            return Ok(None);
        }
        let value = self.0.get(shared, key)?;
        Ok((!self.nullish(&value)).then_some(value))
    }
    fn style_text(&mut self, value: Option<R::Value>) -> Result<Option<String>, PublicError> {
        match value {
            Some(v) => self.strict_text(&v, "startsWith", "s").map(Some),
            None => Ok(None),
        }
    }
}
impl<R: LiveRenderRead> PhaseInput for HostPhase<'_, R> {
    type Error = PublicError;
    type Text = R::Value;
    type Point = R::Value;
    type Points = R::Value;
    fn has_points(&mut self) -> Result<bool, PublicError> {
        let points = self.field("points")?;
        Ok(self.length(&points)? > 0)
    }
    fn title(&mut self) -> Result<Option<String>, PublicError> {
        let value = self.field("title")?;
        if self.0.kind(&value) == LiveValueKind::Undefined {
            return Ok(None);
        }
        self.strict_text(&value, "includes", "t").map(Some)
    }
    fn axis_has_text(&mut self, axis: Axis) -> Result<bool, PublicError> {
        let value = self.axis(axis)?;
        if self.nullish(&value) {
            return Ok(false);
        }
        let text = self.0.get(&value, "text")?;
        Ok(self.0.truthy(&text))
    }
    fn axis(&mut self, axis: Axis) -> Result<R::Value, PublicError> {
        self.field(match axis {
            Axis::XLeft => "xAxisLeft",
            Axis::XRight => "xAxisRight",
            Axis::YBottom => "yAxisBottom",
            Axis::YTop => "yAxisTop",
        })
    }
    fn present(&self, value: &R::Value) -> bool {
        self.0.truthy(value)
    }
    fn text(&mut self, value: &R::Value) -> Result<Option<QuadrantText>, PublicError> {
        if self.nullish(value) {
            return Ok(None);
        }
        let label = self.0.get(value, "text")?;
        let kind = self.0.get(value, "type")?;
        let markdown = self
            .0
            .string_units(&kind)?
            .is_some_and(|v| v == JsString::from("markdown"));
        // The common label helper treats undefined as an empty label, but all
        // present text values retain its strict string operation/error lane.
        if self.0.kind(&label) == LiveValueKind::Undefined {
            return Ok(None);
        }
        Ok(Some(QuadrantText {
            text: self.strict_text(&label, "includes", "t")?,
            label_type: if markdown { "markdown" } else { "text" }.into(),
        }))
    }
    fn quadrant(&mut self, index: usize) -> Result<R::Value, PublicError> {
        let values = self.field("quadrants")?;
        self.0.index(&values, index)
    }
    fn points(&mut self) -> Result<R::Value, PublicError> {
        self.field("points")
    }
    fn next_point(
        &mut self,
        points: &R::Value,
        index: usize,
    ) -> Result<Option<R::Value>, PublicError> {
        if index >= self.length(points)? {
            return Ok(None);
        }
        self.0.index(points, index).map(Some)
    }
    fn geometry(&mut self, point: &R::Value) -> Result<Option<PointGeometry>, PublicError> {
        // Keep the native renderer's declared numeric domain; arbitrary host
        // coercion objects are not silently converted into a typed projection.
        if !(self.number_field(point, "x")? >= 0.
            && self.number_field(point, "x")? <= 1.
            && self.number_field(point, "y")? >= 0.
            && self.number_field(point, "y")? <= 1.)
        {
            return Ok(None);
        }
        let classes = self.field("classes")?;
        let class_name = self.0.get(point, "className")?;
        let shared = self.0.map_get(&classes, &class_name)?;
        let radius = self.style(point, &shared, "radius")?;
        let radius = radius
            .map(|v| {
                self.0
                    .number(&v)
                    .ok_or_else(|| invalid("Quadrant point radius must be a number"))
            })
            .transpose()?;
        let color = self.style(point, &shared, "color")?;
        let stroke_color = self.style(point, &shared, "strokeColor")?;
        let stroke_width = self.style(point, &shared, "strokeWidth")?;
        let color = self.style_text(color)?;
        let stroke_color = self.style_text(stroke_color)?;
        let stroke_width = self.style_text(stroke_width)?;
        let x = self.number_field(point, "x")?;
        let y = self.number_field(point, "y")?;
        Ok(Some(PointGeometry {
            x,
            y,
            style: PointStyle {
                radius,
                color,
                stroke_color,
                stroke_width,
                order: Vec::new(),
            },
        }))
    }
    fn point_text(&mut self, point: &R::Value) -> Result<R::Value, PublicError> {
        self.0.get(point, "text")
    }
    fn point_identity(&mut self, point: &R::Value) -> Result<(String, String), PublicError> {
        let class_name = self.0.get(point, "className")?;
        let class_name = self.strict_text(&class_name, "replace", "e")?;
        let label = self.0.get(point, "text")?;
        let label = self.0.get(&label, "text")?;
        let label = self.strict_text(&label, "replace", "e")?;
        Ok((class_name, label))
    }
    fn accessibility(&mut self, description: bool) -> Result<Option<String>, PublicError> {
        let key = if description { "accDescr" } else { "accTitle" };
        let value = self.field(key)?;
        if !self.0.truthy(&value) {
            return Ok(None);
        }
        let value = self.field(key)?;
        self.strict_text(&value, "includes", "e").map(Some)
    }
}
