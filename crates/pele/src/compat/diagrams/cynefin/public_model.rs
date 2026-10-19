//! Native mutable returned Cynefin model, pinned Pele 458322a5 (MIT).
//! The returned object is plain; CynefinDb is a discarded internal builder.
use super::{Cynefin, CynefinDomain, CynefinItem, CynefinModel, CynefinTransition, DomainName};
use crate::compat::{
    public::*, DiagramModule, ErrorCode, Object, PeleError, RenderResources, RenderSettings,
    Rendered,
};
use std::collections::HashMap;
const ITEMS: usize = 1_000_000;
const TEXT: usize = 16_000_000;
const NAMES: [DomainName; 5] = [
    DomainName::Complex,
    DomainName::Complicated,
    DomainName::Clear,
    DomainName::Chaotic,
    DomainName::Confusion,
];
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
static SCHEMA: ModelSchema = ModelSchema {
    id: "pele/cynefin@1",
    version: 1,
    root: Some(RecordId(1)),
    records: &[RecordSchema {
        id: RecordId(1),
        name: "CynefinModel",
        kind: RecordKind::Object,
        prototype: None,
        members: &[],
        fields: &[
            f("type", FieldKind::String, false),
            f("title", FieldKind::String, true),
            f("accTitle", FieldKind::String, true),
            f("accDescr", FieldKind::String, true),
            f("domains", FieldKind::Map, false),
            f("transitions", FieldKind::Array, false),
        ],
    }],
    prototypes: &[],
    methods: &[],
};
pub struct CynefinPublicState {
    graph: ModelGraph,
    projected: Result<CynefinModel, PublicError>,
}
fn error(message: impl Into<String>) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("cynefin".into());
    e
}
fn limit() -> PeleError {
    let mut e = error("Cynefin public model exceeds native graph/work budget");
    e.code = ErrorCode::Limit;
    e
}
fn ty(message: &str) -> PublicError {
    PublicError::JavaScript {
        name: JavaScriptErrorKind::TypeError,
        message: message.into(),
    }
}
fn legacy(e: PublicError) -> PeleError {
    match e {
        PublicError::Pele(e) => e,
        PublicError::JavaScript { message, .. } => error(
            message
                .well_formed()
                .unwrap_or_else(|| "Cynefin native TypeError".into()),
        ),
        _ => error("Cynefin native host exception"),
    }
}
fn reference(a: &Atom) -> Option<u32> {
    if let Atom::Ref(i) = a {
        Some(*i)
    } else {
        None
    }
}
fn get<'a>(g: &'a ModelGraph, id: u32, key: &str) -> &'a Atom {
    g.nodes[id as usize]
        .properties()
        .iter()
        .find(|p| g.strings[p.key as usize].is(key))
        .map_or(&Atom::Undefined, |p| &p.value)
}
fn truth(g: &ModelGraph, a: &Atom) -> bool {
    match a {
        Atom::Undefined | Atom::Null => false,
        Atom::Bool(v) => *v,
        Atom::Number(v) => *v != 0. && !v.is_nan(),
        Atom::String(i) => !g.strings[*i as usize].0.is_empty(),
        _ => true,
    }
}
fn name(g: &ModelGraph, a: &Atom) -> Option<DomainName> {
    let Atom::String(i) = a else { return None };
    NAMES
        .iter()
        .copied()
        .find(|n| g.strings[*i as usize].is(n.as_str()))
}
struct Writer {
    out: GraphWriter,
    strings: HashMap<JsString, u32>,
    units: usize,
    items: usize,
    work: usize,
}
impl Writer {
    fn new() -> Self {
        Self {
            out: GraphWriter::default(),
            strings: HashMap::new(),
            units: 0,
            items: 0,
            work: 0,
        }
    }
    fn text(&mut self, value: impl Into<JsString>) -> Result<Atom, PeleError> {
        let value = value.into();
        if let Some(i) = self.strings.get(&value) {
            return Ok(Atom::String(*i));
        }
        self.units = self.units.saturating_add(value.0.len());
        if self.strings.len() >= ITEMS || self.units > TEXT {
            return Err(limit());
        }
        let id = self.out.string(value.clone());
        self.strings.insert(value, id);
        Ok(Atom::String(id))
    }
    fn node(&mut self, value: Node) -> Result<Atom, PeleError> {
        self.items += 1;
        self.work += value.properties().len()
            + match &value {
                Node::Map { entries, .. } => entries.len() * 2,
                _ => 0,
            };
        if self.items > ITEMS || self.work > 8_000_000 {
            return Err(limit());
        }
        Ok(Atom::Ref(self.out.node(value)))
    }
    fn record(&mut self, fields: Vec<(&str, Atom)>) -> Result<Atom, PeleError> {
        let mut props = Vec::with_capacity(fields.len());
        for (key, value) in fields {
            let Atom::String(key) = self.text(key)? else {
                unreachable!()
            };
            props.push(Property {
                key,
                value,
                flags: 7,
            })
        }
        self.node(Node::Object { properties: props })
    }
    fn array(&mut self, values: Vec<Atom>) -> Result<Atom, PeleError> {
        if values.len() > ITEMS {
            return Err(limit());
        }
        let length = values.len() as u32;
        let mut props = Vec::with_capacity(values.len());
        for (i, value) in values.into_iter().enumerate() {
            let Atom::String(key) = self.text(i.to_string().as_str())? else {
                unreachable!()
            };
            props.push(Property {
                key,
                value,
                flags: 7,
            })
        }
        self.node(Node::Array {
            length,
            properties: props,
        })
    }
    fn opt(&mut self, value: Option<&String>) -> Result<Atom, PeleError> {
        value.map_or(Ok(Atom::Undefined), |value| self.text(value.as_str()))
    }
}
fn new_graph(m: &CynefinModel) -> Result<ModelGraph, PeleError> {
    let mut w = Writer::new();
    let mut domains = vec![];
    for (key, v) in &m.domains {
        let mut items = Vec::with_capacity(v.items.len());
        for item in &v.items {
            let label = w.text(item.label.as_str())?;
            items.push(w.record(vec![("label", label)])?)
        }
        let items = w.array(items)?;
        let name = w.text(v.name.as_str())?;
        let domain = w.record(vec![("name", name), ("items", items)])?;
        domains.push((w.text(key.as_str())?, domain))
    }
    let domains = w.node(Node::Map {
        entries: domains,
        properties: vec![],
    })?;
    let mut transitions = vec![];
    for v in &m.transitions {
        let from = w.text(v.from.as_str())?;
        let to = w.text(v.to.as_str())?;
        let label = w.opt(v.label.as_ref())?;
        transitions.push(w.record(vec![("from", from), ("to", to), ("label", label)])?)
    }
    let transitions = w.array(transitions)?;
    let kind = w.text("cynefin")?;
    let title = w.opt(m.title.as_ref())?;
    let acc_title = w.opt(m.acc_title.as_ref())?;
    let acc_descr = w.opt(m.acc_descr.as_ref())?;
    let root = w.record(vec![
        ("type", kind),
        ("title", title),
        ("accTitle", acc_title),
        ("accDescr", acc_descr),
        ("domains", domains),
        ("transitions", transitions),
    ])?;
    w.out.finish(root)
}
struct Import<'a> {
    g: &'a ModelGraph,
    units: usize,
}
impl<'a> Import<'a> {
    fn text(&mut self, a: &Atom) -> Result<String, PublicError> {
        let Atom::String(i) = a else {
            return Err(ty("Cynefin render text must be a string"));
        };
        let s = &self.g.strings[*i as usize];
        self.units = self.units.saturating_add(s.0.len());
        if self.units > TEXT {
            return Err(limit().into());
        }
        s.well_formed()
            .ok_or_else(|| error("Cynefin rendering requires well-formed UTF16 text").into())
    }
    fn optional(&mut self, a: &Atom, accessible: bool) -> Result<Option<String>, PublicError> {
        if matches!(a, Atom::Undefined) || accessible && !truth(self.g, a) {
            Ok(None)
        } else {
            self.text(a).map(Some)
        }
    }
    fn array(&self, a: &Atom) -> Result<(u32, HashMap<u32, &'a Atom>), PublicError> {
        let Some(id) = reference(a) else {
            return Err(ty("Cynefin render collection must be an Array"));
        };
        let Node::Array { length, properties } = &self.g.nodes[id as usize] else {
            return Err(ty("Cynefin render collection must be an Array"));
        };
        if *length as usize > ITEMS {
            return Err(limit().into());
        }
        let mut values = HashMap::with_capacity(properties.len());
        for p in properties {
            let Some(key) = self.g.strings[p.key as usize].well_formed() else {
                continue;
            };
            if let Ok(index) = key.parse::<u32>() {
                if index < *length && index.to_string() == key {
                    values.insert(index, &p.value);
                }
            }
        }
        Ok((*length, values))
    }
}
fn project(g: &ModelGraph) -> Result<CynefinModel, PublicError> {
    let Some(root) = reference(&g.root) else {
        return Err(ty("Cynefin model must be an object"));
    };
    let mut i = Import { g, units: 0 };
    let title = i.optional(get(g, root, "title"), false)?;
    let acc_title = i.optional(get(g, root, "accTitle"), true)?;
    let acc_descr = i.optional(get(g, root, "accDescr"), true)?;
    let Some(domains) = reference(get(g, root, "domains")) else {
        return Err(ty("Cynefin domains.get is not a function"));
    };
    let Node::Map {
        entries,
        properties,
    } = &g.nodes[domains as usize]
    else {
        return Err(ty("Cynefin domains.get is not a function"));
    };
    if properties
        .iter()
        .any(|p| g.strings[p.key as usize].is("get"))
    {
        return Err(error("Cynefin overridden domain getter dispatch is not implemented").into());
    }
    let mut out = vec![];
    for n in NAMES {
        let domain = entries
            .iter()
            .find(|(key, _)| name(g, key) == Some(n))
            .map(|(_, v)| v);
        let items = domain
            .and_then(reference)
            .map_or(&Atom::Undefined, |id| get(g, id, "items"));
        if matches!(items, Atom::Undefined | Atom::Null) {
            continue;
        }
        let (length, values) = i.array(items)?;
        let shown = if n == DomainName::Confusion {
            length.min(3)
        } else {
            length
        };
        let mut items = Vec::with_capacity(length as usize);
        for at in 0..shown {
            let value = values.get(&at).copied().unwrap_or(&Atom::Undefined);
            let Some(id) = reference(value) else {
                return Err(ty("Cannot read properties of undefined (reading 'label')"));
            };
            items.push(CynefinItem {
                label: i.text(get(g, id, "label"))?,
            })
        }
        items.resize_with(length as usize, || CynefinItem {
            label: String::new(),
        });
        out.push((n, CynefinDomain { name: n, items }))
    }
    let mut transitions = vec![];
    let (length, values) = i.array(get(g, root, "transitions"))?;
    for at in 0..length {
        let value = values.get(&at).copied().unwrap_or(&Atom::Undefined);
        let Some(id) = reference(value) else {
            if matches!(value, Atom::Undefined | Atom::Null) {
                return Err(ty("Cannot read properties of undefined (reading 'from')"));
            }
            continue;
        };
        let Some(from) = name(g, get(g, id, "from")) else {
            continue;
        };
        let Some(to) = name(g, get(g, id, "to")) else {
            continue;
        };
        if from == to {
            continue;
        }
        let label = get(g, id, "label");
        let label = if truth(g, label) {
            Some(i.text(label)?)
        } else {
            None
        };
        transitions.push(CynefinTransition { from, to, label })
    }
    Ok(CynefinModel {
        diagram_type: "cynefin",
        title,
        acc_title,
        acc_descr,
        domains: out,
        transitions,
    })
}
impl PublicDiagramModule for Cynefin {
    type PublicState = CynefinPublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let projected = project(&graph);
        Ok(CynefinPublicState { graph, projected })
    }
    fn create_public(&self, m: &CynefinModel) -> Result<Self::PublicState, PeleError> {
        let graph = new_graph(m)?;
        let projected = project(&graph);
        Ok(CynefinPublicState { graph, projected })
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
    fn apply_public(&self, s: &mut Self::PublicState, graph: ModelGraph) -> Result<(), PeleError> {
        s.projected = project(&graph);
        s.graph = graph;
        Ok(())
    }
    fn render_public_operation(
        &self,
        s: &mut Self::PublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError> {
        self.render(s.projected.as_ref().map_err(Clone::clone)?, c, o, h)
            .map_err(Into::into)
    }
    fn render_public(
        &self,
        s: &mut Self::PublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render_public_operation(s, c, o, h).map_err(legacy)
    }
}
