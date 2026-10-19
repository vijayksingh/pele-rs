//! Mutable Pie codec. The graph retains ordinary own fields and Map identity.
use super::{Pie, PieModel, PieSection};
use crate::compat::{
    public::{Atom, GraphWriter, ModelGraph, Node, PublicDiagramModule},
    DiagramModule, DiagramType, ErrorCode, Object, PeleError, RenderResources, RenderSettings,
    Rendered,
};

pub struct PiePublicState {
    graph: ModelGraph,
}
fn invalid(message: &str) -> PeleError {
    let mut error = PeleError::new(message, ErrorCode::Semantic);
    error.diagram_type = Some("pie".into());
    error
}
fn text(graph: &ModelGraph, value: &Atom) -> Result<String, PeleError> {
    graph
        .text(value)?
        .well_formed()
        .ok_or_else(|| invalid("UTF-16 Pie text rendering is not implemented"))
}
fn optional_text(graph: &ModelGraph, key: &str) -> Result<Option<String>, PeleError> {
    match graph.field(key)? {
        None | Some(Atom::Undefined | Atom::Null) => Ok(None),
        Some(value) => text(graph, value).map(Some),
    }
}
fn typed(graph: &ModelGraph) -> Result<PieModel, PeleError> {
    let sections = match graph.field("sections")? {
        Some(Atom::Ref(id)) => match graph.nodes.get(*id as usize) {
            Some(Node::Map { entries, .. }) => entries,
            _ => return Err(invalid("Pie sections must be a Map")),
        },
        _ => return Err(invalid("Pie sections must be a Map")),
    };
    let sections = sections
        .iter()
        .map(|(key, value)| {
            let Atom::Number(value) = value else {
                return Err(invalid("Pie section values must be numbers"));
            };
            Ok(PieSection {
                label: text(graph, key)?,
                value: *value,
            })
        })
        .collect::<Result<Vec<_>, PeleError>>()?;
    let show_data = match graph.field("showData")? {
        None | Some(Atom::Undefined | Atom::Null) => false,
        Some(Atom::Bool(v)) => *v,
        Some(Atom::Number(v)) => *v != 0.0 && !v.is_nan(),
        Some(Atom::String(id)) => !graph.strings[*id as usize].0.is_empty(),
        _ => true,
    };
    Ok(PieModel {
        diagram_type: DiagramType::Pie,
        title: optional_text(graph, "title")?,
        acc_title: optional_text(graph, "accTitle")?,
        acc_descr: optional_text(graph, "accDescr")?,
        show_data,
        sections,
    })
}
impl PublicDiagramModule for Pie {
    type PublicState = PiePublicState;
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        Ok(PiePublicState { graph })
    }
    fn create_public(&self, model: &PieModel) -> Result<PiePublicState, PeleError> {
        let mut out = GraphWriter::default();
        let mut entries = Vec::with_capacity(model.sections.len());
        for section in &model.sections {
            let label = out.string(section.label.as_str());
            entries.push((Atom::String(label), Atom::Number(section.value)));
        }
        let sections = out.node(Node::Map {
            entries,
            properties: vec![],
        });
        let pie = out.string("pie");
        let mut fields = vec![out.property("type", Atom::String(pie))];
        for (key, value) in [
            ("title", &model.title),
            ("accTitle", &model.acc_title),
            ("accDescr", &model.acc_descr),
        ] {
            let atom = if let Some(value) = value {
                Atom::String(out.string(value.as_str()))
            } else {
                Atom::Undefined
            };
            fields.push(out.property(key, atom));
        }
        fields.push(out.property("showData", Atom::Bool(model.show_data)));
        fields.push(out.property("sections", Atom::Ref(sections)));
        let root = out.node(Node::Object { properties: fields });
        Ok(PiePublicState {
            graph: out.finish(Atom::Ref(root))?,
        })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, state: &PiePublicState) -> Result<ModelGraph, PeleError> {
        Ok(state.graph.clone())
    }
    fn apply_public(&self, state: &mut PiePublicState, graph: ModelGraph) -> Result<(), PeleError> {
        state.graph = graph;
        Ok(())
    }
    fn render_public(
        &self,
        state: &mut PiePublicState,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render(&typed(&state.graph)?, config, options, host)
    }
}
