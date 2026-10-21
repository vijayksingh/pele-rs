//! Pele agentflow public model/database, 458322a5 (MIT, kepano).
use crate::compat::{
    edge_limit::EdgePolicy,
    source::{space, yaml::parse as parse_yaml},
    DiagramType, ErrorCode, Object, ParseContext, PeleError, Value,
};
use serde::{Serialize, Serializer};
use std::collections::{HashMap, HashSet};
#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LabelType {
    #[default]
    Text,
    String,
    Markdown,
}
#[derive(Clone, Debug)]
pub(super) struct FlowText {
    pub text: String,
    pub kind: LabelType,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentNode {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub label_type: LabelType,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,
    pub styles: Vec<String>,
    pub classes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<Object>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Object>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_connector: Option<bool>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEdge {
    pub start: String,
    pub end: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub edge_type: Option<String>,
    pub text: String,
    pub label_type: LabelType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_semantic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Object>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<Vec<String>>,
    pub classes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub is_user_defined_id: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interpolate: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animate: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation: Option<Value>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentClassDef {
    pub id: String,
    pub styles: Vec<String>,
    pub text_styles: Vec<String>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSubgraph {
    pub id: String,
    #[serde(rename = "type")]
    pub group_type: String,
    pub nodes: Vec<String>,
    pub title: String,
    pub classes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
    pub label_type: LabelType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Object>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentflowModel {
    #[serde(skip)]
    pub public_origin: super::public_model::AgentflowPublicOrigin,
    #[serde(rename = "type")]
    pub diagram_type: DiagramType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    pub connectors: Vec<(String, AgentNode)>,
    pub mappings: Vec<ElementMapping>,
    pub diagnostics: Vec<Diagnostic>,
    pub line_offset: usize,
    #[serde(skip)]
    pub(super) graph: Option<super::graph::AgentGraph>,
    #[serde(serialize_with = "pairs")]
    pub nodes: Vec<(String, AgentNode)>,
    pub edges: Vec<AgentEdge>,
    pub subgraphs: Vec<AgentSubgraph>,
    #[serde(serialize_with = "pairs")]
    pub classes: Vec<(String, AgentClassDef)>,
    #[serde(serialize_with = "pairs")]
    pub tooltips: Vec<(String, String)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_edge_style: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_interpolate: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acc_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acc_descr: Option<String>,
}
fn pairs<T: Serialize, S: Serializer>(v: &[(String, T)], s: S) -> Result<S::Ok, S::Error> {
    v.serialize(s)
}
#[derive(Clone, Debug)]
pub(super) struct LinkData {
    pub kind: String,
    pub stroke: String,
    pub length: Option<f64>,
    pub text: Option<FlowText>,
    pub id: Option<String>,
    pub edge_semantic: Option<String>,
}
#[derive(Clone, Debug)]
pub(super) enum DocItem {
    Text(String),
    Nodes(Vec<String>),
    Dir(String),
}
pub(super) struct AgentDb {
    pub model: AgentflowModel,
    nodes: HashMap<String, usize>,
    classes: HashMap<String, usize>,
    tooltips: HashMap<String, usize>,
    subgraphs: HashMap<String, usize>,
    members: HashSet<String>,
    edge_by_id: HashMap<String, usize>,
    pair_count: HashMap<(String, String), usize>,
    sub_count: usize,
    edge_policy: EdgePolicy,
    inherit_dir: bool,
    config_direction: Option<String>,
    connectors: HashMap<String, usize>,
    connector_order: Vec<String>,
    global_nodes: HashSet<String>,
    bare_mappings: HashSet<usize>,
    first_mapping: HashMap<String, usize>,
}
fn trim(s: &str) -> &str {
    s.trim_matches(space)
}
fn unquote(s: &str) -> String {
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        s[1..s.len() - 1].into()
    } else {
        s.into()
    }
}
fn label_type(v: Option<&Value>) -> LabelType {
    match v {
        Some(Value::String(s)) if s == "text" => LabelType::Text,
        Some(Value::String(s)) if s == "string" => LabelType::String,
        _ => LabelType::Markdown,
    }
}
fn err(message: String, code: ErrorCode) -> PeleError {
    let mut e = PeleError::new(message, code);
    e.diagram_type = Some("agentflow".into());
    e
}
impl AgentDb {
    pub fn new(context: &ParseContext<'_>) -> Self {
        let mut model = AgentflowModel::empty();
        model.public_origin.max_edges = context.max_edges;
        model.public_origin.inherit_dir = context
            .config
            .get("flowchart")
            .and_then(Value::as_object)
            .and_then(|o| o.get("inheritDir"))
            == Some(&Value::Bool(true));
        model.public_origin.direction = context.config.get("direction").and_then(|v| {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        });
        model.line_offset = context.line_offset;
        model.title = context
            .title
            .filter(|title| !title.is_empty())
            .map(str::to_string);
        Self {
            model,
            nodes: HashMap::new(),
            classes: HashMap::new(),
            tooltips: HashMap::new(),
            subgraphs: HashMap::new(),
            members: HashSet::new(),
            edge_by_id: HashMap::new(),
            pair_count: HashMap::new(),
            sub_count: 0,
            edge_policy: EdgePolicy::Legacy(context.max_edges),
            config_direction: match context.config.get("direction") {
                Some(Value::String(s)) => Some(s.clone()),
                _ => None,
            },
            connectors: HashMap::new(),
            connector_order: vec![],
            global_nodes: HashSet::new(),
            bare_mappings: HashSet::new(),
            first_mapping: HashMap::new(),
            inherit_dir: context
                .config
                .get("flowchart")
                .and_then(Value::as_object)
                .and_then(|o| o.get("inheritDir"))
                == Some(&Value::Bool(true)),
        }
    }
    /// Rendering a caller-authored model has no parser/builder history. Only
    /// the real graph builder consumes this transient database adapter.
    pub(super) fn from_render_model(model: AgentflowModel) -> Self {
        Self {
            model,
            nodes: HashMap::new(),
            classes: HashMap::new(),
            tooltips: HashMap::new(),
            subgraphs: HashMap::new(),
            members: HashSet::new(),
            edge_by_id: HashMap::new(),
            pair_count: HashMap::new(),
            sub_count: 0,
            edge_policy: EdgePolicy::Exact(crate::compat::EdgeLimit::Number(f64::NAN)),
            inherit_dir: false,
            config_direction: None,
            connectors: HashMap::new(),
            connector_order: vec![],
            global_nodes: HashSet::new(),
            bare_mappings: HashSet::new(),
            first_mapping: HashMap::new(),
        }
    }
    pub fn new_with_edge_limit(
        context: &ParseContext<'_>,
        limit: crate::compat::EdgeLimit,
    ) -> Self {
        let mut db = Self::new(context);
        db.edge_policy = EdgePolicy::Exact(limit);
        db
    }
    pub fn add_vertex(
        &mut self,
        id: &str,
        text: Option<FlowText>,
        shape: Option<&str>,
        styles: Option<Vec<String>>,
        props: Option<Object>,
        metadata: Option<String>,
    ) -> Result<(), PeleError> {
        self.add_vertex_loc(id, text, shape, styles, props, metadata, None)
    }
    pub fn add_vertex_loc(
        &mut self,
        id: &str,
        text: Option<FlowText>,
        shape: Option<&str>,
        styles: Option<Vec<String>>,
        props: Option<Object>,
        metadata: Option<String>,
        loc: Option<Loc>,
    ) -> Result<(), PeleError> {
        if trim(id).is_empty() {
            return Ok(());
        }
        let mut doc = metadata
            .as_deref()
            .map(|s| self.metadata(id, s, loc))
            .transpose()?
            .filter(Value::truthy);
        let mut authored = None;
        if let Some(Value::Object(fields)) = &mut doc {
            if let Some(Value::String(s)) = fields.get("shape") {
                authored = Some(s.clone());
                let resolved = super::shapes::resolve(s);
                fields.insert("shape".into(), Value::String(resolved.into()));
            }
        }
        let fields = doc.as_ref().and_then(Value::as_object).cloned();
        if let Some(&i) = self.subgraphs.get(id) {
            if let Some(doc) = &doc {
                super::public_model::AgentflowPublicOrigin::metadata(
                    &mut self.model.public_origin.raw_group_metadata,
                    i,
                    &self.model.subgraphs[i].metadata,
                    doc,
                );
                merge(&mut self.model.subgraphs[i].metadata, doc);
                return Ok(());
            }
        }
        if let Some(&i) = self.connectors.get(id) {
            if let Some(doc) = &doc {
                super::public_model::AgentflowPublicOrigin::key(
                    &mut self.model.public_origin.nodes[i],
                    "metadata",
                );
                super::public_model::AgentflowPublicOrigin::metadata(
                    &mut self.model.public_origin.raw_node_metadata,
                    i,
                    &self.model.nodes[i].1.metadata,
                    doc,
                );
                merge(&mut self.model.nodes[i].1.metadata, doc);
                return Ok(());
            }
        }
        if id == "connectors" {
            return Ok(());
        }
        if let Some(&i) = self.edge_by_id.get(id) {
            if let Some(fields) = &fields {
                let e = &mut self.model.edges[i];
                let keys = &mut self.model.public_origin.edges[i];
                if let Some(v) = fields.get("animate") {
                    super::public_model::AgentflowPublicOrigin::key(keys, "animate");
                    e.animate = Some(v.clone())
                }
                if let Some(v) = fields.get("animation") {
                    super::public_model::AgentflowPublicOrigin::key(keys, "animation");
                    e.animation = Some(v.clone())
                }
                if let Some(v) = fields.get("curve") {
                    super::public_model::AgentflowPublicOrigin::key(keys, "interpolate");
                    e.interpolate = Some(v.clone())
                }
            }
            if let Some(doc) = &doc {
                super::public_model::AgentflowPublicOrigin::key(
                    &mut self.model.public_origin.edges[i],
                    "metadata",
                );
                super::public_model::AgentflowPublicOrigin::metadata(
                    &mut self.model.public_origin.raw_edge_metadata,
                    i,
                    &self.model.edges[i].metadata,
                    doc,
                );
                merge(&mut self.model.edges[i].metadata, doc);
            }
            return Ok(());
        }
        let i = if let Some(&i) = self.nodes.get(id) {
            i
        } else {
            let i = self.model.nodes.len();
            self.nodes.insert(id.into(), i);
            self.model
                .public_origin
                .nodes
                .push(vec!["id", "labelType", "styles", "classes"]);
            self.model.nodes.push((
                id.into(),
                AgentNode {
                    id: id.into(),
                    text: None,
                    label_type: LabelType::Text,
                    shape: None,
                    styles: vec![],
                    classes: vec![],
                    dir: None,
                    props: None,
                    link: None,
                    link_target: None,
                    metadata: None,
                    is_connector: None,
                },
            ));
            i
        };
        let keys = &mut self.model.public_origin.nodes[i];
        let key =
            |keys: &mut Vec<_>, name| super::public_model::AgentflowPublicOrigin::key(keys, name);
        if text.is_some() || self.model.nodes[i].1.text.is_none() {
            key(keys, "text")
        }
        if shape.is_some() {
            key(keys, "type")
        }
        if self.model.nodes[i].1.props.is_none() {
            key(keys, "props")
        }
        if doc.is_some() {
            key(keys, "metadata")
        }
        let v = &mut self.model.nodes[i].1;
        if let Some(text) = text {
            v.label_type = text.kind;
            v.text = Some(unquote(trim(&text.text)))
        } else if v.text.is_none() {
            v.text = Some(id.into())
        }
        if let Some(shape) = shape {
            v.shape = Some(shape.into())
        }
        if let Some(styles) = styles {
            v.styles.extend(styles)
        }
        if v.props.is_none() {
            v.props = Some(props.unwrap_or_default())
        } else if let Some(props) = props {
            for (k, value) in props.entries() {
                v.props.as_mut().unwrap().insert(k.into(), value.clone());
            }
        }
        if let Some(doc) = &doc {
            super::public_model::AgentflowPublicOrigin::metadata(
                &mut self.model.public_origin.raw_node_metadata,
                i,
                &v.metadata,
                doc,
            );
            merge(&mut v.metadata, doc)
        } else {
            return Ok(());
        }
        let Some(fields) = fields else { return Ok(()) };
        if let Some(shape) = fields.get("shape").filter(|v| v.truthy()) {
            let raw = authored.map(Value::String).unwrap_or_else(|| shape.clone());
            let name = match &raw {
                Value::String(s) => s.as_str(),
                _ => {
                    return Err(shape_error(
                        format!("No such shape: {}.", serde_json::to_string(&raw).unwrap()),
                        loc,
                        self.model.line_offset,
                    ))
                }
            };
            if name != name.to_lowercase() || name.contains('_') {
                return Err(shape_error(
                    format!("No such shape: {name}. Shape names should be lowercase."),
                    loc,
                    self.model.line_offset,
                ));
            }
            let shape = shape.js_string();
            if !super::shapes::valid(&shape) {
                return Err(shape_error(
                    format!("No such shape: {shape}."),
                    loc,
                    self.model.line_offset,
                ));
            }
            key(keys, "type");
            self.model.nodes[i].1.shape = Some(shape);
        }
        if let Some(label) = fields.get("label").filter(|v| v.truthy()) {
            self.model.nodes[i].1.text = Some(label.js_string());
            self.model.nodes[i].1.label_type = label_type(fields.get("labelType"));
        }
        Ok(())
    }
    pub fn add_link(
        &mut self,
        starts: &[String],
        ends: &[String],
        info: &LinkData,
    ) -> Result<(), PeleError> {
        let id = info.id.as_ref().map(|id| id.replacen('@', "", 1));
        for (start_i, start) in starts.iter().enumerate() {
            for (end_i, end) in ends.iter().enumerate() {
                let id = id
                    .as_ref()
                    .filter(|_| start_i + 1 == starts.len() && end_i == 0);
                let supplied = id.filter(|id| !self.edge_by_id.contains_key(id.as_str()));
                let pair = (start.clone(), end.clone());
                let existing = *self.pair_count.get(&pair).unwrap_or(&0);
                let id = supplied.cloned().unwrap_or_else(|| {
                    format!(
                        "L_{start}_{end}_{}",
                        if existing == 0 { 0 } else { existing + 1 }
                    )
                });
                let text = info.text.as_ref();
                let edge = AgentEdge {
                    start: start.clone(),
                    end: end.clone(),
                    edge_type: Some(info.kind.clone()),
                    text: text.map_or_else(String::new, |t| unquote(trim(&t.text))),
                    label_type: text.map_or(LabelType::Text, |t| t.kind.clone()),
                    stroke: Some(info.stroke.clone()),
                    length: info.length.map(|n| Value::Number(n.min(10.))),
                    edge_semantic: info.edge_semantic.clone(),
                    metadata: None,
                    style: None,
                    classes: Vec::new(),
                    id: Some(id.clone()),
                    is_user_defined_id: supplied.is_some(),
                    interpolate: self.model.default_interpolate.clone(),
                    animate: None,
                    animation: None,
                };
                let len = self.model.edges.len();
                self.edge_policy.check(len, "agentflow")?;
                let mut keys = vec![
                    "start",
                    "end",
                    "type",
                    "text",
                    "labelType",
                    "classes",
                    "isUserDefinedId",
                    "interpolate",
                    "stroke",
                    "length",
                ];
                if edge.edge_semantic.is_some() {
                    keys.push("edgeSemantic")
                }
                keys.push("id");
                self.model.public_origin.edges.push(keys);
                self.model.public_origin.edge_styles.push(None);
                self.model.edges.push(edge);
                self.pair_count.insert(pair, existing + 1);
                self.edge_by_id.entry(id).or_insert(len);
            }
        }
        Ok(())
    }
    pub fn set_direction(&mut self, dir: &str) {
        let mut d = trim(dir);
        if d.contains('<') {
            d = "RL"
        }
        if d.contains('^') {
            d = "BT"
        }
        if d.contains('>') {
            d = "LR"
        }
        if d.contains('v') {
            d = "TB"
        }
        if d == "TD" {
            d = "TB"
        }
        self.model.direction = Some(d.into())
    }
    pub fn set_class(&mut self, ids: &str, class: &str) {
        for id in ids.split(',') {
            if let Some(&i) = self.nodes.get(id) {
                self.model.nodes[i].1.classes.push(class.into())
            }
            if let Some(&i) = self.edge_by_id.get(id) {
                self.model.edges[i].classes.push(class.into())
            }
            if let Some(&i) = self.subgraphs.get(id) {
                self.model.subgraphs[i].classes.push(class.into())
            }
        }
    }
    pub fn set_tooltip(&mut self, ids: &str, text: &str) {
        for id in ids.split(',') {
            if let Some(&i) = self.tooltips.get(id) {
                self.model.tooltips[i].1 = text.into()
            } else {
                self.tooltips.insert(id.into(), self.model.tooltips.len());
                self.model.tooltips.push((id.into(), text.into()))
            }
        }
    }
    pub fn set_link(
        &mut self,
        ids: &str,
        link: &str,
        target: Option<String>,
    ) -> Result<(), PeleError> {
        let link = trim(link);
        let url = if link.is_empty() {
            None
        } else {
            Some(crate::compat::source::url::safe_url(link)?)
        };
        for id in ids.split(',') {
            if let Some(&i) = self.nodes.get(id) {
                super::public_model::AgentflowPublicOrigin::key(
                    &mut self.model.public_origin.nodes[i],
                    "link",
                );
                super::public_model::AgentflowPublicOrigin::key(
                    &mut self.model.public_origin.nodes[i],
                    "linkTarget",
                );
                self.model.nodes[i].1.link = url.clone();
                self.model.nodes[i].1.link_target = target.clone()
            }
        }
        self.set_class(ids, "clickable");
        Ok(())
    }
    pub fn add_class(&mut self, ids: &str, styles: Vec<String>) {
        let replaced = styles
            .join(",")
            .replace("\\,", "§§§")
            .replace(',', ";")
            .replace("§§§", ",");
        let styles: Vec<_> = replaced.split(';').map(str::to_string).collect();
        for id in ids.split(',') {
            let i = if let Some(&i) = self.classes.get(id) {
                i
            } else {
                let i = self.model.classes.len();
                self.classes.insert(id.into(), i);
                self.model.classes.push((
                    id.into(),
                    AgentClassDef {
                        id: id.into(),
                        styles: Vec::new(),
                        text_styles: Vec::new(),
                    },
                ));
                i
            };
            let c = &mut self.model.classes[i].1;
            for s in &styles {
                if s.contains("color") {
                    c.text_styles.push(s.replacen("fill", "bgFill", 1))
                }
                c.styles.push(s.clone())
            }
        }
    }
    pub fn update_link(
        &mut self,
        positions: &[String],
        mut styles: Vec<String>,
        interpolate: Option<&str>,
    ) -> Result<(), PeleError> {
        let style_group = self.model.public_origin.next_style;
        self.model.public_origin.next_style += 1;
        for pos in positions {
            if pos == "default" {
                if let Some(interpolate) = interpolate {
                    self.model.default_interpolate = Some(Value::String(interpolate.into()))
                } else {
                    self.model.public_origin.default_style = Some(style_group);
                    self.model.default_edge_style = Some(styles.clone())
                }
                continue;
            }
            let i=pos.parse::<usize>().ok().filter(|i|*i<self.model.edges.len()).ok_or_else(||err(format!("The index {pos} for linkStyle is out of bounds. Valid indices for linkStyle are between 0 and {}. (Help: Ensure that the index is within the range of existing edges.)",self.model.edges.len() as isize-1),ErrorCode::Semantic))?;
            let e = &mut self.model.edges[i];
            if let Some(interpolate) = interpolate {
                e.interpolate = Some(Value::String(interpolate.into()))
            } else {
                if !styles.is_empty() && !styles.iter().any(|s| s.starts_with("fill")) {
                    styles.push("fill:none".into())
                }
                super::public_model::AgentflowPublicOrigin::key(
                    &mut self.model.public_origin.edges[i],
                    "style",
                );
                self.model.public_origin.edge_styles[i] = Some(style_group);
                e.style = Some(styles.clone())
            }
        }
        Ok(())
    }
    pub fn acc_title(&mut self, s: String) {
        self.model.acc_title = Some(s.trim_start_matches(space).into())
    }
    pub fn acc_description(&mut self, s: String) {
        let mut out = String::new();
        let mut after = false;
        for c in s.chars() {
            if after && space(c) {
                continue;
            }
            out.push(c);
            after = c == '\n'
        }
        self.model.acc_descr = Some(out)
    }
    pub fn add_subgraph(
        &mut self,
        id: Option<FlowText>,
        title: Option<FlowText>,
        same: bool,
        list: Vec<DocItem>,
    ) -> String {
        let mut name = id.as_ref().map(|id| trim(&id.text).to_string());
        let title_text = title.as_ref().map_or("", |t| t.text.as_str());
        if same && title_text.chars().any(space) {
            name = None
        }
        let mut seen = HashSet::new();
        let mut nodes = Vec::new();
        let mut dir = None;
        for item in list {
            match item {
                DocItem::Dir(d) => dir = Some(d),
                DocItem::Text(s) => {
                    if !trim(&s).is_empty() && seen.insert(s.clone()) {
                        nodes.push(s)
                    }
                }
                DocItem::Nodes(ids) => {
                    for id in ids {
                        if !trim(&id).is_empty() && seen.insert(id.clone()) {
                            nodes.push(id)
                        }
                    }
                }
            }
        }
        if dir.is_none() && self.inherit_dir {
            dir = self
                .model
                .direction
                .clone()
                .or_else(|| self.config_direction.clone())
        }
        let id = name.unwrap_or_else(|| format!("subGraph{}", self.sub_count));
        self.sub_count += 1;
        self.model.public_origin.sub_count = self.sub_count;
        nodes.retain(|node| {
            node != &id && !self.global_nodes.contains(node) && !self.members.contains(node)
        });
        self.members.extend(nodes.iter().cloned());
        self.model
            .public_origin
            .members
            .extend(nodes.iter().cloned());
        if let Some(&i) = self.subgraphs.get(&id) {
            self.model.subgraphs[i].nodes.extend(nodes);
            if !trim(title_text).is_empty() {
                self.model.subgraphs[i].title = trim(title_text).into()
            }
            if dir.is_some() {
                self.model.subgraphs[i].dir = dir;
            }
        } else {
            self.subgraphs
                .insert(id.clone(), self.model.subgraphs.len());
            self.model.subgraphs.push(AgentSubgraph {
                id: id.clone(),
                group_type: "flow".into(),
                nodes,
                title: trim(title_text).into(),
                classes: Vec::new(),
                dir,
                label_type: title
                    .as_ref()
                    .map_or(LabelType::Markdown, |t| t.kind.clone()),
                metadata: None,
            })
        }
        id
    }
    pub fn destruct_link(end: &str, start: Option<&str>) -> LinkData {
        let end = trim(end);
        let mut line = &end[..end.len().saturating_sub(1)];
        let kind = match end.chars().next_back() {
            Some('x') => "arrow_cross",
            Some('>') => "arrow_point",
            Some('-' | '.') => {
                line = end;
                "arrow_open"
            }
            _ => "arrow_open",
        };
        let dots = line.bytes().filter(|b| *b == b'.').count();
        let stroke = if dots > 0 { "dotted" } else { "normal" };
        let length = if dots > 0 {
            dots as f64
        } else {
            line.len() as f64 - 1.
        };
        if let Some(start) = start.filter(|s| !s.is_empty()) {
            let stroke_start = if start.contains('.') {
                "dotted"
            } else {
                "normal"
            };
            if stroke_start != stroke {
                return LinkData {
                    kind: "INVALID".into(),
                    stroke: "INVALID".into(),
                    length: None,
                    text: None,
                    id: None,
                    edge_semantic: None,
                };
            }
        }
        let semantic = match (kind, stroke) {
            ("arrow_point", "normal") => Some("sequence"),
            ("arrow_cross", "normal") => Some("failure"),
            ("arrow_open", "dotted") => Some("reference"),
            _ => None,
        };
        LinkData {
            kind: kind.into(),
            stroke: stroke.into(),
            length: Some(length),
            text: None,
            id: None,
            edge_semantic: semantic.map(str::to_owned),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Loc {
    pub first_line: usize,
    pub first_column: usize,
    pub last_line: usize,
    pub last_column: usize,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementPosition {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub start_index: usize,
    pub end_index: usize,
}
#[derive(Clone, Debug, Serialize)]
pub struct ElementMapping {
    pub id: String,
    #[serde(rename = "type")]
    pub statement_type: String,
    pub position: ElementPosition,
}
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiagnosticId {
    ShapeUnsupported,
    ShapeRemoved,
    EdgeOperatorUnsupported,
    ReferenceEdgeLabelRejected,
    ConnectorRefUnresolved,
    ConnectorRefNotAConnector,
    MetadataKeyMisapplied,
    DuplicateIdNode,
    ReservedSyntheticId,
    ContainmentViolation,
    EdgeSemanticContradiction,
    FlowNoInput,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub id: DiagnosticId,
    pub severity: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<ElementPosition>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticVertex {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,
    pub vertex_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Object>,
}
#[derive(Clone, Debug, Serialize)]
pub struct SemanticConnector {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Object>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticEdge {
    pub start: String,
    pub end: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub edge_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_semantic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Object>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticSubgraph {
    pub id: String,
    pub nodes: Vec<String>,
    #[serde(rename = "type")]
    pub group_type: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Object>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticModel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    pub vertices: Vec<SemanticVertex>,
    pub edges: Vec<SemanticEdge>,
    pub sub_graphs: Vec<SemanticSubgraph>,
    pub connectors: Vec<SemanticConnector>,
    pub diagnostics: Vec<Diagnostic>,
}
fn semantic_metadata(v: &Option<Object>) -> Option<Object> {
    let keys = [
        "shape",
        "view",
        "icon",
        "img",
        "form",
        "pos",
        "w",
        "h",
        "class",
        "style",
        "labelType",
    ];
    v.as_ref().and_then(|v| {
        let mut out = Object::default();
        for (k, v) in v.entries() {
            if !keys.contains(&k) {
                out.insert(k.into(), v.clone());
            }
        }
        if out.entries().is_empty() {
            None
        } else {
            Some(out)
        }
    })
}
pub(super) fn vertex_kind(v: &AgentNode) -> &'static str {
    match v.shape.as_deref() {
        Some("subroutine" | "subprocess" | "subproc" | "framed-rectangle" | "tool") => {
            return "tool"
        }
        _ => {}
    }
    match super::shapes::resolve(v.shape.as_deref().unwrap_or("")) {
        "hexagon" | "hex" => "action",
        "lean-right" | "lean_right" => "input",
        "lin-doc" | "lined-document" => "refdoc",
        "diamond" => "decision",
        _ => "task",
    }
}
impl AgentflowModel {
    pub fn semantic_model(&self) -> SemanticModel {
        let mut vertices = vec![];
        let mut connectors = vec![];
        let groups: HashSet<_> = self.subgraphs.iter().map(|s| s.id.as_str()).collect();
        for (id, v) in &self.nodes {
            if groups.contains(id.as_str()) {
                continue;
            }
            let metadata = semantic_metadata(&v.metadata);
            if v.is_connector == Some(true) {
                connectors.push(SemanticConnector {
                    id: id.clone(),
                    title: v.text.clone().filter(|s| s != id),
                    metadata,
                });
            } else {
                vertices.push(SemanticVertex {
                    id: id.clone(),
                    label: v.text.clone(),
                    shape: v.shape.as_deref().map(|s| super::shapes::resolve(s).into()),
                    vertex_kind: vertex_kind(v).into(),
                    metadata,
                });
            }
        }
        let edges = self
            .edges
            .iter()
            .map(|e| SemanticEdge {
                start: e.start.clone(),
                end: e.end.clone(),
                id: e.id.clone(),
                label: (!e.text.is_empty()).then(|| e.text.clone()),
                edge_type: e.edge_type.clone(),
                stroke: e.stroke.clone(),
                edge_semantic: e.edge_semantic.clone(),
                length: e.length.clone(),
                metadata: e.metadata.clone().filter(|m| !m.entries().is_empty()),
            })
            .collect();
        let sub_graphs = self
            .subgraphs
            .iter()
            .map(|s| SemanticSubgraph {
                id: s.id.clone(),
                nodes: s.nodes.clone(),
                group_type: s.group_type.clone(),
                title: s.title.clone(),
                direction: s.dir.clone(),
                metadata: semantic_metadata(&s.metadata),
            })
            .collect();
        SemanticModel {
            direction: self.direction.clone(),
            vertices,
            edges,
            sub_graphs,
            connectors,
            diagnostics: self.diagnostics.clone(),
        }
    }
}
fn strip(mut v: Value) -> Value {
    match &mut v {
        Value::Array(a) => {
            for v in a {
                *v = strip(std::mem::replace(v, Value::Null));
            }
        }
        Value::Object(o) => {
            let mut clean = Object::default();
            for (k, v) in o.entries() {
                if !matches!(k, "__proto__" | "constructor" | "prototype") {
                    clean.insert(k.into(), strip(v.clone()));
                }
            }
            *o = clean;
        }
        _ => {}
    }
    v
}
fn spread(v: &Value) -> Object {
    let mut out = Object::default();
    match v {
        Value::Object(o) => return o.clone(),
        Value::Array(a) => {
            for (i, v) in a.iter().enumerate() {
                out.insert(i.to_string(), v.clone());
            }
        }
        Value::String(s) => {
            for (i, c) in s.chars().enumerate() {
                out.insert(i.to_string(), Value::String(c.to_string()));
            }
        }
        _ => {}
    }
    out
}
fn merge(target: &mut Option<Object>, v: &Value) {
    let out = target.get_or_insert_with(Object::default);
    let v = spread(v);
    for (k, v) in v.entries() {
        out.insert(k.into(), v.clone());
    }
}
fn shape_error(message: String, loc: Option<Loc>, offset: usize) -> PeleError {
    let mut e = err(message, ErrorCode::Semantic);
    if let Some(loc) = loc {
        e.line = loc.first_line + offset;
        e.column = loc.first_column + 1;
    }
    e
}
fn trailing_commas(body: &str) -> String {
    let (mut single, mut double, mut flow) = (false, false, 0_usize);
    let mut block: Option<usize> = None;
    let mut out = vec![];
    for line in body.split('\n') {
        let indent = line.len() - line.trim_start_matches(space).len();
        if let Some(level) = block {
            if line.trim_matches(space).is_empty() || indent > level {
                out.push(line.to_string());
                continue;
            }
            block = None;
        }
        let mut comment = None;
        let chars: Vec<_> = line.char_indices().collect();
        let mut i = 0;
        while i < chars.len() {
            let (at, c) = chars[i];
            if double {
                if c == '\\' {
                    i += 1
                } else if c == '"' {
                    double = false
                }
            } else if single {
                if c == '\'' {
                    single = false
                }
            } else if c == '"' {
                double = true
            } else if c == '\'' {
                single = true
            } else if c == '#' && (i == 0 || matches!(chars[i - 1].1, ' ' | '\t')) {
                comment = Some(at);
                break;
            } else if matches!(c, '[' | '{') {
                flow += 1
            } else if matches!(c, ']' | '}') {
                flow = flow.saturating_sub(1)
            }
            i += 1;
        }
        if single || double || flow > 0 {
            out.push(line.to_string());
            continue;
        }
        let end = comment.unwrap_or(line.len());
        let code = &line[..end];
        let scalar = code.rfind(':').is_some_and(|at| {
            let t = code[at + 1..].trim_matches(space);
            let mut chars = t.chars();
            matches!(chars.next(), Some('>' | '|'))
                && chars.all(|c| c.is_ascii_digit() || matches!(c, '+' | '-'))
        });
        if scalar {
            block = Some(indent);
            out.push(line.to_string());
            continue;
        }
        let end_code = code.trim_end_matches([' ', '\t']);
        if let Some(prefix) = end_code.strip_suffix(',') {
            out.push(format!(
                "{prefix}{}{}",
                &code[end_code.len()..],
                &line[end..]
            ));
        } else {
            out.push(line.to_string())
        }
    }
    out.join("\n")
}
impl AgentDb {
    pub(super) fn metadata(
        &self,
        id: &str,
        source: &str,
        loc: Option<Loc>,
    ) -> Result<Value, PeleError> {
        let multiline = source.contains('\n');
        let text = if multiline {
            format!("{source}\n")
        } else {
            format!("{{\n{source}\n}}")
        };
        let parsed = match parse_yaml(&text) {
            Ok(v) => v,
            Err(error) => {
                let stripped = if multiline {
                    trailing_commas(source)
                } else {
                    source.into()
                };
                if stripped != source {
                    if let Ok(v) = parse_yaml(&format!("{stripped}\n")) {
                        return Ok(strip(if v == Value::Null {
                            Value::Object(Object::default())
                        } else {
                            v
                        }));
                    }
                }
                let at = loc.map_or(String::new(), |loc| {
                    format!(
                        " ({}:{})",
                        loc.first_line + self.model.line_offset,
                        loc.first_column + 1
                    )
                });
                let mut e = err(
                    format!("Metadata of \"{id}\" is not valid YAML. {error}{at}"),
                    ErrorCode::Syntax,
                );
                if let Some(loc) = loc {
                    e.line = loc.first_line + self.model.line_offset;
                    e.column = loc.first_column + 1;
                }
                return Err(e);
            }
        };
        Ok(strip(if parsed == Value::Null {
            Value::Object(Object::default())
        } else {
            parsed
        }))
    }
    pub fn add_global(&mut self, list: Vec<DocItem>) {
        let mut release = false;
        for item in list {
            let ids = match item {
                DocItem::Text(s) => vec![s],
                DocItem::Nodes(v) => v,
                DocItem::Dir(_) => continue,
            };
            for id in ids {
                let id = trim(&id);
                if id.is_empty() {
                    continue;
                }
                if !self.global_nodes.contains(id) {
                    self.model.public_origin.global_nodes.push(id.into());
                }
                self.global_nodes.insert(id.into());
                release |= self.members.remove(id);
            }
        }
        if release {
            self.model
                .public_origin
                .members
                .retain(|id| !self.global_nodes.contains(id));
            for s in &mut self.model.subgraphs {
                s.nodes.retain(|id| !self.global_nodes.contains(id));
            }
        }
    }
    pub fn add_connector(&mut self, id: Option<FlowText>, title: Option<FlowText>) -> String {
        let id = id.as_ref().map_or("", |v| trim(&v.text));
        if id.is_empty() {
            return String::new();
        }
        let title = title.as_ref().map_or("", |v| trim(&v.text));
        if let Some(&i) = self.connectors.get(id) {
            if !title.is_empty() {
                self.model.nodes[i].1.text = Some(title.into());
            }
        } else {
            self.connector_order.push(id.into());
            let i = self.model.nodes.len();
            let v = AgentNode {
                id: id.into(),
                text: Some(if title.is_empty() { id } else { title }.into()),
                label_type: LabelType::Text,
                shape: None,
                styles: vec![],
                classes: vec![],
                dir: None,
                props: None,
                link: None,
                link_target: None,
                metadata: None,
                is_connector: Some(true),
            };
            if let Some(&old) = self.nodes.get(id) {
                self.model.public_origin.raw_node_metadata.remove(&old);
                self.model.public_origin.nodes[old] = vec![
                    "id",
                    "labelType",
                    "styles",
                    "classes",
                    "isConnector",
                    "text",
                ];
                self.model.nodes[old].1 = v;
                self.connectors.insert(id.into(), old);
            } else {
                self.model.public_origin.nodes.push(vec![
                    "id",
                    "labelType",
                    "styles",
                    "classes",
                    "isConnector",
                    "text",
                ]);
                self.model.nodes.push((id.into(), v));
                self.nodes.insert(id.into(), i);
                self.connectors.insert(id.into(), i);
            }
        }
        id.into()
    }
    fn position(&self, loc: Option<Loc>) -> ElementPosition {
        let loc = loc.unwrap_or_default();
        ElementPosition {
            start_line: loc.first_line + self.model.line_offset,
            start_column: loc.first_column,
            end_line: loc.last_line + self.model.line_offset,
            end_column: loc.last_column,
            start_index: 0,
            end_index: 0,
        }
    }
    fn mapping(&mut self, id: String, kind: &str, position: ElementPosition) -> usize {
        let i = self.model.mappings.len();
        self.first_mapping.entry(id.clone()).or_insert(i);
        self.model.mappings.push(ElementMapping {
            id,
            statement_type: kind.into(),
            position,
        });
        i
    }
    pub fn vertex_mapping(&mut self, id: &str, bare: bool, loc: Loc) {
        if id.is_empty() {
            return;
        }
        let i = self.mapping(id.into(), "vertex", self.position(Some(loc)));
        if bare {
            self.bare_mappings.insert(i);
            self.model.public_origin.bare_mappings.push(i);
        }
    }
    pub fn extend_mapping(&mut self, id: &str, loc: Loc) {
        if id.is_empty() {
            return;
        }
        let end = self.position(Some(loc));
        for (i, m) in self.model.mappings.iter_mut().enumerate().rev() {
            if m.statement_type != "vertex" || m.id != id {
                continue;
            }
            if self.bare_mappings.remove(&i) {
                self.model
                    .public_origin
                    .bare_mappings
                    .retain(|index| *index != i);
                m.statement_type = "attachment".into();
            }
            let p = &mut m.position;
            if end.end_line > p.end_line
                || (end.end_line == p.end_line && end.end_column > p.end_column)
            {
                p.end_line = end.end_line;
                p.end_column = end.end_column;
            }
            return;
        }
        self.mapping(id.into(), "attachment", end);
    }
    pub fn edge_mapping(&mut self, ids: &[String], loc: Loc) {
        let names: Vec<_> = ids.iter().filter(|s| !s.is_empty()).cloned().collect();
        self.mapping(
            if names.is_empty() {
                "edge".into()
            } else {
                names.join(">")
            },
            "edge",
            self.position(Some(loc)),
        );
    }
    pub fn block_mapping(&mut self, id: &str, kind: &str, start: Loc, end: Loc) {
        if id.is_empty() {
            return;
        }
        let mut p = self.position(Some(start));
        let end = self.position(Some(end));
        p.end_line = end.end_line;
        p.end_column = end.end_column;
        self.mapping(id.into(), kind, p);
    }
    pub fn emit(
        &mut self,
        id: DiagnosticId,
        severity: &str,
        message: String,
        node: Option<&str>,
        edge: Option<&str>,
    ) {
        let node = node.filter(|s| !s.is_empty());
        let edge = if node.is_some() {
            None
        } else {
            edge.filter(|s| !s.is_empty())
        };
        if self.model.diagnostics.iter().any(|d| {
            d.id == id
                && d.severity == severity
                && d.message == message
                && d.node_id.as_deref() == node
                && d.edge_id.as_deref() == edge
        }) {
            return;
        }
        let position = node
            .or(edge)
            .and_then(|id| self.first_mapping.get(id))
            .map(|i| self.model.mappings[*i].position.clone());
        self.model.public_origin.diagnostic_mappings.push(
            node.or(edge)
                .and_then(|id| self.first_mapping.get(id))
                .copied(),
        );
        self.model.diagnostics.push(Diagnostic {
            id,
            severity: severity.into(),
            message,
            node_id: node.map(str::to_owned),
            edge_id: edge.map(str::to_owned),
            position,
        });
    }
    pub fn finish_connectors(&mut self) {
        self.model.connectors = self
            .connector_order
            .iter()
            .map(|id| (id.clone(), self.model.nodes[self.connectors[id]].1.clone()))
            .collect();
    }
}

impl AgentflowModel {
    pub(super) fn empty() -> Self {
        Self {
            diagram_type: DiagramType::Agentflow,
            public_origin: super::public_model::AgentflowPublicOrigin::default(),
            direction: None,
            connectors: vec![],
            mappings: vec![],
            diagnostics: vec![],
            line_offset: 0,
            graph: None,
            nodes: vec![],
            edges: vec![],
            subgraphs: vec![],
            classes: vec![],
            tooltips: vec![],
            default_edge_style: None,
            default_interpolate: None,
            title: None,
            acc_title: None,
            acc_descr: None,
        }
    }
}
