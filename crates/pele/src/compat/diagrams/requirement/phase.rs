//! Requirement descriptor capture and field-read phases shared by both owners.
use super::model::{Requirement, RequirementElement, RequirementModel};
use crate::compat::PeleError;
use std::collections::HashSet;
pub(super) struct CapturedNode<N, S> {
    pub id: String,
    pub requirement: bool,
    pub record: N,
    pub styles: S,
    pub classes: String,
}
pub(super) struct CapturedEdge {
    pub id: String,
    pub start: Option<String>,
    pub end: Option<String>,
    pub kind: String,
    pub contains: bool,
}
pub(super) struct Scene<N, S> {
    pub nodes: Vec<CapturedNode<N, S>>,
    pub edges: Vec<CapturedEdge>,
    pub direction: String,
}
pub(super) trait PhaseInput {
    type Node: Clone;
    type Styles: Clone;
    type Error: From<PeleError>;
    fn direction(&mut self) -> Result<String, Self::Error>;
    fn capture(&mut self) -> Result<Scene<Self::Node, Self::Styles>, Self::Error>;
    fn styles(&mut self, value: &Self::Styles) -> Result<Vec<String>, Self::Error>;
    fn kind(&mut self, node: &Self::Node) -> Result<String, Self::Error>;
    fn fields(&mut self, node: &Self::Node) -> Result<Vec<(&'static str, String)>, Self::Error>;
    fn metadata(&mut self, key: &'static str) -> Result<Option<String>, Self::Error>;
}
#[derive(Clone)]
pub(super) enum TypedNode<'a> {
    Requirement(&'a Requirement),
    Element(&'a RequirementElement),
}
pub(super) struct Typed<'a>(pub &'a RequirementModel);
impl<'a> PhaseInput for Typed<'a> {
    type Node = TypedNode<'a>;
    type Styles = &'a [String];
    type Error = PeleError;
    fn direction(&mut self) -> Result<String, PeleError> {
        Ok(self.0.direction.clone())
    }
    fn capture(&mut self) -> Result<Scene<Self::Node, Self::Styles>, PeleError> {
        let m = self.0;
        let mut nodes = Vec::new();
        for (_, v) in &m.requirements {
            nodes.push(CapturedNode {
                id: v.name.clone(),
                requirement: true,
                record: TypedNode::Requirement(v),
                styles: v.css_styles.as_slice(),
                classes: v.classes.join(" "),
            });
        }
        for (_, v) in &m.elements {
            nodes.push(CapturedNode {
                id: v.name.clone(),
                requirement: false,
                record: TypedNode::Element(v),
                styles: v.css_styles.as_slice(),
                classes: v.classes.join(" "),
            });
        }
        let keys: HashSet<&str> = m
            .requirements
            .iter()
            .map(|(k, _)| k.as_str())
            .chain(m.elements.iter().map(|(k, _)| k.as_str()))
            .collect();
        let edges = m
            .relations
            .iter()
            .enumerate()
            .map(|(i, e)| CapturedEdge {
                id: format!("{}-{}-{i}", e.src, e.dst),
                start: keys.contains(e.src.as_str()).then(|| e.src.clone()),
                end: keys.contains(e.dst.as_str()).then(|| e.dst.clone()),
                kind: e.relation_type.clone(),
                contains: e.relation_type == "contains",
            })
            .collect();
        Ok(Scene {
            nodes,
            edges,
            direction: m.direction.clone(),
        })
    }
    fn styles(&mut self, value: &Self::Styles) -> Result<Vec<String>, PeleError> {
        Ok(value.to_vec())
    }
    fn kind(&mut self, node: &Self::Node) -> Result<String, PeleError> {
        Ok(match node {
            TypedNode::Requirement(v) => v.requirement_type.clone(),
            TypedNode::Element(_) => "Element".into(),
        })
    }
    fn fields(&mut self, node: &Self::Node) -> Result<Vec<(&'static str, String)>, PeleError> {
        Ok(match node {
            TypedNode::Requirement(v) => vec![
                ("ID", v.requirement_id.clone()),
                ("Text", v.text.clone()),
                ("Risk", v.risk.clone()),
                ("Verification", v.verify_method.clone()),
            ],
            TypedNode::Element(v) => vec![
                ("Type", v.element_type.clone()),
                ("Doc Ref", v.doc_ref.clone()),
            ],
        })
    }
    fn metadata(&mut self, key: &'static str) -> Result<Option<String>, PeleError> {
        Ok(match key {
            "title" => &self.0.title,
            "accTitle" => &self.0.acc_title,
            _ => &self.0.acc_descr,
        }
        .clone())
    }
}
