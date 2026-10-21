//! Pele flowchart/graph.ts model-to-drawing normalization, 458322a5 (MIT, kepano).
use super::model::{FlowNode, FlowchartModel, LabelType};
use crate::compat::{Object, Value};
use std::collections::{HashMap, HashSet};
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct GraphNode {
    pub id: String,
    pub label: Option<String>,
    pub label_type: LabelType,
    pub shape: String,
    pub is_group: bool,
    pub parent_id: Option<String>,
    pub css_styles: Vec<String>,
    pub css_styles_origin: Option<usize>,
    pub css_compiled_styles: Vec<String>,
    pub css_classes: String,
    pub dir: Option<String>,
    pub link: Option<String>,
    pub link_target: Option<String>,
    pub tooltip: Option<String>,
    pub icon: Option<String>,
    pub pos: Option<String>,
    pub img: Option<String>,
    pub asset_width: Option<Value>,
    pub asset_height: Option<Value>,
    pub constraint: Option<String>,
    pub color_index: Option<usize>,
    pub metadata: Option<Object>,
}
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct GraphEdge {
    pub id: String,
    pub is_user_defined_id: bool,
    pub start: String,
    pub end: String,
    pub edge_type: String,
    pub label: String,
    pub label_type: LabelType,
    pub thickness: Option<String>,
    pub pattern: Option<String>,
    pub minlen: Option<Value>,
    pub arrow_type_start: String,
    pub arrow_type_end: String,
    pub css_compiled_styles: Vec<String>,
    pub style: Vec<String>,
    pub classes: String,
    pub animate: Option<Value>,
    pub animation: Option<Value>,
    pub curve: Option<Value>,
}
#[derive(Clone, Debug)]
pub(crate) struct FlowGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}
fn shape_of(v: &FlowNode) -> String {
    if v.img.as_ref().is_some_and(|s| !s.is_empty()) {
        return "imageSquare".into();
    }
    if v.icon.as_ref().is_some_and(|s| !s.is_empty()) {
        return match v.form.as_deref() {
            Some("circle") => "iconCircle",
            Some("square") => "iconSquare",
            Some("rounded") => "iconRounded",
            _ => "icon",
        }
        .into();
    }
    match v.shape.as_deref() {
        Some("square") | None => "squareRect",
        Some("round") => "roundedRect",
        Some(s) => s,
    }
    .into()
}
fn arrows(kind: Option<&str>) -> (&str, &str) {
    match kind {
        Some(s @ ("arrow_point" | "arrow_circle" | "arrow_cross")) => ("none", s),
        Some("double_arrow_point") => ("arrow_point", "arrow_point"),
        Some("double_arrow_circle") => ("arrow_circle", "arrow_circle"),
        Some("double_arrow_cross") => ("arrow_cross", "arrow_cross"),
        _ => ("none", "arrow_point"),
    }
}
pub(crate) fn build_flow_graph(model: &FlowchartModel, default_curve: Option<&str>) -> FlowGraph {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let subgraphs = &model.subgraphs;
    let lookup: HashMap<_, _> = subgraphs
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id.as_str(), i))
        .collect();
    let classes: HashMap<_, _> = model.classes.iter().map(|(k, v)| (k.as_str(), v)).collect();
    let compiled = |names: &[String]| {
        let mut out = Vec::new();
        for name in names {
            if let Some(def) = classes.get(name.as_str()) {
                out.extend(
                    def.styles
                        .iter()
                        .map(|s| s.trim_matches(crate::compat::source::space).to_string()),
                );
                out.extend(
                    def.text_styles
                        .iter()
                        .map(|s| s.trim_matches(crate::compat::source::space).to_string()),
                );
                for s in &mut out {
                    *s = s.trim_matches(crate::compat::source::space).into()
                }
            }
        }
        out
    };
    let mut sub_parent = HashMap::new();
    for sg in subgraphs {
        for child in &sg.nodes {
            if lookup.contains_key(child.as_str()) {
                sub_parent.insert(child.as_str(), sg.id.as_str());
            }
        }
    }
    let mut children: HashMap<&str, Vec<&str>> = HashMap::new();
    for sg in subgraphs {
        if let Some(parent) = sub_parent.get(sg.id.as_str()) {
            children.entry(parent).or_default().push(&sg.id)
        }
    }
    let mut declaration = HashMap::new();
    let mut next = 0;
    for sg in subgraphs {
        if sub_parent.contains_key(sg.id.as_str()) {
            continue;
        }
        let mut pending = vec![sg.id.as_str()];
        while let Some(id) = pending.pop() {
            if declaration.contains_key(id) {
                continue;
            }
            declaration.insert(id, next);
            next += 1;
            if let Some(kids) = children.get(id) {
                pending.extend(kids.iter().rev().copied())
            }
        }
    }
    let collapsed = |id: &str| {
        lookup.get(id).is_some_and(|i| {
            subgraphs[*i].metadata.as_ref().and_then(|m| m.get("view"))
                == Some(&Value::String("collapsed".into()))
        })
    };
    let mut outermost: HashMap<&str, Option<&str>> = HashMap::new();
    let mut hidden = HashSet::new();
    let mut collapsed_into = HashMap::new();
    if subgraphs.iter().any(|sg| collapsed(&sg.id)) {
        for sg in subgraphs {
            let mut chain = Vec::new();
            let mut seen = HashSet::new();
            let mut current = Some(sg.id.as_str());
            while let Some(id) = current {
                if outermost.contains_key(id) || !seen.insert(id) {
                    break;
                }
                chain.push(id);
                current = sub_parent.get(id).copied()
            }
            let mut result = current.and_then(|id| outermost.get(id).copied().flatten());
            for &id in chain.iter().rev() {
                if result.is_none() && collapsed(id) {
                    result = Some(id)
                }
                outermost.insert(id, result);
            }
            let Some(ancestor) = result else { continue };
            if sg.id != ancestor {
                hidden.insert(sg.id.as_str());
                collapsed_into.insert(sg.id.as_str(), ancestor);
            }
            for child in &sg.nodes {
                if child == ancestor {
                    continue;
                }
                hidden.insert(child.as_str());
                collapsed_into.insert(child.as_str(), ancestor);
            }
        }
    }
    let mut parent_of = HashMap::new();
    let mut groups = HashSet::new();
    for sg in subgraphs.iter().rev() {
        if hidden.contains(sg.id.as_str()) {
            continue;
        }
        if !sg.nodes.is_empty() {
            groups.insert(sg.id.as_str());
        }
        for id in &sg.nodes {
            parent_of.insert(id.as_str(), sg.id.as_str());
        }
    }
    let mut by_id = HashMap::new();
    for sg in subgraphs.iter().rev() {
        if hidden.contains(sg.id.as_str()) {
            continue;
        }
        let is_collapsed = collapsed(&sg.id);
        let node = GraphNode {
            id: sg.id.clone(),
            label: Some(sg.title.clone()),
            label_type: sg.label_type.clone(),
            parent_id: parent_of.get(sg.id.as_str()).map(|s| (*s).into()),
            css_styles: Vec::new(),
            css_styles_origin: None,
            css_compiled_styles: compiled(&sg.classes),
            css_classes: sg.classes.join(" "),
            shape: if is_collapsed {
                "collapsedGroup"
            } else {
                "rect"
            }
            .into(),
            dir: sg.dir.clone(),
            is_group: !is_collapsed,
            color_index: declaration.get(sg.id.as_str()).copied(),
            metadata: if is_collapsed {
                None
            } else {
                sg.metadata.clone()
            },
            link: None,
            link_target: None,
            tooltip: None,
            icon: None,
            pos: None,
            img: None,
            asset_width: None,
            asset_height: None,
            constraint: None,
        };
        by_id.insert(sg.id.as_str(), nodes.len());
        nodes.push(node)
    }
    let tooltips: HashMap<_, _> = model
        .tooltips
        .iter()
        .map(|(k, v)| (k.as_str(), v))
        .collect();
    for (origin, (_, v)) in model.nodes.iter().enumerate() {
        if hidden.contains(v.id.as_str()) {
            continue;
        }
        if let Some(&i) = by_id.get(v.id.as_str()) {
            let node = &mut nodes[i];
            node.css_styles = v.styles.clone();
            node.css_styles_origin = Some(origin);
            node.css_compiled_styles = compiled(&v.classes);
            node.css_classes = v.classes.join(" ");
            continue;
        }
        let is_group = groups.contains(v.id.as_str());
        let mut class_names = vec!["default".into(), "node".into()];
        class_names.extend(v.classes.iter().cloned());
        let node = GraphNode {
            id: v.id.clone(),
            label: v.text.clone(),
            label_type: v.label_type.clone(),
            parent_id: parent_of.get(v.id.as_str()).map(|s| (*s).into()),
            css_styles: v.styles.clone(),
            css_styles_origin: Some(origin),
            css_compiled_styles: compiled(&class_names),
            css_classes: format!("default {}", v.classes.join(" ")),
            dir: v.dir.clone(),
            link: v.link.clone(),
            link_target: v.link_target.clone(),
            tooltip: tooltips.get(v.id.as_str()).map(|s| (*s).clone()),
            icon: v.icon.clone(),
            pos: v.pos.clone(),
            img: v.img.clone(),
            asset_width: v.asset_width.clone(),
            asset_height: v.asset_height.clone(),
            constraint: v.constraint.clone(),
            is_group,
            shape: if is_group { "rect".into() } else { shape_of(v) },
            color_index: None,
            metadata: None,
        };
        by_id.insert(&v.id, nodes.len());
        nodes.push(node)
    }
    for (index, raw) in model.edges.iter().enumerate() {
        let start = collapsed_into
            .get(raw.start.as_str())
            .copied()
            .unwrap_or(&raw.start);
        let end = collapsed_into
            .get(raw.end.as_str())
            .copied()
            .unwrap_or(&raw.end);
        if start == end
            && (collapsed_into.contains_key(raw.start.as_str())
                || collapsed_into.contains_key(raw.end.as_str()))
        {
            continue;
        }
        let (arrow_start, arrow_end) = arrows(raw.edge_type.as_deref());
        let mut styles = model.default_edge_style.clone().unwrap_or_default();
        if let Some(style) = &raw.style {
            styles.extend(style.iter().cloned())
        }
        let plain = raw.stroke.as_deref() == Some("invisible")
            || raw.edge_type.as_deref() == Some("arrow_open");
        edges.push(GraphEdge {
            id: raw
                .id
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| format!("L_{start}_{end}_{index}")),
            is_user_defined_id: raw.is_user_defined_id,
            start: start.into(),
            end: end.into(),
            edge_type: raw.edge_type.clone().unwrap_or_else(|| "normal".into()),
            label: raw.text.clone(),
            label_type: raw.label_type.clone(),
            thickness: raw.stroke.clone(),
            pattern: raw.stroke.clone(),
            minlen: raw.length.clone(),
            classes: if raw.stroke.as_deref() == Some("invisible") {
                String::new()
            } else {
                "edge-thickness-normal edge-pattern-solid flowchart-link".into()
            },
            arrow_type_start: if plain { "none" } else { arrow_start }.into(),
            arrow_type_end: if plain { "none" } else { arrow_end }.into(),
            css_compiled_styles: compiled(&raw.classes),
            style: styles,
            animate: raw.animate.clone(),
            animation: raw.animation.clone(),
            curve: raw
                .interpolate
                .as_ref()
                .filter(|v| v.truthy())
                .cloned()
                .or_else(|| {
                    model
                        .default_interpolate
                        .as_ref()
                        .filter(|v| v.truthy())
                        .cloned()
                })
                .or_else(|| default_curve.map(|s| Value::String(s.into()))),
        })
    }
    FlowGraph { nodes, edges }
}
