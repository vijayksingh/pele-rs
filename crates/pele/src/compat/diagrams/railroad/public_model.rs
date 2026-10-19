//! Native RailroadDb arena codec and actual returned methods (Pele 458322a5, MIT).
//! Flat IR references preserve shared nodes and allow iterative deep import/export.
//! Source byName preserves first key order and aliases the latest rule of each name;
//! typed factory reconstruction uses current rules, not erased private lookup history.
//! Default Array length flags2 is implicit; nondefault own length is explicit.
//! Constructor/callback dispatch and active lone-surrogate SVG transport remain
//! explicit capabilities; no JavaScript parser or renderer is used at runtime.
use super::*;
use crate::compat::public::{
    Atom, FieldKind, FieldSchema, GraphWriter, JavaScriptErrorKind, JsString, MemberKind,
    MemberSchema, MethodCall, MethodId, MethodSchema, ModelGraph, ModelSchema, Node, Property,
    PrototypeId, PrototypeParent, PrototypeSchema, PublicDiagramModule, PublicError, RecordId,
    RecordKind, RecordSchema,
};
use crate::compat::ErrorCode;
use std::collections::HashMap;
const PROTO: PrototypeId = PrototypeId(1);
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
const fn method(id: u32, name: &'static str, arity: u32) -> MethodSchema {
    MethodSchema {
        id: MethodId(id),
        name,
        arity,
    }
}
const fn member(id: u32, name: &'static str) -> MemberSchema {
    MemberSchema {
        name,
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(id),
        },
    }
}
static SCHEMA: ModelSchema = ModelSchema {
    id: "pele/railroad@1",
    version: 1,
    root: Some(RecordId(1)),
    records: &[
        RecordSchema {
            id: RecordId(1),
            name: "RailroadDb",
            kind: RecordKind::Object,
            prototype: Some(PROTO),
            fields: &[
                f("type", FieldKind::String, false),
                f("notation", FieldKind::String, false),
                f("title", FieldKind::String, true),
                f("accTitle", FieldKind::String, true),
                f("accDescr", FieldKind::String, true),
                f("rules", FieldKind::Array, false),
                f("byName", FieldKind::Map, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(2),
            name: "RailroadRule",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("name", FieldKind::String, false),
                f("definition", FieldKind::Record(RecordId(3)), false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(3),
            name: "RailroadNode",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("type", FieldKind::String, false),
                FieldSchema {
                    optional: true,
                    ..f("value", FieldKind::String, false)
                },
                FieldSchema {
                    optional: true,
                    ..f("name", FieldKind::String, false)
                },
                FieldSchema {
                    optional: true,
                    ..f("text", FieldKind::String, false)
                },
                FieldSchema {
                    optional: true,
                    ..f("elements", FieldKind::Array, false)
                },
                FieldSchema {
                    optional: true,
                    ..f("alternatives", FieldKind::Array, false)
                },
                FieldSchema {
                    optional: true,
                    ..f("element", FieldKind::Record(RecordId(3)), false)
                },
                FieldSchema {
                    optional: true,
                    ..f("min", FieldKind::Number, false)
                },
                FieldSchema {
                    optional: true,
                    ..f("max", FieldKind::Number, false)
                },
            ],
            members: &[],
        },
    ],
    prototypes: &[PrototypeSchema {
        id: PROTO,
        name: "RailroadDb",
        parent: PrototypeParent::Object,
        members: &[
            member(1, "setTitle"),
            member(2, "setAccTitle"),
            member(3, "setAccDescription"),
            member(4, "addRule"),
            member(5, "getRule"),
        ],
    }],
    methods: &[
        method(1, "setTitle", 1),
        method(2, "setAccTitle", 1),
        method(3, "setAccDescription", 1),
        method(4, "addRule", 1),
        method(5, "getRule", 1),
    ],
};
pub struct RailroadPublicState {
    graph: ModelGraph,
    model: Result<RailroadModel, PeleError>,
}
fn invalid(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("railroad".into());
    e
}
fn limit() -> PeleError {
    PeleError::new(
        "Railroad native graph operation exceeds work budget",
        ErrorCode::Limit,
    )
}
fn own<'a>(g: &'a ModelGraph, id: u32, key: &str) -> Option<&'a Atom> {
    g.nodes
        .get(id as usize)?
        .properties()
        .iter()
        .find_map(|p| g.strings[p.key as usize].is(key).then_some(&p.value))
}
fn required<'a>(g: &'a ModelGraph, a: &Atom, k: &str) -> Result<&'a Atom, PeleError> {
    let Atom::Ref(id) = a else {
        return Err(invalid("Railroad drawable records must be objects"));
    };
    own(g, *id, k).ok_or_else(|| invalid(&format!("Railroad drawable field {k} is missing")))
}
struct Import<'a> {
    g: &'a ModelGraph,
    bytes: usize,
    work: usize,
}
impl Import<'_> {
    fn text(&mut self, a: &Atom) -> Result<String, PeleError> {
        let t = self.g.text(a)?;
        self.bytes = self.bytes.saturating_add(t.0.len().saturating_mul(3));
        if self.bytes > 128 * 1024 * 1024 {
            return Err(limit());
        }
        t.well_formed().ok_or_else(|| {
            invalid("Railroad rendering of lone UTF-16 surrogates is not implemented")
        })
    }
    fn field_text(&mut self, a: &Atom, k: &str) -> Result<String, PeleError> {
        self.text(required(self.g, a, k)?)
    }
    fn optional(&mut self, a: &Atom, k: &str) -> Result<Option<String>, PeleError> {
        let Atom::Ref(id) = a else {
            return Err(invalid("Railroad drawable records must be objects"));
        };
        match own(self.g, *id, k) {
            None | Some(Atom::Undefined) => Ok(None),
            Some(v) if k != "title" && !truth(self.g, v) => Ok(None),
            Some(v) => self.text(v).map(Some),
        }
    }
    fn array(&mut self, a: &Atom) -> Result<Vec<Atom>, PeleError> {
        let Atom::Ref(id) = a else {
            return Err(invalid("Railroad drawable list must be an Array"));
        };
        let Node::Array { length, properties } = &self.g.nodes[*id as usize] else {
            return Err(invalid("Railroad drawable list must be an Array"));
        };
        self.work = self
            .work
            .saturating_add(*length as usize)
            .saturating_add(properties.len());
        if self.work > 8_000_000 {
            return Err(limit());
        }
        let mut out = vec![Atom::Undefined; *length as usize];
        for p in properties {
            if let Some(i) = self.g.strings[p.key as usize]
                .well_formed()
                .and_then(|k| k.parse::<u32>().ok())
            {
                if i < *length && self.g.strings[p.key as usize].is(&i.to_string()) {
                    out[i as usize] = p.value.clone()
                }
            }
        }
        Ok(out)
    }
}
fn imported(g: &ModelGraph) -> Result<RailroadModel, PeleError> {
    let mut i = Import {
        g,
        bytes: 0,
        work: 0,
    };
    let root = &g.root;
    // Notation and byName are not read by rendering. Preserve raw edits in the arena.
    let mut m = RailroadModel::new("railroad", None);
    m.title = i.optional(root, "title")?;
    m.acc_title = i.optional(root, "accTitle")?;
    m.acc_descr = i.optional(root, "accDescr")?;
    let rules = i.array(required(g, root, "rules")?)?;
    let mut ids = HashMap::<u32, usize>::new();
    let mut colors = HashMap::<u32, u8>::new();
    for rule in rules {
        let name = i.field_text(&rule, "name")?;
        let definition = required(g, &rule, "definition")?.clone();
        let Atom::Ref(node) = definition else {
            return Err(invalid("Railroad definitions must be objects"));
        };
        let mut todo = vec![(node, false)];
        while let Some((n, done)) = todo.pop() {
            i.work = i.work.saturating_add(1);
            if i.work > 8_000_000 {
                return Err(limit());
            }
            if !done {
                match colors.get(&n) {
                    Some(2) => continue,
                    Some(1) => {
                        return Err(invalid("Railroad model contains a cyclic node reference."))
                    }
                    _ => {}
                }
                colors.insert(n, 1);
                let index = m.nodes.len();
                ids.insert(n, index);
                m.nodes.push(RailroadNode::Sequence(vec![]));
                todo.push((n, true));
                let a = Atom::Ref(n);
                let kind = i.field_text(&a, "type")?;
                let children = match kind.as_str() {
                    "sequence" => i.array(required(g, &a, "elements")?)?,
                    "choice" => i.array(required(g, &a, "alternatives")?)?,
                    "optional" | "repetition" => vec![required(g, &a, "element")?.clone()],
                    "terminal" | "nonterminal" | "special" => vec![],
                    _ => return Err(invalid(
                        "Railroad unknown IR node type is not representable in the typed renderer",
                    )),
                };
                // Extra structural fields affect the pinned generic fold and cannot be erased.
                for key in [
                    "alternatives",
                    "elements",
                    "element",
                    "primary",
                    "suffix",
                    "base",
                ] {
                    let expected = match kind.as_str() {
                        "sequence" => key == "elements",
                        "choice" => key == "alternatives",
                        "optional" | "repetition" => key == "element",
                        _ => false,
                    };
                    if !expected
                        && own(g, n, key)
                            .is_some_and(|a| !matches!(a, Atom::Undefined | Atom::Null))
                    {
                        return Err(invalid(
                            "Railroad cross-kind structural fields require an extended typed IR",
                        ));
                    }
                }
                for child in children.into_iter().rev() {
                    let Atom::Ref(c) = child else {
                        return Err(invalid("Railroad child definitions must be objects"));
                    };
                    todo.push((c, false));
                }
            } else {
                let a = Atom::Ref(n);
                let kind = i.field_text(&a, "type")?;
                let child = |key: &str| -> Result<usize, PeleError> {
                    let Atom::Ref(c) = required(g, &a, key)? else {
                        return Err(invalid("Railroad child definitions must be objects"));
                    };
                    ids.get(c)
                        .copied()
                        .ok_or_else(|| invalid("Railroad missing child definition"))
                };
                let value = match kind.as_str() {
                    "terminal" => RailroadNode::Terminal(i.field_text(&a, "value")?),
                    "nonterminal" => RailroadNode::Nonterminal(i.field_text(&a, "name")?),
                    "special" => RailroadNode::Special(i.field_text(&a, "text")?),
                    "optional" => RailroadNode::Optional(child("element")?),
                    "repetition" => {
                        let element = child("element")?;
                        let number = |k| match required(g, &a, k)? {
                            Atom::Number(n) => Ok(*n),
                            _ => Err(invalid("Railroad repetition bounds must be numbers")),
                        };
                        RailroadNode::Repetition {
                            element,
                            min: number("min")?,
                            max: number("max")?,
                        }
                    }
                    "sequence" | "choice" => {
                        let key = if kind == "sequence" {
                            "elements"
                        } else {
                            "alternatives"
                        };
                        let mut list = vec![];
                        for c in i.array(required(g, &a, key)?)? {
                            let Atom::Ref(c) = c else {
                                return Err(invalid("Railroad child definitions must be objects"));
                            };
                            list.push(ids[&c]);
                        }
                        if kind == "sequence" {
                            RailroadNode::Sequence(list)
                        } else {
                            RailroadNode::Choice(list)
                        }
                    }
                    _ => unreachable!(),
                };
                m.nodes[ids[&n]] = value;
                colors.insert(n, 2);
            }
        }
        m.rules.push(RailroadRule {
            name,
            definition: ids[&node],
        });
    }
    Ok(m)
}
fn rec(w: &mut GraphWriter, fields: Vec<(&str, Atom)>) -> u32 {
    let properties = fields.into_iter().map(|(k, v)| w.property(k, v)).collect();
    w.node(Node::Object { properties })
}
fn arr(w: &mut GraphWriter, values: impl IntoIterator<Item = Atom>) -> u32 {
    let properties: Vec<_> = values
        .into_iter()
        .enumerate()
        .map(|(n, v)| w.property(&n.to_string(), v))
        .collect();
    w.node(Node::Array {
        length: properties.len() as u32,
        properties,
    })
}
fn text(w: &mut GraphWriter, s: &str) -> Atom {
    Atom::String(w.string(s))
}
fn initial(m: &RailroadModel) -> Result<ModelGraph, PeleError> {
    m.validate_nodes()?;
    let mut w = GraphWriter::default();
    let mut replacements = vec![];
    let mut ids = vec![None; m.nodes.len()];
    let mut pending = vec![];
    for r in &m.rules {
        if ids[r.definition].is_none() {
            ids[r.definition] = Some(w.node(Node::Object { properties: vec![] }));
            pending.push(r.definition);
        }
    }
    let mut at = 0;
    while at < pending.len() {
        let index = pending[at];
        at += 1;
        for &child in m.nodes[index].children() {
            if ids[child].is_none() {
                ids[child] = Some(w.node(Node::Object { properties: vec![] }));
                pending.push(child);
            }
        }
    }
    for index in pending {
        let n = &m.nodes[index];
        let mut fields = vec![];
        let kind = match n {
            RailroadNode::Terminal(_) => "terminal",
            RailroadNode::Nonterminal(_) => "nonterminal",
            RailroadNode::Special(_) => "special",
            RailroadNode::Sequence(_) => "sequence",
            RailroadNode::Choice(_) => "choice",
            RailroadNode::Optional(_) => "optional",
            RailroadNode::Repetition { .. } => "repetition",
        };
        fields.push(("type", text(&mut w, kind)));
        match n {
            RailroadNode::Terminal(v) => fields.push(("value", text(&mut w, v))),
            RailroadNode::Nonterminal(v) => fields.push(("name", text(&mut w, v))),
            RailroadNode::Special(v) => fields.push(("text", text(&mut w, v))),
            RailroadNode::Sequence(v) | RailroadNode::Choice(v) => {
                let a = arr(&mut w, v.iter().map(|n| Atom::Ref(ids[*n].unwrap())));
                fields.push((
                    if kind == "sequence" {
                        "elements"
                    } else {
                        "alternatives"
                    },
                    Atom::Ref(a),
                ));
            }
            RailroadNode::Optional(v) => fields.push(("element", Atom::Ref(ids[*v].unwrap()))),
            RailroadNode::Repetition { element, min, max } => {
                fields.push(("element", Atom::Ref(ids[*element].unwrap())));
                fields.push(("min", Atom::Number(*min)));
                fields.push(("max", Atom::Number(*max)));
            }
        }
        let properties = fields.into_iter().map(|(k, v)| w.property(k, v)).collect();
        replacements.push((ids[index].unwrap(), Node::Object { properties }));
    }
    let mut rules = vec![];
    let mut entries = vec![];
    let mut names = HashMap::new();
    for rule in &m.rules {
        let name = text(&mut w, &rule.name);
        let r = rec(
            &mut w,
            vec![
                ("name", name.clone()),
                ("definition", Atom::Ref(ids[rule.definition].unwrap())),
            ],
        );
        let value = Atom::Ref(r);
        rules.push(value.clone());
        if let Some(at) = names.get(&rule.name) {
            entries[*at] = (name, value);
        } else {
            names.insert(rule.name.clone(), entries.len());
            entries.push((name, value));
        }
    }
    let rules = arr(&mut w, rules);
    let by_name = w.node(Node::Map {
        entries,
        properties: vec![],
    });
    let mut properties = vec![];
    for (k, v) in [("type", "railroad"), ("notation", m.notation.as_str())] {
        let a = text(&mut w, v);
        properties.push(w.property(k, a));
    }
    for (k, v) in [
        ("title", &m.title),
        ("accTitle", &m.acc_title),
        ("accDescr", &m.acc_descr),
    ] {
        let a = v
            .as_deref()
            .map(|s| text(&mut w, s))
            .unwrap_or(Atom::Undefined);
        properties.push(w.property(k, a));
    }
    properties.push(w.property("rules", Atom::Ref(rules)));
    properties.push(w.property("byName", Atom::Ref(by_name)));
    let root = w.node(Node::Instance {
        prototype: PROTO,
        properties,
    });
    let mut g = w.finish(Atom::Ref(root))?;
    for (id, node) in replacements {
        g.nodes[id as usize] = node;
    }
    g.validate()?;
    Ok(g)
}
impl PublicDiagramModule for Railroad {
    type PublicState = RailroadPublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let model = imported(&graph);
        Ok(RailroadPublicState { graph, model })
    }
    fn create_public(&self, m: &RailroadModel) -> Result<RailroadPublicState, PeleError> {
        let graph = initial(m)?;
        let model = imported(&graph);
        Ok(RailroadPublicState { graph, model })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, s: &RailroadPublicState) -> Result<ModelGraph, PeleError> {
        Ok(s.graph.clone())
    }
    fn apply_public(&self, s: &mut RailroadPublicState, g: ModelGraph) -> Result<(), PeleError> {
        g.validate()?;
        s.model = imported(&g);
        s.graph = g;
        Ok(())
    }
    fn invoke_public(
        &self,
        s: &mut RailroadPublicState,
        c: MethodCall,
        _: &Object,
        _: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        let result = Context::new(&mut s.graph).invoke(c);
        s.model = imported(&s.graph);
        result
    }
    fn render_public(
        &self,
        s: &mut RailroadPublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render(s.model.as_ref().map_err(Clone::clone)?, c, o, h)
    }
}
fn type_error(s: &str) -> PublicError {
    PublicError::JavaScript {
        name: JavaScriptErrorKind::TypeError,
        message: JsString::from(s),
    }
}
fn truth(g: &ModelGraph, a: &Atom) -> bool {
    match a {
        Atom::Undefined | Atom::Null => false,
        Atom::Bool(b) => *b,
        Atom::Number(n) => *n != 0. && !n.is_nan(),
        Atom::String(id) => !g.strings[*id as usize].0.is_empty(),
        _ => true,
    }
}
fn space(c: u16) -> bool {
    matches!(c,0x9..=0xd|0x20|0xa0|0x1680|0x2000..=0x200a|0x2028|0x2029|0x202f|0x205f|0x3000|0xfeff)
}
#[derive(Clone, Hash, PartialEq, Eq)]
enum Key {
    Undefined,
    Null,
    Bool(bool),
    Number(u64),
    String(u32),
    Ref(u32),
    Function(u32),
    Host(u32),
}
struct Context<'a> {
    g: &'a mut ModelGraph,
    strings: HashMap<JsString, u32>,
    canonical: Vec<u32>,
    maps: HashMap<u32, HashMap<Key, usize>>,
    properties: HashMap<u32, HashMap<Vec<u16>, usize>>,
    work: usize,
    refs: usize,
    units: usize,
}
impl<'a> Context<'a> {
    fn new(g: &'a mut ModelGraph) -> Self {
        let mut strings = HashMap::new();
        let mut canonical = vec![];
        let mut units = 0;
        let mut refs = 0;
        for (n, s) in g.strings.iter().enumerate() {
            units += s.0.len();
            canonical.push(*strings.entry(s.clone()).or_insert(n as u32));
        }
        for n in &g.nodes {
            refs += n.properties().len()
                + match n {
                    Node::Map { entries, .. } => entries.len() * 2,
                    Node::Set { values, .. } => values.len(),
                    _ => 0,
                };
        }
        Self {
            g,
            strings,
            canonical,
            maps: HashMap::new(),
            properties: HashMap::new(),
            work: 0,
            refs,
            units,
        }
    }
    fn tick(&mut self, n: usize) -> Result<(), PublicError> {
        self.work = self.work.saturating_add(n);
        if self.work > 32_000_000 {
            Err(limit().into())
        } else {
            Ok(())
        }
    }
    fn room(&mut self, refs: usize) -> Result<(), PublicError> {
        self.refs = self.refs.saturating_add(refs);
        if self.refs > 8_000_000 || self.g.nodes.len() >= 1_000_000 {
            Err(limit().into())
        } else {
            Ok(())
        }
    }
    fn intern(&mut self, s: JsString) -> Result<Atom, PublicError> {
        if let Some(id) = self.strings.get(&s) {
            return Ok(Atom::String(*id));
        }
        if self.g.strings.len() >= 1_000_000 || self.units.saturating_add(s.0.len()) > 16_000_000 {
            return Err(limit().into());
        }
        self.units += s.0.len();
        let id = self.g.strings.len() as u32;
        self.strings.insert(s.clone(), id);
        self.canonical.push(id);
        self.g.strings.push(s);
        Ok(Atom::String(id))
    }
    fn text(&mut self, s: &str) -> Result<Atom, PublicError> {
        self.intern(JsString::from(s))
    }
    fn key(&self, a: &Atom) -> Key {
        match a {
            Atom::Undefined => Key::Undefined,
            Atom::Null => Key::Null,
            Atom::Bool(b) => Key::Bool(*b),
            Atom::Number(n) => Key::Number(if n.is_nan() {
                f64::NAN.to_bits()
            } else if *n == 0. {
                0
            } else {
                n.to_bits()
            }),
            Atom::String(id) => Key::String(self.canonical[*id as usize]),
            Atom::Ref(id) => Key::Ref(*id),
            Atom::BuiltinFunction(id) => Key::Function(*id),
            Atom::HostReference(id) => Key::Host(*id),
        }
    }
    fn property_index(&mut self, id: u32, k: &str) -> Result<Option<usize>, PublicError> {
        if !self.properties.contains_key(&id) {
            self.tick(self.g.nodes[id as usize].properties().len())?;
            let index = self.g.nodes[id as usize]
                .properties()
                .iter()
                .enumerate()
                .map(|(n, p)| (self.g.strings[p.key as usize].0.clone(), n))
                .collect();
            self.properties.insert(id, index);
        }
        self.tick(1)?;
        Ok(self.properties[&id]
            .get(&k.encode_utf16().collect::<Vec<_>>())
            .copied())
    }
    fn own(&mut self, id: u32, k: &str) -> Result<Option<Atom>, PublicError> {
        Ok(self
            .property_index(id, k)?
            .map(|n| self.g.nodes[id as usize].properties()[n].value.clone()))
    }
    fn read(&mut self, a: &Atom, k: &str) -> Result<Atom, PublicError> {
        match a {
            Atom::Undefined | Atom::Null => Err(type_error(&format!(
                "Cannot read properties of {} (reading '{k}')",
                if matches!(a, Atom::Null) {
                    "null"
                } else {
                    "undefined"
                }
            ))),
            Atom::Ref(id) => {
                if k == "length" {
                    if let Node::Array { length, .. } = self.g.nodes[*id as usize] {
                        return Ok(Atom::Number(length as f64));
                    }
                }
                if let Some(value) = self.own(*id, k)? {
                    return Ok(value);
                }
                if k == "size" {
                    if let Node::Map { entries, .. } = &self.g.nodes[*id as usize] {
                        return Ok(Atom::Number(entries.len() as f64));
                    }
                }
                Ok(Atom::Undefined)
            }
            Atom::String(id) => {
                let units = &self.g.strings[*id as usize].0;
                if k == "length" {
                    return Ok(Atom::Number(units.len() as f64));
                }
                if let Some(n) = k.parse::<usize>().ok().filter(|n| n.to_string() == k) {
                    if let Some(c) = units.get(n).copied() {
                        return self.intern(JsString(vec![c]));
                    }
                }
                Ok(Atom::Undefined)
            }
            _ => Ok(Atom::Undefined),
        }
    }
    fn set(&mut self, a: &Atom, k: &str, v: Atom) -> Result<(), PublicError> {
        let Atom::Ref(id) = a else {
            if matches!(a, Atom::Null | Atom::Undefined) {
                return Err(type_error(&format!(
                    "Cannot set properties of {} (setting '{k}')",
                    if matches!(a, Atom::Null) {
                        "null"
                    } else {
                        "undefined"
                    }
                )));
            }
            let kind = match a {
                Atom::String(_) => "string",
                Atom::Bool(_) => "boolean",
                Atom::Number(_) => "number",
                _ => {
                    return Err(
                        invalid("Railroad host property receiver dispatch is unavailable").into(),
                    )
                }
            };
            let mut units = format!("Cannot create property '{k}' on {kind} '")
                .encode_utf16()
                .collect::<Vec<_>>();
            units.extend(self.coercion(a)?);
            units.push(39);
            return Err(PublicError::JavaScript {
                name: JavaScriptErrorKind::TypeError,
                message: JsString(units),
            });
        };
        let at = self.property_index(*id, k)?;
        if let Some(at) = at {
            if self.g.nodes[*id as usize].properties()[at].flags & 2 == 0 {
                let name = if matches!(
                    self.g.nodes[*id as usize],
                    Node::Instance {
                        prototype: PROTO,
                        ..
                    }
                ) {
                    "mf"
                } else {
                    "Object"
                };
                return Err(type_error(&format!(
                    "Cannot assign to read only property '{k}' of object '#<{name}>'"
                )));
            }
            self.g.nodes[*id as usize].properties_mut()[at].value = v;
        } else {
            self.room(1)?;
            let Atom::String(key) = self.text(k)? else {
                unreachable!()
            };
            let n = self.g.nodes[*id as usize].properties().len();
            self.g.nodes[*id as usize].properties_mut().push(Property {
                key,
                value: v,
                flags: 7,
            });
            self.properties
                .get_mut(id)
                .unwrap()
                .insert(k.encode_utf16().collect(), n);
        }
        Ok(())
    }
    fn native(&mut self, a: &Atom, p: &str, expr: &str, kind: u8) -> Result<u32, PublicError> {
        let found = self.read(a, p)?;
        if matches!(found, Atom::BuiltinFunction(_) | Atom::HostReference(_)) {
            return Err(invalid(&format!(
                "Railroad overridden {expr} callback dispatch is unavailable"
            ))
            .into());
        }
        let Atom::Ref(id) = a else {
            return Err(type_error(&format!("{expr} is not a function")));
        };
        let good = matches!(
            (&self.g.nodes[*id as usize], kind),
            (Node::Map { .. }, 0) | (Node::Array { .. }, 1) | (Node::Set { .. }, 2)
        );
        if !good || self.own(*id, p)?.is_some() {
            return Err(type_error(&format!("{expr} is not a function")));
        }
        Ok(*id)
    }
    fn map_index(&mut self, id: u32) -> Result<(), PublicError> {
        if !self.maps.contains_key(&id) {
            let Node::Map { entries, .. } = &self.g.nodes[id as usize] else {
                unreachable!()
            };
            let entries = entries.clone();
            self.tick(entries.len())?;
            let mut index = HashMap::new();
            for (n, (k, _)) in entries.iter().enumerate() {
                index.insert(self.key(k), n);
            }
            self.maps.insert(id, index);
        }
        Ok(())
    }
    fn map_get(&mut self, a: &Atom, k: &Atom, has: bool, expr: &str) -> Result<Atom, PublicError> {
        let id = self.native(a, if has { "has" } else { "get" }, expr, 0)?;
        self.map_index(id)?;
        let at = self.maps[&id].get(&self.key(k));
        let Node::Map { entries, .. } = &self.g.nodes[id as usize] else {
            unreachable!()
        };
        Ok(if has {
            Atom::Bool(at.is_some())
        } else {
            at.map_or(Atom::Undefined, |n| entries[*n].1.clone())
        })
    }
    fn map_set(&mut self, a: &Atom, k: Atom, v: Atom, expr: &str) -> Result<(), PublicError> {
        let id = self.native(a, "set", expr, 0)?;
        self.map_index(id)?;
        let key = self.key(&k);
        let at = self.maps[&id].get(&key).copied();
        if at.is_none() {
            self.room(2)?
        }
        let Node::Map { entries, .. } = &mut self.g.nodes[id as usize] else {
            unreachable!()
        };
        if let Some(n) = at {
            entries[n].1 = v
        } else {
            let n = entries.len();
            entries.push((
                if matches!(k,Atom::Number(n) if n==0.) {
                    Atom::Number(0.)
                } else {
                    k
                },
                v,
            ));
            self.maps.get_mut(&id).unwrap().insert(key, n);
        }
        Ok(())
    }
    fn push(&mut self, a: &Atom, v: Atom, expr: &str) -> Result<(), PublicError> {
        let id = self.native(a, "push", expr, 1)?;
        let Node::Array { length, .. } = self.g.nodes[id as usize] else {
            unreachable!()
        };
        if let Some(Atom::Number(_)) = self.own(id, "length")? {
            let p = self.g.nodes[id as usize]
                .properties()
                .iter()
                .find(|p| self.g.strings[p.key as usize].is("length"))
                .unwrap();
            if p.flags & 2 == 0 {
                return Err(type_error(
                    "Cannot assign to read only property 'length' of object '[object Array]'",
                ));
            }
        }
        if length >= 1_000_000 {
            return Err(limit().into());
        }
        self.set(a, &length.to_string(), v)?;
        let length_at = self.property_index(id, "length")?;
        if let Node::Array {
            length: len,
            properties,
        } = &mut self.g.nodes[id as usize]
        {
            *len = length + 1;
            if let Some(at) = length_at {
                properties[at].value = Atom::Number(*len as f64);
            }
        }
        Ok(())
    }
    fn string_units(&mut self, a: &Atom, expr: &str) -> Result<Vec<u16>, PublicError> {
        if let Atom::String(id) = a {
            self.tick(self.g.strings[*id as usize].0.len())?;
            Ok(self.g.strings[*id as usize].0.clone())
        } else {
            let property = expr.rsplit('.').next().unwrap_or(expr);
            let found = self.read(a, property)?;
            if matches!(found, Atom::BuiltinFunction(_) | Atom::HostReference(_)) {
                return Err(invalid(&format!(
                    "Railroad overridden {expr} callback dispatch is unavailable"
                ))
                .into());
            }
            Err(type_error(&format!("{expr} is not a function")))
        }
    }
    fn coercion(&mut self, a: &Atom) -> Result<Vec<u16>, PublicError> {
        match a {
            Atom::String(id) => {
                self.tick(self.g.strings[*id as usize].0.len())?;
                Ok(self.g.strings[*id as usize].0.clone())
            }
            Atom::Undefined => Ok("undefined".encode_utf16().collect()),
            Atom::Null => Ok("null".encode_utf16().collect()),
            Atom::Bool(b) => Ok(if *b { "true" } else { "false" }.encode_utf16().collect()),
            Atom::Number(n) => Ok(crate::compat::svg::js_num(*n).encode_utf16().collect()),
            _ => {
                Err(invalid("Railroad object primitive-conversion dispatch is unavailable").into())
            }
        }
    }
    fn invoke(&mut self, c: MethodCall) -> Result<Atom, PublicError> {
        let arg = |n| c.arguments.get(n).cloned().unwrap_or(Atom::Undefined);
        let r = &c.receiver;
        match c.method.0 {
            1 => self.set(r, "title", arg(0))?,
            2 | 3 => {
                let units = self.string_units(&arg(0), "t.replace")?;
                let changed = if c.method.0 == 2 {
                    let at = units.iter().position(|c| !space(*c)).unwrap_or(units.len());
                    units[at..].to_vec()
                } else {
                    let mut v = vec![];
                    let mut n = 0;
                    while n < units.len() {
                        v.push(units[n]);
                        if units[n] == 10 {
                            let start = n + 1;
                            if units.get(start).is_some_and(|c| space(*c)) {
                                n = start;
                                while n < units.len() && space(units[n]) {
                                    n += 1;
                                }
                                continue;
                            }
                        }
                        n += 1;
                    }
                    v
                };
                let value = self.intern(JsString(changed))?;
                self.set(
                    r,
                    if c.method.0 == 2 {
                        "accTitle"
                    } else {
                        "accDescr"
                    },
                    value,
                )?;
            }
            4 => {
                let rules = self.read(r, "rules")?;
                self.push(&rules, arg(0), "this.rules.push")?;
                let map = self.read(r, "byName")?;
                // Resolve the member, then evaluate rule.name, then check callability.
                self.read(&map, "set")?;
                let name = self.read(&arg(0), "name")?;
                self.map_set(&map, name, arg(0), "this.byName.set")?;
            }
            5 => {
                let map = self.read(r, "byName")?;
                return self.map_get(&map, &arg(0), false, "this.byName.get");
            }
            _ => return Err(invalid("Railroad method is unavailable").into()),
        }
        Ok(Atom::Undefined)
    }
}
