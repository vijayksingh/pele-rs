//! Opt-in mutable Sankey data codec for Pele 458322a5 (MIT, kepano).
//!
//! The arena owns identity: node array entries, link endpoints and the observable
//! `lookup` Map share object IDs. Direct edits do not refresh that Map or
//! renumber nodes. The existing Sankey renderer consumes a concrete typed
//! projection imported at the edit boundary; rendering never reparses source.
use super::{Sankey, SankeyLink, SankeyModel, SankeyNode};
use crate::compat::{
    public::{
        Atom, FieldKind, FieldSchema, GraphWriter, JavaScriptErrorKind, JsString, MemberKind,
        MemberSchema, MethodCall, MethodId, MethodSchema, ModelGraph, ModelSchema, Node, Property,
        PrototypeId, PrototypeParent, PrototypeSchema, PublicDiagramModule, PublicError, RecordId,
        RecordKind, RecordSchema,
    },
    DiagramModule, DiagramType, ErrorCode, Object, PeleError, RenderResources, RenderSettings,
    Rendered,
};
use std::collections::HashMap;

const SANKEY_PROTO: PrototypeId = PrototypeId(1);
// Constructor call/construction is an explicit separate capability. These are
// every ordinary builder method from the pinned returned instance prototype.
static SCHEMA: ModelSchema = ModelSchema {
    id: "pele/sankey@1",
    version: 1,
    root: Some(RecordId(1)),
    records: &[
        RecordSchema {
            id: RecordId(1),
            name: "SankeyDb",
            kind: RecordKind::Object,
            prototype: Some(SANKEY_PROTO),
            fields: &[
                FieldSchema {
                    name: "type",
                    kind: FieldKind::String,
                    optional: false,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "nodes",
                    kind: FieldKind::Array,
                    optional: false,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "links",
                    kind: FieldKind::Array,
                    optional: false,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "title",
                    kind: FieldKind::String,
                    optional: false,
                    undefined: true,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "accTitle",
                    kind: FieldKind::String,
                    optional: false,
                    undefined: true,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "accDescr",
                    kind: FieldKind::String,
                    optional: false,
                    undefined: true,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "lookup",
                    kind: FieldKind::Map,
                    optional: false,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(2),
            name: "SankeyNode",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                FieldSchema {
                    name: "id",
                    kind: FieldKind::String,
                    optional: false,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "index",
                    kind: FieldKind::Number,
                    optional: false,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(3),
            name: "SankeyLink",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                FieldSchema {
                    name: "source",
                    kind: FieldKind::Record(RecordId(2)),
                    optional: false,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "target",
                    kind: FieldKind::Record(RecordId(2)),
                    optional: false,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "value",
                    kind: FieldKind::Number,
                    optional: false,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
            ],
            members: &[],
        },
    ],
    prototypes: &[PrototypeSchema {
        id: SANKEY_PROTO,
        name: "SankeyDb",
        parent: PrototypeParent::Object,
        members: &[
            MemberSchema {
                name: "findOrCreateNode",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(1),
                },
            },
            MemberSchema {
                name: "addLink",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(2),
                },
            },
        ],
    }],
    methods: &[
        MethodSchema {
            id: MethodId(1),
            name: "findOrCreateNode",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(2),
            name: "addLink",
            arity: 3,
        },
    ],
};
pub struct SankeyPublicState {
    graph: ModelGraph,
    model: Result<SankeyModel, PeleError>,
}
fn invalid(message: &str) -> PeleError {
    let mut error = PeleError::new(message, ErrorCode::Semantic);
    error.diagram_type = Some("sankey".into());
    error
}
fn object_id(atom: &Atom) -> Result<u32, PeleError> {
    match atom {
        Atom::Ref(id) => Ok(*id),
        _ => Err(invalid("Sankey node and link records must be objects")),
    }
}
fn field<'a>(graph: &'a ModelGraph, id: u32, name: &str) -> Option<&'a Atom> {
    graph
        .nodes
        .get(id as usize)?
        .properties()
        .iter()
        .find_map(|p| graph.strings[p.key as usize].is(name).then_some(&p.value))
}
fn string(graph: &ModelGraph, atom: &Atom) -> Result<String, PeleError> {
    graph
        .text(atom)?
        .well_formed()
        .ok_or_else(|| invalid("Sankey rendering of lone UTF-16 surrogates is not implemented"))
}
fn optional_text(graph: &ModelGraph, key: &str) -> Result<Option<String>, PeleError> {
    match graph.field(key)? {
        None | Some(Atom::Undefined | Atom::Null) => Ok(None),
        Some(value) => string(graph, value).map(Some),
    }
}
// Canonical own array indexes only. Holes are retained in the arena, but a
// renderer read of a missing node/link record fails instead of allocating from
// an untrusted sparse length or silently shortening the array.
fn array<'a>(graph: &'a ModelGraph, name: &str) -> Result<Vec<&'a Atom>, PeleError> {
    let Some(Atom::Ref(id)) = graph.field(name)? else {
        return Err(invalid(&format!("Sankey {name} must be an Array")));
    };
    let Some(Node::Array { length, properties }) = graph.nodes.get(*id as usize) else {
        return Err(invalid(&format!("Sankey {name} must be an Array")));
    };
    let mut indexed = HashMap::new();
    for p in properties {
        let key = &graph.strings[p.key as usize].0;
        if key.is_empty() || key.len() > 10 || (key.len() > 1 && key[0] == 48) {
            continue;
        }
        let mut index = 0u64;
        let mut canonical = true;
        for &unit in key {
            if !(48..=57).contains(&unit) {
                canonical = false;
                break;
            }
            index = index * 10 + u64::from(unit - 48);
        }
        if canonical && index < u64::from(*length) {
            indexed.entry(index as u32).or_insert(&p.value);
        }
    }
    if indexed.len() != *length as usize {
        return Err(invalid(&format!(
            "Sankey {name} contains a missing array record"
        )));
    }
    Ok((0..*length).map(|i| indexed[&i]).collect())
}
fn index(graph: &ModelGraph, id: u32, length: usize) -> Result<usize, PeleError> {
    match field(graph, id, "index") {
        Some(Atom::Number(n)) if n.is_finite() && *n >= 0. && n.fract() == 0. => {
            if *n < length as f64 {
                Ok(*n as usize)
            } else {
                Err(invalid(
                    "Sankey link endpoint index is outside the node array",
                ))
            }
        }
        _ => Err(invalid(
            "Sankey link endpoint index must be a finite array index",
        )),
    }
}
fn imported(graph: &ModelGraph) -> Result<SankeyModel, PeleError> {
    let array_nodes = array(graph, "nodes")?;
    let array_links = array(graph, "links")?;
    let mut nodes = Vec::with_capacity(array_nodes.len());
    let mut labels: HashMap<u32, String> = HashMap::new();
    let mut indexes: HashMap<u32, usize> = HashMap::new();
    // A compact graph can repeat one long label in arbitrarily many array
    // slots. Bound concrete owned-string expansion before cloning it.
    let mut text_bytes = 0usize;
    let reserve_text = |used: &mut usize, bytes: usize| -> Result<(), PeleError> {
        *used = used.saturating_add(bytes);
        if *used > 128 * 1024 * 1024 {
            return Err(PeleError::new(
                "Sankey imported text exceeds native allocation budget",
                ErrorCode::Limit,
            ));
        }
        Ok(())
    };
    for value in array_nodes {
        let id = object_id(value)?;
        let label = if let Some(label) = labels.get(&id) {
            reserve_text(&mut text_bytes, label.len())?;
            label.clone()
        } else {
            let value =
                field(graph, id, "id").ok_or_else(|| invalid("Sankey node ID must be a string"))?;
            let label = string(graph, value)?;
            reserve_text(&mut text_bytes, label.len().saturating_mul(2))?;
            labels.insert(id, label.clone());
            label
        };
        // Array node.index is not read by Pele's layout or labels. It may be
        // stale, absent or nonfinite until that record is used as an endpoint.
        // Keep the original value in the arena; this unused typed slot is inert.
        let unused_index = match field(graph, id, "index") {
            Some(Atom::Number(n))
                if n.is_finite() && *n >= 0. && n.fract() == 0. && *n <= usize::MAX as f64 =>
            {
                *n as usize
            }
            _ => 0,
        };
        nodes.push(SankeyNode {
            id: label,
            index: unused_index,
        });
    }
    let mut links = Vec::with_capacity(array_links.len());
    for value in array_links {
        let id = object_id(value)?;
        let mut endpoint = |name| -> Result<SankeyNode, PeleError> {
            let value = field(graph, id, name)
                .ok_or_else(|| invalid("Sankey links require source and target objects"))?;
            let object = object_id(value)?;
            let index = if let Some(index) = indexes.get(&object) {
                *index
            } else {
                let index = index(graph, object, nodes.len())?;
                indexes.insert(object, index);
                index
            };
            // The link's endpoint.id is not read by the renderer. An endpoint
            // may remain detached after nodes[i] is replaced; its own index
            // still selects the current array slot, exactly as in Pele.
            Ok(SankeyNode {
                id: String::new(),
                index,
            })
        };
        let source = endpoint("source")?;
        let target = endpoint("target")?;
        let value = match field(graph, id, "value") {
            Some(Atom::Number(n)) => *n,
            // Number.isFinite rejects these before layout ever adds them.
            _ => f64::NAN,
        };
        links.push(SankeyLink {
            source,
            target,
            value,
        });
    }
    Ok(SankeyModel {
        diagram_type: DiagramType::Sankey,
        nodes,
        links,
        title: optional_text(graph, "title")?,
        acc_title: optional_text(graph, "accTitle")?,
        acc_descr: optional_text(graph, "accDescr")?,
    })
}
fn node(out: &mut GraphWriter, node: &SankeyNode) -> u32 {
    let label = out.string(node.id.as_str());
    let properties = vec![
        out.property("id", Atom::String(label)),
        out.property("index", Atom::Number(node.index as f64)),
    ];
    out.node(Node::Object { properties })
}
fn array_node(out: &mut GraphWriter, entries: impl Iterator<Item = Atom>) -> u32 {
    let properties: Vec<_> = entries
        .enumerate()
        .map(|(i, value)| out.property(&i.to_string(), value))
        .collect();
    out.node(Node::Array {
        length: properties.len() as u32,
        properties,
    })
}
fn initial_graph(model: &SankeyModel) -> Result<ModelGraph, PeleError> {
    let mut out = GraphWriter::default();
    let node_objects: Vec<_> = model.nodes.iter().map(|n| node(&mut out, n)).collect();
    let mut original_nodes = HashMap::new();
    for (record, object) in model.nodes.iter().zip(&node_objects) {
        original_nodes
            .entry((record.index, record.id.as_str()))
            .or_insert(*object);
    }
    let nodes = array_node(&mut out, node_objects.iter().copied().map(Atom::Ref));
    let mut link_objects = Vec::with_capacity(model.links.len());
    for link in &model.links {
        let mut endpoint = |endpoint: &SankeyNode| {
            // The source parser's records are owned value copies in the
            // immutable API. Recover its guaranteed index/record alias
            // once here; all subsequent edits use arena IDs exclusively.
            original_nodes
                .get(&(endpoint.index, endpoint.id.as_str()))
                .copied()
                .unwrap_or_else(|| node(&mut out, endpoint))
        };
        let source = endpoint(&link.source);
        let target = endpoint(&link.target);
        let properties = vec![
            out.property("source", Atom::Ref(source)),
            out.property("target", Atom::Ref(target)),
            out.property("value", Atom::Number(link.value)),
        ];
        link_objects.push(out.node(Node::Object { properties }));
    }
    let links = array_node(&mut out, link_objects.into_iter().map(Atom::Ref));
    let entries = model
        .nodes
        .iter()
        .zip(&node_objects)
        .map(|(n, id)| (Atom::String(out.string(n.id.as_str())), Atom::Ref(*id)))
        .collect();
    let lookup = out.node(Node::Map {
        entries,
        properties: vec![],
    });
    let kind = out.string("sankey");
    let mut properties = vec![
        out.property("type", Atom::String(kind)),
        out.property("nodes", Atom::Ref(nodes)),
        out.property("links", Atom::Ref(links)),
    ];
    for (key, text) in [
        ("title", &model.title),
        ("accTitle", &model.acc_title),
        ("accDescr", &model.acc_descr),
    ] {
        let value = text.as_ref().map_or(Atom::Undefined, |text| {
            Atom::String(out.string(text.as_str()))
        });
        properties.push(out.property(key, value));
    }
    properties.push(out.property("lookup", Atom::Ref(lookup)));
    let root = out.node(Node::Instance {
        prototype: SANKEY_PROTO,
        properties,
    });
    out.finish(Atom::Ref(root))
}
impl PublicDiagramModule for Sankey {
    type PublicState = SankeyPublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn invoke_public(
        &self,
        s: &mut Self::PublicState,
        call: MethodCall,
        _: &Object,
        _: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        let result = invoke_graph(&mut s.graph, call);
        // lookup.set precedes nodes.push: a throwing call can leave a cached
        // node absent from the array. Never rebuild that cache or roll it back.
        s.model = imported(&s.graph);
        result
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let model = imported(&graph);
        Ok(SankeyPublicState { graph, model })
    }
    fn create_public(&self, model: &SankeyModel) -> Result<Self::PublicState, PeleError> {
        let graph = initial_graph(model)?;
        let model = imported(&graph);
        Ok(SankeyPublicState { graph, model })
    }
    fn create_public_parsed(
        &self,
        parsed: crate::compat::public::ParsedPublicModel<SankeyModel>,
    ) -> Result<Self::PublicState, PeleError> {
        if !parsed.is_from_module::<Self>() || parsed.is_prepared() {
            return parsed.create_legacy(self);
        }
        let mut model = parsed.into_model();
        let graph = initial_graph(&model)?;
        // The canonical parser has unique labels, dense nodes[i].index == i,
        // and endpoint copies drawn from those exact nodes. Graph admission
        // bounds indexes to an exactly representable f64 range. Preserve the
        // importer’s per-source-node text budget and latent error boundary.
        let mut text_bytes = 0usize;
        let mut error = None;
        for node in &model.nodes {
            text_bytes = text_bytes.saturating_add(node.id.len().saturating_mul(2));
            if text_bytes > 128 * 1024 * 1024 {
                error = Some(PeleError::new(
                    "Sankey imported text exceeds native allocation budget",
                    ErrorCode::Limit,
                ));
                break;
            }
        }
        let model = match error {
            Some(error) => Err(error),
            None => {
                // These labels are deliberately absent from the render
                // projection. Drop their allocations, not only their lengths.
                for link in &mut model.links {
                    drop(std::mem::take(&mut link.source.id));
                    drop(std::mem::take(&mut link.target.id));
                }
                Ok(model)
            }
        };
        Ok(SankeyPublicState { graph, model })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, state: &Self::PublicState) -> Result<ModelGraph, PeleError> {
        Ok(state.graph.clone())
    }
    fn apply_public(
        &self,
        state: &mut Self::PublicState,
        graph: ModelGraph,
    ) -> Result<(), PeleError> {
        graph.validate()?;
        let model = imported(&graph);
        state.graph = graph;
        state.model = model;
        Ok(())
    }
    fn render_public(
        &self,
        state: &mut Self::PublicState,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render(
            state.model.as_ref().map_err(Clone::clone)?,
            config,
            options,
            host,
        )
    }
}

fn type_error(message: &str) -> PublicError {
    PublicError::JavaScript {
        name: JavaScriptErrorKind::TypeError,
        message: JsString::from(message),
    }
}
fn intern(g: &mut ModelGraph, s: &str) -> u32 {
    if let Some(id) = g.strings.iter().position(|x| x.is(s)) {
        return id as u32;
    }
    let id = g.strings.len() as u32;
    g.strings.push(JsString::from(s));
    id
}
fn read(g: &ModelGraph, receiver: &Atom, key: &str) -> Result<Atom, PublicError> {
    match receiver {
        Atom::Null | Atom::Undefined => Err(type_error(&format!(
            "Cannot read properties of {} (reading '{key}')",
            if matches!(receiver, Atom::Null) {
                "null"
            } else {
                "undefined"
            }
        ))),
        Atom::Ref(id) => {
            if key == "length" {
                if let Node::Array { length, .. } = &g.nodes[*id as usize] {
                    return Ok(Atom::Number(f64::from(*length)));
                }
            }
            Ok(field(g, *id, key).cloned().unwrap_or(Atom::Undefined))
        }
        Atom::String(id) if key == "length" => {
            Ok(Atom::Number(g.strings[*id as usize].0.len() as f64))
        }
        _ => Ok(Atom::Undefined),
    }
}
fn same_key(g: &ModelGraph, a: &Atom, b: &Atom) -> bool {
    match (a, b) {
        (Atom::Undefined, Atom::Undefined) | (Atom::Null, Atom::Null) => true,
        (Atom::Bool(a), Atom::Bool(b)) => a == b,
        (Atom::Number(a), Atom::Number(b)) => a == b || (a.is_nan() && b.is_nan()),
        (Atom::String(a), Atom::String(b)) => g.strings[*a as usize] == g.strings[*b as usize],
        (Atom::Ref(a), Atom::Ref(b))
        | (Atom::BuiltinFunction(a), Atom::BuiltinFunction(b))
        | (Atom::HostReference(a), Atom::HostReference(b)) => a == b,
        _ => false,
    }
}
/// Only the real native Map/Array prototype operation is implicit. A supplied
/// function needs an explicit host/native callback dispatch capability.
fn collection(
    g: &ModelGraph,
    value: &Atom,
    property: &str,
    expression: &str,
    map: bool,
) -> Result<u32, PublicError> {
    let callable = read(g, value, property)?;
    if matches!(callable, Atom::HostReference(_) | Atom::BuiltinFunction(_)) {
        return Err(invalid(&format!(
            "Sankey overridden {expression} callback dispatch is unavailable"
        ))
        .into());
    }
    let id = match value {
        Atom::Ref(id) => *id,
        _ => return Err(type_error(&format!("{expression} is not a function"))),
    };
    let native = if map {
        matches!(g.nodes[id as usize], Node::Map { .. })
    } else {
        matches!(g.nodes[id as usize], Node::Array { .. })
    };
    if !native || field(g, id, property).is_some() {
        return Err(type_error(&format!("{expression} is not a function")));
    }
    Ok(id)
}
fn room(g: &ModelGraph, nodes: usize, keys: &[&str], references: usize) -> Result<(), PublicError> {
    let missing = keys
        .iter()
        .filter(|key| !g.strings.iter().any(|s| s.is(key)))
        .collect::<Vec<_>>();
    let units = g
        .strings
        .iter()
        .fold(0usize, |n, s| n.saturating_add(s.0.len()))
        .saturating_add(missing.iter().map(|s| s.len()).sum::<usize>());
    let work = g.nodes.iter().fold(0usize, |n, node| {
        n.saturating_add(node.properties().len())
            .saturating_add(match node {
                Node::Map { entries, .. } => entries.len().saturating_mul(2),
                Node::Set { values, .. } => values.len(),
                _ => 0,
            })
    });
    if g.nodes.len().saturating_add(nodes) > 1_000_000
        || g.strings.len().saturating_add(missing.len()) > 1_000_000
        || units > 16_000_000
        || work.saturating_add(references) > 8_000_000
    {
        return Err(PeleError::new(
            "Sankey method exceeds native graph work budget",
            ErrorCode::Limit,
        )
        .into());
    }
    Ok(())
}
// Nondefault own Array length descriptors live in properties alongside the
// scalar Node::Array.length. Default flags2 is implicit; readonly flags0 is not.
fn push_index(g: &ModelGraph, id: u32) -> Result<(u32, String), PublicError> {
    let Node::Array { length, properties } = &g.nodes[id as usize] else {
        unreachable!()
    };
    if properties
        .iter()
        .any(|p| g.strings[p.key as usize].is("length") && p.flags & 2 == 0)
    {
        return Err(type_error(
            "Cannot assign to read only property 'length' of object '[object Array]'",
        ));
    }
    let key = length.to_string();
    if properties
        .iter()
        .any(|p| g.strings[p.key as usize].is(&key) && p.flags & 2 == 0)
    {
        return Err(type_error(&format!(
            "Cannot assign to read only property '{key}' of object '[object Array]'"
        )));
    }
    if *length >= 1_000_000 {
        return Err(
            PeleError::new("Sankey array exceeds native work budget", ErrorCode::Limit).into(),
        );
    }
    Ok((*length, key))
}
fn push(g: &mut ModelGraph, id: u32, length: u32, key: &str, value: Atom) {
    let existing = g.nodes[id as usize]
        .properties()
        .iter()
        .position(|p| g.strings[p.key as usize].is(key));
    if let Some(at) = existing {
        g.nodes[id as usize].properties_mut()[at].value = value
    } else {
        let key = intern(g, key);
        g.nodes[id as usize].properties_mut().push(Property {
            key,
            value,
            flags: 7,
        })
    }
    if let Node::Array {
        length: len,
        properties,
    } = &mut g.nodes[id as usize]
    {
        *len = length + 1;
        for p in properties {
            if g.strings[p.key as usize].is("length") {
                p.value = Atom::Number(f64::from(*len))
            }
        }
    }
}
fn record(g: &mut ModelGraph, fields: Vec<(&str, Atom)>) -> Atom {
    let properties = fields
        .into_iter()
        .map(|(key, value)| Property {
            key: intern(g, key),
            value,
            flags: 7,
        })
        .collect();
    let id = g.nodes.len() as u32;
    g.nodes.push(Node::Object { properties });
    Atom::Ref(id)
}
fn invoke_graph(g: &mut ModelGraph, call: MethodCall) -> Result<Atom, PublicError> {
    let arg = |i| call.arguments.get(i).cloned().unwrap_or(Atom::Undefined);
    match call.method.0 {
        1 => {
            let key = arg(0);
            let lookup = read(g, &call.receiver, "lookup")?;
            let lookup_id = collection(g, &lookup, "get", "this.lookup.get", true)?;
            let found = if let Node::Map { entries, .. } = &g.nodes[lookup_id as usize] {
                entries
                    .iter()
                    .find(|(k, _)| same_key(g, k, &key))
                    .map(|(_, v)| v.clone())
                    .unwrap_or(Atom::Undefined)
            } else {
                unreachable!()
            };
            if !matches!(found, Atom::Undefined) {
                return Ok(found);
            }
            let nodes = read(g, &call.receiver, "nodes")?;
            let index = read(g, &nodes, "length")?;
            let lookup = read(g, &call.receiver, "lookup")?;
            let lookup_id = collection(g, &lookup, "set", "this.lookup.set", true)?;
            let at = if let Node::Map { entries, .. } = &g.nodes[lookup_id as usize] {
                entries.iter().position(|(k, _)| same_key(g, k, &key))
            } else {
                unreachable!()
            };
            room(g, 1, &["id", "index"], 2 + if at.is_none() { 2 } else { 0 })?;
            let node = record(g, vec![("id", key.clone()), ("index", index)]);
            if let Node::Map { entries, .. } = &mut g.nodes[lookup_id as usize] {
                if let Some(at) = at {
                    entries[at].1 = node.clone()
                } else {
                    // SameValueZero key storage canonicalizes -0, while node.id
                    // retains the exact argument, including signed zero/NaN bits.
                    let key = match key {
                        Atom::Number(n) if n == 0. => Atom::Number(0.),
                        key => key,
                    };
                    entries.push((key, node.clone()))
                }
            }
            // Deliberately after Map publication, including every throwing path.
            let nodes = read(g, &call.receiver, "nodes")?;
            let id = collection(g, &nodes, "push", "this.nodes.push", false)?;
            let (length, key) = push_index(g, id)?;
            room(g, 0, &[&key], 1)?;
            push(g, id, length, &key, node.clone());
            Ok(node)
        }
        2 => {
            let links = read(g, &call.receiver, "links")?;
            let id = collection(g, &links, "push", "this.links.push", false)?;
            let (length, key) = push_index(g, id)?;
            room(g, 1, &["source", "target", "value", &key], 4)?;
            let link = record(
                g,
                vec![("source", arg(0)), ("target", arg(1)), ("value", arg(2))],
            );
            push(g, id, length, &key, link);
            Ok(Atom::Undefined)
        }
        _ => Err(invalid("Sankey public method is unavailable").into()),
    }
}
