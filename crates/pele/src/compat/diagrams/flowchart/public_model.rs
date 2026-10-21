//! Native mutable FlowDb and ordered identity graph, Pele 458322a5 (MIT, kepano).
use super::{FlowClassDef, FlowEdge, FlowNode, FlowSubgraph, Flowchart, FlowchartModel, LabelType};
use crate::compat::{
    public::*,
    source::{space, yaml},
    DiagramModule, DiagramType, ErrorCode, Object, PeleError, RenderResources, RenderSettings,
    Rendered, Value,
};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
const MAX_ITEMS: usize = 1_000_000;
const MAX_WORK: usize = 8_000_000;
const MAX_TEXT: usize = 16_000_000;
const PROTOTYPE: PrototypeId = PrototypeId(1);
/// Parse-only observables erased by the value convenience model. Never serialized.
#[derive(Clone, Debug, Default)]
pub struct FlowchartPublicOrigin {
    pub nodes: Vec<Vec<&'static str>>,
    pub edges: Vec<Vec<&'static str>>,
    pub members: Vec<String>,
    pub sub_count: usize,
    pub max_edges: usize,
    pub inherit_dir: bool,
    pub edge_styles: Vec<Option<usize>>,
    pub default_style: Option<usize>,
    pub next_style: usize,
}
impl FlowchartPublicOrigin {
    pub(super) fn key(keys: &mut Vec<&'static str>, key: &'static str) {
        if !keys.contains(&key) {
            keys.push(key)
        }
    }
}
const fn f(name: &'static str, kind: FieldKind, undefined: bool) -> FieldSchema {
    FieldSchema {
        name,
        kind,
        optional: false,
        undefined,
        nullable: false,
        flags: 7,
    }
}
const fn m(id: u32, name: &'static str, arity: u32) -> MethodSchema {
    MethodSchema {
        id: MethodId(id),
        name,
        arity,
    }
}
const fn p(id: u32, name: &'static str) -> MemberSchema {
    MemberSchema {
        name,
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(id),
        },
    }
}
static SCHEMA: ModelSchema = ModelSchema {
    id: "pele/flowchart@1",
    version: 1,
    root: Some(RecordId(1)),
    records: &[RecordSchema {
        id: RecordId(1),
        name: "FlowchartModel",
        kind: RecordKind::Object,
        prototype: Some(PROTOTYPE),
        fields: &[
            f("type", FieldKind::String, false),
            f("direction", FieldKind::String, true),
            f("nodes", FieldKind::Map, false),
            f("edges", FieldKind::Array, false),
            f("subgraphs", FieldKind::Array, false),
            f("classes", FieldKind::Map, false),
            f("tooltips", FieldKind::Map, false),
            f("defaultEdgeStyle", FieldKind::Array, true),
            f("defaultInterpolate", FieldKind::Any, true),
            f("title", FieldKind::String, true),
            f("accTitle", FieldKind::String, true),
            f("accDescr", FieldKind::String, true),
            f("subgraphLookup", FieldKind::Map, false),
            f("members", FieldKind::Set, false),
            f("edgeById", FieldKind::Map, false),
            f("pairCount", FieldKind::Map, false),
            f("subCount", FieldKind::Number, false),
            FieldSchema {
                name: "options",
                kind: FieldKind::Any,
                optional: true,
                undefined: true,
                nullable: true,
                flags: 7,
            },
        ],
        members: &[],
    }],
    prototypes: &[PrototypeSchema {
        id: PROTOTYPE,
        name: "FlowDb",
        parent: PrototypeParent::Object,
        members: &[
            p(1, "addVertex"),
            p(2, "addSingleLink"),
            p(3, "addLink"),
            p(4, "updateLinkInterpolate"),
            p(5, "updateLink"),
            p(6, "edgeAt"),
            p(7, "addClass"),
            p(8, "setDirection"),
            p(9, "setClass"),
            p(10, "setTooltip"),
            p(11, "setLink"),
            p(12, "setClickEvent"),
            p(13, "setAccTitle"),
            p(14, "setAccDescription"),
            p(15, "addSubGraph"),
            p(16, "exists"),
            p(17, "makeUniq"),
            p(18, "destructLink"),
            p(19, "subgraph"),
        ],
    }],
    methods: &[
        m(1, "addVertex", 6),
        m(2, "addSingleLink", 4),
        m(3, "addLink", 3),
        m(4, "updateLinkInterpolate", 2),
        m(5, "updateLink", 2),
        m(6, "edgeAt", 1),
        m(7, "addClass", 2),
        m(8, "setDirection", 1),
        m(9, "setClass", 2),
        m(10, "setTooltip", 2),
        m(11, "setLink", 3),
        m(12, "setClickEvent", 3),
        m(13, "setAccTitle", 1),
        m(14, "setAccDescription", 1),
        m(15, "addSubGraph", 3),
        m(16, "exists", 2),
        m(17, "makeUniq", 2),
        m(18, "destructLink", 2),
        m(19, "subgraph", 1),
    ],
};
pub struct FlowchartPublicState {
    graph: ModelGraph,
    imported: std::cell::OnceCell<Result<FlowchartModel, PeleError>>,
}
impl FlowchartPublicState {
    pub(crate) fn detached_seed(&self) -> super::FlowchartDetachedInput {
        super::FlowchartDetachedInput::new(self.prepared_render_model().err())
    }

    /// Shared DB projection for actual renderer variants such as Swimlane.
    /// Registration remains owned by the calling public document.
    pub(crate) fn prepared_render_model(&self) -> Result<&FlowchartModel, PublicError> {
        let model = self.render_model()?;
        if matches!(self.graph.root,Atom::Ref(id) if matches!(self.graph.nodes[id as usize],Node::Object{..}))
            && model.subgraphs.iter().any(|group| !group.nodes.is_empty())
        {
            return Err(ty("o.subgraph is not a function"));
        }
        Ok(model)
    }
    pub(crate) fn render_model(&self) -> Result<&FlowchartModel, PeleError> {
        self.imported
            .get_or_init(|| imported(&self.graph))
            .as_ref()
            .map_err(Clone::clone)
    }
}
fn error(message: impl Into<String>) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("flowchart".into());
    e
}
fn limit() -> PeleError {
    let mut e = error("Flowchart public model expansion exceeds the work budget");
    e.code = ErrorCode::Limit;
    e
}
fn ty(message: &str) -> PublicError {
    PublicError::JavaScript {
        name: JavaScriptErrorKind::TypeError,
        message: message.into(),
    }
}
fn get<'a>(g: &'a ModelGraph, id: u32, key: &str) -> &'a Atom {
    g.nodes[id as usize]
        .properties()
        .iter()
        .find(|p| g.strings[p.key as usize].is(key))
        .map_or(&Atom::Undefined, |p| &p.value)
}
fn reference(a: &Atom) -> Result<u32, PeleError> {
    if let Atom::Ref(id) = a {
        Ok(*id)
    } else {
        Err(error("Flowchart public model requires an object reference"))
    }
}
fn text(g: &ModelGraph, a: &Atom) -> Result<String, PeleError> {
    g.text(a)?
        .well_formed()
        .ok_or_else(|| error("Flowchart public text must be a well-formed string"))
}
fn dense(g: &ModelGraph, a: &Atom) -> Result<Vec<Atom>, PeleError> {
    let id = reference(a)?;
    let Node::Array { length, properties } = &g.nodes[id as usize] else {
        return Err(error("Flowchart public collection must be an Array"));
    };
    if *length as usize > MAX_ITEMS {
        return Err(limit());
    }
    let mut values = vec![None; *length as usize];
    for p in properties {
        let Some(k) = g.strings[p.key as usize].well_formed() else {
            continue;
        };
        if let Ok(i) = k.parse::<usize>() {
            if i < values.len() && i.to_string() == k {
                values[i] = Some(p.value.clone())
            }
        }
    }
    values
        .into_iter()
        .map(|v| v.ok_or_else(|| error("Sparse Flowchart public collections are not implemented")))
        .collect()
}
fn entries(g: &ModelGraph, a: &Atom) -> Result<Vec<(Atom, Atom)>, PeleError> {
    let id = reference(a)?;
    if let Node::Map { entries, .. } = &g.nodes[id as usize] {
        if entries.len() > MAX_ITEMS {
            return Err(limit());
        }
        Ok(entries.clone())
    } else {
        Err(error("Flowchart public collection must be a Map"))
    }
}
struct Reader<'a> {
    g: &'a ModelGraph,
    work: usize,
    units: usize,
    active: HashSet<u32>,
}
impl Reader<'_> {
    fn charge(&mut self, n: usize) -> Result<(), PeleError> {
        self.work = self.work.checked_add(n).ok_or_else(limit)?;
        if self.work > MAX_WORK {
            Err(limit())
        } else {
            Ok(())
        }
    }
    fn string(&mut self, a: &Atom) -> Result<String, PeleError> {
        if let Atom::String(i) = a {
            self.units = self
                .units
                .checked_add(self.g.strings[*i as usize].0.len())
                .ok_or_else(limit)?;
            if self.units > MAX_TEXT {
                return Err(limit());
            }
        }
        text(self.g, a)
    }
    fn opt(&mut self, a: &Atom) -> Result<Option<String>, PeleError> {
        if matches!(a, Atom::Undefined) {
            Ok(None)
        } else {
            self.string(a).map(Some)
        }
    }
    fn strings(&mut self, a: &Atom) -> Result<Vec<String>, PeleError> {
        let values = dense(self.g, a)?;
        self.charge(values.len())?;
        values.iter().map(|a| self.string(a)).collect()
    }
    fn value(&mut self, a: &Atom, depth: usize) -> Result<Value, PeleError> {
        self.charge(1)?;
        if depth > 128 {
            return Err(limit());
        }
        Ok(match a {
            Atom::Null => Value::Null,
            Atom::Bool(v) => Value::Bool(*v),
            Atom::Number(v) => Value::Number(*v),
            Atom::String(_) => Value::String(self.string(a)?),
            Atom::Ref(id) => {
                if !self.active.insert(*id) {
                    return Err(error(
                        "Cyclic rendered Flowchart properties are not implemented",
                    ));
                }
                let v = match &self.g.nodes[*id as usize] {
                    Node::Array { .. } => {
                        let values = dense(self.g, a)?;
                        Value::Array(
                            values
                                .iter()
                                .map(|v| self.value(v, depth + 1))
                                .collect::<Result<_, _>>()?,
                        )
                    }
                    Node::Object { properties } | Node::Instance { properties, .. } => {
                        let properties = properties.clone();
                        let mut o = Object::default();
                        for p in properties {
                            if p.flags & 1 == 0 || matches!(p.value, Atom::Undefined) {
                                continue;
                            }
                            let key = self.string(&Atom::String(p.key))?;
                            o.insert(key, self.value(&p.value, depth + 1)?)
                        }
                        Value::Object(o)
                    }
                    _ => return Err(error("Flowchart rendered properties require plain objects")),
                };
                self.active.remove(id);
                v
            }
            _ => return Err(error("Flowchart rendered property type is not implemented")),
        })
    }
    fn option_value(&mut self, a: &Atom) -> Result<Option<Value>, PeleError> {
        if matches!(a, Atom::Undefined) {
            Ok(None)
        } else {
            self.value(a, 0).map(Some)
        }
    }
    fn label(&self, a: &Atom) -> Result<LabelType, PeleError> {
        match text(self.g, a)?.as_str() {
            "text" => Ok(LabelType::Text),
            "string" => Ok(LabelType::String),
            "markdown" => Ok(LabelType::Markdown),
            _ => Err(error("Unknown Flowchart public label type")),
        }
    }
}
pub(crate) fn imported(g: &ModelGraph) -> Result<FlowchartModel, PeleError> {
    let root = reference(&g.root)?;
    let mut r = Reader {
        g,
        work: 0,
        units: 0,
        active: HashSet::new(),
    };
    let mut nodes = Vec::new();
    for (k, a) in entries(g, get(g, root, "nodes"))? {
        r.charge(1)?;
        let key = r.string(&k)?;
        let id = reference(&a)?;
        nodes.push((
            key,
            FlowNode {
                id: r.string(get(g, id, "id"))?,
                text: r.opt(get(g, id, "text"))?,
                label_type: r.label(get(g, id, "labelType"))?,
                shape: r.opt(get(g, id, "type"))?,
                styles: r.strings(get(g, id, "styles"))?,
                classes: r.strings(get(g, id, "classes"))?,
                dir: r.opt(get(g, id, "dir"))?,
                // props is builder bookkeeping; graph/layout/render never reads it.
                // Retain its exact aliases, cycles and UTF-16 in the authoritative graph.
                props: None,
                link: r.opt(get(g, id, "link"))?,
                link_target: r.opt(get(g, id, "linkTarget"))?,
                icon: r.opt(get(g, id, "icon"))?,
                form: r.opt(get(g, id, "form"))?,
                pos: r.opt(get(g, id, "pos"))?,
                img: r.opt(get(g, id, "img"))?,
                constraint: r.opt(get(g, id, "constraint"))?,
                asset_width: r.option_value(get(g, id, "assetWidth"))?,
                asset_height: r.option_value(get(g, id, "assetHeight"))?,
            },
        ));
    }
    let mut edges = Vec::new();
    for a in dense(g, get(g, root, "edges"))? {
        r.charge(1)?;
        let id = reference(&a)?;
        edges.push(FlowEdge {
            start: r.string(get(g, id, "start"))?,
            end: r.string(get(g, id, "end"))?,
            edge_type: r.opt(get(g, id, "type"))?,
            text: r.string(get(g, id, "text"))?,
            label_type: r.label(get(g, id, "labelType"))?,
            stroke: r.opt(get(g, id, "stroke"))?,
            length: r.option_value(get(g, id, "length"))?,
            style: if matches!(get(g, id, "style"), Atom::Undefined) {
                None
            } else {
                Some(r.strings(get(g, id, "style"))?)
            },
            classes: r.strings(get(g, id, "classes"))?,
            id: r.opt(get(g, id, "id"))?,
            is_user_defined_id: matches!(get(g, id, "isUserDefinedId"), Atom::Bool(true)),
            interpolate: r.option_value(get(g, id, "interpolate"))?,
            animate: r.option_value(get(g, id, "animate"))?,
            animation: r.option_value(get(g, id, "animation"))?,
        });
    }
    let mut subgraphs = Vec::new();
    for a in dense(g, get(g, root, "subgraphs"))? {
        r.charge(1)?;
        let id = reference(&a)?;
        subgraphs.push(FlowSubgraph {
            id: r.string(get(g, id, "id"))?,
            nodes: r.strings(get(g, id, "nodes"))?,
            title: r.string(get(g, id, "title"))?,
            classes: r.strings(get(g, id, "classes"))?,
            dir: r.opt(get(g, id, "dir"))?,
            label_type: r.label(get(g, id, "labelType"))?,
            metadata: {
                // Only metadata.view is observed by graph construction.
                let metadata = get(g,id,"metadata");
                if let Atom::Ref(mid) = metadata {
                    if matches!(get(g,*mid,"view"),Atom::String(i) if g.strings[*i as usize].is("collapsed")) {
                        let mut o=Object::default();o.insert("view".into(),Value::String("collapsed".into()));Some(o)
                    } else {None}
                } else {None}
            },
        });
    }
    let mut classes = Vec::new();
    for (k, a) in entries(g, get(g, root, "classes"))? {
        r.charge(1)?;
        let key = r.string(&k)?;
        let id = reference(&a)?;
        classes.push((
            key,
            FlowClassDef {
                id: r.string(get(g, id, "id"))?,
                styles: r.strings(get(g, id, "styles"))?,
                text_styles: r.strings(get(g, id, "textStyles"))?,
            },
        ));
    }
    let mut tooltips = Vec::new();
    for (k, v) in entries(g, get(g, root, "tooltips"))? {
        r.charge(1)?;
        tooltips.push((r.string(&k)?, r.string(&v)?));
    }
    Ok(FlowchartModel {
        diagram_type: DiagramType::Flowchart,
        direction: r.opt(get(g, root, "direction"))?,
        nodes,
        edges,
        subgraphs,
        classes,
        tooltips,
        default_edge_style: if matches!(get(g, root, "defaultEdgeStyle"), Atom::Undefined) {
            None
        } else {
            Some(r.strings(get(g, root, "defaultEdgeStyle"))?)
        },
        default_interpolate: r.option_value(get(g, root, "defaultInterpolate"))?,
        title: r.opt(get(g, root, "title"))?,
        acc_title: r.opt(get(g, root, "accTitle"))?,
        acc_descr: r.opt(get(g, root, "accDescr"))?,
        public_origin: FlowchartPublicOrigin::default(),
    })
}
fn label(t: &LabelType) -> &'static str {
    match t {
        LabelType::Text => "text",
        LabelType::String => "string",
        LabelType::Markdown => "markdown",
    }
}
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum Key {
    Undefined,
    Null,
    Bool(bool),
    Number(u64),
    String(u32),
    Ref(u32),
    Builtin(u32),
    Host(u32),
}
struct Arena<'a> {
    g: &'a mut ModelGraph,
    units: usize,
    work: usize,
    string_ids: HashMap<JsString, u32>,
    canonical_strings: Vec<u32>,
    text_work: usize,
    maps: RefCell<HashMap<u32, HashMap<Key, usize>>>,
    properties: RefCell<HashMap<u32, HashMap<u32, usize>>>,
}
impl<'a> Arena<'a> {
    fn new(g: &'a mut ModelGraph) -> Self {
        let units = g.strings.iter().map(|s| s.0.len()).sum();
        let work = g
            .nodes
            .iter()
            .map(|n| {
                n.properties().len()
                    + match n {
                        Node::Map { entries, .. } => entries.len() * 2,
                        Node::Set { values, .. } => values.len(),
                        _ => 0,
                    }
            })
            .sum();
        let mut string_ids = HashMap::new();
        let mut canonical_strings = Vec::with_capacity(g.strings.len());
        for (i, s) in g.strings.iter().enumerate() {
            canonical_strings.push(*string_ids.entry(s.clone()).or_insert(i as u32));
        }
        Self {
            g,
            units,
            work,
            string_ids,
            canonical_strings,
            text_work: 0,
            maps: RefCell::default(),
            properties: RefCell::default(),
        }
    }
    fn key(&self, a: &Atom) -> Key {
        match a {
            Atom::Undefined => Key::Undefined,
            Atom::Null => Key::Null,
            Atom::Bool(v) => Key::Bool(*v),
            Atom::Number(v) => Key::Number(if v.is_nan() {
                f64::NAN.to_bits()
            } else if *v == 0. {
                0
            } else {
                v.to_bits()
            }),
            Atom::String(i) => Key::String(self.canonical_strings[*i as usize]),
            Atom::Ref(i) => Key::Ref(*i),
            Atom::BuiltinFunction(i) => Key::Builtin(*i),
            Atom::HostReference(i) => Key::Host(*i),
        }
    }
    fn property_position(&self, id: u32, key: &str) -> Option<usize> {
        let mut buffer = [0; 64];
        let canonical = *match JsString::short_utf16(key, &mut buffer) {
            Some(units) => self.string_ids.get(units),
            None => self.string_ids.get(&JsString::from(key)),
        }?;
        let mut cache = self.properties.borrow_mut();
        let index = cache.entry(id).or_insert_with(|| {
            self.g.nodes[id as usize]
                .properties()
                .iter()
                .enumerate()
                .map(|(i, p)| (self.canonical_strings[p.key as usize], i))
                .collect()
        });
        index.get(&canonical).copied()
    }
    fn map_position(&self, id: u32, key: &Atom) -> Result<Option<usize>, PublicError> {
        let Node::Map { entries, .. } = &self.g.nodes[id as usize] else {
            return Err(ty("Flowchart collection must be a Map"));
        };
        let key = self.key(key);
        let mut cache = self.maps.borrow_mut();
        let index = cache.entry(id).or_insert_with(|| {
            let mut index = HashMap::new();
            for (i, (k, _)) in entries.iter().enumerate() {
                index.entry(self.key(k)).or_insert(i);
            }
            index
        });
        Ok(index.get(&key).copied())
    }
    fn intern(&mut self, s: JsString) -> Result<u32, PublicError> {
        self.text_work = self.text_work.checked_add(s.0.len()).ok_or_else(limit)?;
        if self.text_work > MAX_TEXT {
            return Err(limit().into());
        }
        if let Some(id) = self.string_ids.get(&s) {
            return Ok(*id);
        }
        if self.g.strings.len() >= MAX_ITEMS || self.units.saturating_add(s.0.len()) > MAX_TEXT {
            return Err(limit().into());
        }
        self.units += s.0.len();
        let id = self.g.strings.len() as u32;
        self.string_ids.insert(s.clone(), id);
        self.canonical_strings.push(id);
        self.g.strings.push(s);
        Ok(id)
    }
    fn intern_key(&mut self, key: &str) -> Result<u32, PublicError> {
        let mut buffer = [0; 64];
        if let Some(units) = JsString::short_utf16(key, &mut buffer) {
            if let Some(id) = self.string_ids.get(units).copied() {
                // A repeated key still consumes the same text-work budget as
                // intern. Only its temporary owned UTF-16 allocation disappears.
                self.text_work = self.text_work.checked_add(units.len()).ok_or_else(limit)?;
                if self.text_work > MAX_TEXT {
                    return Err(limit().into());
                }
                return Ok(id);
            }
        }
        // Misses and long keys use the original admission/order/budget path,
        // and are charged exactly once there.
        self.intern(key.into())
    }
    fn string(&mut self, s: impl Into<JsString>) -> Result<Atom, PublicError> {
        self.intern(s.into()).map(Atom::String)
    }
    fn node(&mut self, n: Node) -> Result<u32, PublicError> {
        if self.g.nodes.len() >= MAX_ITEMS {
            return Err(limit().into());
        }
        let id = self.g.nodes.len() as u32;
        self.g.nodes.push(n);
        Ok(id)
    }
    fn object(&mut self) -> Result<u32, PublicError> {
        self.node(Node::Object { properties: vec![] })
    }
    fn assign(&mut self, id: u32, key: &str, value: Atom) -> Result<(), PublicError> {
        if let Some(i) = self.property_position(id, key) {
            let p = &mut self.g.nodes[id as usize].properties_mut()[i];
            if p.flags & 2 == 0 {
                return Err(ty("Cannot assign to a read only Flowchart property"));
            }
            p.value = value;
            return Ok(());
        }
        if self.work >= MAX_WORK {
            return Err(limit().into());
        }
        self.work += 1;
        let key = self.intern_key(key)?;
        let index = self.g.nodes[id as usize].properties().len();
        self.g.nodes[id as usize].properties_mut().push(Property {
            key,
            value,
            flags: 7,
        });
        if let Some(properties) = self.properties.get_mut().get_mut(&id) {
            properties.insert(key, index);
        }

        Ok(())
    }
    fn field(&self, id: u32, key: &str) -> Atom {
        self.property_position(id, key)
            .map_or(Atom::Undefined, |i| {
                self.g.nodes[id as usize].properties()[i].value.clone()
            })
    }
    fn id(&self, a: &Atom) -> Result<u32, PublicError> {
        reference(a).map_err(|_| ty("Flowchart value must be an object"))
    }
    fn field_id(&self, id: u32, key: &str) -> Result<u32, PublicError> {
        self.id(get(self.g, id, key))
    }
    fn array(&mut self, values: Vec<Atom>) -> Result<u32, PublicError> {
        if values.len() > MAX_ITEMS {
            return Err(limit().into());
        }
        let id = self.node(Node::Array {
            length: values.len() as u32,
            properties: vec![],
        })?;
        for (i, v) in values.into_iter().enumerate() {
            self.assign(id, &i.to_string(), v)?
        }
        Ok(id)
    }
    fn strings(&mut self, values: &[String]) -> Result<u32, PublicError> {
        let values = values
            .iter()
            .map(|s| self.string(s.as_str()))
            .collect::<Result<Vec<_>, _>>()?;
        self.array(values)
    }
    // Only for objects allocated by record/ordered in this call. Track keys in
    // insertion order rather than building the general imported-node index.
    fn assign_fresh<'k>(
        &mut self,
        id: u32,
        keys: &mut Vec<&'k str>,
        key: &'k str,
        value: Atom,
    ) -> Result<(), PublicError> {
        if let Some(index) = keys.iter().position(|existing| *existing == key) {
            self.g.nodes[id as usize].properties_mut()[index].value = value;
            return Ok(());
        }
        if self.work >= MAX_WORK {
            return Err(limit().into());
        }
        self.work += 1;
        let string_id = self.intern_key(key)?;
        self.g.nodes[id as usize].properties_mut().push(Property {
            key: string_id,
            value,
            flags: 7,
        });
        keys.push(key);
        Ok(())
    }
    fn record(&mut self, values: Vec<(&str, Atom)>) -> Result<u32, PublicError> {
        let id = self.object()?;
        let mut keys = Vec::with_capacity(values.len());
        for (k, v) in values {
            self.assign_fresh(id, &mut keys, k, v)?
        }
        Ok(id)
    }
    fn push(&mut self, id: u32, value: Atom) -> Result<(), PublicError> {
        if self.g.nodes[id as usize]
            .properties()
            .iter()
            .any(|p| self.g.strings[p.key as usize].is("push"))
        {
            return Err(ty(
                "Overridden Flowchart collection push is not implemented",
            ));
        }
        let Node::Array { length, .. } = self.g.nodes[id as usize] else {
            return Err(ty("Flowchart collection push is not a function"));
        };
        if length as usize >= MAX_ITEMS {
            return Err(limit().into());
        }
        self.assign(id, &length.to_string(), value)?;
        let Node::Array { length: len, .. } = &mut self.g.nodes[id as usize] else {
            unreachable!()
        };
        *len = length + 1;
        Ok(())
    }
    fn map(&mut self) -> Result<u32, PublicError> {
        self.node(Node::Map {
            entries: vec![],
            properties: vec![],
        })
    }
    fn map_get(&self, id: u32, key: &Atom) -> Result<Atom, PublicError> {
        let i = self.map_position(id, key)?;
        let Node::Map { entries, .. } = &self.g.nodes[id as usize] else {
            unreachable!()
        };
        Ok(i.map_or(Atom::Undefined, |i| entries[i].1.clone()))
    }
    fn map_has(&self, id: u32, key: &Atom) -> Result<bool, PublicError> {
        Ok(self.map_position(id, key)?.is_some())
    }
    fn map_set(&mut self, id: u32, key: Atom, value: Atom) -> Result<(), PublicError> {
        let position = self.map_position(id, &key)?;
        let cache_key = self.key(&key);
        let Node::Map { entries, .. } = &mut self.g.nodes[id as usize] else {
            unreachable!()
        };
        if let Some(i) = position {
            entries[i].1 = value
        } else {
            if entries.len() >= MAX_ITEMS || self.work + 2 > MAX_WORK {
                return Err(limit().into());
            }
            self.work += 2;
            let i = entries.len();
            entries.push((key, value));
            self.maps
                .get_mut()
                .get_mut(&id)
                .unwrap()
                .insert(cache_key, i);
        }
        Ok(())
    }
    fn value(&mut self, v: &Value) -> Result<Atom, PublicError> {
        Ok(match v {
            Value::Null => Atom::Null,
            Value::Bool(v) => Atom::Bool(*v),
            Value::Number(v) => Atom::Number(*v),
            Value::String(s) => self.string(s.as_str())?,
            Value::Array(values) => {
                let values = values
                    .iter()
                    .map(|v| self.value(v))
                    .collect::<Result<Vec<_>, _>>()?;
                Atom::Ref(self.array(values)?)
            }
            Value::Object(o) => {
                let id = self.object()?;
                for (k, v) in o.entries() {
                    let v = self.value(v)?;
                    self.assign(id, k, v)?
                }
                Atom::Ref(id)
            }
        })
    }
    fn opt(&mut self, s: Option<&String>) -> Result<Atom, PublicError> {
        s.map_or(Ok(Atom::Undefined), |s| self.string(s.as_str()))
    }
    fn opt_value(&mut self, v: Option<&Value>) -> Result<Atom, PublicError> {
        v.map_or(Ok(Atom::Undefined), |v| self.value(v))
    }
    fn ordered(
        &mut self,
        fields: Vec<(&'static str, Atom)>,
        order: &[&'static str],
    ) -> Result<u32, PublicError> {
        let id = self.object()?;
        let mut keys = Vec::with_capacity(fields.len());
        for key in order {
            if let Some((_, v)) = fields.iter().find(|(k, _)| k == key) {
                self.assign_fresh(id, &mut keys, key, v.clone())?
            }
        }
        for (k, v) in fields {
            if !order.contains(&k) && !matches!(v, Atom::Undefined) {
                self.assign_fresh(id, &mut keys, k, v)?
            }
        }
        Ok(id)
    }
}
fn same(g: &ModelGraph, a: &Atom, b: &Atom) -> bool {
    match (a, b) {
        (Atom::Undefined, Atom::Undefined) | (Atom::Null, Atom::Null) => true,
        (Atom::Bool(a), Atom::Bool(b)) => a == b,
        (Atom::Number(a), Atom::Number(b)) => a == b || a.is_nan() && b.is_nan(),
        (Atom::String(a), Atom::String(b)) => g.strings[*a as usize] == g.strings[*b as usize],
        (Atom::Ref(a), Atom::Ref(b))
        | (Atom::BuiltinFunction(a), Atom::BuiltinFunction(b))
        | (Atom::HostReference(a), Atom::HostReference(b)) => a == b,
        _ => false,
    }
}
fn new_graph(
    m: &FlowchartModel,
    limit: Option<crate::compat::EdgeLimit>,
) -> Result<ModelGraph, PublicError> {
    new_graph_with_options(m, limit, None)
}
fn new_graph_with_options(
    m: &FlowchartModel,
    limit: Option<crate::compat::EdgeLimit>,
    constructor: Option<ConstructorArgument>,
) -> Result<ModelGraph, PublicError> {
    let mut graph = ModelGraph {
        version: 1,
        root: Atom::Undefined,
        strings: vec![],
        nodes: vec![],
    };
    let mut a = Arena::new(&mut graph);
    let root = a.node(Node::Instance {
        prototype: PROTOTYPE,
        properties: vec![],
    })?;
    a.g.root = Atom::Ref(root);
    let nodes = a.map()?;
    let classes = a.map()?;
    let tooltips = a.map()?;
    let sg_lookup = a.map()?;
    let edge_lookup = a.map()?;
    let pair_count = a.map()?;
    let mut edge_ids = Vec::new();
    let mut sg_ids = Vec::new();
    let mut style_arrays = HashMap::new();
    for (i, (key, n)) in m.nodes.iter().enumerate() {
        let fields = vec![
            ("id", a.string(n.id.as_str())?),
            ("text", a.opt(n.text.as_ref())?),
            ("labelType", a.string(label(&n.label_type))?),
            ("type", a.opt(n.shape.as_ref())?),
            ("styles", Atom::Ref(a.strings(&n.styles)?)),
            ("classes", Atom::Ref(a.strings(&n.classes)?)),
            ("dir", a.opt(n.dir.as_ref())?),
            (
                "props",
                n.props
                    .as_ref()
                    .map_or(Ok(Atom::Undefined), |o| a.value(&Value::Object(o.clone())))?,
            ),
            ("link", a.opt(n.link.as_ref())?),
            ("linkTarget", a.opt(n.link_target.as_ref())?),
            ("icon", a.opt(n.icon.as_ref())?),
            ("form", a.opt(n.form.as_ref())?),
            ("pos", a.opt(n.pos.as_ref())?),
            ("img", a.opt(n.img.as_ref())?),
            ("constraint", a.opt(n.constraint.as_ref())?),
            ("assetWidth", a.opt_value(n.asset_width.as_ref())?),
            ("assetHeight", a.opt_value(n.asset_height.as_ref())?),
        ];
        let order = m.public_origin.nodes.get(i).map_or(
            &[
                "id",
                "labelType",
                "styles",
                "classes",
                "text",
                "type",
                "props",
            ][..],
            Vec::as_slice,
        );
        let id = a.ordered(fields, order)?;
        let key = a.string(key.as_str())?;
        a.map_set(nodes, key, Atom::Ref(id))?;
    }
    for (i, e) in m.edges.iter().enumerate() {
        let mut fields = vec![
            ("start", a.string(e.start.as_str())?),
            ("end", a.string(e.end.as_str())?),
            ("type", a.opt(e.edge_type.as_ref())?),
            ("text", a.string(e.text.as_str())?),
            ("labelType", a.string(label(&e.label_type))?),
            ("classes", Atom::Ref(a.strings(&e.classes)?)),
            ("isUserDefinedId", Atom::Bool(e.is_user_defined_id)),
            ("interpolate", a.opt_value(e.interpolate.as_ref())?),
            ("stroke", a.opt(e.stroke.as_ref())?),
            ("length", a.opt_value(e.length.as_ref())?),
            ("id", a.opt(e.id.as_ref())?),
            ("animate", a.opt_value(e.animate.as_ref())?),
            ("animation", a.opt_value(e.animation.as_ref())?),
        ];
        if let Some(style) = &e.style {
            let group = m.public_origin.edge_styles.get(i).copied().flatten();
            let sid = if let Some(id) = group.and_then(|g| style_arrays.get(&g)).copied() {
                id
            } else {
                let id = a.strings(style)?;
                if let Some(g) = group {
                    style_arrays.insert(g, id);
                }
                id
            };
            fields.push(("style", Atom::Ref(sid)));
        }
        let order = m.public_origin.edges.get(i).map_or(
            &[
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
            ][..],
            Vec::as_slice,
        );
        let id = a.ordered(fields, order)?;
        edge_ids.push(Atom::Ref(id));
        if let Some(key) = &e.id {
            let key = a.string(key.as_str())?;
            if matches!(a.map_get(edge_lookup, &key)?, Atom::Undefined) {
                a.map_set(edge_lookup, key, Atom::Ref(id))?;
            }
        }
        let pair = a.string(format!("{}\0{}", e.start, e.end).as_str())?;
        let count = match a.map_get(pair_count, &pair)? {
            Atom::Number(n) => n,
            _ => 0.,
        };
        a.map_set(pair_count, pair, Atom::Number(count + 1.))?;
    }
    for s in &m.subgraphs {
        let fields = vec![
            ("id", a.string(s.id.as_str())?),
            ("nodes", Atom::Ref(a.strings(&s.nodes)?)),
            ("title", a.string(s.title.as_str())?),
            ("classes", Atom::Ref(a.strings(&s.classes)?)),
            ("dir", a.opt(s.dir.as_ref())?),
            ("labelType", a.string(label(&s.label_type))?),
            (
                "metadata",
                s.metadata
                    .as_ref()
                    .map_or(Ok(Atom::Undefined), |o| a.value(&Value::Object(o.clone())))?,
            ),
        ];
        let id = a.ordered(
            fields,
            &["id", "nodes", "title", "classes", "dir", "labelType"],
        )?;
        sg_ids.push(Atom::Ref(id));
        let key = a.string(s.id.as_str())?;
        a.map_set(sg_lookup, key, Atom::Ref(id))?;
    }
    for (key, c) in &m.classes {
        let s = a.strings(&c.styles)?;
        let t = a.strings(&c.text_styles)?;
        let cid = a.string(c.id.as_str())?;
        let id = a.record(vec![
            ("id", cid),
            ("styles", Atom::Ref(s)),
            ("textStyles", Atom::Ref(t)),
        ])?;
        let key = a.string(key.as_str())?;
        a.map_set(classes, key, Atom::Ref(id))?;
    }
    for (k, v) in &m.tooltips {
        let k = a.string(k.as_str())?;
        let v = a.string(v.as_str())?;
        a.map_set(tooltips, k, v)?;
    }
    let members = m
        .public_origin
        .members
        .iter()
        .map(|s| a.string(s.as_str()))
        .collect::<Result<Vec<_>, _>>()?;
    let members = a.node(Node::Set {
        values: members,
        properties: vec![],
    })?;
    let edges = a.array(edge_ids)?;
    let subgraphs = a.array(sg_ids)?;
    let default_style = if let Some(style) = &m.default_edge_style {
        let id = if let Some(id) = m
            .public_origin
            .default_style
            .and_then(|g| style_arrays.get(&g))
            .copied()
        {
            id
        } else {
            a.strings(style)?
        };
        Atom::Ref(id)
    } else {
        Atom::Undefined
    };
    let options = if let Some(argument) = constructor {
        match argument {
            ConstructorArgument::Undefined => Atom::Ref(a.object()?),
            value => value.into_atom(|text| a.string(text))?,
        }
    } else {
        let options = a.object()?;
        a.assign(
            options,
            "inheritDir",
            Atom::Bool(m.public_origin.inherit_dir),
        )?;
        let max_edges = match limit {
            Some(limit) => a.value(&limit.value())?,
            None => Atom::Number(m.public_origin.max_edges as f64),
        };
        a.assign(options, "maxEdges", max_edges)?;
        Atom::Ref(options)
    };
    let fields = vec![
        ("type", a.string("flowchart")?),
        ("direction", a.opt(m.direction.as_ref())?),
        ("nodes", Atom::Ref(nodes)),
        ("edges", Atom::Ref(edges)),
        ("subgraphs", Atom::Ref(subgraphs)),
        ("classes", Atom::Ref(classes)),
        ("tooltips", Atom::Ref(tooltips)),
        ("defaultEdgeStyle", default_style),
        (
            "defaultInterpolate",
            a.opt_value(m.default_interpolate.as_ref())?,
        ),
        ("title", a.opt(m.title.as_ref())?),
        ("accTitle", a.opt(m.acc_title.as_ref())?),
        ("accDescr", a.opt(m.acc_descr.as_ref())?),
        ("subgraphLookup", Atom::Ref(sg_lookup)),
        ("members", Atom::Ref(members)),
        ("edgeById", Atom::Ref(edge_lookup)),
        ("pairCount", Atom::Ref(pair_count)),
        ("subCount", Atom::Number(m.public_origin.sub_count as f64)),
        ("options", options),
    ];
    for (k, v) in fields {
        a.assign(root, k, v)?
    }
    Ok(graph)
}
fn truthy(g: &ModelGraph, a: &Atom) -> bool {
    match a {
        Atom::Undefined | Atom::Null => false,
        Atom::Bool(v) => *v,
        Atom::Number(v) => *v != 0. && !v.is_nan(),
        Atom::String(i) => !g.strings[*i as usize].0.is_empty(),
        _ => true,
    }
}
fn number(g: &ModelGraph, a: &Atom) -> Result<f64, PublicError> {
    Ok(match a {
        Atom::Undefined => f64::NAN,
        Atom::Null => 0.,
        Atom::Bool(v) => u8::from(*v) as f64,
        Atom::Number(v) => *v,
        Atom::String(_) => {
            let s = text(g, a)?;
            let s = s.trim_matches(space);
            if s.is_empty() {
                0.
            } else if matches!(s, "Infinity" | "+Infinity") {
                f64::INFINITY
            } else if s == "-Infinity" {
                f64::NEG_INFINITY
            } else if let Some((radix, rest)) = [
                ("0x", 16),
                ("0X", 16),
                ("0o", 8),
                ("0O", 8),
                ("0b", 2),
                ("0B", 2),
            ]
            .into_iter()
            .find_map(|(p, r)| s.strip_prefix(p).map(|s| (r, s)))
            {
                u64::from_str_radix(rest, radix).map_or(f64::NAN, |v| v as f64)
            } else {
                s.parse().unwrap_or(f64::NAN)
            }
        }
        _ => return Err(error("Flowchart object numeric coercion is not implemented").into()),
    })
}
fn js_string(g: &ModelGraph, a: &Atom) -> Result<String, PublicError> {
    Ok(match a {
        Atom::Undefined => "undefined".into(),
        Atom::Null => "null".into(),
        Atom::Bool(v) => v.to_string(),
        Atom::Number(n) => Value::Number(*n).js_string(),
        Atom::String(_) => text(g, a)?,
        _ => return Err(error("Flowchart object string coercion is not implemented").into()),
    })
}
fn raw_trim(g: &ModelGraph, a: &Atom, left_only: bool) -> Result<JsString, PublicError> {
    let Atom::String(i) = a else {
        return Err(ty("Flowchart argument must be a string"));
    };
    let units = &g.strings[*i as usize].0;
    let blank = |u: &u16| char::from_u32(*u as u32).is_some_and(space);
    let start = units.iter().take_while(|u| blank(u)).count();
    let end = if left_only {
        units.len()
    } else {
        units.len() - units[start..].iter().rev().take_while(|u| blank(u)).count()
    };
    Ok(JsString(units[start..end].to_vec()))
}
fn unquote_units(mut text: JsString) -> JsString {
    if text.0.first() == Some(&34) && text.0.last() == Some(&34) && text.0.len() >= 2 {
        text.0.remove(0);
        text.0.pop();
    }
    text
}
fn arg(call: &MethodCall, n: usize) -> Atom {
    call.arguments.get(n).cloned().unwrap_or(Atom::Undefined)
}
struct LinkPending {
    root: u32,
    edge: u32,
    start: Atom,
    end: Atom,
    id: Atom,
}
struct VertexPending {
    root: u32,
    id: Atom,
    text_obj: Atom,
    shape: Atom,
    style: Atom,
    classes: Atom,
    direction: Atom,
    props: Atom,
    doc: Option<Object>,
    existing: Atom,
}
impl VertexPending {
    fn needs_warning(&self) -> bool {
        matches!(self.existing, Atom::Undefined)
            && matches!(self.text_obj, Atom::Undefined)
            && matches!(self.shape, Atom::Undefined)
            && !matches!(self.style, Atom::Undefined | Atom::Null)
    }
}
struct LinkBatch {
    root: u32,
    starts: Vec<Atom>,
    ends: Vec<Atom>,
    info: Atom,
    id: Atom,
    start: usize,
    end: usize,
}
impl LinkBatch {
    fn next(&mut self, g: &ModelGraph) -> Option<(Atom, Atom, Atom)> {
        if self.start >= self.starts.len() || self.ends.is_empty() {
            return None;
        }
        let start = self.starts[self.start].clone();
        let end = self.ends[self.end].clone();
        let id = if self.starts.last().is_some_and(|s| same(g, s, &start))
            && self.ends.first().is_some_and(|e| same(g, e, &end))
        {
            self.id.clone()
        } else {
            Atom::Undefined
        };
        self.end += 1;
        if self.end == self.ends.len() {
            self.end = 0;
            self.start += 1;
        }
        Some((start, end, id))
    }
}
impl Arena<'_> {
    fn link_batch(&mut self, root: u32, call: &MethodCall) -> Result<LinkBatch, PublicError> {
        self.guard(root, "addSingleLink", 2)?;
        let starts = dense(self.g, &arg(call, 0))?;
        let ends = dense(self.g, &arg(call, 1))?;
        if starts
            .len()
            .checked_mul(ends.len())
            .is_none_or(|n| n > MAX_WORK)
        {
            return Err(limit().into());
        }
        let info = arg(call, 2);
        let id = if let Atom::Ref(i) = info {
            let id = self.field(i, "id");
            if let Atom::String(s) = id {
                let s = self.g.strings[s as usize].0.clone();
                let mut out = Vec::with_capacity(s.len());
                let mut removed = false;
                for c in s {
                    if c == 64 && !removed {
                        removed = true
                    } else {
                        out.push(c)
                    }
                }
                self.string(JsString(out))?
            } else {
                Atom::Undefined
            }
        } else {
            Atom::Undefined
        };
        Ok(LinkBatch {
            root,
            starts,
            ends,
            info,
            id,
            start: 0,
            end: 0,
        })
    }
    fn guard(&self, id: u32, name: &str, builtin: u32) -> Result<(), PublicError> {
        if let Some(p) = self.g.nodes[id as usize]
            .properties()
            .iter()
            .find(|p| self.g.strings[p.key as usize].is(name))
        {
            if matches!(p.value,Atom::BuiltinFunction(v)if v==builtin) {
                return Ok(());
            }
            return Err(
                if matches!(p.value, Atom::BuiltinFunction(_) | Atom::HostReference(_)) {
                    error("Flowchart overridden method dispatch is not implemented").into()
                } else {
                    ty("Flowchart method is not a function")
                },
            );
        }
        Ok(())
    }
    fn map_field(&self, root: u32, name: &str, key: &Atom) -> Result<Atom, PublicError> {
        self.map_get(self.field_id(root, name)?, key)
    }
    fn set_class(&mut self, root: u32, ids: &str, class: Atom) -> Result<(), PublicError> {
        for id in ids.split(',') {
            let key = self.string(id)?;
            for name in ["nodes", "edgeById", "subgraphLookup"] {
                let node = self.map_field(root, name, &key)?;
                if !matches!(node, Atom::Undefined | Atom::Null) {
                    let node = self.id(&node)?;
                    let classes = self.field_id(node, "classes")?;
                    self.push(classes, class.clone())?
                }
            }
        }
        Ok(())
    }
    fn edge_at(&self, root: u32, pos: &Atom) -> Result<Atom, PublicError> {
        let id = self.field_id(root, "edges")?;
        let n = number(self.g, pos)?;
        let value = self.field(id, &Value::Number(n).js_string());
        if matches!(value, Atom::Undefined) {
            let length = match self.g.nodes[id as usize] {
                Node::Array { length, .. } => length as f64,
                _ => number(self.g, &self.field(id, "length"))?,
            };
            return Err(error(format!("The index {} for linkStyle is out of bounds. Valid indices for linkStyle are between 0 and {}. (Help: Ensure that the index is within the range of existing edges.)",js_string(self.g,pos)?,Value::Number(length-1.).js_string())).into());
        }
        Ok(value)
    }
    fn single_link_prepare(
        &mut self,
        root: u32,
        start: Atom,
        end: Atom,
        info: Atom,
        id: Atom,
    ) -> Result<LinkPending, PublicError> {
        let classes = self.array(vec![])?;
        let interpolate = self.field(root, "defaultInterpolate");
        let empty = self.string("")?;
        let text_kind = self.string("text")?;
        let edge = self.record(vec![
            ("start", start.clone()),
            ("end", end.clone()),
            ("type", Atom::Undefined),
            ("text", empty),
            ("labelType", text_kind),
            ("classes", Atom::Ref(classes)),
            ("isUserDefinedId", Atom::Bool(false)),
            ("interpolate", interpolate),
        ])?;
        if !matches!(info, Atom::Undefined | Atom::Null) {
            let data = self.id(&info)?;
            let label = self.field(data, "text");
            if !matches!(label, Atom::Undefined) {
                let label = self.id(&label)?;
                let t = unquote_units(raw_trim(self.g, &self.field(label, "text"), false)?);
                let t = self.string(t)?;
                self.assign(edge, "text", t)?;
                let kind = self.field(label, "type");
                let kind = if ["text", "string", "markdown"]
                    .iter()
                    .any(|s| matches!(&kind,Atom::String(i)if self.g.strings[*i as usize].is(s)))
                {
                    kind
                } else {
                    self.string("markdown")?
                };
                self.assign(edge, "labelType", kind)?;
            }
            for key in ["type", "stroke"] {
                self.assign(edge, key, self.field(data, key))?
            }
            let length = self.field(data, "length");
            let length = if number(self.g, &length)? > 10. {
                Atom::Number(10.)
            } else {
                length
            };
            self.assign(edge, "length", length)?;
        } else if matches!(info, Atom::Null) {
            return Err(ty("Cannot read Flowchart link properties of null"));
        }
        let lookup = self.field_id(root, "edgeById")?;
        let custom = truthy(self.g, &id) && !self.map_has(lookup, &id)?;
        let pair = self.string(
            format!(
                "{}\0{}",
                js_string(self.g, &start)?,
                js_string(self.g, &end)?
            )
            .as_str(),
        )?;
        let old = self.map_field(root, "pairCount", &pair)?;
        let count = if matches!(old, Atom::Null | Atom::Undefined) {
            0.
        } else {
            number(self.g, &old)?
        };
        let id = if custom {
            id
        } else {
            self.string(
                format!(
                    "L_{}_{}_{}",
                    js_string(self.g, &start)?,
                    js_string(self.g, &end)?,
                    Value::Number(if count == 0. { 0. } else { count + 1. }).js_string()
                )
                .as_str(),
            )?
        };
        self.assign(edge, "id", id.clone())?;
        self.assign(edge, "isUserDefinedId", Atom::Bool(custom))?;
        Ok(LinkPending {
            root,
            edge,
            start,
            end,
            id,
        })
    }
    fn single_link_finish(&mut self, pending: LinkPending, max: Atom) -> Result<(), PublicError> {
        let edges = self.field_id(pending.root, "edges")?;
        let Node::Array { length, .. } = self.g.nodes[edges as usize] else {
            return Err(ty("Flowchart edges must be an Array"));
        };
        if !matches!(max, Atom::Undefined) && length as f64 >= number(self.g, &max)? {
            let mut e = error(format!(
                "Edge limit exceeded. {length} edges found, but the limit is {}.",
                js_string(self.g, &max)?
            ));
            e.code = ErrorCode::Limit;
            return Err(e.into());
        }
        self.single_link_commit(pending)
    }
    fn single_link_commit(&mut self, pending: LinkPending) -> Result<(), PublicError> {
        let LinkPending {
            root,
            edge,
            start,
            end,
            id,
        } = pending;
        let edges = self.field_id(root, "edges")?;
        self.push(edges, Atom::Ref(edge))?;
        let pair = self.string(
            format!(
                "{}\0{}",
                js_string(self.g, &start)?,
                js_string(self.g, &end)?
            )
            .as_str(),
        )?;
        let counts = self.field_id(root, "pairCount")?;
        let old = self.map_get(counts, &pair)?;
        let count = if matches!(old, Atom::Null | Atom::Undefined) {
            0.
        } else {
            number(self.g, &old)?
        };
        self.map_set(counts, pair, Atom::Number(count + 1.))?;
        let lookup = self.field_id(root, "edgeById")?;
        if !self.map_has(lookup, &id)? {
            self.map_set(lookup, id, Atom::Ref(edge))?
        }
        Ok(())
    }
    fn single_link(
        &mut self,
        root: u32,
        start: Atom,
        end: Atom,
        info: Atom,
        id: Atom,
    ) -> Result<(), PublicError> {
        let pending = self.single_link_prepare(root, start, end, info, id)?;
        let options = self.field_id(root, "options")?;
        let max = self.field(options, "maxEdges");
        self.single_link_finish(pending, max)
    }
    fn vertex_prepare(
        &mut self,
        root: u32,
        args: &[Atom],
    ) -> Result<Option<VertexPending>, PublicError> {
        let argument = |n| args.get(n).cloned().unwrap_or(Atom::Undefined);
        let id = argument(0);
        if !truthy(self.g, &id) {
            return Ok(None);
        }
        let name = text(self.g, &id)?;
        if name.trim_matches(space).is_empty() {
            return Ok(None);
        }
        let text_obj = argument(1);
        let shape = argument(2);
        let style = argument(3);
        let classes = argument(4);
        let direction = argument(5);
        let props = if matches!(argument(6), Atom::Undefined) {
            Atom::Ref(self.object()?)
        } else {
            argument(6)
        };
        let meta = argument(7);
        let doc = if matches!(meta, Atom::Undefined) {
            None
        } else {
            let meta = text(self.g, &meta)?;
            let yaml = if meta.contains('\n') {
                format!("{meta}\n")
            } else {
                format!("{{\n{meta}\n}}")
            };
            match yaml::parse(&yaml).map_err(|e| {
                let mut err = error(format!("Metadata of \"{name}\" is not valid YAML. {e}"));
                err.code = ErrorCode::Syntax;
                err
            })? {
                Value::Object(o) => Some(o),
                _ => None,
            }
        };
        let sub = self.map_field(root, "subgraphLookup", &id)?;
        if truthy(self.g, &sub) && doc.is_some() {
            let sub = self.id(&sub)?;
            let mut target = self.field(sub, "metadata");
            if matches!(target, Atom::Undefined | Atom::Null) {
                target = Atom::Ref(self.object()?)
            }
            let target_id = self.id(&target)?;
            for (k, v) in doc.unwrap().entries() {
                let v = self.value(v)?;
                self.assign(target_id, k, v)?
            }
            self.assign(sub, "metadata", target)?;
            return Ok(None);
        }
        let edge = self.map_field(root, "edgeById", &id)?;
        if truthy(self.g, &edge) {
            if let Some(doc) = doc {
                let edge = self.id(&edge)?;
                for (from, to) in [
                    ("animate", "animate"),
                    ("animation", "animation"),
                    ("curve", "interpolate"),
                ] {
                    if let Some(v) = doc.get(from) {
                        let v = self.value(v)?;
                        self.assign(edge, to, v)?
                    }
                }
            }
            return Ok(None);
        }
        let nodes = self.field_id(root, "nodes")?;
        let existing = self.map_get(nodes, &id)?;
        Ok(Some(VertexPending {
            root,
            id,
            text_obj,
            shape,
            style,
            classes,
            direction,
            props,
            doc,
            existing,
        }))
    }
    fn vertex_finish(&mut self, pending: VertexPending) -> Result<(), PublicError> {
        let VertexPending {
            root,
            id,
            text_obj,
            shape,
            style,
            classes,
            direction,
            props,
            doc,
            existing,
        } = pending;
        let vertex = if matches!(existing, Atom::Undefined) {
            let styles = self.array(vec![])?;
            let classes = self.array(vec![])?;
            let kind = self.string("text")?;
            let vertex = self.record(vec![
                ("id", id.clone()),
                ("labelType", kind),
                ("styles", Atom::Ref(styles)),
                ("classes", Atom::Ref(classes)),
            ])?;
            let nodes = self.field_id(root, "nodes")?;
            self.map_set(nodes, id.clone(), Atom::Ref(vertex))?;
            vertex
        } else {
            self.id(&existing)?
        };
        if !matches!(text_obj, Atom::Undefined) {
            let tid = self.id(&text_obj)?;
            self.assign(vertex, "labelType", self.field(tid, "type"))?;
            let t = unquote_units(raw_trim(self.g, &self.field(tid, "text"), false)?);
            let t = self.string(t)?;
            self.assign(vertex, "text", t)?;
        } else if matches!(self.field(vertex, "text"), Atom::Undefined) {
            self.assign(vertex, "text", id.clone())?
        }
        if !matches!(shape, Atom::Undefined) {
            self.assign(vertex, "type", shape)?
        }
        for (key, value) in [("styles", style), ("classes", classes)] {
            if !matches!(value, Atom::Undefined | Atom::Null) {
                let target = self.field_id(vertex, key)?;
                for value in dense(self.g, &value)? {
                    self.push(target, value)?
                }
            }
        }
        if !matches!(direction, Atom::Undefined) {
            self.assign(vertex, "dir", direction)?
        }
        let old_props = self.field(vertex, "props");
        if matches!(old_props, Atom::Undefined) {
            self.assign(vertex, "props", props)?;
        } else if !matches!(props, Atom::Undefined | Atom::Null) {
            let target = self.id(&old_props)?;
            let source = self.id(&props)?;
            let fields = self.g.nodes[source as usize].properties().to_vec();
            for p in fields {
                if p.flags & 1 != 0 {
                    let key = text(self.g, &Atom::String(p.key))?;
                    self.assign(target, &key, p.value)?
                }
            }
        }
        let Some(doc) = doc else { return Ok(()) };
        if let Some(v) = doc.get("shape").filter(|v| v.truthy()) {
            let shape = v.js_string();
            if shape != shape.to_lowercase() || shape.contains('_') {
                return Err(error(format!(
                    "No such shape: {shape}. Shape names should be lowercase."
                ))
                .into());
            }
            if !super::shapes::valid(&shape) {
                return Err(error(format!("No such shape: {shape}.")).into());
            }
            let shape = self.string(shape.as_str())?;
            self.assign(vertex, "type", shape)?;
        }
        let supplied_label = doc.get("label").and_then(|v| {
            if let Value::String(s) = v {
                Some(s)
            } else {
                None
            }
        });
        if let Some(v) = doc.get("label").filter(|v| v.truthy()) {
            let t = self.string(v.js_string().as_str())?;
            self.assign(vertex, "text", t)?;
            let kind = match doc.get("labelType") {
                Some(Value::String(s)) if matches!(s.as_str(), "text" | "string" | "markdown") => {
                    s.as_str()
                }
                _ => "markdown",
            };
            let kind = self.string(kind)?;
            self.assign(vertex, "labelType", kind)?;
        }
        for key in ["icon", "form", "pos", "img", "constraint"] {
            if let Some(v) = doc.get(key).filter(|v| v.truthy()) {
                let value = self.string(v.js_string().as_str())?;
                self.assign(vertex, key, value)?;
                if matches!(key, "icon" | "img")
                    && supplied_label.is_none_or(|s| s.trim_matches(space).is_empty())
                    && same(self.g, &self.field(vertex, "text"), &id)
                {
                    let empty = self.string("")?;
                    self.assign(vertex, "text", empty)?;
                }
            }
        }
        for (from, to) in [("w", "assetWidth"), ("h", "assetHeight")] {
            if let Some(v) = doc.get(from).filter(|v| v.truthy()) {
                let value = self.value(v)?;
                let n = number(self.g, &value)?;
                self.assign(vertex, to, Atom::Number(n))?;
            }
        }
        Ok(())
    }
    fn vertex(&mut self, root: u32, args: &[Atom]) -> Result<(), PublicError> {
        let Some(pending) = self.vertex_prepare(root, args)? else {
            return Ok(());
        };
        if pending.needs_warning() {
            let options = self.field_id(root, "options")?;
            let warn = self.field(options, "warn");
            if !matches!(warn, Atom::Undefined | Atom::Null) {
                return Err(
                    if matches!(warn, Atom::HostReference(_) | Atom::BuiltinFunction(_)) {
                        error("Flowchart warning callback dispatch is not implemented").into()
                    } else {
                        ty("Flowchart warning callback is not a function")
                    },
                );
            }
        }
        self.vertex_finish(pending)
    }
    fn subgraph_prepare(
        &mut self,
        root: u32,
        id_text: Atom,
        list: Atom,
        title_text: Atom,
    ) -> Result<SubgraphPending, PublicError> {
        let mut id = if matches!(id_text, Atom::Undefined | Atom::Null) {
            None
        } else {
            let i = self.id(&id_text)?;
            Some(raw_trim(self.g, &self.field(i, "text"), false)?)
        };
        let title = if matches!(title_text, Atom::Undefined | Atom::Null) {
            self.string("")?
        } else {
            let t = self.id(&title_text)?;
            self.field(t, "text")
        };
        if same(self.g, &id_text, &title_text) && !matches!(id_text, Atom::Undefined | Atom::Null) {
            if let Atom::String(t) = title {
                if self.g.strings[t as usize]
                    .0
                    .iter()
                    .any(|u| char::from_u32(*u as u32).is_some_and(space))
                {
                    id = None
                }
            }
        }
        let mut nodes = Vec::new();
        let mut seen = HashSet::new();
        let mut dir = Atom::Undefined;
        for entry in dense(self.g, &list)? {
            let items = match entry {
                Atom::String(_) => vec![entry],
                Atom::Ref(i) => {
                    if matches!(self.g.nodes[i as usize], Node::Array { .. }) {
                        dense(self.g, &entry)?
                    } else {
                        dir = self.field(i, "value");
                        vec![]
                    }
                }
                _ => {
                    return Err(ty(
                        "Flowchart subgraph entry must be a string, array, or direction",
                    ))
                }
            };
            for item in items {
                if !raw_trim(self.g, &item, false)?.0.is_empty() && seen.insert(self.key(&item)) {
                    nodes.push(item)
                }
            }
        }
        Ok(SubgraphPending {
            root,
            id,
            nodes,
            dir,
            title,
            title_text,
        })
    }
    fn add_subgraph(
        &mut self,
        root: u32,
        id_text: Atom,
        list: Atom,
        title_text: Atom,
    ) -> Result<Atom, PublicError> {
        let mut pending = self.subgraph_prepare(root, id_text, list, title_text)?;
        if matches!(pending.dir, Atom::Undefined | Atom::Null) {
            let options = self.field_id(root, "options")?;
            if truthy(self.g, &self.field(options, "inheritDir")) {
                let direction = self.field(root, "direction");
                pending.dir = if matches!(direction, Atom::Undefined | Atom::Null) {
                    self.field(options, "direction")
                } else {
                    direction
                };
            } else {
                pending.dir = Atom::Undefined;
            }
        }
        self.subgraph_finish(pending)
    }
    fn subgraph_finish(&mut self, pending: SubgraphPending) -> Result<Atom, PublicError> {
        let SubgraphPending {
            root,
            id,
            mut nodes,
            dir,
            title,
            title_text,
        } = pending;
        let count = number(self.g, &self.field(root, "subCount"))?;
        let id = match id {
            Some(id) => self.string(id)?,
            None => {
                self.string(format!("subGraph{}", Value::Number(count).js_string()).as_str())?
            }
        };
        self.assign(root, "subCount", Atom::Number(count + 1.))?;
        let members = self.field_id(root, "members")?;
        let Node::Set { values, .. } = &self.g.nodes[members as usize] else {
            return Err(ty("Flowchart members must be a Set"));
        };
        let existing = values.iter().map(|v| self.key(v)).collect::<HashSet<_>>();
        nodes.retain(|n| !same(self.g, n, &id) && !existing.contains(&self.key(n)));
        if nodes.len() + values.len() > MAX_ITEMS {
            return Err(limit().into());
        }
        let Node::Set { values, .. } = &mut self.g.nodes[members as usize] else {
            unreachable!()
        };
        values.extend(nodes.iter().cloned());
        let node_list = self.array(nodes.clone())?;
        let classes = self.array(vec![])?;
        let title = if truthy(self.g, &title) {
            self.string(raw_trim(self.g, &title, false)?)?
        } else {
            self.string("")?
        };
        let kind = if let Atom::Ref(t) = title_text {
            self.field(t, "type")
        } else {
            Atom::Undefined
        };
        let kind = if ["text", "string", "markdown"]
            .iter()
            .any(|s| matches!(&kind,Atom::String(i)if self.g.strings[*i as usize].is(s)))
        {
            kind
        } else {
            self.string("markdown")?
        };
        let sub = self.record(vec![
            ("id", id.clone()),
            ("nodes", Atom::Ref(node_list)),
            ("title", title),
            ("classes", Atom::Ref(classes)),
            ("dir", dir),
            ("labelType", kind),
        ])?;
        let lookup = self.field_id(root, "subgraphLookup")?;
        let old = self.map_get(lookup, &id)?;
        if truthy(self.g, &old) {
            let old = self.id(&old)?;
            let target = self.field_id(old, "nodes")?;
            for n in nodes {
                self.push(target, n)?
            }
        } else {
            let subgraphs = self.field_id(root, "subgraphs")?;
            self.push(subgraphs, Atom::Ref(sub))?
        }
        self.map_set(
            lookup,
            id.clone(),
            if truthy(self.g, &old) {
                old
            } else {
                Atom::Ref(sub)
            },
        )?;
        Ok(id)
    }
}
/// Actual public destructLink helper counts/slices UTF-16, including lone units.
fn destruct(end: &JsString, start: Option<&JsString>) -> (String, &'static str, Option<f64>) {
    let units = &end.0;
    let mut line = &units[..units.len().saturating_sub(1)];
    let mut kind = "arrow_open".to_string();
    if let Some((first, name)) = match units.last() {
        Some(120) => Some((120, "arrow_cross")),
        Some(62) => Some((60, "arrow_point")),
        Some(111) => Some((111, "arrow_circle")),
        _ => None,
    } {
        kind = name.into();
        if units.first() == Some(&first) {
            kind = format!("double_{name}");
            line = line.get(1..).unwrap_or(&[]);
        }
    }
    let mut stroke = match line.first() {
        Some(61) => "thick",
        Some(126) => "invisible",
        _ => "normal",
    };
    let dots = line.iter().filter(|c| **c == 46).count();
    let length = if dots > 0 {
        stroke = "dotted";
        dots as f64
    } else {
        line.len() as f64 - 1.
    };
    if let Some(start) = start {
        let mut units = &start.0[..];
        let start_kind = match units.first() {
            Some(60) => {
                units = &units[1..];
                "arrow_point"
            }
            Some(120) => {
                units = &units[1..];
                "arrow_cross"
            }
            Some(111) => {
                units = &units[1..];
                "arrow_circle"
            }
            _ => "arrow_open",
        };
        let start_stroke = if units.contains(&46) {
            "dotted"
        } else if units.contains(&61) {
            "thick"
        } else {
            "normal"
        };
        if start_stroke != stroke || (start_kind != "arrow_open" && start_kind != kind) {
            return ("INVALID".into(), "INVALID", None);
        }
        if start_kind != "arrow_open" {
            kind = format!("double_{start_kind}");
        }
    }
    (kind, stroke, Some(length))
}
fn invoke(g: &mut ModelGraph, call: MethodCall) -> Result<Atom, PublicError> {
    let mut a = Arena::new(g);
    let root = a.id(&call.receiver)?;
    match call.method.0 {
        1 => a.vertex(root, &call.arguments)?,
        2 => a.single_link(
            root,
            arg(&call, 0),
            arg(&call, 1),
            arg(&call, 2),
            arg(&call, 3),
        )?,
        3 => {
            let mut batch = a.link_batch(root, &call)?;
            while let Some((start, end, id)) = batch.next(a.g) {
                a.single_link(root, start, end, batch.info.clone(), id)?;
            }
        }
        4 | 5 => {
            let value = arg(&call, 1);
            for pos in dense(a.g, &arg(&call, 0))? {
                if matches!(pos,Atom::String(i)if a.g.strings[i as usize].is("default")) {
                    a.assign(
                        root,
                        if call.method.0 == 4 {
                            "defaultInterpolate"
                        } else {
                            "defaultEdgeStyle"
                        },
                        value.clone(),
                    )?;
                    continue;
                }
                a.guard(root, "edgeAt", 6)?;
                let edge = a.edge_at(root, &pos)?;
                let edge = a.id(&edge)?;
                a.assign(
                    edge,
                    if call.method.0 == 4 {
                        "interpolate"
                    } else {
                        "style"
                    },
                    value.clone(),
                )?;
                if call.method.0 == 5 {
                    let styles = dense(a.g, &value)?;
                    if !styles.is_empty() {
                        let mut fill = false;
                        for s in styles {
                            if matches!(s, Atom::Undefined | Atom::Null) {
                                continue;
                            }
                            if text(a.g, &s)?.starts_with("fill") {
                                fill = true;
                                break;
                            }
                        }
                        if !fill {
                            let styles = a.id(&value)?;
                            let v = a.string("fill:none")?;
                            a.push(styles, v)?
                        }
                    }
                }
            }
        }
        6 => return a.edge_at(root, &arg(&call, 0)),
        7 => {
            let ids = text(a.g, &arg(&call, 0))?;
            let styles = dense(a.g, &arg(&call, 1))?;
            let mut units = styles.len().saturating_sub(1);
            for v in &styles {
                units = units
                    .checked_add(match v {
                        Atom::String(i) => a.g.strings[*i as usize].0.len(),
                        Atom::Null | Atom::Undefined => 0,
                        _ => js_string(a.g, v)?.encode_utf16().count(),
                    })
                    .ok_or_else(limit)?;
                if units > MAX_TEXT {
                    return Err(limit().into());
                }
            }
            let joined = styles
                .iter()
                .map(|v| {
                    if matches!(v, Atom::Null | Atom::Undefined) {
                        Ok(String::new())
                    } else {
                        js_string(a.g, v)
                    }
                })
                .collect::<Result<Vec<_>, PublicError>>()?
                .join(",")
                .replace("\\,", "§§§")
                .replace(',', ";")
                .replace("§§§", ",");
            let classes = a.field_id(root, "classes")?;
            for id in ids.split(',') {
                let key = a.string(id)?;
                let old = a.map_get(classes, &key)?;
                let node = if matches!(old, Atom::Undefined) {
                    let styles = a.array(vec![])?;
                    let text = a.array(vec![])?;
                    let node = a.record(vec![
                        ("id", key.clone()),
                        ("styles", Atom::Ref(styles)),
                        ("textStyles", Atom::Ref(text)),
                    ])?;
                    a.map_set(classes, key, Atom::Ref(node))?;
                    node
                } else {
                    a.id(&old)?
                };
                for style in joined.split(';') {
                    if style.contains("color") {
                        let s = a.string(style.replacen("fill", "bgFill", 1).as_str())?;
                        let target = a.field_id(node, "textStyles")?;
                        a.push(target, s)?;
                    }
                    let s = a.string(style)?;
                    let target = a.field_id(node, "styles")?;
                    a.push(target, s)?;
                }
            }
        }
        8 => {
            let mut d = raw_trim(a.g, &arg(&call, 0), false)?;
            for (c, to) in [(60, "RL"), (94, "BT"), (62, "LR"), (118, "TB")] {
                if d.0.contains(&c) {
                    d = to.into()
                }
            }
            if d.is("TD") {
                d = "TB".into()
            }
            let d = a.string(d)?;
            a.assign(root, "direction", d)?;
        }
        9 => {
            let ids = text(a.g, &arg(&call, 0))?;
            a.set_class(root, &ids, arg(&call, 1))?;
        }
        10 => {
            let tooltip = arg(&call, 1);
            if !matches!(tooltip, Atom::Undefined) {
                let ids = text(a.g, &arg(&call, 0))?;
                let map = a.field_id(root, "tooltips")?;
                for id in ids.split(',') {
                    let id = a.string(id)?;
                    a.map_set(map, id, tooltip.clone())?;
                }
            }
        }
        11 => {
            let ids = text(a.g, &arg(&call, 0))?;
            let url = raw_trim(a.g, &arg(&call, 1), false)?;
            for id in ids.split(',') {
                let key = a.string(id)?;
                let node = a.map_field(root, "nodes", &key)?;
                if !matches!(node, Atom::Undefined) {
                    let node = a.id(&node)?;
                    let link = if url.0.is_empty() {
                        Atom::Undefined
                    } else {
                        let url = url.well_formed().ok_or_else(|| {
                            error("Flowchart URL UTF-16 rendering is not implemented")
                        })?;
                        a.string(crate::compat::source::url::safe_url(&url)?.as_str())?
                    };
                    a.assign(node, "link", link)?;
                    a.assign(node, "linkTarget", arg(&call, 2))?;
                }
            }
            a.guard(root, "setClass", 9)?;
            let clickable = a.string("clickable")?;
            a.set_class(root, &ids, clickable)?;
        }
        12 => {
            let ids = text(a.g, &arg(&call, 0))?;
            a.guard(root, "setClass", 9)?;
            let clickable = a.string("clickable")?;
            a.set_class(root, &ids, clickable)?;
        }
        13 | 14 => {
            let value = arg(&call, 0);
            let value = if call.method.0 == 13 {
                raw_trim(a.g, &value, true)?
            } else {
                let Atom::String(i) = value else {
                    return Err(ty("Flowchart accessibility argument must be a string"));
                };
                let mut out = Vec::new();
                let mut after = false;
                for &c in &a.g.strings[i as usize].0 {
                    if after && char::from_u32(c as u32).is_some_and(space) {
                        continue;
                    }
                    out.push(c);
                    after = c == 10;
                }
                JsString(out)
            };
            let value = a.string(value)?;
            a.assign(
                root,
                if call.method.0 == 13 {
                    "accTitle"
                } else {
                    "accDescr"
                },
                value,
            )?;
        }
        15 => return a.add_subgraph(root, arg(&call, 0), arg(&call, 1), arg(&call, 2)),
        16 => {
            let id = a.key(&arg(&call, 1));
            let all = dense(a.g, &arg(&call, 0))?;
            let mut work = all.len();
            for sg in all {
                let sg = a.id(&sg)?;
                let nodes = a.field(sg, "nodes");
                let nid = a.id(&nodes)?;
                let Node::Array { length, .. } = a.g.nodes[nid as usize] else {
                    return Err(ty("Flowchart nodes must be an Array"));
                };
                work = work.checked_add(length as usize).ok_or_else(limit)?;
                if work > MAX_WORK {
                    return Err(limit().into());
                }
                if dense(a.g, &nodes)?.iter().any(|v| a.key(v) == id) {
                    return Ok(Atom::Bool(true));
                }
            }
            return Ok(Atom::Bool(false));
        }
        17 => {
            let sg = a.id(&arg(&call, 0))?;
            let all = dense(a.g, &arg(&call, 1))?;
            let mut work = all.len();
            // Preflight occurrences, not distinct values: compact repeated aliases
            // must not expand into an unbounded membership scan.
            for other in &all {
                let other = a.id(other)?;
                let nodes = a.field(other, "nodes");
                let nid = a.id(&nodes)?;
                let Node::Array { length, .. } = a.g.nodes[nid as usize] else {
                    return Err(ty("Flowchart nodes must be an Array"));
                };
                work = work.checked_add(length as usize).ok_or_else(limit)?;
                if work > MAX_WORK {
                    return Err(limit().into());
                }
            }
            let mut taken = HashSet::new();
            for other in all {
                let other = a.id(&other)?;
                taken.extend(
                    dense(a.g, &a.field(other, "nodes"))?
                        .iter()
                        .map(|v| a.key(v)),
                );
            }
            let nodes = dense(a.g, &a.field(sg, "nodes"))?
                .into_iter()
                .filter(|v| !taken.contains(&a.key(v)))
                .collect();
            let nodes = a.array(nodes)?;
            return Ok(Atom::Ref(a.record(vec![("nodes", Atom::Ref(nodes))])?));
        }
        18 => {
            let end = raw_trim(a.g, &arg(&call, 0), false)?;
            let start = arg(&call, 1);
            let start = if truthy(a.g, &start) {
                Some(raw_trim(a.g, &start, false)?)
            } else {
                None
            };
            let (kind, stroke, length) = destruct(&end, start.as_ref());
            let kind = a.string(kind.as_str())?;
            let stroke = a.string(stroke)?;
            let mut fields = vec![("type", kind), ("stroke", stroke)];
            if let Some(length) = length {
                fields.push(("length", Atom::Number(length)));
            }
            return Ok(Atom::Ref(a.record(fields)?));
        }
        19 => return a.map_field(root, "subgraphLookup", &arg(&call, 0)),
        _ => return Err(error("Flowchart public method is unavailable").into()),
    }
    Ok(Atom::Undefined)
}
fn create_state(
    m: &FlowchartModel,
    limit: Option<crate::compat::EdgeLimit>,
) -> Result<FlowchartPublicState, PeleError> {
    let graph = new_graph(m, limit).map_err(|e| match e {
        PublicError::Pele(e) => e,
        _ => error("Flowchart public export failed"),
    })?;
    let imported = std::cell::OnceCell::new();
    Ok(FlowchartPublicState { graph, imported })
}
pub(crate) const LIVE_FIELDS: &[LiveConstructorField] = &[LiveConstructorField {
    key: "options",
    methods: &[MethodId(1), MethodId(2), MethodId(3), MethodId(15)],
}];
pub(crate) fn construct_with(
    argument: ConstructorArgument,
) -> Result<FlowchartPublicState, PeleError> {
    let graph = new_graph_with_options(&FlowchartModel::empty(), None, Some(argument)).map_err(
        |e| match e {
            PublicError::Pele(e) => e,
            _ => error("Flowchart public construction failed"),
        },
    )?;
    Ok(FlowchartPublicState {
        graph,
        imported: std::cell::OnceCell::new(),
    })
}
impl PublicDiagramModule for Flowchart {
    fn detached_public_render_capability(&self) -> Option<DetachedRenderCapability> {
        Some(DetachedRenderCapability::flowchart())
    }
    fn detached_public_render(&self, state: &Self::PublicState) -> Option<DetachedPublicRender> {
        Some(DetachedPublicRender::flowchart(state.detached_seed()))
    }

    fn public_constructor(
        &self,
        prototype: PrototypeId,
    ) -> Option<PublicConstructorFactory<Self::PublicState>> {
        (prototype == PROTOTYPE).then(|| {
            PublicConstructorFactory::first_value::<Self>(PROTOTYPE, construct_with, LIVE_FIELDS)
        })
    }
    fn detached_public_invocation(&self, call: MethodCall) -> Option<DetachedPublicInvocation> {
        detached_method::<Self>(call)
    }

    type PublicState = FlowchartPublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let imported = std::cell::OnceCell::new();
        Ok(FlowchartPublicState { graph, imported })
    }
    fn create_public(&self, m: &FlowchartModel) -> Result<Self::PublicState, PeleError> {
        create_state(m, None)
    }
    fn create_public_with_edge_limit(
        &self,
        m: &FlowchartModel,
        limit: crate::compat::EdgeLimit,
    ) -> Result<Self::PublicState, PeleError> {
        create_state(m, Some(limit))
    }
    fn create_public_prepared(
        &self,
        model: &FlowchartModel,
        limit: crate::compat::EdgeLimit,
    ) -> Result<Self::PublicState, PeleError> {
        crate::compat::public::create_prepared_edge_public(self, model, limit)
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, s: &Self::PublicState) -> Result<ModelGraph, PeleError> {
        Ok(s.graph.clone())
    }
    fn apply_public(&self, s: &mut Self::PublicState, g: ModelGraph) -> Result<(), PeleError> {
        s.imported.take();
        s.graph = g;
        Ok(())
    }
    fn invoke_public(
        &self,
        s: &mut Self::PublicState,
        call: MethodCall,
        _: &Object,
        _: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        let result = invoke(&mut s.graph, call);
        s.imported.take();
        result
    }
    fn render_public(
        &self,
        s: &mut Self::PublicState,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render(s.render_model()?, config, options, host)
    }
    fn render_public_operation(
        &self,
        s: &mut Self::PublicState,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError> {
        self.render(s.prepared_render_model()?, config, options, host)
            .map_err(Into::into)
    }
}

struct SubgraphPending {
    root: u32,
    id: Option<JsString>,
    nodes: Vec<Atom>,
    dir: Atom,
    title: Atom,
    title_text: Atom,
}
struct FlowInvocation {
    text_work: usize,
    stage: FlowStage,
    batch: Option<LinkBatch>,
}
enum FlowStage {
    Start(MethodCall),
    WarnOptions(VertexPending),
    WarnGet(VertexPending, Atom),
    WarnCall(VertexPending),
    MaxOptions(LinkPending),
    MaxGet(LinkPending),
    MaxCompare(LinkPending, u32, Atom),
    MaxError(u32),
    SubOptions(SubgraphPending),
    SubInherit(SubgraphPending),
    SubTruthy(SubgraphPending),
    SubDirectionOptions(SubgraphPending),
    SubDirection(SubgraphPending),
    Finished,
}
impl FlowInvocation {
    fn request_max(&mut self, pending: LinkPending) -> PublicInvocationPhase {
        let receiver = Atom::Ref(pending.root);
        self.stage = FlowStage::MaxOptions(pending);
        PublicInvocationPhase::Host(PublicHostOperation::Get {
            receiver,
            key: "options",
        })
    }
    fn next_link(&mut self, a: &mut Arena<'_>) -> Result<PublicInvocationPhase, PublicError> {
        let Some(batch) = self.batch.as_mut() else {
            return Ok(PublicInvocationPhase::Complete(Ok(Atom::Undefined)));
        };
        let Some((start, end, id)) = batch.next(a.g) else {
            return Ok(PublicInvocationPhase::Complete(Ok(Atom::Undefined)));
        };
        let pending = a.single_link_prepare(batch.root, start, end, batch.info.clone(), id)?;
        Ok(self.request_max(pending))
    }
    fn step(
        &mut self,
        a: &mut Arena<'_>,
        reply: Option<Atom>,
    ) -> Result<PublicInvocationPhase, PublicError> {
        let stage = std::mem::replace(&mut self.stage, FlowStage::Finished);
        match stage {
            FlowStage::Start(call) => {
                let root = a.id(&call.receiver)?;
                match call.method.0 {
                    1 => {
                        let Some(pending) = a.vertex_prepare(root, &call.arguments)? else {
                            return Ok(PublicInvocationPhase::Complete(Ok(Atom::Undefined)));
                        };
                        if pending.needs_warning() {
                            let receiver = Atom::Ref(root);
                            self.stage = FlowStage::WarnOptions(pending);
                            Ok(PublicInvocationPhase::Host(PublicHostOperation::Get {
                                receiver,
                                key: "options",
                            }))
                        } else {
                            a.vertex_finish(pending)?;
                            Ok(PublicInvocationPhase::Complete(Ok(Atom::Undefined)))
                        }
                    }
                    2 => {
                        let pending = a.single_link_prepare(
                            root,
                            arg(&call, 0),
                            arg(&call, 1),
                            arg(&call, 2),
                            arg(&call, 3),
                        )?;
                        Ok(self.request_max(pending))
                    }
                    3 => {
                        self.batch = Some(a.link_batch(root, &call)?);
                        self.next_link(a)
                    }
                    15 => {
                        let pending =
                            a.subgraph_prepare(root, arg(&call, 0), arg(&call, 1), arg(&call, 2))?;
                        if matches!(pending.dir, Atom::Undefined | Atom::Null) {
                            self.stage = FlowStage::SubOptions(pending);
                            Ok(PublicInvocationPhase::Host(PublicHostOperation::Get {
                                receiver: Atom::Ref(root),
                                key: "options",
                            }))
                        } else {
                            Ok(PublicInvocationPhase::Complete(a.subgraph_finish(pending)))
                        }
                    }
                    _ => Err(error("Flowchart detached method is unavailable").into()),
                }
            }
            FlowStage::WarnOptions(pending) => {
                let receiver = reply.expect("protocol requires reply");
                self.stage = FlowStage::WarnGet(pending, receiver.clone());
                Ok(PublicInvocationPhase::HostWithContext {
                    operation: PublicHostOperation::Get {
                        receiver,
                        key: "warn",
                    },
                    expression: PublicHostErrorExpression::ThisOptionsWarn,
                })
            }
            FlowStage::MaxOptions(pending) => {
                let receiver = reply.expect("protocol requires reply");
                self.stage = FlowStage::MaxGet(pending);
                Ok(PublicInvocationPhase::Host(PublicHostOperation::Get {
                    receiver,
                    key: "maxEdges",
                }))
            }
            FlowStage::WarnGet(pending, receiver) => {
                let function = reply.expect("protocol requires reply");
                if matches!(function, Atom::Undefined | Atom::Null) {
                    a.vertex_finish(pending)?;
                    return Ok(PublicInvocationPhase::Complete(Ok(Atom::Undefined)));
                }
                let name = text(a.g, &pending.id)?;
                let message=a.string(format!("Style applied to unknown node \"{name}\". This may indicate a typo. The node will be created automatically.").as_str())?;
                self.stage = FlowStage::WarnCall(pending);
                Ok(PublicInvocationPhase::HostWithContext {
                    operation: PublicHostOperation::Call {
                        function,
                        receiver,
                        arguments: vec![message],
                        not_callable_error: "this.options.warn is not a function",
                        result: PublicHostResult::Discard,
                    },
                    expression: PublicHostErrorExpression::OptionalWarnStyle,
                })
            }
            FlowStage::WarnCall(pending) => {
                a.vertex_finish(pending)?;
                Ok(PublicInvocationPhase::Complete(Ok(Atom::Undefined)))
            }
            FlowStage::MaxGet(pending) => {
                let max = reply.expect("protocol requires reply");
                if matches!(max, Atom::Undefined) {
                    a.single_link_commit(pending)?;
                    return self.next_link(a);
                }
                let edges = a.field_id(pending.root, "edges")?;
                let Node::Array { length, .. } = a.g.nodes[edges as usize] else {
                    return Err(ty("Flowchart edges must be an Array"));
                };
                self.stage = FlowStage::MaxCompare(pending, length, max.clone());
                Ok(PublicInvocationPhase::Host(
                    PublicHostOperation::GreaterEqual {
                        left: Atom::Number(length as f64),
                        right: max,
                    },
                ))
            }
            FlowStage::MaxCompare(pending, length, max) => {
                match reply.expect("protocol requires reply") {
                    Atom::Bool(false) => {
                        a.single_link_commit(pending)?;
                        self.next_link(a)
                    }
                    Atom::Bool(true) => {
                        self.stage = FlowStage::MaxError(length);
                        Ok(PublicInvocationPhase::Host(PublicHostOperation::ToString {
                            value: max,
                        }))
                    }
                    _ => Err(error("Invalid host comparison result").into()),
                }
            }
            FlowStage::MaxError(length) => {
                let value = reply.expect("protocol requires reply");
                let limit = text(a.g, &value)?;
                let mut error = error(format!(
                    "Edge limit exceeded. {length} edges found, but the limit is {limit}."
                ));
                error.code = ErrorCode::Limit;
                Err(error.into())
            }
            FlowStage::SubOptions(pending) => {
                self.stage = FlowStage::SubInherit(pending);
                Ok(PublicInvocationPhase::Host(PublicHostOperation::Get {
                    receiver: reply.expect("protocol requires reply"),
                    key: "inheritDir",
                }))
            }
            FlowStage::SubInherit(pending) => {
                self.stage = FlowStage::SubTruthy(pending);
                Ok(PublicInvocationPhase::Host(PublicHostOperation::Truthy {
                    value: reply.expect("protocol requires reply"),
                }))
            }
            FlowStage::SubTruthy(mut pending) => {
                match reply.expect("protocol requires reply") {
                    Atom::Bool(false) => pending.dir = Atom::Undefined,
                    Atom::Bool(true) => {
                        pending.dir = a.field(pending.root, "direction");
                        if matches!(pending.dir, Atom::Undefined | Atom::Null) {
                            let receiver = Atom::Ref(pending.root);
                            self.stage = FlowStage::SubDirectionOptions(pending);
                            return Ok(PublicInvocationPhase::Host(PublicHostOperation::Get {
                                receiver,
                                key: "options",
                            }));
                        }
                    }
                    _ => return Err(error("Invalid host truthiness result").into()),
                }
                Ok(PublicInvocationPhase::Complete(a.subgraph_finish(pending)))
            }
            FlowStage::SubDirectionOptions(pending) => {
                self.stage = FlowStage::SubDirection(pending);
                Ok(PublicInvocationPhase::Host(PublicHostOperation::Get {
                    receiver: reply.expect("protocol requires reply"),
                    key: "direction",
                }))
            }
            FlowStage::SubDirection(mut pending) => {
                pending.dir = reply.expect("protocol requires reply");
                Ok(PublicInvocationPhase::Complete(a.subgraph_finish(pending)))
            }
            FlowStage::Finished => Err(error("Flowchart invocation already finished").into()),
        }
    }
}
fn resume_flow(
    state: &mut FlowchartPublicState,
    phase: &mut FlowInvocation,
    reply: Option<PublicHostValue>,
) -> PublicInvocationPhase {
    state.imported.take();
    let mut arena = Arena::new(&mut state.graph);
    arena.text_work = phase.text_work;
    let reply = reply
        .map(|value| value.into_atom(|text| arena.string(text)))
        .transpose();
    let result = reply.and_then(|reply| phase.step(&mut arena, reply));
    phase.text_work = arena.text_work;
    match result {
        Ok(step) => step,
        Err(error) => PublicInvocationPhase::Complete(Err(error)),
    }
}
pub(crate) fn detached_method<D: 'static>(call: MethodCall) -> Option<DetachedPublicInvocation> {
    matches!(call.method.0, 1..=3 | 15).then(|| {
        DetachedPublicInvocation::new::<D, FlowchartPublicState, _>(
            FlowInvocation {
                text_work: 0,
                stage: FlowStage::Start(call),
                batch: None,
            },
            resume_flow,
        )
    })
}
