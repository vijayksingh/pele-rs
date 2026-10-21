//! Pele architecture/db.ts, 458322a5 (MIT, kepano).
use super::parser::Ast;
use crate::compat::{ErrorCode, PeleError};
use serde::{ser::SerializeMap, Serialize};
use std::collections::HashMap;
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "in", skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
pub struct ArchGroup {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "in", skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchEdge {
    pub lhs_id: String,
    pub lhs_dir: char,
    pub lhs_into: bool,
    pub lhs_group: bool,
    pub rhs_id: String,
    pub rhs_dir: char,
    pub rhs_into: bool,
    pub rhs_group: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
pub struct ArchLayoutHint {
    pub direction: String,
    pub members: Vec<String>,
}
#[derive(Clone, Debug)]
pub struct ArchitectureModel {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub nodes: Vec<(String, ArchNode)>,
    pub groups: Vec<(String, ArchGroup)>,
    pub edges: Vec<ArchEdge>,
    pub layout_hints: Vec<ArchLayoutHint>,
    pub registered: Vec<(String, String)>,
}
impl Serialize for ArchitectureModel {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut m = s.serialize_map(None)?;
        m.serialize_entry("type", "architecture")?;
        for (k, v) in [
            ("title", &self.title),
            ("accTitle", &self.acc_title),
            ("accDescr", &self.acc_descr),
        ] {
            if let Some(v) = v {
                m.serialize_entry(k, v)?
            }
        }
        m.serialize_entry("nodes", &self.nodes)?;
        m.serialize_entry("groups", &self.groups)?;
        m.serialize_entry("edges", &self.edges)?;
        m.serialize_entry("layoutHints", &self.layout_hints)?;
        m.serialize_entry("registered", &self.registered)?;
        m.end()
    }
}
fn fail(message: String) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("architecture".into());
    e
}
fn register(
    reg: &mut HashMap<String, String>,
    order: &mut Vec<(String, String)>,
    kind: &str,
    id: &str,
    parent: Option<&str>,
) -> Result<(), PeleError> {
    if let Some(prior) = reg.get(id) {
        return Err(fail(format!(
            "The {kind} id [{id}] is already in use by another {prior}"
        )));
    }
    if let Some(p) = parent {
        if id == p {
            return Err(fail(format!(
                "The {kind} [{id}] cannot be placed within itself"
            )));
        }
        if !reg.contains_key(p) {
            return Err(fail(format!("The {kind} [{id}]'s parent does not exist. Please make sure the parent is created before this {kind}")));
        }
        if reg.get(p).map(String::as_str) == Some("node") {
            return Err(fail(format!("The {kind} [{id}]'s parent is not a group")));
        }
    }
    let kind = if kind == "group" { "group" } else { "node" };
    reg.insert(id.into(), kind.into());
    order.push((id.into(), kind.into()));
    Ok(())
}
pub fn populate(a: Ast, title: Option<&str>) -> Result<ArchitectureModel, PeleError> {
    let mut m = ArchitectureModel::empty(title);
    m.acc_title = a.acc_title.filter(|s| !s.is_empty());
    m.acc_descr = a.acc_descr.filter(|s| !s.is_empty());
    if let Some(t) = a.title.filter(|s| !s.is_empty()) {
        m.title = Some(t)
    }
    let mut reg = HashMap::new();
    for g in a.groups {
        register(
            &mut reg,
            &mut m.registered,
            "group",
            &g.id,
            g.parent.as_deref(),
        )?;
        m.groups.push((g.id.clone(), g))
    }
    for n in a.services.into_iter().chain(a.junctions) {
        register(
            &mut reg,
            &mut m.registered,
            &n.node_type,
            &n.id,
            n.parent.as_deref(),
        )?;
        m.nodes.push((n.id.clone(), n))
    }
    let at: HashMap<&str, usize> = m
        .nodes
        .iter()
        .enumerate()
        .map(|(i, (k, _))| (k.as_str(), i))
        .collect();
    for e in a.edges {
        let ends = [
            ("left", &e.lhs_id, e.lhs_group),
            ("right", &e.rhs_id, e.rhs_group),
        ];
        for (side, id, _) in ends {
            if !reg.contains_key(id) {
                return Err(fail(format!("The {side}-hand id [{id}] does not yet exist. Please create the service/group before declaring an edge to it.")));
            }
        }
        for (side, id, _) in ends {
            if !at.contains_key(id.as_str()) {
                return Err(fail(format!("The {side}-hand id [{id}] is a group. Add {{group}} to a service inside it instead.")));
            }
        }
        let parent = m.nodes[at[e.lhs_id.as_str()]].1.parent.as_deref();
        if parent.is_some() && parent == m.nodes[at[e.rhs_id.as_str()]].1.parent.as_deref() {
            for (side, id, crosses) in ends {
                if crosses {
                    return Err(fail(format!("The {side}-hand id [{id}] is modified to traverse the group boundary, but the edge does not pass through two groups.")));
                }
            }
        }
        m.edges.push(e)
    }
    for h in a.hints {
        let mut seen = std::collections::HashSet::new();
        for id in &h.members {
            if reg.get(id).map(String::as_str) != Some("node") {
                return Err(fail(format!(
                    "align {} references [{id}], which is not a service or junction",
                    h.direction
                )));
            }
            if !seen.insert(id) {
                return Err(fail(format!(
                    "align {} lists [{id}] more than once",
                    h.direction
                )));
            }
        }
        m.layout_hints.push(h)
    }
    Ok(m)
}

impl ArchitectureModel {
    /// Empty class state shared by parser population and native construction.
    pub(super) fn empty(title: Option<&str>) -> Self {
        Self {
            title: title.map(str::to_owned),
            acc_title: None,
            acc_descr: None,
            nodes: vec![],
            groups: vec![],
            edges: vec![],
            layout_hints: vec![],
            registered: vec![],
        }
    }
}
