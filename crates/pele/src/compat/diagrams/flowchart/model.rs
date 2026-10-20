//! Pele flowchart public model/database, 458322a5 (MIT, kepano).
use crate::compat::{
    edge_limit::EdgePolicy,
    source::{space, yaml::parse as parse_yaml},
    DiagramType, EdgeLimit, ErrorCode, Object, ParseContext, PeleError, Value,
};
use serde::{Serialize, Serializer};
use std::collections::{HashMap, HashSet};
#[derive(Clone, Debug, Default, Serialize)]
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
pub struct FlowNode {
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
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub form: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pos: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub img: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_width: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_height: Option<Value>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowEdge {
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
pub struct FlowClassDef {
    pub id: String,
    pub styles: Vec<String>,
    pub text_styles: Vec<String>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowSubgraph {
    pub id: String,
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
pub struct FlowchartModel {
    #[serde(skip)]
    pub public_origin: super::public_model::FlowchartPublicOrigin,
    #[serde(rename = "type")]
    pub diagram_type: DiagramType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(serialize_with = "pairs")]
    pub nodes: Vec<(String, FlowNode)>,
    pub edges: Vec<FlowEdge>,
    pub subgraphs: Vec<FlowSubgraph>,
    #[serde(serialize_with = "pairs")]
    pub classes: Vec<(String, FlowClassDef)>,
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
}
#[derive(Clone, Debug)]
pub(super) enum DocItem {
    Text(String),
    Nodes(Vec<String>),
    Dir(String),
}
pub(super) struct FlowDb {
    pub model: FlowchartModel,
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
fn number(v: &Value) -> f64 {
    match v {
        Value::Null => 0.,
        Value::Bool(b) => {
            if *b {
                1.
            } else {
                0.
            }
        }
        Value::Number(n) => *n,
        Value::String(s) => {
            let s = trim(s);
            if s.is_empty() {
                return 0.;
            }
            if s == "Infinity" || s == "+Infinity" {
                return f64::INFINITY;
            }
            if s == "-Infinity" {
                return f64::NEG_INFINITY;
            }
            if s.len() > 2 {
                let radix = match s.get(..2) {
                    Some("0x" | "0X") => Some(16),
                    Some("0o" | "0O") => Some(8),
                    Some("0b" | "0B") => Some(2),
                    _ => None,
                };
                if let Some(radix) = radix {
                    let mut n = 0.;
                    for c in s[2..].chars() {
                        let Some(d) = c.to_digit(radix) else {
                            return f64::NAN;
                        };
                        n = n * radix as f64 + d as f64
                    }
                    return n;
                }
            }
            if s.chars()
                .any(|c| !matches!(c, '0'..='9' | '+' | '-' | '.' | 'e' | 'E'))
            {
                return f64::NAN;
            }
            s.parse().unwrap_or(f64::NAN)
        }
        _ => number(&Value::String(v.js_string())),
    }
}

fn err(message: String, code: ErrorCode) -> PeleError {
    let mut e = PeleError::new(message, code);
    e.diagram_type = Some("flowchart".into());
    e
}
impl FlowchartModel {
    pub(super) fn empty() -> Self {
        Self {
            diagram_type: DiagramType::Flowchart,
            public_origin: Default::default(),
            direction: None,
            nodes: Vec::new(),
            edges: Vec::new(),
            subgraphs: Vec::new(),
            classes: Vec::new(),
            tooltips: Vec::new(),
            default_edge_style: None,
            default_interpolate: None,
            title: None,
            acc_title: None,
            acc_descr: None,
        }
    }
}
impl FlowDb {
    pub fn new(context: &ParseContext<'_>) -> Self {
        Self {
            model: {
                let mut model = FlowchartModel::empty();
                model.public_origin.max_edges = context.max_edges;
                model.public_origin.inherit_dir = context
                    .config
                    .get("flowchart")
                    .and_then(Value::as_object)
                    .and_then(|o| o.get("inheritDir"))
                    == Some(&Value::Bool(true));
                model.title = context.title.filter(|t| !t.is_empty()).map(str::to_string);
                model
            },
            nodes: HashMap::new(),
            classes: HashMap::new(),
            tooltips: HashMap::new(),
            subgraphs: HashMap::new(),
            members: HashSet::new(),
            edge_by_id: HashMap::new(),
            pair_count: HashMap::new(),
            sub_count: 0,
            edge_policy: EdgePolicy::Legacy(context.max_edges),
            inherit_dir: context
                .config
                .get("flowchart")
                .and_then(Value::as_object)
                .and_then(|o| o.get("inheritDir"))
                == Some(&Value::Bool(true)),
        }
    }
    pub fn new_with_edge_limit(context: &ParseContext<'_>, limit: EdgeLimit) -> Self {
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
        if trim(id).is_empty() {
            return Ok(());
        }
        let doc = if let Some(meta) = metadata {
            let yaml = if meta.contains('\n') {
                format!("{meta}\n")
            } else {
                format!("{{\n{meta}\n}}")
            };
            match parse_yaml(&yaml).map_err(|e| {
                err(
                    format!("Metadata of \"{id}\" is not valid YAML. {e}"),
                    ErrorCode::Syntax,
                )
            })? {
                Value::Object(doc) => Some(doc),
                _ => None,
            }
        } else {
            None
        };
        if let (Some(&sg), Some(doc)) = (self.subgraphs.get(id), doc.as_ref()) {
            let metadata = self.model.subgraphs[sg]
                .metadata
                .get_or_insert_with(Object::default);
            for (k, v) in doc.entries() {
                metadata.insert(k.into(), v.clone())
            }
            return Ok(());
        }
        if let Some(&i) = self.edge_by_id.get(id) {
            if let Some(doc) = doc {
                let e = &mut self.model.edges[i];
                let keys = &mut self.model.public_origin.edges[i];
                if let Some(v) = doc.get("animate") {
                    super::public_model::FlowchartPublicOrigin::key(keys, "animate");
                    e.animate = Some(v.clone())
                }
                if let Some(v) = doc.get("animation") {
                    super::public_model::FlowchartPublicOrigin::key(keys, "animation");
                    e.animation = Some(v.clone())
                }
                if let Some(v) = doc.get("curve") {
                    super::public_model::FlowchartPublicOrigin::key(keys, "interpolate");
                    e.interpolate = Some(v.clone())
                }
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
                FlowNode {
                    id: id.into(),
                    text: None,
                    label_type: LabelType::Text,
                    shape: None,
                    styles: Vec::new(),
                    classes: Vec::new(),
                    dir: None,
                    props: None,
                    link: None,
                    link_target: None,
                    icon: None,
                    form: None,
                    pos: None,
                    img: None,
                    constraint: None,
                    asset_width: None,
                    asset_height: None,
                },
            ));
            i
        };
        let keys = &mut self.model.public_origin.nodes[i];
        let key =
            |keys: &mut Vec<_>, name| super::public_model::FlowchartPublicOrigin::key(keys, name);
        if text.is_some() || self.model.nodes[i].1.text.is_none() {
            key(keys, "text")
        }
        if shape.is_some() {
            key(keys, "type")
        }
        if self.model.nodes[i].1.props.is_none() {
            key(keys, "props")
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
                v.props.as_mut().unwrap().insert(k.into(), value.clone())
            }
        }
        let Some(doc) = doc else { return Ok(()) };
        for (input, output) in [
            ("shape", "type"),
            ("label", "text"),
            ("icon", "icon"),
            ("form", "form"),
            ("pos", "pos"),
            ("img", "img"),
            ("constraint", "constraint"),
            ("w", "assetWidth"),
            ("h", "assetHeight"),
        ] {
            if doc.get(input).is_some_and(Value::truthy) {
                key(keys, output);
            }
        }
        if let Some(shape) = doc.get("shape").filter(|v| v.truthy()) {
            let shape = shape.js_string();
            if shape.to_lowercase() != shape || shape.contains('_') {
                return Err(err(
                    format!("No such shape: {shape}. Shape names should be lowercase."),
                    ErrorCode::Semantic,
                ));
            }
            if !super::shapes::valid(&shape) {
                return Err(err(format!("No such shape: {shape}."), ErrorCode::Semantic));
            }
            v.shape = Some(shape)
        }
        let label = doc.get("label").and_then(|v| {
            if let Value::String(s) = v {
                Some(s)
            } else {
                None
            }
        });
        if let Some(label) = doc.get("label").filter(|v| v.truthy()) {
            v.text = Some(label.js_string());
            v.label_type = label_type(doc.get("labelType"))
        }
        if let Some(icon) = doc.get("icon").filter(|v| v.truthy()) {
            v.icon = Some(icon.js_string());
            if label.is_none_or(|s| trim(s).is_empty()) && v.text.as_deref() == Some(id) {
                v.text = Some(String::new())
            }
        }
        for (key, field) in [
            ("form", &mut v.form),
            ("pos", &mut v.pos),
            ("constraint", &mut v.constraint),
        ] {
            if let Some(value) = doc.get(key).filter(|v| v.truthy()) {
                *field = Some(value.js_string())
            }
        }
        if let Some(img) = doc.get("img").filter(|v| v.truthy()) {
            v.img = Some(img.js_string());
            if label.is_none_or(|s| trim(s).is_empty()) && v.text.as_deref() == Some(id) {
                v.text = Some(String::new())
            }
        }
        if let Some(w) = doc.get("w").filter(|v| v.truthy()) {
            v.asset_width = Some(Value::Number(number(w)))
        }
        if let Some(h) = doc.get("h").filter(|v| v.truthy()) {
            v.asset_height = Some(Value::Number(number(h)))
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
                let edge = FlowEdge {
                    start: start.clone(),
                    end: end.clone(),
                    edge_type: Some(info.kind.clone()),
                    text: text.map_or_else(String::new, |t| unquote(trim(&t.text))),
                    label_type: text.map_or(LabelType::Text, |t| t.kind.clone()),
                    stroke: Some(info.stroke.clone()),
                    length: info.length.map(|n| Value::Number(n.min(10.))),
                    style: None,
                    classes: Vec::new(),
                    id: Some(id.clone()),
                    is_user_defined_id: supplied.is_some(),
                    interpolate: self.model.default_interpolate.clone(),
                    animate: None,
                    animation: None,
                };
                let len = self.model.edges.len();
                self.edge_policy.check(len, "flowchart")?;
                self.model.public_origin.edges.push(vec![
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
                    "id",
                ]);
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
                super::public_model::FlowchartPublicOrigin::key(
                    &mut self.model.public_origin.nodes[i],
                    "link",
                );
                super::public_model::FlowchartPublicOrigin::key(
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
                    FlowClassDef {
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
                super::public_model::FlowchartPublicOrigin::key(
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
            dir = self.model.direction.clone()
        }
        let id = name.unwrap_or_else(|| format!("subGraph{}", self.sub_count));
        self.sub_count += 1;
        self.model.public_origin.sub_count = self.sub_count;
        nodes.retain(|node| node != &id && !self.members.contains(node));
        self.members.extend(nodes.iter().cloned());
        self.model
            .public_origin
            .members
            .extend(nodes.iter().cloned());
        if let Some(&i) = self.subgraphs.get(&id) {
            self.model.subgraphs[i].nodes.extend(nodes)
        } else {
            self.subgraphs
                .insert(id.clone(), self.model.subgraphs.len());
            self.model.subgraphs.push(FlowSubgraph {
                id: id.clone(),
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
        let mut line = &end[..end.len() - end.chars().next_back().map_or(0, char::len_utf8)];
        let mut kind = "arrow_open".to_string();
        match end.chars().next_back() {
            Some('x') | Some('>') | Some('o') => {
                let (c, arrow) = match end.chars().next_back().unwrap() {
                    'x' => ('x', "arrow_cross"),
                    '>' => ('<', "arrow_point"),
                    _ => ('o', "arrow_circle"),
                };
                kind = arrow.into();
                if end.starts_with(c) {
                    kind = format!("double_{kind}");
                    line = &line[1..]
                }
            }
            _ => {}
        }
        let mut stroke = "normal";
        let mut len = line.len() as f64 - 1.;
        if line.starts_with('=') {
            stroke = "thick"
        }
        if line.starts_with('~') {
            stroke = "invisible"
        }
        let dots = line.bytes().filter(|c| *c == b'.').count();
        if dots > 0 {
            stroke = "dotted";
            len = dots as f64
        }
        if let Some(start) = start.filter(|s| !s.is_empty()) {
            let mut s = trim(start);
            let start_kind = match s.chars().next() {
                Some('<') => {
                    s = &s[1..];
                    "arrow_point"
                }
                Some('x') => {
                    s = &s[1..];
                    "arrow_cross"
                }
                Some('o') => {
                    s = &s[1..];
                    "arrow_circle"
                }
                _ => "arrow_open",
            };
            let start_stroke = if s.contains('.') {
                "dotted"
            } else if s.contains('=') {
                "thick"
            } else {
                "normal"
            };
            if start_stroke != stroke || (start_kind != "arrow_open" && start_kind != kind) {
                return LinkData {
                    kind: "INVALID".into(),
                    stroke: "INVALID".into(),
                    length: None,
                    text: None,
                    id: None,
                };
            }
            if start_kind != "arrow_open" {
                kind = format!("double_{start_kind}")
            }
        }
        LinkData {
            kind,
            stroke: stroke.into(),
            length: Some(len),
            text: None,
            id: None,
        }
    }
}
