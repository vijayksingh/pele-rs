//! Captured FlowDb graph inputs; styles, direction and metadata remain phase-live.
//! Pele 458322a5 (MIT, kepano). Geometry stays in the shared compound renderer.
use super::{
    graph::{finish_flow_graph, FlowGraph, GraphNode},
    model::*,
    phase::PhaseInput,
};
use crate::compat::{
    public::{LiveCallArgument, LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    DiagramType, ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
pub(crate) struct FlowchartDetachedInput {
    initial_error: Option<PublicError>,
}
impl FlowchartDetachedInput {
    pub(crate) fn new(error: Option<PublicError>) -> Self {
        Self {
            initial_error: error.filter(|e| {
                matches!(e,PublicError::Pele(e)if e.code==ErrorCode::Limit)
                    || matches!(e, PublicError::JavaScript { .. })
            }),
        }
    }
}

fn error(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("flowchart".into());
    e
}
fn limit() -> PeleError {
    let mut e = error("Flowchart public model expansion exceeds the work budget");
    e.code = ErrorCode::Limit;
    e
}
struct Host<'a, R: LiveRenderRead> {
    read: &'a mut R,
    styles: Vec<R::Value>,
    work: usize,
    units: usize,
}
impl<R: LiveRenderRead> Host<'_, R> {
    fn tick(&mut self, n: usize) -> Result<(), PublicError> {
        self.work = self.work.saturating_add(n);
        if self.work > 8_000_000 {
            Err(limit().into())
        } else {
            Ok(())
        }
    }
    fn root_field(&mut self, key: &str) -> Result<R::Value, PublicError> {
        let root = self.read.root();
        self.read.get(&root, key)
    }
    fn string(&mut self, v: &R::Value) -> Result<String, PublicError> {
        let s = self.read.require_string(
            v,
            LiveStringUse {
                method: "trim",
                expression_receiver: "t",
            },
        )?;
        self.units = self.units.saturating_add(s.0.len());
        if self.units > 16_000_000 {
            return Err(limit().into());
        }
        s.well_formed()
            .ok_or_else(|| error("Flowchart rendering requires well-formed UTF16 text").into())
    }
    fn text(&mut self, v: &R::Value, key: &str) -> Result<String, PublicError> {
        let a = self.read.get(v, key)?;
        self.string(&a)
    }
    fn optional(&mut self, v: &R::Value, key: &str) -> Result<Option<String>, PublicError> {
        let a = self.read.get(v, key)?;
        if matches!(
            self.read.kind(&a),
            LiveValueKind::Undefined | LiveValueKind::Null
        ) {
            Ok(None)
        } else {
            self.string(&a).map(Some)
        }
    }
    fn values(&mut self, v: &R::Value) -> Result<Vec<R::Value>, PublicError> {
        let items = self.read.snapshot_iterable(v, 1_000_000)?;
        self.tick(items.len())?;
        Ok(items)
    }
    fn strings(&mut self, v: &R::Value) -> Result<Vec<String>, PublicError> {
        let items = self.values(v)?;
        items.iter().map(|x| self.string(x)).collect()
    }
    fn texts(&mut self, v: &R::Value, key: &str) -> Result<Vec<String>, PublicError> {
        let a = self.read.get(v, key)?;
        self.strings(&a)
    }
    fn scalar(&mut self, v: &R::Value, key: &str) -> Result<Option<Value>, PublicError> {
        let a = self.read.get(v, key)?;
        Ok(match self.read.kind(&a) {
            LiveValueKind::Undefined => None,
            LiveValueKind::Null => Some(Value::Null),
            LiveValueKind::Boolean => Some(Value::Bool(self.read.truthy(&a))),
            LiveValueKind::Number => Some(Value::Number(
                self.read
                    .number(&a)
                    .ok_or_else(|| error("Flowchart expected number"))?,
            )),
            LiveValueKind::String => Some(Value::String(self.string(&a)?)),
            _ => return Err(error("Flowchart object option coercion is not implemented").into()),
        })
    }
    fn label(&mut self, v: &R::Value) -> Result<LabelType, PublicError> {
        Ok(match self.optional(v, "labelType")?.as_deref() {
            Some("markdown") => LabelType::Markdown,
            Some("string") => LabelType::String,
            _ => LabelType::Text,
        })
    }
    // Account expanded copies before build_flow_graph materializes them. Shared
    // class arrays can be small in source storage and huge per drawing occurrence.
    fn admit_compiled_styles(&mut self, m: &FlowchartModel) -> Result<(), PublicError> {
        use std::collections::{HashMap, HashSet};
        let mut classes = HashMap::new();
        for (key, class) in &m.classes {
            let mut count = 0usize;
            let mut units = 0usize;
            for style in class.styles.iter().chain(&class.text_styles) {
                count = count.saturating_add(1);
                units = units.saturating_add(
                    style
                        .trim_matches(crate::compat::source::space)
                        .encode_utf16()
                        .count(),
                );
            }
            classes.insert(key.as_str(), (count, units));
        }
        let mut work = 0usize;
        let mut units = 0usize;
        let mut copies = 0usize;
        let mut charge = |names: Vec<&str>| {
            let mut accumulated = 0usize;
            let mut accumulated_units = 0usize;
            for name in names {
                if let Some((c, u)) = classes.get(name) {
                    accumulated = accumulated.saturating_add(*c);
                    accumulated_units = accumulated_units.saturating_add(*u);
                    units = units.saturating_add(*u);
                    work = work.saturating_add(accumulated);
                    copies = copies.saturating_add(accumulated_units);
                }
            }
        };
        let mut existing = HashSet::new();
        for group in &m.subgraphs {
            existing.insert(group.id.as_str());
            charge(group.classes.iter().map(String::as_str).collect());
        }
        for (_, node) in &m.nodes {
            let mut names = Vec::new();
            if existing.insert(node.id.as_str()) {
                names.extend(["default", "node"]);
            }
            names.extend(node.classes.iter().map(String::as_str));
            charge(names);
        }
        for edge in &m.edges {
            charge(edge.classes.iter().map(String::as_str).collect());
        }
        let defaults = m.default_edge_style.as_deref().unwrap_or_default();
        let default_units = defaults
            .iter()
            .map(|s| s.encode_utf16().count())
            .sum::<usize>();
        units = units.saturating_add(default_units.saturating_mul(m.edges.len()));
        work = work.saturating_add(defaults.len().saturating_mul(m.edges.len()));
        copies = copies.saturating_add(default_units.saturating_mul(m.edges.len()));
        // Own edge styles are cloned again into the drawing graph as well.
        for edge in &m.edges {
            if let Some(styles) = &edge.style {
                units = units.saturating_add(
                    styles
                        .iter()
                        .map(|s| s.encode_utf16().count())
                        .sum::<usize>(),
                );
                work = work.saturating_add(styles.len());
            }
        }
        self.tick(work)?;
        self.units = self.units.saturating_add(units);
        if self.units > 16_000_000 || copies > 64_000_000 {
            return Err(limit().into());
        }
        Ok(())
    }
    fn graph_input(&mut self, groups: &R::Value) -> Result<FlowchartModel, PublicError> {
        self.styles.clear();
        let root = self.read.root();
        let mut m = FlowchartModel::empty();
        m.diagram_type = DiagramType::Flowchart;
        let values = self.root_field("nodes")?;
        let values = self.read.snapshot_map_values(&values, 1_000_000)?;
        self.tick(values.len())?;
        for (i, v) in values.into_iter().enumerate() {
            let styles = self.read.get(&v, "styles")?;
            self.styles.push(styles);
            m.nodes.push((
                i.to_string(),
                FlowNode {
                    id: self.text(&v, "id")?,
                    text: self.optional(&v, "text")?,
                    label_type: self.label(&v)?,
                    shape: self.optional(&v, "type")?,
                    styles: Vec::new(),
                    classes: self.texts(&v, "classes")?,
                    dir: self.optional(&v, "dir")?,
                    props: None,
                    link: self.optional(&v, "link")?,
                    link_target: self.optional(&v, "linkTarget")?,
                    icon: self.optional(&v, "icon")?,
                    form: self.optional(&v, "form")?,
                    pos: self.optional(&v, "pos")?,
                    img: self.optional(&v, "img")?,
                    constraint: self.optional(&v, "constraint")?,
                    asset_width: self.scalar(&v, "assetWidth")?,
                    asset_height: self.scalar(&v, "assetHeight")?,
                },
            ));
        }
        let edges = self.root_field("edges")?;
        for v in self.values(&edges)? {
            let styles = self.read.get(&v, "style")?;
            let style = if self.read.truthy(&styles) {
                Some(self.strings(&styles)?)
            } else {
                None
            };
            let user = self.read.get(&v, "isUserDefinedId")?;
            m.edges.push(FlowEdge {
                start: self.text(&v, "start")?,
                end: self.text(&v, "end")?,
                edge_type: self.optional(&v, "type")?,
                text: self.text(&v, "text")?,
                label_type: self.label(&v)?,
                stroke: self.optional(&v, "stroke")?,
                length: self.scalar(&v, "length")?,
                style,
                classes: self.texts(&v, "classes")?,
                id: self.optional(&v, "id")?,
                is_user_defined_id: self.read.truthy(&user),
                interpolate: self.scalar(&v, "interpolate")?,
                animate: self.scalar(&v, "animate")?,
                animation: self.scalar(&v, "animation")?,
            });
        }
        for v in self.values(groups)? {
            let meta = self.read.get(&v, "metadata")?;
            let metadata = if matches!(self.read.kind(&meta), LiveValueKind::Object) {
                let view = self.optional(&meta, "view")?;
                if view.as_deref() == Some("collapsed") {
                    let mut o = Object::default();
                    o.insert("view".into(), Value::String("collapsed".into()));
                    Some(o)
                } else {
                    None
                }
            } else {
                None
            };
            m.subgraphs.push(FlowSubgraph {
                id: self.text(&v, "id")?,
                nodes: self.texts(&v, "nodes")?,
                title: self.text(&v, "title")?,
                classes: self.texts(&v, "classes")?,
                dir: self.optional(&v, "dir")?,
                label_type: self.label(&v)?,
                metadata,
            });
        }
        let classes = self.root_field("classes")?;
        for pair in self.values(&classes)? {
            let k = self.read.index(&pair, 0)?;
            let v = self.read.index(&pair, 1)?;
            let key = self.string(&k)?;
            m.classes.push((
                key.clone(),
                FlowClassDef {
                    id: key,
                    styles: self.texts(&v, "styles")?,
                    text_styles: self.texts(&v, "textStyles")?,
                },
            ));
        }
        let tooltips = self.root_field("tooltips")?;
        for (_, node) in &m.nodes {
            let v = self.read.map_get_text(&tooltips, &node.id)?;
            if !matches!(self.read.kind(&v), LiveValueKind::Undefined) {
                m.tooltips.push((node.id.clone(), self.string(&v)?));
            }
        }
        let styles = self.root_field("defaultEdgeStyle")?;
        m.default_edge_style = if matches!(
            self.read.kind(&styles),
            LiveValueKind::Undefined | LiveValueKind::Null
        ) {
            None
        } else {
            Some(self.strings(&styles)?)
        };
        m.default_interpolate = self.scalar(&root, "defaultInterpolate")?;
        self.tick(
            m.subgraphs.len().saturating_mul(
                m.subgraphs
                    .len()
                    .saturating_add(m.nodes.len())
                    .saturating_add(1),
            ),
        )?;
        Ok(m)
    }
}
struct LiveTopology<'a, 'b, R: LiveRenderRead> {
    host: &'a mut Host<'b, R>,
    groups: R::Value,
    resources: &'a mut dyn RenderResources,
}
impl<R: LiveRenderRead> LiveTopology<'_, '_, R> {
    fn lookup(&mut self, id: &str) -> Result<R::Value, PublicError> {
        let root = self.host.read.root();
        let function = self.host.read.get(&root, "subgraph")?;
        self.host.tick(1)?;
        self.host.read.call(
            &function,
            &root,
            &[LiveCallArgument::Text(id.into())],
            self.resources,
            "model.subgraph is not a function",
        )
    }
    fn collapsed(&mut self, value: &R::Value) -> Result<bool, PublicError> {
        if matches!(
            self.host.read.kind(value),
            LiveValueKind::Undefined | LiveValueKind::Null
        ) {
            return Ok(false);
        }
        let metadata = self.host.read.get(value, "metadata")?;
        if matches!(
            self.host.read.kind(&metadata),
            LiveValueKind::Undefined | LiveValueKind::Null
        ) {
            return Ok(false);
        }
        let view = self.host.read.get(&metadata, "view")?;
        if self.host.read.kind(&view) != LiveValueKind::String {
            return Ok(false);
        }
        Ok(self.host.string(&view)? == "collapsed")
    }
}
impl<R: LiveRenderRead> super::topology::Source for LiveTopology<'_, '_, R> {
    type Value = R::Value;
    type Iter = R::Value;
    type Error = PublicError;
    fn groups(&mut self) -> R::Value {
        self.groups.clone()
    }
    fn members(&mut self, group: &R::Value) -> Result<R::Value, PublicError> {
        self.host.read.get(group, "nodes")
    }
    fn begin(&mut self, value: &R::Value) -> Result<R::Value, PublicError> {
        self.host.read.iterator_begin(value)
    }
    fn next(&mut self, it: &mut R::Value) -> Result<Option<R::Value>, PublicError> {
        self.host.read.iterator_next(it)
    }
    fn close(&mut self, it: &mut R::Value) {
        let _ = self.host.read.iterator_close(it);
    }
    fn id(&mut self, group: &R::Value) -> Result<String, PublicError> {
        self.host.text(group, "id")
    }
    fn child(&mut self, value: &R::Value) -> Result<String, PublicError> {
        self.host.string(value)
    }
    fn lookup_exists(&mut self, id: &str) -> Result<bool, PublicError> {
        let value = self.lookup(id)?;
        Ok(self.host.read.truthy(&value))
    }
    fn lookup_collapsed(&mut self, id: &str) -> Result<bool, PublicError> {
        let value = self.lookup(id)?;
        self.collapsed(&value)
    }
    fn own_collapsed(&mut self, group: &R::Value) -> Result<bool, PublicError> {
        self.collapsed(group)
    }
    fn work(&mut self, n: usize) -> Result<(), PublicError> {
        self.host.tick(n)
    }
    fn copy_text(&mut self, text: &str) -> Result<(), PublicError> {
        self.host.units = self.host.units.saturating_add(text.encode_utf16().count());
        if self.host.units > 16_000_000 {
            Err(limit().into())
        } else {
            Ok(())
        }
    }
}
impl<R: LiveRenderRead> PhaseInput for Host<'_, R> {
    type Error = PublicError;
    fn begin_draw(
        &mut self,
        curve: Option<&str>,
        resources: &mut dyn RenderResources,
    ) -> Result<FlowGraph, PublicError> {
        self.units = 0;
        let groups = self.root_field("subgraphs")?;
        let topology = super::topology::build(&mut LiveTopology {
            host: self,
            groups: groups.clone(),
            resources,
        })?;
        let m = self.graph_input(&groups)?;
        self.admit_compiled_styles(&m)?;
        Ok(finish_flow_graph(&m, curve, topology))
    }
    fn resolve_styles(&mut self, node: &mut GraphNode) -> Result<(), PublicError> {
        if let Some(i) = node.css_styles_origin {
            let v = self.styles.get(i).ok_or_else(limit)?.clone();
            node.css_styles = self.strings(&v)?;
        }
        Ok(())
    }
    fn direction(&mut self) -> Result<Option<String>, PublicError> {
        let root = self.read.root();
        self.optional(&root, "direction")
    }
    fn metadata(&mut self, key: &'static str) -> Result<Option<String>, PublicError> {
        let root = self.read.root();
        self.optional(&root, key)
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    input: &FlowchartDetachedInput,
    config: &Object,
    settings: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    render_variant(input, config, settings, read, resources, None)
}
pub(crate) fn render_variant<R: LiveRenderRead>(
    input: &FlowchartDetachedInput,
    config: &Object,
    settings: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
    variant: Option<&dyn super::render::FlowVariant>,
) -> Result<Rendered, PublicError> {
    if let Some(e) = &input.initial_error {
        return Err(e.clone().into());
    }
    super::render::render_phases(
        &mut Host {
            read,
            styles: Vec::new(),
            work: 0,
            units: 0,
        },
        config,
        settings,
        resources,
        variant,
    )
}
