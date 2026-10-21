//! Shared Agentflow graph-construction phases. Sources own all live handles.
use super::{
    graph::{AgentGraph, GraphEdge, GraphNode},
    model::{AgentClassDef, AgentDb, AgentEdge, AgentNode, AgentSubgraph, DiagnosticId},
};
use crate::compat::{Object, Value};
use std::collections::{HashMap, HashSet};

pub(super) trait Source {
    type Group: Clone;
    type Members;
    type Error;
    fn work(&mut self, count: usize) -> Result<(), Self::Error>;
    fn group_count(&mut self) -> Result<usize, Self::Error>;
    fn group(&mut self, index: usize) -> Result<Self::Group, Self::Error>;
    fn group_id(&mut self, group: &Self::Group) -> Result<String, Self::Error>;
    fn group_collapsed(&mut self, group: &Self::Group) -> Result<bool, Self::Error>;
    fn members(&mut self, group: &Self::Group) -> Result<Self::Members, Self::Error>;
    fn next_member(&mut self, members: &mut Self::Members) -> Result<Option<String>, Self::Error>;
    fn close_members(&mut self, members: &mut Self::Members) -> Result<(), Self::Error>;
    fn subgraph(&mut self, id: &str) -> Result<Option<Self::Group>, Self::Error>;
    fn project_group(&mut self, group: &Self::Group) -> Result<AgentSubgraph, Self::Error>;
    fn nodes(&mut self) -> Result<Vec<AgentNode>, Self::Error>;
    fn edges(&mut self) -> Result<Vec<AgentEdge>, Self::Error>;
    fn class(&mut self, name: &str) -> Result<Option<AgentClassDef>, Self::Error>;
    fn tooltip(&mut self, id: &str) -> Result<Option<String>, Self::Error>;
    fn default_styles(&mut self) -> Result<Vec<String>, Self::Error>;
    fn default_curve(&mut self) -> Result<Option<Value>, Self::Error>;
    fn kind(&mut self, id: &str) -> Result<Option<String>, Self::Error>;
    fn emit(
        &mut self,
        id: DiagnosticId,
        severity: &str,
        message: String,
        node: &str,
    ) -> Result<(), Self::Error>;
}
fn compiled<S: Source>(source: &mut S, names: &[String]) -> Result<Vec<String>, S::Error> {
    let mut out = Vec::new();
    for name in names {
        source.work(1)?;
        if let Some(class) = source.class(name)? {
            out.extend(
                class
                    .styles
                    .into_iter()
                    .chain(class.text_styles)
                    .map(|s| s.trim_matches(crate::compat::source::space).to_owned()),
            );
        }
    }
    Ok(out)
}
fn each_member<S: Source>(
    source: &mut S,
    group: &S::Group,
    mut action: impl FnMut(&mut S, String) -> Result<(), S::Error>,
) -> Result<(), S::Error> {
    let mut members = source.members(group)?;
    loop {
        // A failure from next itself does not close the iterator.
        let Some(id) = source.next_member(&mut members)? else {
            return Ok(());
        };
        if let Err(error) = action(source, id) {
            let _ = source.close_members(&mut members);
            return Err(error);
        }
    }
}
pub(super) fn build<S: Source>(source: &mut S, config: &Object) -> Result<AgentGraph, S::Error> {
    let mut hidden = HashSet::new();
    let mut hidden_order = Vec::new();
    let mut owner = HashMap::new();
    let mut walked = HashSet::new();
    let mut index = 0;
    while index < source.group_count()? {
        source.work(1)?;
        let group = source.group(index)?;
        index += 1;
        if !source.group_collapsed(&group)? {
            continue;
        }
        let id = source.group_id(&group)?;
        if hidden.contains(&id) {
            continue;
        }
        let mut pending = vec![id.clone()];
        while let Some(member) = pending.pop() {
            source.work(1)?;
            let Some(children) = source.subgraph(&member)? else {
                continue;
            };
            each_member(source, &children, |source, child| {
                source.work(1)?;
                let id = source.group_id(&group)?;
                if child == id || hidden.contains(&child) {
                    return Ok(());
                }
                hidden.insert(child.clone());
                hidden_order.push(child.clone());
                owner.insert(child.clone(), id);
                if !walked.contains(&child) {
                    pending.push(child);
                }
                Ok(())
            })?;
        }
        walked.insert(source.group_id(&group)?);
    }
    let mut collapsed_into = HashMap::new();
    for id in &hidden_order {
        let mut path = vec![id.clone()];
        let mut top = owner[id].clone();
        loop {
            source.work(1)?;
            let known = collapsed_into.get(&top);
            let Some(up) = known.or_else(|| owner.get(&top)) else {
                break;
            };
            if known.is_none() {
                path.push(top.clone());
            }
            top = up.clone();
            if known.is_some() {
                break;
            }
        }
        for id in path {
            collapsed_into.insert(id, top.clone());
        }
    }
    let mut positions = HashMap::new();
    let count = source.group_count()?;
    for index in 0..count {
        source.work(1)?;
        let group = source.group(index)?;
        positions.insert(source.group_id(&group)?, index);
    }
    let mut parent_of = HashMap::new();
    let count = source.group_count()?;
    for index in (0..count).rev() {
        source.work(1)?;
        let group = source.group(index)?;
        if hidden.contains(&source.group_id(&group)?) {
            continue;
        }
        each_member(source, &group, |source, id| {
            source.work(1)?;
            let parent = source.group_id(&group)?;
            let mut closes = false;
            if positions.get(&id).is_some_and(|at| *at >= index) {
                let mut current = Some(parent.clone());
                let mut hops = 0;
                while hops <= source.group_count()? {
                    source.work(1)?;
                    let Some(value) = current else {
                        break;
                    };
                    if value == id {
                        closes = true;
                        break;
                    }
                    current = parent_of.get(&value).cloned();
                    hops += 1;
                }
            }
            if closes {
                source.emit(DiagnosticId::ContainmentViolation, "warning", format!("Container \"{parent}\" cannot contain \"{id}\" because \"{id}\" already contains it. The nesting that would close the loop is dropped."), &id)?;
            } else {
                parent_of.insert(id, parent);
            }
            Ok(())
        })?;
    }
    let mut nodes = Vec::new();
    let mut by_id = HashMap::new();
    let count = source.group_count()?;
    for index in (0..count).rev() {
        source.work(1)?;
        let group = source.group(index)?;
        if hidden.contains(&source.group_id(&group)?) {
            continue;
        }
        let group = source.project_group(&group)?;
        let collapsed = group.metadata.as_ref().and_then(|m| m.get("view"))
            == Some(&Value::String("collapsed".into()));
        let metadata = if collapsed {
            let mut value = group.metadata.clone().unwrap_or_default();
            value.insert(
                "containerType".into(),
                Value::String(group.group_type.clone()),
            );
            Some(value)
        } else {
            group.metadata
        };
        let node = GraphNode {
            id: group.id.clone(),
            label: Some(group.title),
            label_type: group.label_type,
            parent_id: parent_of.get(&group.id).cloned(),
            css_styles: vec![],
            css_compiled_styles: compiled(source, &group.classes)?,
            css_classes: group.classes.join(" "),
            shape: if collapsed {
                "collapsedGroup"
            } else {
                "flowGroup"
            }
            .into(),
            dir: group.dir,
            is_group: !collapsed,
            metadata,
            link: None,
            link_target: None,
            tooltip: None,
            color_index: None,
            kind: None,
        };
        by_id.insert(node.id.clone(), nodes.len());
        nodes.push(node);
    }
    for vertex in source.nodes()? {
        source.work(1)?;
        if hidden.contains(&vertex.id) {
            continue;
        }
        if let Some(&index) = by_id.get(&vertex.id) {
            let node = &mut nodes[index];
            node.css_styles = vertex.styles;
            node.css_compiled_styles = compiled(source, &vertex.classes)?;
            node.css_classes = vertex.classes.join(" ");
            continue;
        }
        let shape = vertex.shape.as_deref().map(super::shapes::resolve);
        let mut classes = vec!["default".into(), "node".into()];
        classes.extend(vertex.classes.iter().cloned());
        let node = GraphNode {
            id: vertex.id.clone(),
            label: vertex.text,
            label_type: vertex.label_type,
            parent_id: parent_of.get(&vertex.id).cloned(),
            css_styles: vertex.styles,
            css_compiled_styles: compiled(source, &classes)?,
            css_classes: format!("default {}", vertex.classes.join(" ")),
            shape: match shape {
                None | Some("square") => "squareRect",
                Some(shape) => shape,
            }
            .into(),
            dir: vertex.dir,
            link: vertex.link,
            link_target: vertex.link_target,
            tooltip: source.tooltip(&vertex.id)?,
            metadata: vertex.metadata,
            is_group: false,
            color_index: None,
            kind: None,
        };
        by_id.insert(node.id.clone(), nodes.len());
        nodes.push(node);
    }
    let default_curve = config
        .get("flowchart")
        .and_then(Value::as_object)
        .and_then(|c| c.get("curve"))
        .filter(|v| matches!(v, Value::String(_)))
        .cloned();
    let mut edges = Vec::new();
    for (index, raw) in source.edges()?.into_iter().enumerate() {
        source.work(1)?;
        let start = collapsed_into.get(&raw.start).unwrap_or(&raw.start);
        let end = collapsed_into.get(&raw.end).unwrap_or(&raw.end);
        if start == end && raw.start != raw.end {
            continue;
        }
        let mut styles = source.default_styles()?;
        if let Some(style) = raw.style {
            styles.extend(style);
        }
        let arrow = match raw.edge_type.as_deref() {
            Some("arrow_open") => "none",
            Some("arrow_cross") => "arrow_cross",
            _ => "arrow_point",
        };
        edges.push(GraphEdge {
            id: raw
                .id
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| format!("L_{start}_{end}_{index}")),
            is_user_defined_id: raw.is_user_defined_id,
            start: start.clone(),
            end: end.clone(),
            edge_type: raw.edge_type.unwrap_or_else(|| "normal".into()),
            label: raw.text,
            label_type: raw.label_type,
            thickness: raw.stroke.clone(),
            pattern: raw.stroke,
            minlen: raw.length,
            classes: "edge-thickness-normal edge-pattern-solid flowchart-link".into(),
            arrow_type_start: "none".into(),
            arrow_type_end: arrow.into(),
            css_compiled_styles: compiled(source, &raw.classes)?,
            style: styles,
            animate: raw.animate,
            animation: raw.animation,
            curve: match raw.interpolate.filter(Value::truthy) {
                Some(v) => Some(v),
                None => source
                    .default_curve()?
                    .filter(Value::truthy)
                    .or_else(|| default_curve.clone()),
            },
            metadata: raw.metadata,
        });
    }
    super::shapes::normalize_with(&mut nodes, |id, severity, message, node| {
        source.emit(id, severity, message, node)
    })?;
    let mut children: HashMap<String, Vec<String>> = HashMap::new();
    let mut index = 0;
    while index < source.group_count()? {
        source.work(1)?;
        let group = source.group(index)?;
        index += 1;
        let id = source.group_id(&group)?;
        if let Some(parent) = parent_of.get(&id) {
            children.entry(parent.clone()).or_default().push(id);
        }
    }
    let mut order = HashMap::new();
    let mut index = 0;
    while index < source.group_count()? {
        source.work(1)?;
        let group = source.group(index)?;
        index += 1;
        let id = source.group_id(&group)?;
        if parent_of.contains_key(&id) {
            continue;
        }
        let mut pending = vec![id];
        while let Some(id) = pending.pop() {
            source.work(1)?;
            if order.contains_key(&id) {
                continue;
            }
            order.insert(id.clone(), order.len());
            if let Some(kids) = children.get(&id) {
                pending.extend(kids.iter().rev().cloned());
            }
        }
    }
    let mut fallback = order.len();
    for node in &mut nodes {
        source.work(1)?;
        if node.is_group || node.shape == "collapsedGroup" {
            let n = if let Some(i) = order.get(&node.id) {
                *i
            } else {
                let i = fallback;
                fallback += 1;
                i
            };
            node.color_index = Some(super::graph::container_slot(n, 8));
            continue;
        }
        if let Some(kind) = source.kind(&node.id)? {
            if super::shapes::kind_slot(&kind).is_some() {
                node.kind = Some(kind);
            }
        }
    }
    Ok(AgentGraph { nodes, edges })
}

struct Typed<'a> {
    db: &'a mut AgentDb,
    groups: HashMap<String, usize>,
    nodes: HashMap<String, usize>,
    classes: HashMap<String, usize>,
    tooltips: HashMap<String, usize>,
    connectors: HashSet<String>,
}
impl Source for Typed<'_> {
    type Group = usize;
    type Members = (usize, usize);
    type Error = std::convert::Infallible;
    fn work(&mut self, _: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    fn group_count(&mut self) -> Result<usize, Self::Error> {
        Ok(self.db.model.subgraphs.len())
    }
    fn group(&mut self, i: usize) -> Result<usize, Self::Error> {
        Ok(i)
    }
    fn group_id(&mut self, g: &usize) -> Result<String, Self::Error> {
        Ok(self.db.model.subgraphs[*g].id.clone())
    }
    fn group_collapsed(&mut self, g: &usize) -> Result<bool, Self::Error> {
        Ok(self.db.model.subgraphs[*g]
            .metadata
            .as_ref()
            .and_then(|m| m.get("view"))
            == Some(&Value::String("collapsed".into())))
    }
    fn members(&mut self, g: &usize) -> Result<Self::Members, Self::Error> {
        Ok((*g, 0))
    }
    fn next_member(&mut self, it: &mut Self::Members) -> Result<Option<String>, Self::Error> {
        let value = self.db.model.subgraphs[it.0].nodes.get(it.1).cloned();
        it.1 += 1;
        Ok(value)
    }
    fn close_members(&mut self, _: &mut Self::Members) -> Result<(), Self::Error> {
        Ok(())
    }
    fn subgraph(&mut self, id: &str) -> Result<Option<usize>, Self::Error> {
        Ok(self.groups.get(id).copied())
    }
    fn project_group(&mut self, g: &usize) -> Result<AgentSubgraph, Self::Error> {
        Ok(self.db.model.subgraphs[*g].clone())
    }
    fn nodes(&mut self) -> Result<Vec<AgentNode>, Self::Error> {
        Ok(self.db.model.nodes.iter().map(|(_, v)| v.clone()).collect())
    }
    fn edges(&mut self) -> Result<Vec<AgentEdge>, Self::Error> {
        Ok(self.db.model.edges.clone())
    }
    fn class(&mut self, key: &str) -> Result<Option<AgentClassDef>, Self::Error> {
        Ok(self
            .classes
            .get(key)
            .map(|i| self.db.model.classes[*i].1.clone()))
    }
    fn tooltip(&mut self, key: &str) -> Result<Option<String>, Self::Error> {
        Ok(self
            .tooltips
            .get(key)
            .map(|i| self.db.model.tooltips[*i].1.clone()))
    }
    fn default_styles(&mut self) -> Result<Vec<String>, Self::Error> {
        Ok(self.db.model.default_edge_style.clone().unwrap_or_default())
    }
    fn default_curve(&mut self) -> Result<Option<Value>, Self::Error> {
        Ok(self.db.model.default_interpolate.clone())
    }
    fn kind(&mut self, id: &str) -> Result<Option<String>, Self::Error> {
        Ok(if self.connectors.contains(id) {
            Some("connector".into())
        } else {
            self.nodes
                .get(id)
                .map(|i| super::model::vertex_kind(&self.db.model.nodes[*i].1).to_owned())
        })
    }
    fn emit(
        &mut self,
        id: DiagnosticId,
        severity: &str,
        message: String,
        node: &str,
    ) -> Result<(), Self::Error> {
        self.db.emit(id, severity, message, Some(node), None);
        Ok(())
    }
}
pub(super) fn typed(db: &mut AgentDb, config: &Object) -> AgentGraph {
    let mut source = Typed {
        groups: db
            .model
            .subgraphs
            .iter()
            .enumerate()
            .map(|(i, g)| (g.id.clone(), i))
            .collect(),
        nodes: db
            .model
            .nodes
            .iter()
            .enumerate()
            .map(|(i, (k, _))| (k.clone(), i))
            .collect(),
        classes: db
            .model
            .classes
            .iter()
            .enumerate()
            .map(|(i, (k, _))| (k.clone(), i))
            .collect(),
        tooltips: db
            .model
            .tooltips
            .iter()
            .enumerate()
            .map(|(i, (k, _))| (k.clone(), i))
            .collect(),
        connectors: db.model.connectors.iter().map(|(k, _)| k.clone()).collect(),
        db,
    };
    match build(&mut source, config) {
        Ok(graph) => graph,
        Err(never) => match never {},
    }
}
