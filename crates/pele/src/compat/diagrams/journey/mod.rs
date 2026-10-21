//! Pele diagrams/journey/{index,db,lexer,parser,render}.ts, 458322a5 (MIT, kepano).
mod live_phase;
pub(crate) use live_phase::{render_detached, JourneyDetachedInput};
mod parser;
mod public_model;
pub use public_model::JourneyPublicState;
mod render;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, ParseContext, PeleError, RenderResources,
    RenderSettings, Rendered, Value,
};
use serde::{Serialize, Serializer};
#[derive(Clone, Debug, Serialize)]
pub struct JourneyTask {
    pub task: String,
    #[serde(serialize_with = "number")]
    pub score: f64,
    pub people: Vec<String>,
    pub section: String,
}
fn number<S: Serializer>(n: &f64, s: S) -> Result<S::Ok, S::Error> {
    Value::Number(*n).serialize(s)
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JourneyModel {
    #[serde(rename = "type")]
    pub diagram_type: DiagramType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acc_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acc_descr: Option<String>,
    pub sections: Vec<String>,
    pub tasks: Vec<JourneyTask>,
    pub current_section: String,
}
impl JourneyModel {
    pub fn actors(&self) -> Vec<String> {
        let unique: std::collections::HashSet<&str> = self
            .tasks
            .iter()
            .flat_map(|t| t.people.iter().map(String::as_str))
            .collect();
        let mut names: Vec<_> = unique.into_iter().map(str::to_string).collect();
        names.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
        names.dedup();
        names
    }
}
#[derive(Default)]
pub struct Journey;
impl DiagramModule for Journey {
    type Model = JourneyModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Journey)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<Self::Model, PeleError> {
        parser::parse(c.source, c.title)
    }
    fn render(
        &self,
        m: &Self::Model,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(m, c, o, r)
    }
}

impl JourneyModel {
    /// Empty class state shared by parser population and native construction.
    pub(super) fn empty(title: Option<&str>) -> Self {
        Self {
            diagram_type: DiagramType::Journey,
            title: title.map(str::to_owned),
            acc_title: None,
            acc_descr: None,
            sections: Vec::new(),
            tasks: Vec::new(),
            current_section: String::new(),
        }
    }
}
