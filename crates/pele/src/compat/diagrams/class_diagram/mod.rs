//! Class diagram module ported from Pele src/diagrams/class, 458322a5 (MIT, kepano).
mod graph;
mod lexer;
mod markers;
mod model;
mod parser;
mod render;
mod tokens;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, OutputLimit, ParseContext, PeleError,
    RenderResources, RenderSettings, Rendered,
};
pub use model::{
    ClassInterface, ClassMember, ClassModel, ClassNode, ClassNote, ClassRelation, NamespaceNode,
    RelationEnd, StyleClass,
};
#[derive(Default)]
pub struct Class;
impl DiagramModule for Class {
    type Model = ClassModel;
    fn validate_model_projection(&self, m: &ClassModel) -> Result<(), PeleError> {
        let mut depths = std::collections::HashMap::new();
        for (id, ns) in &m.namespaces {
            let depth = ns
                .parent
                .as_deref()
                .and_then(|p| depths.get(p))
                .copied()
                .unwrap_or(0usize)
                + 1;
            if depth >= 36 {
                return Err(PeleError::new(
                    "Class model exceeds safe JSON projection depth",
                    crate::compat::ErrorCode::Limit,
                ));
            }
            depths.insert(id.as_str(), depth);
        }
        Ok(())
    }
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Class)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<ClassModel, PeleError> {
        let mut db = ClassModel::new(c.title);
        parser::parse(c.source, &mut db)?;
        Ok(db)
    }
    fn render(
        &self,
        m: &ClassModel,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(m, c, o, r)
    }
    fn render_raw(
        &self,
        m: &ClassModel,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render_exact(m, c, o, r)
    }
    fn render_with_output_limit(
        &self,
        m: &ClassModel,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
        _limit: OutputLimit,
    ) -> Result<Rendered, PeleError> {
        render::render_exact(m, c, o, r)
    }
}

mod public_model;
pub use public_model::ClassPublicState;

mod live_phase;
pub(crate) use live_phase::{render_detached, ClassDetachedInput};
