//! Authoritative flat arena preserves Maps, record/array aliases, UTF-16 and IEEE values.
//! Source provenance retains the real builder counter and pending fields. Typed factory
//! reconstruction has no erased JS mutation history. Array scalar length's default
//! flags2 descriptor is implicit; nondefault own length is explicit. Host callbacks,
//! accessors, arbitrary primitive conversion and active lone-surrogate SVG text
//! remain explicit capabilities rather than a JavaScript interpreter.
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
    id: "pele/requirement@1",
    version: 1,
    root: Some(RecordId(1)),
    records: &[
        RecordSchema {
            id: RecordId(1),
            name: "RequirementDb",
            kind: RecordKind::Object,
            prototype: Some(PROTO),
            fields: &[
                f("type", FieldKind::String, false),
                f("direction", FieldKind::String, false),
                f("requirements", FieldKind::Map, false),
                f("elements", FieldKind::Map, false),
                f("relations", FieldKind::Array, false),
                f("classes", FieldKind::Map, false),
                f("title", FieldKind::String, true),
                f("accTitle", FieldKind::String, true),
                f("accDescr", FieldKind::String, true),
                f("newId", FieldKind::String, false),
                f("newText", FieldKind::String, false),
                f("newRisk", FieldKind::String, false),
                f("newVerifyMethod", FieldKind::String, false),
                f("newElementType", FieldKind::String, false),
                f("newDocRef", FieldKind::String, false),
                f("styleCopies", FieldKind::Number, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(2),
            name: "Requirement",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("name", FieldKind::String, false),
                f("type", FieldKind::String, false),
                f("requirementId", FieldKind::String, false),
                f("text", FieldKind::String, false),
                f("risk", FieldKind::String, false),
                f("verifyMethod", FieldKind::String, false),
                f("cssStyles", FieldKind::Array, false),
                f("classes", FieldKind::Array, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(3),
            name: "RequirementElement",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("name", FieldKind::String, false),
                f("type", FieldKind::String, false),
                f("docRef", FieldKind::String, false),
                f("cssStyles", FieldKind::Array, false),
                f("classes", FieldKind::Array, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(4),
            name: "Relation",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("type", FieldKind::String, false),
                f("src", FieldKind::String, false),
                f("dst", FieldKind::String, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(5),
            name: "RequirementClass",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("id", FieldKind::String, false),
                f("styles", FieldKind::Array, false),
                f("textStyles", FieldKind::Array, false),
            ],
            members: &[],
        },
    ],
    prototypes: &[PrototypeSchema {
        id: PROTO,
        name: "RequirementDb",
        parent: PrototypeParent::Object,
        members: &[
            member(1, "copy"),
            member(2, "setDirection"),
            member(3, "addRequirement"),
            member(4, "setNewReqId"),
            member(5, "setNewReqText"),
            member(6, "setNewReqRisk"),
            member(7, "setNewReqVerifyMethod"),
            member(8, "addElement"),
            member(9, "setNewElementType"),
            member(10, "setNewElementDocRef"),
            member(11, "addRelationship"),
            member(12, "node"),
            member(13, "setCssStyle"),
            member(14, "setClass"),
            member(15, "defineClass"),
            member(16, "setAccTitle"),
            member(17, "setAccDescription"),
        ],
    }],
    methods: &[
        method(1, "copy", 2),
        method(2, "setDirection", 1),
        method(3, "addRequirement", 2),
        method(4, "setNewReqId", 1),
        method(5, "setNewReqText", 1),
        method(6, "setNewReqRisk", 1),
        method(7, "setNewReqVerifyMethod", 1),
        method(8, "addElement", 1),
        method(9, "setNewElementType", 1),
        method(10, "setNewElementDocRef", 1),
        method(11, "addRelationship", 3),
        method(12, "node", 1),
        method(13, "setCssStyle", 2),
        method(14, "setClass", 2),
        method(15, "defineClass", 2),
        method(16, "setAccTitle", 1),
        method(17, "setAccDescription", 1),
    ],
};
pub struct RequirementPublicState {
    graph: ModelGraph,
    model: Result<RequirementModel, PeleError>,
}
fn invalid(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("requirement".into());
    e
}
fn limit() -> PeleError {
    PeleError::new(
        "Requirement native graph operation exceeds work budget",
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
        return Err(invalid("Requirement drawable records must be objects"));
    };
    own(g, *id, k).ok_or_else(|| invalid(&format!("Requirement drawable field {k} is missing")))
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
            invalid("Requirement rendering of lone UTF-16 surrogates is not implemented")
        })
    }
    fn field_text(&mut self, a: &Atom, k: &str) -> Result<String, PeleError> {
        self.text(required(self.g, a, k)?)
    }
    // The pinned renderer omits falsy body rows before asking for label text.
    fn body(&mut self, a: &Atom, k: &str) -> Result<String, PeleError> {
        let Atom::Ref(id) = a else {
            return Err(invalid("Requirement drawable records must be objects"));
        };
        let v = own(self.g, *id, k).cloned().unwrap_or(Atom::Undefined);
        if !truth(self.g, &v) {
            Ok(String::new())
        } else {
            self.text(&v)
        }
    }
    fn optional(&mut self, a: &Atom, k: &str) -> Result<Option<String>, PeleError> {
        let Atom::Ref(id) = a else {
            return Err(invalid("Requirement drawable records must be objects"));
        };
        match own(self.g, *id, k) {
            None | Some(Atom::Undefined | Atom::Null) => Ok(None),
            Some(v) => self.text(v).map(Some),
        }
    }
    fn array(&mut self, a: &Atom) -> Result<Vec<Atom>, PeleError> {
        let Atom::Ref(id) = a else {
            return Err(invalid("Requirement drawable list must be an Array"));
        };
        let Node::Array { length, properties } = &self.g.nodes[*id as usize] else {
            return Err(invalid("Requirement drawable list must be an Array"));
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
    fn texts(&mut self, a: &Atom) -> Result<Vec<String>, PeleError> {
        self.array(a)?.iter().map(|a| self.text(a)).collect()
    }
    fn map(&mut self, a: &Atom) -> Result<Vec<(Atom, Atom)>, PeleError> {
        let Atom::Ref(id) = a else {
            return Err(invalid("Requirement drawable lookup must be a Map"));
        };
        let Node::Map { entries, .. } = &self.g.nodes[*id as usize] else {
            return Err(invalid("Requirement drawable lookup must be a Map"));
        };
        self.work = self.work.saturating_add(entries.len());
        if self.work > 8_000_000 {
            return Err(limit());
        }
        Ok(entries.clone())
    }
}
fn imported(g: &ModelGraph) -> Result<RequirementModel, PeleError> {
    let mut i = Import {
        g,
        bytes: 0,
        work: 0,
    };
    let r = &g.root;
    let mut m = model::Db::new(None).finish();
    m.direction = i.field_text(r, "direction")?;
    m.title = i.optional(r, "title")?;
    m.acc_title = i.optional(r, "accTitle")?;
    m.acc_descr = i.optional(r, "accDescr")?;
    for (k, a) in i.map(required(g, r, "requirements")?)? {
        m.requirements.push((
            i.text(&k)?,
            Requirement {
                name: i.field_text(&a, "name")?,
                requirement_type: i.field_text(&a, "type")?,
                requirement_id: i.body(&a, "requirementId")?,
                text: i.body(&a, "text")?,
                risk: i.body(&a, "risk")?,
                verify_method: i.body(&a, "verifyMethod")?,
                css_styles: i.texts(required(g, &a, "cssStyles")?)?,
                classes: i.texts(required(g, &a, "classes")?)?,
            },
        ));
    }
    for (k, a) in i.map(required(g, r, "elements")?)? {
        m.elements.push((
            i.text(&k)?,
            RequirementElement {
                name: i.field_text(&a, "name")?,
                element_type: i.body(&a, "type")?,
                doc_ref: i.body(&a, "docRef")?,
                css_styles: i.texts(required(g, &a, "cssStyles")?)?,
                classes: i.texts(required(g, &a, "classes")?)?,
            },
        ));
    }
    for a in i.array(required(g, r, "relations")?)? {
        m.relations.push(Relation {
            relation_type: i.field_text(&a, "type")?,
            src: i.field_text(&a, "src")?,
            dst: i.field_text(&a, "dst")?,
        });
    }
    // Classes and pending builder fields are consumed by methods, never by rendering.
    // Keep their complete values/aliases in the arena; do not demand scalar typed copies.
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
fn texts(w: &mut GraphWriter, values: &[String]) -> u32 {
    let values = values
        .iter()
        .map(|s| Atom::String(w.string(s.as_str())))
        .collect::<Vec<_>>();
    arr(w, values)
}
fn text(w: &mut GraphWriter, s: &str) -> Atom {
    Atom::String(w.string(s))
}
fn initial(m: &RequirementModel) -> Result<ModelGraph, PeleError> {
    let mut w = GraphWriter::default();
    let mut properties = vec![];
    for (k, v) in [("type", "requirement"), ("direction", m.direction.as_str())] {
        let a = text(&mut w, v);
        properties.push(w.property(k, a));
    }
    let mut req = vec![];
    for (k, v) in &m.requirements {
        let mut fs = vec![];
        for (k, v) in [
            ("name", v.name.as_str()),
            ("type", v.requirement_type.as_str()),
            ("requirementId", v.requirement_id.as_str()),
            ("text", v.text.as_str()),
            ("risk", v.risk.as_str()),
            ("verifyMethod", v.verify_method.as_str()),
        ] {
            fs.push((k, text(&mut w, v)));
        }
        fs.push(("cssStyles", Atom::Ref(texts(&mut w, &v.css_styles))));
        fs.push(("classes", Atom::Ref(texts(&mut w, &v.classes))));
        let v = rec(&mut w, fs);
        let k = text(&mut w, k);
        req.push((k, Atom::Ref(v)));
    }
    let req = w.node(Node::Map {
        entries: req,
        properties: vec![],
    });
    properties.push(w.property("requirements", Atom::Ref(req)));
    let mut els = vec![];
    for (k, v) in &m.elements {
        let mut fs = vec![];
        for (k, v) in [
            ("name", v.name.as_str()),
            ("type", v.element_type.as_str()),
            ("docRef", v.doc_ref.as_str()),
        ] {
            fs.push((k, text(&mut w, v)));
        }
        fs.push(("cssStyles", Atom::Ref(texts(&mut w, &v.css_styles))));
        fs.push(("classes", Atom::Ref(texts(&mut w, &v.classes))));
        let v = rec(&mut w, fs);
        let k = text(&mut w, k);
        els.push((k, Atom::Ref(v)));
    }
    let els = w.node(Node::Map {
        entries: els,
        properties: vec![],
    });
    properties.push(w.property("elements", Atom::Ref(els)));
    let mut relations = vec![];
    for v in &m.relations {
        let mut fs = vec![];
        for (k, v) in [
            ("type", v.relation_type.as_str()),
            ("src", v.src.as_str()),
            ("dst", v.dst.as_str()),
        ] {
            fs.push((k, text(&mut w, v)));
        }
        relations.push(Atom::Ref(rec(&mut w, fs)));
    }
    let a = arr(&mut w, relations);
    properties.push(w.property("relations", Atom::Ref(a)));
    let mut classes = vec![];
    for (k, v) in &m.classes {
        let id = text(&mut w, &v.id);
        let styles = texts(&mut w, &v.styles);
        let ts = texts(&mut w, &v.text_styles);
        let a = rec(
            &mut w,
            vec![
                ("id", id),
                ("styles", Atom::Ref(styles)),
                ("textStyles", Atom::Ref(ts)),
            ],
        );
        let k = text(&mut w, k);
        classes.push((k, Atom::Ref(a)));
    }
    let a = w.node(Node::Map {
        entries: classes,
        properties: vec![],
    });
    properties.push(w.property("classes", Atom::Ref(a)));
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
    for (k, v) in [
        "newId",
        "newText",
        "newRisk",
        "newVerifyMethod",
        "newElementType",
        "newDocRef",
    ]
    .into_iter()
    .zip(&m.public_provenance.pending)
    {
        let a = text(&mut w, v);
        properties.push(w.property(k, a));
    }
    properties.push(w.property(
        "styleCopies",
        Atom::Number(m.public_provenance.style_copies as f64),
    ));
    let root = w.node(Node::Instance {
        prototype: PROTO,
        properties,
    });
    w.finish(Atom::Ref(root))
}
fn construct_default() -> Result<RequirementPublicState, PeleError> {
    // Implicit Pele constructor: field defaults only, no source or argument reads.
    RequirementDiagram.create_public(&super::model::Db::new(None).finish())
}

impl PublicDiagramModule for RequirementDiagram {
    type PublicState = RequirementPublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn public_constructor(
        &self,
        prototype: PrototypeId,
    ) -> Option<crate::compat::public::PublicConstructorFactory<Self::PublicState>> {
        (prototype == PROTO).then(|| {
            crate::compat::public::PublicConstructorFactory::ignoring_arguments::<Self>(
                PROTO,
                construct_default,
            )
        })
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let model = imported(&graph);
        Ok(RequirementPublicState { graph, model })
    }
    fn create_public(&self, m: &RequirementModel) -> Result<RequirementPublicState, PeleError> {
        let graph = initial(m)?;
        let model = imported(&graph);
        Ok(RequirementPublicState { graph, model })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, s: &RequirementPublicState) -> Result<ModelGraph, PeleError> {
        Ok(s.graph.clone())
    }
    fn apply_public(&self, s: &mut RequirementPublicState, g: ModelGraph) -> Result<(), PeleError> {
        g.validate()?;
        s.model = imported(&g);
        s.graph = g;
        Ok(())
    }
    fn invoke_public(
        &self,
        s: &mut RequirementPublicState,
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
        s: &mut RequirementPublicState,
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
fn trimmed(v: &[u16]) -> &[u16] {
    let start = v.iter().position(|c| !space(*c)).unwrap_or(v.len());
    let end = v.iter().rposition(|c| !space(*c)).map_or(start, |n| n + 1);
    &v[start..end]
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
enum Iter {
    Array(u32, u32),
    Map(u32, usize),
    Set(u32, usize),
    String(u32, usize),
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
                    return Err(invalid(
                        "Requirement host property receiver dispatch is unavailable",
                    )
                    .into())
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
                    "Yc"
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
    fn node(&mut self, n: Node) -> Result<Atom, PublicError> {
        self.room(n.properties().len())?;
        let id = self.g.nodes.len() as u32;
        self.g.nodes.push(n);
        Ok(Atom::Ref(id))
    }
    fn record(&mut self, fields: Vec<(&str, Atom)>) -> Result<Atom, PublicError> {
        let a = self.node(Node::Object { properties: vec![] })?;
        for (k, v) in fields {
            self.set(&a, k, v)?
        }
        Ok(a)
    }
    fn array(&mut self, values: Vec<Atom>) -> Result<Atom, PublicError> {
        let a = self.node(Node::Array {
            length: 0,
            properties: vec![],
        })?;
        for v in values {
            self.push(&a, v, "array.push")?
        }
        Ok(a)
    }
    fn native(&mut self, a: &Atom, p: &str, expr: &str, kind: u8) -> Result<u32, PublicError> {
        let found = self.read(a, p)?;
        if matches!(found, Atom::BuiltinFunction(_) | Atom::HostReference(_)) {
            return Err(invalid(&format!(
                "Requirement overridden {expr} callback dispatch is unavailable"
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
    fn iter(&mut self, a: &Atom, expr: &str) -> Result<Iter, PublicError> {
        match a {
            Atom::Ref(id) => match self.g.nodes[*id as usize] {
                Node::Array { .. } => Ok(Iter::Array(*id, 0)),
                Node::Set { .. } => Ok(Iter::Set(*id, 0)),
                Node::Map { .. } => Ok(Iter::Map(*id, 0)),
                _ => Err(type_error(&format!("{expr} is not iterable"))),
            },
            Atom::String(id) => Ok(Iter::String(*id, 0)),
            _ => Err(type_error(&format!("{expr} is not iterable"))),
        }
    }
    fn next(&mut self, it: &mut Iter) -> Result<Option<Atom>, PublicError> {
        self.tick(1)?;
        match it {
            Iter::Array(id, n) => {
                let Node::Array { length, .. } = self.g.nodes[*id as usize] else {
                    unreachable!()
                };
                if *n >= length {
                    return Ok(None);
                }
                let a = self.read(&Atom::Ref(*id), &n.to_string())?;
                *n += 1;
                Ok(Some(a))
            }
            Iter::Map(id, n) => {
                let Node::Map { entries, .. } = &self.g.nodes[*id as usize] else {
                    unreachable!()
                };
                let value = entries.get(*n).cloned();
                *n += 1;
                value.map(|(k, v)| self.array(vec![k, v])).transpose()
            }
            Iter::Set(id, n) => {
                let Node::Set { values, .. } = &self.g.nodes[*id as usize] else {
                    unreachable!()
                };
                let v = values.get(*n).cloned();
                *n += 1;
                Ok(v)
            }
            Iter::String(id, n) => {
                let s = &self.g.strings[*id as usize].0;
                if *n >= s.len() {
                    return Ok(None);
                }
                let count = if (0xd800..=0xdbff).contains(&s[*n])
                    && s.get(*n + 1).is_some_and(|c| (0xdc00..=0xdfff).contains(c))
                {
                    2
                } else {
                    1
                };
                let t = JsString(s[*n..*n + count].to_vec());
                *n += count;
                self.intern(t).map(Some)
            }
        }
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
                    "Requirement overridden {expr} callback dispatch is unavailable"
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
            _ => Err(
                invalid("Requirement object primitive-conversion dispatch is unavailable").into(),
            ),
        }
    }
    fn helper(
        &mut self,
        r: &Atom,
        name: &str,
        id: u32,
        args: Vec<Atom>,
    ) -> Result<Atom, PublicError> {
        let f = self.read(r, name)?;
        let native = matches!(r,Atom::Ref(n)if matches!(self.g.nodes[*n as usize],Node::Instance{prototype:PROTO,..}));
        if matches!(f,Atom::BuiltinFunction(n)if n==id)
            || matches!(f, Atom::Undefined)
                && native
                && self
                    .own(if let Atom::Ref(n) = r { *n } else { 0 }, name)?
                    .is_none()
        {
            self.invoke(MethodCall {
                receiver: r.clone(),
                method: MethodId(id),
                arguments: args,
            })
        } else if matches!(f, Atom::BuiltinFunction(_) | Atom::HostReference(_)) {
            Err(invalid(&format!(
                "Requirement overridden this.{name} callback dispatch is unavailable"
            ))
            .into())
        } else {
            Err(type_error(&format!("this.{name} is not a function")))
        }
    }
    fn number(&mut self, a: &Atom) -> Result<f64, PublicError> {
        Ok(match a {
            Atom::Undefined => f64::NAN,
            Atom::Null => 0.,
            Atom::Bool(b) => {
                if *b {
                    1.
                } else {
                    0.
                }
            }
            Atom::Number(n) => *n,
            Atom::String(id) => {
                let Ok(s) = String::from_utf16(trimmed(&self.g.strings[*id as usize].0)) else {
                    return Ok(f64::NAN);
                };
                if s.is_empty() {
                    0.
                } else if s == "Infinity" || s == "+Infinity" {
                    f64::INFINITY
                } else if s == "-Infinity" {
                    f64::NEG_INFINITY
                } else if let Some((digits, radix)) = s
                    .strip_prefix("0x")
                    .or_else(|| s.strip_prefix("0X"))
                    .map(|v| (v, 16))
                    .or_else(|| {
                        s.strip_prefix("0o")
                            .or_else(|| s.strip_prefix("0O"))
                            .map(|v| (v, 8))
                    })
                    .or_else(|| {
                        s.strip_prefix("0b")
                            .or_else(|| s.strip_prefix("0B"))
                            .map(|v| (v, 2))
                    })
                {
                    if digits.is_empty() {
                        f64::NAN
                    } else {
                        let mut n = 0.;
                        for c in digits.chars() {
                            let Some(d) = c.to_digit(radix) else {
                                return Ok(f64::NAN);
                            };
                            n = n * radix as f64 + d as f64;
                        }
                        n
                    }
                } else if s
                    .bytes()
                    .all(|c| c.is_ascii_digit() || matches!(c, b'.' | b'e' | b'E' | b'+' | b'-'))
                {
                    s.parse().unwrap_or(f64::NAN)
                } else {
                    f64::NAN
                }
            }
            _ => {
                return Err(invalid(
                    "Requirement object numeric conversion dispatch is unavailable",
                )
                .into())
            }
        })
    }
    fn copy_styles(&mut self, r: &Atom, target: Atom, styles: Atom) -> Result<(), PublicError> {
        let old = self.read(r, "styleCopies")?;
        let len = self.read(&styles, "length")?;
        let n = if matches!(old, Atom::String(_)) || matches!(len, Atom::String(_)) {
            let mut units = self.coercion(&old)?;
            units.extend(self.coercion(&len)?);
            self.intern(JsString(units))?
        } else {
            Atom::Number(self.number(&old)? + self.number(&len)?)
        };
        self.set(r, "styleCopies", n)?;
        let total = self.read(r, "styleCopies")?;
        if self.number(&total)? > 1_000_000. {
            let mut e = PeleError::new(
                "Requirement diagram applies too many style declarations.",
                ErrorCode::Limit,
            );
            e.diagram_type = Some("requirement".into());
            return Err(e.into());
        }
        let mut it = self.iter(&styles, "n")?;
        while let Some(s) = self.next(&mut it)? {
            self.push(&target, s, "t.push")?;
        }
        Ok(())
    }
    fn lookup(&mut self, r: &Atom, id: Atom) -> Result<Atom, PublicError> {
        let map = self.read(r, "requirements")?;
        let value = self.map_get(&map, &id, false, "this.requirements.get")?;
        if !matches!(value, Atom::Null | Atom::Undefined) {
            return Ok(value);
        }
        let map = self.read(r, "elements")?;
        self.map_get(&map, &id, false, "this.elements.get")
    }
    fn has_value(&mut self, a: &Atom, needle: &Atom, expr: &str) -> Result<bool, PublicError> {
        if let Atom::String(id) = a {
            let units = self.coercion(needle)?;
            return Ok(units.is_empty()
                || self.g.strings[*id as usize]
                    .0
                    .windows(units.len())
                    .any(|w| w == units));
        }
        let id = self.native(a, "includes", expr, 1)?;
        let Node::Array { length, .. } = self.g.nodes[id as usize] else {
            unreachable!()
        };
        for n in 0..length {
            let value = self.read(a, &n.to_string())?;
            if self.key(&value) == self.key(needle) {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn split(&mut self, s: &Atom, expr: &str) -> Result<Vec<Atom>, PublicError> {
        let units = self.string_units(s, expr)?;
        units
            .split(|c| *c == 44)
            .map(|s| self.intern(JsString(s.to_vec())))
            .collect()
    }
    fn values(&mut self, map: &Atom) -> Result<u32, PublicError> {
        self.native(map, "values", "this.requirements.values", 0)
    }
    fn invoke(&mut self, c: MethodCall) -> Result<Atom, PublicError> {
        let arg = |n| c.arguments.get(n).cloned().unwrap_or(Atom::Undefined);
        let r = &c.receiver;
        match c.method.0 {
            1 => {
                self.copy_styles(r, arg(0), arg(1))?;
            }
            2 => {
                self.set(r, "direction", arg(0))?;
            }
            3 | 8 => {
                let req = c.method.0 == 3;
                let mapname = if req { "requirements" } else { "elements" };
                let map = self.read(r, mapname)?;
                let mut value =
                    self.map_get(&map, &arg(0), false, &format!("this.{mapname}.get"))?;
                if matches!(value, Atom::Undefined) {
                    let mut fields = vec![("name", arg(0))];
                    if req {
                        fields.push(("type", arg(1)));
                        for (k, builder) in [
                            ("requirementId", "newId"),
                            ("text", "newText"),
                            ("risk", "newRisk"),
                            ("verifyMethod", "newVerifyMethod"),
                        ] {
                            fields.push((k, self.read(r, builder)?));
                        }
                    } else {
                        fields.push(("type", self.read(r, "newElementType")?));
                        fields.push(("docRef", self.read(r, "newDocRef")?));
                    }
                    fields.push(("cssStyles", self.array(vec![])?));
                    let default = self.text("default")?;
                    fields.push(("classes", self.array(vec![default])?));
                    value = self.record(fields)?;
                    let map = self.read(r, mapname)?;
                    self.map_set(&map, arg(0), value.clone(), &format!("this.{mapname}.set"))?;
                }
                // Chained assignments execute right to left, including partial throws.
                for key in if req {
                    vec!["newVerifyMethod", "newRisk", "newText", "newId"]
                } else {
                    vec!["newDocRef", "newElementType"]
                } {
                    let empty = self.text("")?;
                    self.set(r, key, empty)?;
                }
                return Ok(value);
            }
            4..=7 | 9 | 10 => {
                let key = match c.method.0 {
                    4 => "newId",
                    5 => "newText",
                    6 => "newRisk",
                    7 => "newVerifyMethod",
                    9 => "newElementType",
                    _ => "newDocRef",
                };
                self.set(r, key, arg(0))?;
            }
            11 => {
                let list = self.read(r, "relations")?;
                self.native(&list, "push", "this.relations.push", 1)?;
                let record =
                    self.record(vec![("type", arg(0)), ("src", arg(1)), ("dst", arg(2))])?;
                self.push(&list, record, "this.relations.push")?;
            }
            12 => return self.lookup(r, arg(0)),
            13 | 14 | 15 => {
                let mut ids = self.iter(&arg(0), "t")?;
                while let Some(id) = self.next(&mut ids)? {
                    if c.method.0 == 15 {
                        let map = self.read(r, "classes")?;
                        let mut def = self.map_get(&map, &id, false, "this.classes.get")?;
                        if matches!(def, Atom::Undefined) {
                            let styles = self.array(vec![])?;
                            let text_styles = self.array(vec![])?;
                            def = self.record(vec![
                                ("id", id.clone()),
                                ("styles", styles),
                                ("textStyles", text_styles),
                            ])?;
                            let map = self.read(r, "classes")?;
                            self.map_set(&map, id.clone(), def.clone(), "this.classes.set")?;
                        }
                        let style = arg(1);
                        let mut values = self.iter(&style, "n")?;
                        while let Some(s) = self.next(&mut values)? {
                            let color = self.text("color")?;
                            if self.has_value(&s, &color, "a.includes")? {
                                let target = self.read(&def, "textStyles")?;
                                self.native(&target, "push", "styleClass.textStyles.push", 1)?;
                                let units = self.string_units(&s, "a.replace")?;
                                let mut changed = units.clone();
                                if let Some(at) =
                                    units.windows(4).position(|w| w == [102, 105, 108, 108])
                                {
                                    changed.splice(at..at + 4, "bgFill".encode_utf16());
                                }
                                let changed = self.intern(JsString(changed))?;
                                self.push(&target, changed, "styleClass.textStyles.push")?;
                            }
                            let target = self.read(&def, "styles")?;
                            self.push(&target, s, "styleClass.styles.push")?;
                        }
                        let sid = self.native(&style, "flatMap", "n.flatMap", 1)?;
                        let Node::Array { length, .. } = self.g.nodes[sid as usize] else {
                            unreachable!()
                        };
                        let mut flattened = vec![];
                        for n in 0..length {
                            if let Some(s) = self.own(sid, &n.to_string())? {
                                flattened.extend(self.split(&s, "a.split")?);
                            }
                        }
                        let split = self.array(flattened)?;
                        for mapname in ["requirements", "elements"] {
                            let map = self.read(r, mapname)?;
                            let mid = self.values(&map)?;
                            let mut n = 0;
                            loop {
                                let Node::Map { entries, .. } = &self.g.nodes[mid as usize] else {
                                    unreachable!()
                                };
                                let Some((_, record)) = entries.get(n).cloned() else {
                                    break;
                                };
                                n += 1;
                                self.tick(1)?;
                                let classes = self.read(&record, "classes")?;
                                if self.has_value(&classes, &id, "req.classes.includes")? {
                                    let target = self.read(&record, "cssStyles")?;
                                    self.helper(r, "copy", 1, vec![target, split.clone()])?;
                                }
                            }
                        }
                    } else {
                        let node = self.helper(r, "node", 12, vec![id])?;
                        if c.method.0 == 13 {
                            if !truth(self.g, &arg(1)) || !truth(self.g, &node) {
                                return Ok(Atom::Undefined);
                            }
                        } else if !truth(self.g, &node) {
                            continue;
                        }
                        let mut values = self.iter(&arg(1), "n")?;
                        while let Some(value) = self.next(&mut values)? {
                            if c.method.0 == 13 {
                                let target = self.read(&node, "cssStyles")?;
                                let comma = self.text(",")?;
                                let styles = if self.has_value(&value, &comma, "s.includes")? {
                                    let parts = self.split(&value, "s.split")?;
                                    self.array(parts)?
                                } else {
                                    self.array(vec![value])?
                                };
                                self.helper(r, "copy", 1, vec![target, styles])?;
                            } else {
                                let target = self.read(&node, "classes")?;
                                self.push(&target, value.clone(), "node.classes.push")?;
                                let map = self.read(r, "classes")?;
                                let class =
                                    self.map_get(&map, &value, false, "this.classes.get")?;
                                let styles = if matches!(class, Atom::Undefined | Atom::Null) {
                                    Atom::Undefined
                                } else {
                                    self.read(&class, "styles")?
                                };
                                if truth(self.g, &styles) {
                                    let target = self.read(&node, "cssStyles")?;
                                    self.helper(r, "copy", 1, vec![target, styles])?;
                                }
                            }
                        }
                    }
                }
            }
            16 | 17 => {
                let units = self.string_units(&arg(0), "t.replace")?;
                let changed = if c.method.0 == 16 {
                    let n = units.iter().position(|u| !space(*u)).unwrap_or(units.len());
                    units[n..].to_vec()
                } else {
                    let mut out = vec![];
                    let mut n = 0;
                    while n < units.len() {
                        out.push(units[n]);
                        if units[n] == 10 {
                            let start = n + 1;
                            if units.get(start).is_some_and(|u| space(*u)) {
                                n = start;
                                while n < units.len() && space(units[n]) {
                                    n += 1;
                                }
                                continue;
                            }
                        }
                        n += 1;
                    }
                    out
                };
                let value = self.intern(JsString(changed))?;
                self.set(
                    r,
                    if c.method.0 == 16 {
                        "accTitle"
                    } else {
                        "accDescr"
                    },
                    value,
                )?;
            }
            _ => return Err(invalid("Requirement method is unavailable").into()),
        }
        Ok(Atom::Undefined)
    }
}
