//! Pele gantt module (458322a5, MIT, kepano). Pure Rust scheduling/layout; host calendar seam.
mod axis;
mod dates;
mod lexer;
mod model;
mod parser;
mod render;
mod scheduler;
mod tokens;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, ParseContext, PeleError, RenderResources,
    RenderSettings, Rendered, Value,
};
pub use dates::parse_date_utc;
pub use model::{GanttModel, GanttTask, TaskRaw};
#[derive(Default)]
pub struct Gantt;
impl DiagramModule for Gantt {
    type Model = GanttModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Gantt)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<GanttModel, PeleError> {
        let mut m = parse_header(&c);
        parser::parse(c.source, &mut m)?;
        Ok(m)
    }
    fn render(
        &self,
        m: &GanttModel,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(m, c, o, r)
    }
}

mod public_model;
pub use public_model::GanttPublicState;

fn parse_header(c: &ParseContext<'_>) -> GanttModel {
    let mode = c
        .config
        .get("gantt")
        .and_then(Value::as_object)
        .and_then(|c| c.get("displayMode"))
        .and_then(|v| {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap_or_default();
    GanttModel::new(c.title, mode)
}
