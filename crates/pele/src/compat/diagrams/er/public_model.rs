//! Opt-in native ErDb codec, Pele 458322a5 (MIT, kepano).
//!
//! The arena is authoritative. The source builder retains attribute own-property
//! order and claimed Set insertion order; private Map values alias group records.
//! `from_module` reconstructs private lookup from the current typed group vector;
//! arbitrary typed edits cannot recover an earlier, erased JS lookup history.
//! Nondefault Array length descriptors accompany the scalar length (flags0 for
//! readonly); default writable flags2 remains implicit. Constructor invocation,
//! overridden host callbacks and active lone-surrogate SVG text remain explicit
//! capabilities rather than an embedded JavaScript runtime.
use super::*;
use crate::compat::public::{
    Atom, FieldKind, FieldSchema, GraphWriter, JavaScriptErrorKind, JsString, MemberKind,
    MemberSchema, MethodCall, MethodId, MethodSchema, ModelGraph, ModelSchema, Node, Property,
    PrototypeId, PrototypeParent, PrototypeSchema, PublicDiagramModule, PublicError, RecordId,
    RecordKind, RecordSchema,
};
use crate::compat::ErrorCode;
use std::collections::{HashMap, HashSet};
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
    id: "pele/er@1",
    version: 1,
    root: Some(RecordId(1)),
    records: &[
        RecordSchema {
            id: RecordId(1),
            name: "ErDb",
            kind: RecordKind::Object,
            prototype: Some(PROTO),
            fields: &[
                f("type", FieldKind::String, false),
                f("direction", FieldKind::String, false),
                f("entities", FieldKind::Map, false),
                f("relationships", FieldKind::Array, false),
                f("classes", FieldKind::Map, false),
                f("subgraphs", FieldKind::Array, false),
                f("title", FieldKind::String, true),
                f("accTitle", FieldKind::String, true),
                f("accDescr", FieldKind::String, true),
                f("subgraphLookup", FieldKind::Map, false),
                f("claimed", FieldKind::Set, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(2),
            name: "ErEntity",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("id", FieldKind::String, false),
                f("label", FieldKind::String, false),
                f("alias", FieldKind::String, false),
                f("attributes", FieldKind::Array, false),
                f("cssClasses", FieldKind::String, false),
                f("cssStyles", FieldKind::Array, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(3),
            name: "ErAttribute",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("type", FieldKind::String, false),
                f("name", FieldKind::String, false),
                f("keys", FieldKind::Array, false),
                f("comment", FieldKind::String, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(4),
            name: "ErRelationship",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("entityA", FieldKind::String, false),
                f("roleA", FieldKind::String, false),
                f("entityB", FieldKind::String, false),
                f("relSpec", FieldKind::Record(RecordId(5)), false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(5),
            name: "RelSpec",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("cardA", FieldKind::String, false),
                f("relType", FieldKind::String, false),
                f("cardB", FieldKind::String, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(6),
            name: "ErClassDef",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("id", FieldKind::String, false),
                f("styles", FieldKind::Array, false),
                f("textStyles", FieldKind::Array, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(7),
            name: "ErSubgraph",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("id", FieldKind::String, false),
                f("nodes", FieldKind::Array, false),
                f("title", FieldKind::String, false),
                f("classes", FieldKind::Array, false),
                f("cssStyles", FieldKind::Array, false),
                f("dir", FieldKind::String, true),
                f("labelType", FieldKind::String, false),
            ],
            members: &[],
        },
    ],
    prototypes: &[PrototypeSchema {
        id: PROTO,
        name: "ErDb",
        parent: PrototypeParent::Object,
        members: &[
            member(1, "addEntity"),
            member(2, "addAttributes"),
            member(3, "addRelationship"),
            member(4, "setDirection"),
            member(5, "addCssStyles"),
            member(6, "addClass"),
            member(7, "setClass"),
            member(8, "addSubGraph"),
            member(9, "setAccTitle"),
            member(10, "setAccDescription"),
        ],
    }],
    methods: &[
        method(1, "addEntity", 1),
        method(2, "addAttributes", 2),
        method(3, "addRelationship", 4),
        method(4, "setDirection", 1),
        method(5, "addCssStyles", 2),
        method(6, "addClass", 2),
        method(7, "setClass", 2),
        method(8, "addSubGraph", 3),
        method(9, "setAccTitle", 1),
        method(10, "setAccDescription", 1),
    ],
};
pub struct ErPublicState {
    graph: ModelGraph,
    model: std::cell::OnceCell<Result<ErModel, PeleError>>,
}
fn invalid(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("er".into());
    e
}
fn limit() -> PeleError {
    PeleError::new(
        "ER native graph operation exceeds work budget",
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
        return Err(invalid("ER drawable records must be objects"));
    };
    own(g, *id, k).ok_or_else(|| invalid(&format!("ER drawable field {k} is missing")))
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
        t.well_formed()
            .ok_or_else(|| invalid("ER rendering of lone UTF-16 surrogates is not implemented"))
    }
    fn field_text(&mut self, a: &Atom, k: &str) -> Result<String, PeleError> {
        self.text(required(self.g, a, k)?)
    }
    fn optional(&mut self, a: &Atom, k: &str) -> Result<Option<String>, PeleError> {
        let Atom::Ref(id) = a else {
            return Err(invalid("ER drawable records must be objects"));
        };
        match own(self.g, *id, k) {
            None | Some(Atom::Undefined | Atom::Null) => Ok(None),
            Some(v) => self.text(v).map(Some),
        }
    }
    fn array(&mut self, a: &Atom) -> Result<Vec<Atom>, PeleError> {
        let Atom::Ref(id) = a else {
            return Err(invalid("ER drawable list must be an Array"));
        };
        let Node::Array { length, properties } = &self.g.nodes[*id as usize] else {
            return Err(invalid("ER drawable list must be an Array"));
        };
        self.work = self
            .work
            .saturating_add(*length as usize)
            .saturating_add(properties.len());
        if self.work > 1_000_000 {
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
            return Err(invalid("ER drawable lookup must be a Map"));
        };
        let Node::Map { entries, .. } = &self.g.nodes[*id as usize] else {
            return Err(invalid("ER drawable lookup must be a Map"));
        };
        self.work = self.work.saturating_add(entries.len());
        if self.work > 1_000_000 {
            return Err(limit());
        }
        Ok(entries.clone())
    }
}
fn imported(g: &ModelGraph) -> Result<ErModel, PeleError> {
    let mut i = Import {
        g,
        bytes: 0,
        work: 0,
    };
    let root = &g.root;
    let mut m = ErModel::new(None);
    m.direction = i.field_text(root, "direction")?;
    m.title = i.optional(root, "title")?;
    m.acc_title = i.optional(root, "accTitle")?;
    m.acc_descr = i.optional(root, "accDescr")?;
    let mut used = HashSet::new();
    for a in i.array(required(g, root, "subgraphs")?)? {
        let classes = i.texts(required(g, &a, "classes")?)?;
        used.extend(classes.iter().cloned());
        m.subgraphs.push(ErSubgraph {
            id: i.field_text(&a, "id")?,
            nodes: i.texts(required(g, &a, "nodes")?)?,
            title: i.field_text(&a, "title")?,
            classes,
            css_styles: i.texts(required(g, &a, "cssStyles")?)?,
            dir: i.optional(&a, "dir")?,
            label_type: String::new(),
        });
    }
    let groups: HashSet<_> = m.subgraphs.iter().map(|s| s.id.as_str()).collect();
    for (key, a) in i.map(required(g, root, "entities")?)? {
        let name = i.text(&key)?;
        // A moved entity only contributes its endpoint ID mapping. Its table,
        // classes and label fields are never read by the pinned renderer.
        if groups.contains(name.as_str()) {
            m.entities.push((
                name,
                ErEntity {
                    id: i.field_text(&a, "id")?,
                    label: String::new(),
                    alias: String::new(),
                    attributes: vec![],
                    css_classes: String::new(),
                    css_styles: vec![],
                },
            ));
            continue;
        }
        let css_classes = i.field_text(&a, "cssClasses")?;
        used.extend(css_classes.split(' ').map(str::to_owned));
        let mut attributes = vec![];
        for attr in i.array(required(g, &a, "attributes")?)? {
            attributes.push(ErAttribute {
                r#type: i.field_text(&attr, "type")?,
                name: i.field_text(&attr, "name")?,
                keys: i.texts(required(g, &attr, "keys")?)?,
                comment: i.field_text(&attr, "comment")?,
            });
        }
        m.entities.push((
            name,
            ErEntity {
                id: i.field_text(&a, "id")?,
                label: i.field_text(&a, "label")?,
                alias: i.field_text(&a, "alias")?,
                attributes,
                css_classes,
                css_styles: i.texts(required(g, &a, "cssStyles")?)?,
            },
        ));
    }
    for (key, a) in i.map(required(g, root, "classes")?)? {
        // Unreferenced class records, including cyclic extras, are inert.
        let Atom::String(id) = key else { continue };
        let Some(key) = g.strings[id as usize].well_formed() else {
            continue;
        };
        if !used.contains(&key) {
            continue;
        }
        m.classes.push((
            key,
            ErClassDef {
                id: String::new(),
                styles: i.texts(required(g, &a, "styles")?)?,
                text_styles: i.texts(required(g, &a, "textStyles")?)?,
            },
        ));
    }
    for a in i.array(required(g, root, "relationships")?)? {
        let spec = required(g, &a, "relSpec")?;
        let mut cardinality = |key| -> Result<Cardinality, PeleError> {
            match i.field_text(spec, key)?.to_ascii_uppercase().as_str() {
                "ZERO_OR_ONE" => Ok(Cardinality::ZERO_OR_ONE),
                "ZERO_OR_MORE" => Ok(Cardinality::ZERO_OR_MORE),
                "ONE_OR_MORE" => Ok(Cardinality::ONE_OR_MORE),
                "ONLY_ONE" => Ok(Cardinality::ONLY_ONE),
                "MD_PARENT" => Ok(Cardinality::MD_PARENT),
                _ => Err(invalid(
                    "ER unknown cardinality marker is not representable in the typed renderer",
                )),
            }
        };
        let card_a = cardinality("cardA")?;
        let card_b = cardinality("cardB")?;
        let rel_type = if matches!(required(g,spec,"relType")?,Atom::String(id) if g.strings[*id as usize].is("IDENTIFYING"))
        {
            Identification::IDENTIFYING
        } else {
            Identification::NON_IDENTIFYING
        };
        m.relationships.push(ErRelationship {
            entity_a: i.field_text(&a, "entityA")?,
            role_a: i.field_text(&a, "roleA")?,
            entity_b: i.field_text(&a, "entityB")?,
            rel_spec: RelSpec {
                card_a,
                rel_type,
                card_b,
            },
        });
    }
    // Class compilation duplicates styles per drawable node in the concrete
    // renderer. Count that expansion before handing it a compact aliased arena.
    // Also bound the existing typed Vec class lookup work, not just input size.
    let class_sizes: HashMap<_, _> = m
        .classes
        .iter()
        .map(|(name, def)| {
            let bytes = def
                .styles
                .iter()
                .chain(&def.text_styles)
                .fold(0usize, |n, s| {
                    n.saturating_add(s.len())
                        .saturating_add(std::mem::size_of::<String>())
                });
            (
                name.as_str(),
                (def.styles.len() + def.text_styles.len(), bytes),
            )
        })
        .collect();
    for name in m
        .subgraphs
        .iter()
        .flat_map(|s| s.classes.iter().map(String::as_str))
        .chain(
            m.entities
                .iter()
                .filter(|(name, _)| !groups.contains(name.as_str()))
                .flat_map(|(_, e)| e.css_classes.split(' ')),
        )
    {
        i.work = i.work.saturating_add(m.classes.len());
        if let Some((count, bytes)) = class_sizes.get(name) {
            i.work = i.work.saturating_add(*count);
            i.bytes = i.bytes.saturating_add(*bytes)
        }
        if i.work > 1_000_000 || i.bytes > 128 * 1024 * 1024 {
            return Err(limit());
        }
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
fn initial(m: &ErModel) -> Result<ModelGraph, PeleError> {
    let mut w = GraphWriter::default();
    let mut entities = vec![];
    for (name, e) in &m.entities {
        let mut attrs = vec![];
        for (index, a) in e.attributes.iter().enumerate() {
            let keys = texts(&mut w, &a.keys);
            let comment = text(&mut w, &a.comment);
            let mut fields = vec![
                ("type", text(&mut w, &a.r#type)),
                ("name", text(&mut w, &a.name)),
            ];
            if m.public_attribute_comment_first(name, index) {
                fields.extend([("comment", comment), ("keys", Atom::Ref(keys))])
            } else {
                fields.extend([("keys", Atom::Ref(keys)), ("comment", comment)])
            }
            attrs.push(Atom::Ref(rec(&mut w, fields)));
        }
        let attributes = arr(&mut w, attrs);
        let styles = texts(&mut w, &e.css_styles);
        let fields = vec![
            ("id", text(&mut w, &e.id)),
            ("label", text(&mut w, &e.label)),
            ("alias", text(&mut w, &e.alias)),
            ("attributes", Atom::Ref(attributes)),
            ("cssClasses", text(&mut w, &e.css_classes)),
            ("cssStyles", Atom::Ref(styles)),
        ];
        let entity = rec(&mut w, fields);
        entities.push((text(&mut w, name), Atom::Ref(entity)));
    }
    let entities = w.node(Node::Map {
        entries: entities,
        properties: vec![],
    });
    let mut relationships = vec![];
    for r in &m.relationships {
        let fields = vec![
            ("cardA", text(&mut w, &format!("{:?}", r.rel_spec.card_a))),
            (
                "relType",
                text(&mut w, &format!("{:?}", r.rel_spec.rel_type)),
            ),
            ("cardB", text(&mut w, &format!("{:?}", r.rel_spec.card_b))),
        ];
        let spec = rec(&mut w, fields);
        let fields = vec![
            ("entityA", text(&mut w, &r.entity_a)),
            ("roleA", text(&mut w, &r.role_a)),
            ("entityB", text(&mut w, &r.entity_b)),
            ("relSpec", Atom::Ref(spec)),
        ];
        relationships.push(Atom::Ref(rec(&mut w, fields)));
    }
    let relationships = arr(&mut w, relationships);
    let mut classes = vec![];
    for (key, c) in &m.classes {
        let styles = texts(&mut w, &c.styles);
        let text_styles = texts(&mut w, &c.text_styles);
        let fields = vec![
            ("id", text(&mut w, &c.id)),
            ("styles", Atom::Ref(styles)),
            ("textStyles", Atom::Ref(text_styles)),
        ];
        let id = rec(&mut w, fields);
        classes.push((text(&mut w, key), Atom::Ref(id)));
    }
    let classes = w.node(Node::Map {
        entries: classes,
        properties: vec![],
    });
    let mut groups = vec![];
    let mut lookup: Vec<(Atom, Atom)> = vec![];
    let mut lookup_positions: HashMap<String, usize> = HashMap::new();
    for s in &m.subgraphs {
        let nodes = texts(&mut w, &s.nodes);
        let classes = texts(&mut w, &s.classes);
        let styles = texts(&mut w, &s.css_styles);
        let dir = s
            .dir
            .as_deref()
            .map_or(Atom::Undefined, |v| text(&mut w, v));
        let fields = vec![
            ("id", text(&mut w, &s.id)),
            ("nodes", Atom::Ref(nodes)),
            ("title", text(&mut w, &s.title)),
            ("classes", Atom::Ref(classes)),
            ("cssStyles", Atom::Ref(styles)),
            ("dir", dir),
            ("labelType", text(&mut w, &s.label_type)),
        ];
        let group = Atom::Ref(rec(&mut w, fields));
        let key = text(&mut w, &s.id);
        if let Some(at) = lookup_positions.get(&s.id).copied() {
            lookup[at].1 = group.clone()
        } else {
            lookup_positions.insert(s.id.clone(), lookup.len());
            lookup.push((key, group.clone()))
        }
        groups.push(group);
    }
    let subgraphs = arr(&mut w, groups);
    let lookup = w.node(Node::Map {
        entries: lookup,
        properties: vec![],
    });
    let claimed = m.public_claimed().iter().map(|s| text(&mut w, s)).collect();
    let claimed = w.node(Node::Set {
        values: claimed,
        properties: vec![],
    });
    let mut properties = vec![];
    for (k, v) in [
        ("type", text(&mut w, "er")),
        ("direction", text(&mut w, &m.direction)),
        ("entities", Atom::Ref(entities)),
        ("relationships", Atom::Ref(relationships)),
        ("classes", Atom::Ref(classes)),
        ("subgraphs", Atom::Ref(subgraphs)),
    ] {
        properties.push(w.property(k, v))
    }
    for (k, v) in [
        ("title", &m.title),
        ("accTitle", &m.acc_title),
        ("accDescr", &m.acc_descr),
    ] {
        let a = v.as_deref().map_or(Atom::Undefined, |s| text(&mut w, s));
        properties.push(w.property(k, a))
    }
    properties.push(w.property("subgraphLookup", Atom::Ref(lookup)));
    properties.push(w.property("claimed", Atom::Ref(claimed)));
    let root = w.node(Node::Instance {
        prototype: PROTO,
        properties,
    });
    w.finish(Atom::Ref(root))
}
impl PublicDiagramModule for Er {
    type PublicState = ErPublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let model = std::cell::OnceCell::new();
        Ok(ErPublicState { graph, model })
    }
    fn create_public(&self, m: &ErModel) -> Result<ErPublicState, PeleError> {
        let graph = initial(m)?;
        let model = std::cell::OnceCell::new();
        Ok(ErPublicState { graph, model })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, s: &ErPublicState) -> Result<ModelGraph, PeleError> {
        Ok(s.graph.clone())
    }
    fn apply_public(&self, s: &mut ErPublicState, g: ModelGraph) -> Result<(), PeleError> {
        g.validate()?;
        s.model.take();
        s.graph = g;
        Ok(())
    }
    fn invoke_public(
        &self,
        s: &mut ErPublicState,
        c: MethodCall,
        _: &Object,
        _: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        let result = Context::new(&mut s.graph).invoke(c);
        s.model.take();
        result
    }
    fn render_public(
        &self,
        s: &mut ErPublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render(
            s.model
                .get_or_init(|| imported(&s.graph))
                .as_ref()
                .map_err(Clone::clone)?,
            c,
            o,
            h,
        )
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
    sets: HashMap<u32, HashSet<Key>>,
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
            sets: HashMap::new(),
            work: 0,
            refs,
            units,
        }
    }
    fn tick(&mut self, n: usize) -> Result<(), PublicError> {
        self.work = self.work.saturating_add(n);
        if self.work > 1_000_000 {
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
    fn own(&mut self, id: u32, k: &str) -> Result<Option<Atom>, PublicError> {
        self.tick(self.g.nodes[id as usize].properties().len())?;
        Ok(own(self.g, id, k).cloned())
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
                    return Err(invalid("ER host property receiver dispatch is unavailable").into())
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
        self.tick(self.g.nodes[*id as usize].properties().len())?;
        let at = self.g.nodes[*id as usize]
            .properties()
            .iter()
            .position(|p| self.g.strings[p.key as usize].is(k));
        if let Some(at) = at {
            if self.g.nodes[*id as usize].properties()[at].flags & 2 == 0 {
                return Err(type_error(&format!(
                    "Cannot assign to read only property '{k}' of object '#<Object>'"
                )));
            }
            self.g.nodes[*id as usize].properties_mut()[at].value = v;
        } else {
            self.room(1)?;
            let Atom::String(key) = self.text(k)? else {
                unreachable!()
            };
            self.g.nodes[*id as usize].properties_mut().push(Property {
                key,
                value: v,
                flags: 7,
            });
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
                "ER overridden {expr} callback dispatch is unavailable"
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
    fn set_index(&mut self, id: u32) -> Result<(), PublicError> {
        if !self.sets.contains_key(&id) {
            let Node::Set { values, .. } = &self.g.nodes[id as usize] else {
                unreachable!()
            };
            let values = values.clone();
            self.tick(values.len())?;
            self.sets
                .insert(id, values.iter().map(|a| self.key(a)).collect());
        }
        Ok(())
    }
    fn set_has(&mut self, a: &Atom, k: &Atom) -> Result<bool, PublicError> {
        let id = self.native(a, "has", "this.claimed.has", 2)?;
        self.set_index(id)?;
        Ok(self.sets[&id].contains(&self.key(k)))
    }
    fn set_add(&mut self, a: &Atom, k: Atom) -> Result<(), PublicError> {
        let id = self.native(a, "add", "this.claimed.add", 2)?;
        self.set_index(id)?;
        let key = self.key(&k);
        if !self.sets[&id].contains(&key) {
            self.room(1)?;
            self.sets.get_mut(&id).unwrap().insert(key);
            let Node::Set { values, .. } = &mut self.g.nodes[id as usize] else {
                unreachable!()
            };
            values.push(if matches!(k,Atom::Number(n) if n==0.) {
                Atom::Number(0.)
            } else {
                k
            });
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
        if let Node::Array {
            length: len,
            properties,
        } = &mut self.g.nodes[id as usize]
        {
            *len = length + 1;
            for p in properties {
                if self.g.strings[p.key as usize].is("length") {
                    p.value = Atom::Number(*len as f64)
                }
            }
        }
        Ok(())
    }
    fn iter(&mut self, a: &Atom) -> Result<Iter, PublicError> {
        match a {
            Atom::Ref(id) => match self.g.nodes[*id as usize] {
                Node::Array { .. } => Ok(Iter::Array(*id, 0)),
                Node::Set { .. } => Ok(Iter::Set(*id, 0)),
                Node::Map { .. } => Ok(Iter::Map(*id, 0)),
                _ => Err(type_error("value is not iterable")),
            },
            Atom::String(id) => Ok(Iter::String(*id, 0)),
            _ => Err(type_error("value is not iterable")),
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
            self.read(a, property)?;
            Err(type_error(&format!("{expr} is not a function")))
        }
    }
    fn includes_color(&mut self, a: &Atom) -> Result<bool, PublicError> {
        if let Atom::String(id) = a {
            self.tick(self.g.strings[*id as usize].0.len())?;
            return Ok(self.g.strings[*id as usize]
                .0
                .windows(5)
                .any(|w| w == [99, 111, 108, 111, 114]));
        }
        let id = self.native(a, "includes", "style.includes", 1)?;
        let Node::Array { length, .. } = self.g.nodes[id as usize] else {
            unreachable!()
        };
        for n in 0..length {
            self.tick(1)?;
            if matches!(self.read(a,&n.to_string())?,Atom::String(id) if self.g.strings[id as usize].is("color"))
            {
                return Ok(true);
            }
        }
        Ok(false)
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
            _ => Err(invalid("ER object primitive-conversion dispatch is unavailable").into()),
        }
    }
    fn helper_entity(&mut self, r: &Atom, name: Atom) -> Result<Atom, PublicError> {
        let f = self.read(r, "addEntity")?;
        let native = matches!(r,Atom::Ref(id) if matches!(self.g.nodes[*id as usize],Node::Instance{prototype:PROTO,..}));
        if matches!(f, Atom::BuiltinFunction(1))
            || matches!(f, Atom::Undefined)
                && native
                && self
                    .own(if let Atom::Ref(id) = r { *id } else { 0 }, "addEntity")?
                    .is_none()
        {
            self.entity(r, name, Atom::Undefined)
        } else if matches!(f, Atom::HostReference(_) | Atom::BuiltinFunction(_)) {
            Err(invalid("ER overridden this.addEntity callback dispatch is unavailable").into())
        } else {
            Err(type_error("this.addEntity is not a function"))
        }
    }
    fn entity(&mut self, r: &Atom, name: Atom, alias: Atom) -> Result<Atom, PublicError> {
        let alias = if matches!(alias, Atom::Undefined) {
            self.text("")?
        } else {
            alias
        };
        let map = self.read(r, "entities")?;
        let found = self.map_get(&map, &name, false, "this.entities.get")?;
        if matches!(found, Atom::Undefined) {
            let mut id = "entity-".encode_utf16().collect::<Vec<_>>();
            id.extend(self.coercion(&name)?);
            id.push(45);
            let size = self.read(&map, "size")?;
            id.extend(self.coercion(&size)?);
            let id = self.intern(JsString(id))?;
            let attributes = self.array(vec![])?;
            let styles = self.array(vec![])?;
            let default = self.text("default")?;
            let e = self.record(vec![
                ("id", id),
                ("label", name.clone()),
                ("alias", alias),
                ("attributes", attributes),
                ("cssClasses", default),
                ("cssStyles", styles),
            ])?;
            let map = self.read(r, "entities")?;
            self.map_set(&map, name, e.clone(), "this.entities.set")?;
            Ok(e)
        } else {
            let old = self.read(&found, "alias")?;
            if !truth(self.g, &old) && truth(self.g, &alias) {
                self.set(&found, "alias", alias)?
            }
            Ok(found)
        }
    }
    fn invoke(&mut self, c: MethodCall) -> Result<Atom, PublicError> {
        let arg = |n| c.arguments.get(n).cloned().unwrap_or(Atom::Undefined);
        let r = &c.receiver;
        match c.method.0 {
            1 => self.entity(r, arg(0), arg(1)),
            2 => {
                let e = self.helper_entity(r, arg(0))?;
                let attrs = arg(1);
                let length = self.read(&attrs, "length")?;
                let n = match length {
                    Atom::Number(n) => n,
                    Atom::Undefined => f64::NAN,
                    _ => {
                        return Err(invalid(
                            "ER nonnumeric attribute length coercion is unavailable",
                        )
                        .into())
                    }
                };
                if n > 1_000_000. {
                    return Err(limit().into());
                }
                let mut j = n - 1.;
                while j >= 0. {
                    self.tick(1)?;
                    let a = self.read(&attrs, &crate::compat::svg::js_num(j))?;
                    let keys = self.read(&a, "keys")?;
                    if matches!(keys, Atom::Undefined | Atom::Null) {
                        let keys = self.array(vec![])?;
                        self.set(&a, "keys", keys)?
                    }
                    let comment = self.read(&a, "comment")?;
                    if matches!(comment, Atom::Undefined | Atom::Null) {
                        let empty = self.text("")?;
                        self.set(&a, "comment", empty)?
                    }
                    let list = self.read(&e, "attributes")?;
                    self.push(&list, a, "entity.attributes.push")?;
                    j -= 1.;
                }
                Ok(Atom::Undefined)
            }
            3 => {
                let mut ends = vec![];
                for name in [arg(0), arg(2)] {
                    let lookup = self.read(r, "subgraphLookup")?;
                    let exists = self.map_get(&lookup, &name, true, "this.subgraphLookup.has")?;
                    ends.push(if truth(self.g, &exists) {
                        name
                    } else {
                        let e = self.helper_entity(r, name)?;
                        self.read(&e, "id")?
                    });
                }
                let a = self.record(vec![
                    ("entityA", ends[0].clone()),
                    ("roleA", arg(1)),
                    ("entityB", ends[1].clone()),
                    ("relSpec", arg(3)),
                ])?;
                let list = self.read(r, "relationships")?;
                self.push(&list, a, "this.relationships.push")?;
                Ok(Atom::Undefined)
            }
            4 => {
                self.set(r, "direction", arg(0))?;
                Ok(Atom::Undefined)
            }
            5..=7 => {
                let ids = arg(0);
                let styles = arg(1);
                if c.method.0 == 5 && !truth(self.g, &styles) {
                    return Ok(Atom::Undefined);
                }
                let mut iter = self.iter(&ids)?;
                while let Some(id) = self.next(&mut iter)? {
                    match c.method.0 {
                        5 | 7 => {
                            for (lookup, key) in [
                                (
                                    "entities",
                                    if c.method.0 == 5 {
                                        "cssStyles"
                                    } else {
                                        "cssClasses"
                                    },
                                ),
                                (
                                    "subgraphLookup",
                                    if c.method.0 == 5 {
                                        "cssStyles"
                                    } else {
                                        "classes"
                                    },
                                ),
                            ] {
                                let map = self.read(r, lookup)?;
                                let target =
                                    self.map_get(&map, &id, false, &format!("this.{lookup}.get"))?;
                                if !truth(self.g, &target) {
                                    continue;
                                }
                                let mut values = self.iter(&styles)?;
                                while let Some(value) = self.next(&mut values)? {
                                    if c.method.0 == 7 && lookup == "entities" {
                                        let old = self.read(&target, key)?;
                                        let mut units = self.coercion(&old)?;
                                        units.push(32);
                                        units.extend(self.coercion(&value)?);
                                        let a = self.intern(JsString(units))?;
                                        self.set(&target, key, a)?
                                    } else {
                                        let a = self.read(&target, key)?;
                                        self.push(&a, value, "record.styles.push")?
                                    }
                                }
                            }
                        }
                        6 => {
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
                                self.map_set(&map, id, def.clone(), "this.classes.set")?
                            }
                            if !truth(self.g, &styles) {
                                continue;
                            }
                            let mut values = self.iter(&styles)?;
                            while let Some(style) = self.next(&mut values)? {
                                if self.includes_color(&style)? {
                                    // Resolve the member before evaluating the replacement
                                    // argument, as the actual native call expression does.
                                    let list = self.read(&def, "textStyles")?;
                                    self.native(&list, "push", "def.textStyles.push", 1)?;
                                    let units = self.string_units(&style, "style.replace")?;
                                    let mut replaced = units.clone();
                                    if let Some(at) =
                                        units.windows(4).position(|w| w == [102, 105, 108, 108])
                                    {
                                        replaced.splice(at..at + 4, "bgFill".encode_utf16());
                                    }
                                    let value = self.intern(JsString(replaced))?;
                                    self.push(&list, value, "def.textStyles.push")?
                                }
                                let list = self.read(&def, "styles")?;
                                self.push(&list, style, "def.styles.push")?
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                Ok(Atom::Undefined)
            }
            8 => self.group(r, arg(0), arg(1), arg(2)),
            9 | 10 => {
                let units = self.string_units(&arg(0), "text.replace")?;
                let changed = if c.method.0 == 9 {
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
                                    n += 1
                                }
                                continue;
                            }
                        }
                        n += 1;
                    }
                    v
                };
                let a = self.intern(JsString(changed))?;
                self.set(
                    r,
                    if c.method.0 == 9 {
                        "accTitle"
                    } else {
                        "accDescr"
                    },
                    a,
                )?;
                Ok(Atom::Undefined)
            }
            _ => Err(invalid("ER method is unavailable").into()),
        }
    }
    fn group(
        &mut self,
        r: &Atom,
        id_text: Atom,
        list: Atom,
        title_text: Atom,
    ) -> Result<Atom, PublicError> {
        let value = self.read(&id_text, "text")?;
        let units = self.string_units(&value, "idText.text.trim")?;
        let id = self.intern(JsString(trimmed(&units).to_vec()))?;
        let list_id = self.native(&list, "flat", "list.flat", 1)?;
        let Node::Array { length, .. } = self.g.nodes[list_id as usize] else {
            unreachable!()
        };
        self.tick(length as usize)?;
        let mut flat = vec![];
        for n in 0..length {
            if let Some(a) = self.own(list_id, &n.to_string())? {
                if let Atom::Ref(sub) = a {
                    if let Node::Array { length, .. } = self.g.nodes[sub as usize] {
                        self.tick(length as usize)?;
                        for j in 0..length {
                            if let Some(v) = self.own(sub, &j.to_string())? {
                                flat.push(v)
                            }
                        }
                        continue;
                    }
                }
                flat.push(a)
            }
            if flat.len() > 1_000_000 {
                return Err(limit().into());
            }
        }
        let mut seen = HashSet::new();
        let mut nodes = vec![];
        let mut dir = Atom::Undefined;
        for item in flat {
            if !matches!(item, Atom::String(_)) {
                if !matches!(item, Atom::Null | Atom::Undefined) {
                    let stmt = self.read(&item, "stmt")?;
                    if matches!(stmt,Atom::String(id) if self.g.strings[id as usize].is("dir")) {
                        dir = self.read(&item, "value")?
                    }
                }
                continue;
            }
            let units = self.string_units(&item, "item.trim")?;
            let trim = trimmed(&units);
            if trim.is_empty() || !seen.insert(trim.to_vec()) {
                continue;
            }
            let claimed = self.read(r, "claimed")?;
            if !self.set_has(&claimed, &item)? {
                nodes.push(item)
            }
        }
        for node in &nodes {
            let claimed = self.read(r, "claimed")?;
            self.set_add(&claimed, node.clone())?
        }
        let title = self.read(&title_text, "text")?;
        let title = if truth(self.g, &title) {
            title
        } else {
            self.text("")?
        };
        let units = self.string_units(&title, "(titleText.text || '').trim")?;
        let title = self.intern(JsString(trimmed(&units).to_vec()))?;
        let ty = self.read(&title_text, "type")?;
        let label_type = if matches!(ty,Atom::String(id) if ["markdown","text","string"].iter().any(|s|self.g.strings[id as usize].is(s)))
        {
            ty
        } else {
            self.text("markdown")?
        };
        let nodes = self.array(nodes)?;
        let classes = self.array(vec![])?;
        let styles = self.array(vec![])?;
        let group = self.record(vec![
            ("id", id.clone()),
            ("nodes", nodes),
            ("title", title),
            ("classes", classes),
            ("cssStyles", styles),
            ("dir", dir),
            ("labelType", label_type),
        ])?;
        let groups = self.read(r, "subgraphs")?;
        self.push(&groups, group.clone(), "this.subgraphs.push")?;
        let lookup = self.read(r, "subgraphLookup")?;
        self.map_set(&lookup, id.clone(), group, "this.subgraphLookup.set")?;
        Ok(id)
    }
}
