//! First-draw graph construction. Only native-selected diagnostics suspend.
use super::{graph::AgentGraph, graph_source::Source, model::*};
use crate::compat::{
    public::{
        JsString, LiveCallArgument, LiveRenderRead, LiveStringUse, LiveValueKind, PublicError,
    },
    Object, RenderResources, Value,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
#[derive(Clone)]
pub(super) struct Built {
    pub graph: AgentGraph,
    pub styles: Vec<Option<u32>>,
}
pub(super) type Cache = Arc<Mutex<Option<Arc<Built>>>>;
pub(super) fn cache() -> Cache {
    Arc::new(Mutex::new(None))
}
struct Host<'a, R: LiveRenderRead> {
    read: &'a mut R,
    resources: &'a mut dyn RenderResources,
    root: R::Value,
    groups: R::Value,
    styles: HashMap<String, u32>,
    texts: HashMap<String, R::Value>,
    work: usize,
    units: usize,
}
impl<R: LiveRenderRead> Host<'_, R> {
    fn tick(&mut self, n: usize) -> Result<(), PublicError> {
        self.work = self.work.saturating_add(n);
        if self.work > 8_000_000 {
            Err(super::live_phase::limit().into())
        } else {
            Ok(())
        }
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
            return Err(super::live_phase::limit().into());
        }
        let s = s.well_formed().ok_or_else(|| {
            super::live_phase::error("Agentflow rendering requires well-formed UTF16 text")
        })?;
        self.texts.entry(s.clone()).or_insert_with(|| v.clone());
        Ok(s)
    }
    fn text(&mut self, v: &R::Value, k: &str) -> Result<String, PublicError> {
        let a = self.read.get(v, k)?;
        self.string(&a)
    }
    fn optional(&mut self, v: &R::Value, k: &str) -> Result<Option<String>, PublicError> {
        let a = self.read.get(v, k)?;
        if matches!(
            self.read.kind(&a),
            LiveValueKind::Undefined | LiveValueKind::Null
        ) {
            Ok(None)
        } else {
            self.string(&a).map(Some)
        }
    }
    fn root_field(&mut self, k: &str) -> Result<R::Value, PublicError> {
        self.read.get(&self.root, k)
    }
    fn values(&mut self, v: &R::Value) -> Result<Vec<R::Value>, PublicError> {
        let out = self.read.snapshot_iterable(v, 1_000_000)?;
        self.tick(out.len())?;
        Ok(out)
    }
    fn strings(&mut self, v: &R::Value) -> Result<Vec<String>, PublicError> {
        let out = self.values(v)?;
        out.iter().map(|a| self.string(a)).collect()
    }
    fn texts(&mut self, v: &R::Value, k: &str) -> Result<Vec<String>, PublicError> {
        let a = self.read.get(v, k)?;
        self.strings(&a)
    }
    fn scalar(&mut self, v: &R::Value, k: &str) -> Result<Option<Value>, PublicError> {
        let a = self.read.get(v, k)?;
        Ok(match self.read.kind(&a) {
            LiveValueKind::Undefined => None,
            LiveValueKind::Null => Some(Value::Null),
            LiveValueKind::Boolean => Some(Value::Bool(self.read.truthy(&a))),
            LiveValueKind::Number => {
                Some(Value::Number(self.read.number(&a).ok_or_else(|| {
                    super::live_phase::error("Agentflow expected number")
                })?))
            }
            LiveValueKind::String => Some(Value::String(self.string(&a)?)),
            _ => {
                return Err(super::live_phase::error(
                    "Agentflow object option coercion is not implemented",
                )
                .into())
            }
        })
    }
    fn label(&mut self, v: &R::Value) -> Result<LabelType, PublicError> {
        Ok(match self.optional(v, "labelType")?.as_deref() {
            Some("markdown") => LabelType::Markdown,
            Some("string") => LabelType::String,
            _ => LabelType::Text,
        })
    }
    fn method(
        &mut self,
        key: &str,
        args: &[LiveCallArgument<R::Value>],
        error: &'static str,
    ) -> Result<R::Value, PublicError> {
        let f = self.read.get(&self.root, key)?;
        self.read.call(&f, &self.root, args, self.resources, error)
    }
    fn metadata(&mut self, v: &R::Value) -> Result<Option<Object>, PublicError> {
        let a = self.read.get(v, "metadata")?;
        if !matches!(self.read.kind(&a), LiveValueKind::Object) {
            return Ok(None);
        }
        let view = self.optional(&a, "view")?;
        let mut out = Object::default();
        if let Some(view) = view {
            out.insert("view".into(), Value::String(view));
        }
        Ok(Some(out))
    }
}
impl<R: LiveRenderRead> Source for Host<'_, R> {
    type Group = R::Value;
    type Members = R::Value;
    type Error = PublicError;
    fn work(&mut self, n: usize) -> Result<(), PublicError> {
        self.tick(n)
    }
    fn group_count(&mut self) -> Result<usize, PublicError> {
        let n = self.read.get(&self.groups, "length")?;
        let n = self
            .read
            .number(&n)
            .ok_or_else(|| super::live_phase::error("Agentflow subgraphs require an array"))?;
        if !n.is_finite() || n < 0.0 || n.fract() != 0.0 || n > 1_000_000.0 {
            return Err(super::live_phase::limit().into());
        }
        Ok(n as usize)
    }
    fn group(&mut self, i: usize) -> Result<R::Value, PublicError> {
        self.read.index(&self.groups, i)
    }
    fn group_id(&mut self, g: &R::Value) -> Result<String, PublicError> {
        self.text(g, "id")
    }
    fn group_collapsed(&mut self, g: &R::Value) -> Result<bool, PublicError> {
        Ok(self.metadata(g)?.and_then(|m| m.get("view").cloned())
            == Some(Value::String("collapsed".into())))
    }
    fn members(&mut self, g: &R::Value) -> Result<R::Value, PublicError> {
        let v = self.read.get(g, "nodes")?;
        self.read.iterator_begin(&v)
    }
    fn next_member(&mut self, it: &mut R::Value) -> Result<Option<String>, PublicError> {
        let Some(v) = self.read.iterator_next(it)? else {
            return Ok(None);
        };
        match self.tick(1).and_then(|_| self.string(&v)) {
            Ok(value) => Ok(Some(value)),
            Err(error) => {
                let _ = self.read.iterator_close(it);
                Err(error)
            }
        }
    }
    fn close_members(&mut self, it: &mut R::Value) -> Result<(), PublicError> {
        self.read.iterator_close(it)
    }
    fn subgraph(&mut self, id: &str) -> Result<Option<R::Value>, PublicError> {
        let v = self.method(
            "subgraph",
            &[LiveCallArgument::Text(id.into())],
            "n.subgraph is not a function",
        )?;
        Ok(
            if matches!(
                self.read.kind(&v),
                LiveValueKind::Undefined | LiveValueKind::Null
            ) {
                None
            } else {
                Some(v)
            },
        )
    }
    fn project_group(&mut self, g: &R::Value) -> Result<AgentSubgraph, PublicError> {
        Ok(AgentSubgraph {
            id: self.text(g, "id")?,
            group_type: self.text(g, "type")?,
            nodes: Vec::new(),
            title: self.text(g, "title")?,
            classes: self.texts(g, "classes")?,
            dir: self.optional(g, "dir")?,
            label_type: self.label(g)?,
            metadata: self.metadata(g)?,
        })
    }
    fn nodes(&mut self) -> Result<Vec<AgentNode>, PublicError> {
        let map = self.root_field("nodes")?;
        let items = self.read.snapshot_map_values(&map, 1_000_000)?;
        self.tick(items.len())?;
        let mut out = Vec::with_capacity(items.len());
        for v in items {
            let id = self.text(&v, "id")?;
            let styles = self.read.get(&v, "styles")?;
            let token = self.read.retain_value(&styles)?;
            self.styles.insert(id.clone(), token);
            out.push(AgentNode {
                id,
                text: self.optional(&v, "text")?,
                label_type: self.label(&v)?,
                shape: self.optional(&v, "type")?,
                styles: vec![],
                classes: self.texts(&v, "classes")?,
                dir: self.optional(&v, "dir")?,
                props: None,
                link: self.optional(&v, "link")?,
                link_target: self.optional(&v, "linkTarget")?,
                metadata: self.metadata(&v)?,
                is_connector: None,
            });
        }
        Ok(out)
    }
    fn edges(&mut self) -> Result<Vec<AgentEdge>, PublicError> {
        let list = self.root_field("edges")?;
        let values = self.values(&list)?;
        let mut out = Vec::with_capacity(values.len());
        for v in values {
            let styles = self.read.get(&v, "style")?;
            let style = if self.read.truthy(&styles) {
                Some(self.strings(&styles)?)
            } else {
                None
            };
            let user = self.read.get(&v, "isUserDefinedId")?;
            out.push(AgentEdge {
                start: self.text(&v, "start")?,
                end: self.text(&v, "end")?,
                edge_type: self.optional(&v, "type")?,
                text: self.text(&v, "text")?,
                label_type: self.label(&v)?,
                stroke: self.optional(&v, "stroke")?,
                length: self.scalar(&v, "length")?,
                edge_semantic: None,
                metadata: self.metadata(&v)?,
                style,
                classes: self.texts(&v, "classes")?,
                id: self.optional(&v, "id")?,
                is_user_defined_id: self.read.truthy(&user),
                interpolate: self.scalar(&v, "interpolate")?,
                animate: self.scalar(&v, "animate")?,
                animation: self.scalar(&v, "animation")?,
            });
        }
        Ok(out)
    }
    fn class(&mut self, key: &str) -> Result<Option<AgentClassDef>, PublicError> {
        let map = self.root_field("classes")?;
        let v = self.read.map_get_text(&map, key)?;
        if matches!(
            self.read.kind(&v),
            LiveValueKind::Undefined | LiveValueKind::Null
        ) {
            return Ok(None);
        }
        Ok(Some(AgentClassDef {
            id: key.into(),
            styles: self.texts(&v, "styles")?,
            text_styles: self.texts(&v, "textStyles")?,
        }))
    }
    fn tooltip(&mut self, key: &str) -> Result<Option<String>, PublicError> {
        let map = self.root_field("tooltips")?;
        let v = self.read.map_get_text(&map, key)?;
        if matches!(
            self.read.kind(&v),
            LiveValueKind::Undefined | LiveValueKind::Null
        ) {
            Ok(None)
        } else {
            self.string(&v).map(Some)
        }
    }
    fn default_styles(&mut self) -> Result<Vec<String>, PublicError> {
        let v = self.root_field("defaultEdgeStyle")?;
        if matches!(
            self.read.kind(&v),
            LiveValueKind::Undefined | LiveValueKind::Null
        ) {
            Ok(vec![])
        } else {
            self.strings(&v)
        }
    }
    fn default_curve(&mut self) -> Result<Option<Value>, PublicError> {
        let root = self.root.clone();
        self.scalar(&root, "defaultInterpolate")
    }
    fn kind(&mut self, id: &str) -> Result<Option<String>, PublicError> {
        // The logger may replace nodes while shape diagnostics are emitted.
        // Resolve the current vertex at this late upstream read point.
        let nodes = self.root_field("nodes")?;
        let vertex = self.read.map_get_text(&nodes, id)?;
        if !self.read.truthy(&vertex) {
            return Ok(None);
        }
        let v = self.method(
            "vertexKind",
            &[LiveCallArgument::Value(vertex)],
            "n.vertexKind is not a function",
        )?;
        if matches!(
            self.read.kind(&v),
            LiveValueKind::Undefined | LiveValueKind::Null
        ) {
            Ok(None)
        } else {
            self.string(&v).map(Some)
        }
    }
    fn emit(
        &mut self,
        id: DiagnosticId,
        severity: &str,
        message: String,
        node: &str,
    ) -> Result<(), PublicError> {
        let (name, code, error) = match id {
            DiagnosticId::ContainmentViolation => (
                "emitWarning",
                "CONTAINMENT_VIOLATION",
                "n.emitWarning is not a function",
            ),
            DiagnosticId::ShapeRemoved => (
                "emitError",
                "SHAPE_REMOVED",
                "t.emitError is not a function",
            ),
            DiagnosticId::ShapeUnsupported => (
                "emitWarning",
                "SHAPE_UNSUPPORTED",
                "t.emitWarning is not a function",
            ),
            _ => return Err(super::live_phase::error("Unexpected graph diagnostic").into()),
        };
        debug_assert_eq!(
            severity,
            if name == "emitError" {
                "error"
            } else {
                "warning"
            }
        );
        let value = self
            .texts
            .get(node)
            .cloned()
            .ok_or_else(|| super::live_phase::error("Missing diagnostic node identity"))?;
        self.method(
            name,
            &[
                LiveCallArgument::Text(code.into()),
                LiveCallArgument::Text(message.as_str().into()),
                LiveCallArgument::Record(vec![(JsString::from("nodeId"), value)]),
            ],
            error,
        )?;
        Ok(())
    }
}
pub(super) fn build<R: LiveRenderRead>(
    read: &mut R,
    resources: &mut dyn RenderResources,
    config: &Object,
) -> Result<Built, PublicError> {
    let root = read.root();
    let groups = read.get(&root, "subgraphs")?;
    let mut host = Host {
        read,
        resources,
        root,
        groups,
        styles: HashMap::new(),
        texts: HashMap::new(),
        work: 0,
        units: 0,
    };
    let graph = super::graph_source::build(&mut host, config)?;
    let styles = graph
        .nodes
        .iter()
        .map(|node| host.styles.get(&node.id).copied())
        .collect();
    Ok(Built { graph, styles })
}
