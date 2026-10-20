//! Opt-in mutable Sankey data codec for Pele 458322a5 (MIT, kepano).
//!
//! The arena owns identity: node array entries, link endpoints and the observable
//! `lookup` Map share object IDs. Direct edits do not refresh that Map or
//! renumber nodes. The existing Sankey renderer consumes a concrete typed
//! projection imported at the edit boundary; rendering never reparses source.
use super::{Sankey, SankeyLink, SankeyModel, SankeyNode};
use crate::compat::public::captured_read::{
    ArenaRead, CapturedRead, NodeKind, PropertyPrefix, PROPERTY_CHUNK, STRING_CHUNK,
};
use crate::compat::public::captured_write::{ArenaWrite, CapturedWrite, OwnQuery};
use crate::compat::{
    public::{
        Atom, FieldKind, FieldSchema, GraphWriter, InitialWriter, JavaScriptErrorKind, JsString,
        MemberKind, MemberSchema, MethodCall, MethodId, MethodSchema, ModelGraph, ModelSchema,
        Node, Property, PrototypeId, PrototypeParent, PrototypeSchema, PublicDiagramModule,
        PublicError, RecordId, RecordKind, RecordSchema,
    },
    DiagramModule, DiagramType, ErrorCode, Object, PeleError, RenderResources, RenderSettings,
    Rendered,
};
use std::{borrow::Borrow, collections::HashMap};

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
fn captured_error(message: &str) -> PeleError {
    PeleError::new(message, ErrorCode::Semantic)
}
fn stored_field<R: CapturedRead>(
    reader: &mut R,
    id: u32,
    name: &str,
) -> Result<Option<R::Value>, PeleError> {
    let mut out = [None];
    reader.own_fields(id, &[name], &mut out)?;
    Ok(out[0].take())
}
fn root_field<R: CapturedRead>(reader: &mut R, name: &str) -> Result<Option<R::Value>, PeleError> {
    let Atom::Ref(id) = reader.root() else {
        return Err(captured_error("Public model root must be an object"));
    };
    let header = reader.header(id)?;
    if !matches!(
        header.map(|header| header.kind),
        Some(NodeKind::Object | NodeKind::Instance)
    ) {
        return Err(captured_error("Public model root must be an object"));
    }
    stored_field(reader, id, name)
}
fn string<R: CapturedRead>(reader: &mut R, atom: &Atom) -> Result<String, PeleError> {
    let Atom::String(id) = atom else {
        return Err(captured_error("Expected a public string"));
    };
    let length = reader.string_length(*id)? as usize;
    let mut text = String::with_capacity(length);
    let mut seen = 0usize;
    let mut high = None;
    let surrogate = || invalid("Sankey rendering of lone UTF-16 surrogates is not implemented");
    reader.with_string_chunks(*id, |units| {
        if units.is_empty() || units.len() > STRING_CHUNK {
            return Err(captured_error("Invalid captured string chunk"));
        }
        seen = seen
            .checked_add(units.len())
            .filter(|n| *n <= length)
            .ok_or_else(|| captured_error("Invalid captured string length"))?;
        for &unit in units {
            if let Some(leading) = high.take() {
                if !(0xdc00..=0xdfff).contains(&unit) {
                    return Err(surrogate());
                }
                let scalar = 0x10000 + (((leading - 0xd800) as u32) << 10) + (unit - 0xdc00) as u32;
                text.push(char::from_u32(scalar).expect("paired UTF16 scalar"));
            } else if (0xd800..=0xdbff).contains(&unit) {
                high = Some(unit);
            } else if (0xdc00..=0xdfff).contains(&unit) {
                return Err(surrogate());
            } else {
                text.push(char::from_u32(unit as u32).expect("non-surrogate UTF16 scalar"));
            }
        }
        Ok(())
    })?;
    if seen != length {
        return Err(captured_error("Invalid captured string length"));
    }
    if high.is_some() {
        return Err(surrogate());
    }
    Ok(text)
}
fn optional_text<R: CapturedRead>(reader: &mut R, key: &str) -> Result<Option<String>, PeleError> {
    match root_field(reader, key)? {
        None => Ok(None),
        Some(value) => match value.borrow() {
            Atom::Undefined | Atom::Null => Ok(None),
            atom => string(reader, atom).map(Some),
        },
    }
}
// Canonical indexes are interpreted here, never by the host storage adapter.
// Both array memberships are admitted before any label or endpoint projection.
fn array<R: CapturedRead>(
    reader: &mut R,
    name: &str,
    scratch: &mut [PropertyPrefix<R::Value>],
) -> Result<Vec<R::Value>, PeleError> {
    let value = root_field(reader, name)?;
    let id = match value.as_ref().map(|value| value.borrow()) {
        Some(Atom::Ref(id)) => *id,
        _ => return Err(invalid(&format!("Sankey {name} must be an Array"))),
    };
    let Some(header) = reader.header(id)? else {
        return Err(invalid(&format!("Sankey {name} must be an Array")));
    };
    if header.kind != NodeKind::Array {
        return Err(invalid(&format!("Sankey {name} must be an Array")));
    }
    let length = header
        .array_length
        .ok_or_else(|| captured_error("Invalid captured array header"))?;
    let mut indexed = HashMap::new();
    let mut cursor = 0u32;
    while cursor < header.property_count {
        let capacity = scratch
            .len()
            .min(PROPERTY_CHUNK)
            .min((header.property_count - cursor) as usize);
        let count = reader.property_prefixes(id, cursor, &mut scratch[..capacity])?;
        if count == 0 || count > capacity {
            return Err(captured_error("Invalid captured property batch"));
        }
        for property in &mut scratch[..count] {
            let value = property
                .value
                .take()
                .ok_or_else(|| captured_error("Invalid captured property value"))?;
            let size = property.key_units as usize;
            let key = &property.prefix;
            if size == 0 || size > key.len() || (size > 1 && key[0] == 48) {
                continue;
            }
            let mut index = 0u64;
            let mut canonical = true;
            for &unit in &key[..size] {
                if !(48..=57).contains(&unit) {
                    canonical = false;
                    break;
                }
                index = index * 10 + u64::from(unit - 48);
            }
            if canonical && index < u64::from(length) {
                indexed.entry(index as u32).or_insert(value);
            }
        }
        cursor = cursor
            .checked_add(count as u32)
            .ok_or_else(|| captured_error("Invalid captured property cursor"))?;
    }
    if indexed.len() != length as usize {
        return Err(invalid(&format!(
            "Sankey {name} contains a missing array record"
        )));
    }
    Ok((0..length).map(|i| indexed[&i].clone()).collect())
}
fn index<R: CapturedRead>(reader: &mut R, id: u32, length: usize) -> Result<usize, PeleError> {
    let value = stored_field(reader, id, "index")?;
    match value.as_ref().map(|value| value.borrow()) {
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
    imported_from(&mut ArenaRead::new(graph))
}
fn imported_from<R: CapturedRead>(reader: &mut R) -> Result<SankeyModel, PeleError> {
    let mut scratch: [PropertyPrefix<R::Value>; PROPERTY_CHUNK] =
        std::array::from_fn(|_| PropertyPrefix::default());
    let array_nodes = array(reader, "nodes", &mut scratch)?;
    let array_links = array(reader, "links", &mut scratch)?;
    let mut nodes = Vec::with_capacity(array_nodes.len());
    let mut labels: HashMap<u32, String> = HashMap::new();
    let mut indexes: HashMap<u32, usize> = HashMap::new();
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
        let id = object_id(value.borrow())?;
        let mut fields = [None, None];
        reader.own_fields(id, &["id", "index"], &mut fields)?;
        let label = if let Some(label) = labels.get(&id) {
            reserve_text(&mut text_bytes, label.len())?;
            label.clone()
        } else {
            let value = fields[0]
                .as_ref()
                .ok_or_else(|| invalid("Sankey node ID must be a string"))?;
            let label = string(reader, value.borrow())?;
            reserve_text(&mut text_bytes, label.len().saturating_mul(2))?;
            labels.insert(id, label.clone());
            label
        };
        // This unused index remains independent of endpoint validation.
        let unused_index = match fields[1].as_ref().map(|value| value.borrow()) {
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
        let id = object_id(value.borrow())?;
        let mut fields = [None, None, None];
        reader.own_fields(id, &["source", "target", "value"], &mut fields)?;
        let [source, target, value] = fields;
        let mut endpoint = |value: Option<R::Value>| -> Result<SankeyNode, PeleError> {
            let value =
                value.ok_or_else(|| invalid("Sankey links require source and target objects"))?;
            let object = object_id(value.borrow())?;
            let index = if let Some(index) = indexes.get(&object) {
                *index
            } else {
                let index = index(reader, object, nodes.len())?;
                indexes.insert(object, index);
                index
            };
            Ok(SankeyNode {
                id: String::new(),
                index,
            })
        };
        let source = endpoint(source)?;
        let target = endpoint(target)?;
        let value = match value.as_ref().map(|value| value.borrow()) {
            Some(Atom::Number(n)) => *n,
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
        title: optional_text(reader, "title")?,
        acc_title: optional_text(reader, "accTitle")?,
        acc_descr: optional_text(reader, "accDescr")?,
    })
}
fn node(out: &mut impl InitialWriter, node: &SankeyNode) -> Result<u32, PeleError> {
    let label = out.string(node.id.as_str())?;
    let properties = vec![
        out.property("id", Atom::String(label))?,
        out.property("index", Atom::Number(node.index as f64))?,
    ];
    out.node(Node::Object { properties })
}
fn array_node(
    out: &mut impl InitialWriter,
    entries: impl Iterator<Item = Atom>,
) -> Result<u32, PeleError> {
    let properties: Vec<_> = entries
        .enumerate()
        .map(|(i, value)| out.property(&i.to_string(), value))
        .collect::<Result<_, _>>()?;
    out.node(Node::Array {
        length: properties.len() as u32,
        properties,
    })
}
fn initial_graph(model: &SankeyModel) -> Result<ModelGraph, PeleError> {
    initial_into(GraphWriter::default(), model)
}
fn initial_into<W: InitialWriter>(mut out: W, model: &SankeyModel) -> Result<W::Output, PeleError> {
    let node_objects: Vec<_> = model
        .nodes
        .iter()
        .map(|n| node(&mut out, n))
        .collect::<Result<_, _>>()?;
    let mut original_nodes = HashMap::new();
    for (record, object) in model.nodes.iter().zip(&node_objects) {
        original_nodes
            .entry((record.index, record.id.as_str()))
            .or_insert(*object);
    }
    let nodes = array_node(&mut out, node_objects.iter().copied().map(Atom::Ref))?;
    let mut link_objects = Vec::with_capacity(model.links.len());
    for link in &model.links {
        let mut endpoint = |endpoint: &SankeyNode| {
            // The source parser's records are owned value copies in the
            // immutable API. Recover its guaranteed index/record alias
            // once here; all subsequent edits use arena IDs exclusively.
            original_nodes
                .get(&(endpoint.index, endpoint.id.as_str()))
                .copied()
                .map(Ok)
                .unwrap_or_else(|| node(&mut out, endpoint))
        };
        let source = endpoint(&link.source)?;
        let target = endpoint(&link.target)?;
        let properties = vec![
            out.property("source", Atom::Ref(source))?,
            out.property("target", Atom::Ref(target))?,
            out.property("value", Atom::Number(link.value))?,
        ];
        link_objects.push(out.node(Node::Object { properties })?);
    }
    let links = array_node(&mut out, link_objects.into_iter().map(Atom::Ref))?;
    let entries = model
        .nodes
        .iter()
        .zip(&node_objects)
        .map(|(n, id)| Ok((Atom::String(out.string(n.id.as_str())?), Atom::Ref(*id))))
        .collect::<Result<_, PeleError>>()?;
    let lookup = out.node(Node::Map {
        entries,
        properties: vec![],
    })?;
    let kind = out.string("sankey")?;
    let mut properties = vec![
        out.property("type", Atom::String(kind))?,
        out.property("nodes", Atom::Ref(nodes))?,
        out.property("links", Atom::Ref(links))?,
    ];
    for (key, text) in [
        ("title", &model.title),
        ("accTitle", &model.acc_title),
        ("accDescr", &model.acc_descr),
    ] {
        let value = match text {
            Some(text) => Atom::String(out.string(text.as_str())?),
            None => Atom::Undefined,
        };
        properties.push(out.property(key, value)?);
    }
    properties.push(out.property("lookup", Atom::Ref(lookup))?);
    let root = out.node(Node::Instance {
        prototype: SANKEY_PROTO,
        properties,
    })?;
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
fn read<W: CapturedWrite>(store: &mut W, receiver: &Atom, key: &str) -> Result<Atom, PublicError> {
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
                if let Some(header) = store.header(*id)? {
                    if header.kind == NodeKind::Array {
                        return Ok(Atom::Number(f64::from(
                            header.array_length.expect("array header"),
                        )));
                    }
                }
            }
            Ok(stored_field(store, *id, key)?.unwrap_or(Atom::Undefined))
        }
        Atom::String(id) if key == "length" => {
            Ok(Atom::Number(f64::from(store.string_length(*id)?)))
        }
        _ => Ok(Atom::Undefined),
    }
}
/// Only the real native Map/Array prototype operation is implicit. A supplied
/// function needs an explicit host/native callback dispatch capability.
fn collection<W: CapturedWrite>(
    store: &mut W,
    value: &Atom,
    property: &str,
    expression: &str,
    map: bool,
) -> Result<u32, PublicError> {
    let callable = read(store, value, property)?;
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
    let native = store
        .header(id)?
        .is_some_and(|header| header.kind == if map { NodeKind::Map } else { NodeKind::Array });
    if !native || stored_field(store, id, property)?.is_some() {
        return Err(type_error(&format!("{expression} is not a function")));
    }
    Ok(id)
}
fn room<W: CapturedWrite>(
    store: &mut W,
    nodes: usize,
    keys: &[&str],
    references: usize,
) -> Result<(), PublicError> {
    // Current helper policies request at most four keys. Keep absent duplicate
    // requests distinct, exactly as the original missing-key vector did.
    let mut present = [None; 4];
    assert!(keys.len() <= present.len());
    store.find_strings(keys, &mut present[..keys.len()])?;
    let missing = keys.iter().zip(&present).filter(|(_, id)| id.is_none());
    let missing_count = missing.clone().count();
    let missing_units = missing.map(|(key, _)| key.len()).sum::<usize>();
    let counts = store.counts()?;
    let units = counts.units.saturating_add(missing_units);
    if counts.nodes.saturating_add(nodes) > 1_000_000
        || counts.strings.saturating_add(missing_count) > 1_000_000
        || units > 16_000_000
        || counts.work.saturating_add(references) > 8_000_000
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
// scalar length. Every matching readonly descriptor participates in rejection.
struct PushPosition {
    length: u32,
    key: String,
    existing: Option<u32>,
}
fn push_index<W: CapturedWrite>(store: &mut W, id: u32) -> Result<PushPosition, PublicError> {
    let header = store.header(id)?.expect("validated array");
    assert!(header.kind == NodeKind::Array);
    let length = header.array_length.expect("array header");
    let key = length.to_string();
    let mut descriptors = [OwnQuery::default(), OwnQuery::default()];
    store.own_descriptors(id, &["length", &key], &mut descriptors)?;
    if descriptors[0].any_nonwritable {
        return Err(type_error(
            "Cannot assign to read only property 'length' of object '[object Array]'",
        ));
    }
    if descriptors[1].any_nonwritable {
        return Err(type_error(&format!(
            "Cannot assign to read only property '{key}' of object '[object Array]'"
        )));
    }
    if length >= 1_000_000 {
        return Err(
            PeleError::new("Sankey array exceeds native work budget", ErrorCode::Limit).into(),
        );
    }
    Ok(PushPosition {
        length,
        key,
        existing: descriptors[1].first_slot,
    })
}
fn push<W: CapturedWrite>(
    store: &mut W,
    id: u32,
    position: PushPosition,
    value: Atom,
) -> Result<(), PublicError> {
    // Since push_index, only read-only room checks and distinct new-record/
    // string appends may run. No callback or target-array edit intervenes.
    let PushPosition {
        length,
        key,
        existing,
    } = position;
    if let Some(slot) = existing {
        store.replace_property_value(id, slot, value)?;
    } else {
        let key = store.intern(&key)?;
        store.append_property(
            id,
            Property {
                key,
                value,
                flags: 7,
            },
        )?;
    }
    store.set_array_length(id, length + 1)?;
    store.replace_matching_property_values(id, "length", Atom::Number(f64::from(length + 1)))?;
    Ok(())
}
fn record<W: CapturedWrite>(store: &mut W, fields: Vec<(&str, Atom)>) -> Result<Atom, PublicError> {
    let properties = fields
        .into_iter()
        .map(|(key, value)| {
            Ok(Property {
                key: store.intern(key)?,
                value,
                flags: 7,
            })
        })
        .collect::<Result<_, PeleError>>()?;
    Ok(Atom::Ref(store.append_node(Node::Object { properties })?))
}
fn invoke_graph(graph: &mut ModelGraph, call: MethodCall) -> Result<Atom, PublicError> {
    invoke_store(&mut ArenaWrite::new(graph), call)
}
fn invoke_store<W: CapturedWrite>(store: &mut W, call: MethodCall) -> Result<Atom, PublicError> {
    let arg = |i| call.arguments.get(i).cloned().unwrap_or(Atom::Undefined);
    match call.method.0 {
        1 => {
            let key = arg(0);
            let lookup = read(store, &call.receiver, "lookup")?;
            let lookup_id = collection(store, &lookup, "get", "this.lookup.get", true)?;
            let found = store
                .map_find(lookup_id, &key)?
                .map(|entry| entry.value)
                .unwrap_or(Atom::Undefined);
            if !matches!(found, Atom::Undefined) {
                return Ok(found);
            }
            let nodes = read(store, &call.receiver, "nodes")?;
            let index = read(store, &nodes, "length")?;
            let lookup = read(store, &call.receiver, "lookup")?;
            let lookup_id = collection(store, &lookup, "set", "this.lookup.set", true)?;
            let at = store.map_find(lookup_id, &key)?.map(|entry| entry.slot);
            room(
                store,
                1,
                &["id", "index"],
                2 + if at.is_none() { 2 } else { 0 },
            )?;
            let node = record(store, vec![("id", key.clone()), ("index", index)])?;
            if let Some(at) = at {
                store.replace_map_value(lookup_id, at, node.clone())?;
            } else {
                // Only newly appended zero keys canonicalize; record ID and
                // existing matching key retain their exact IEEE bits.
                let key = match key {
                    Atom::Number(n) if n == 0. => Atom::Number(0.),
                    key => key,
                };
                store.append_map_entry(lookup_id, key, node.clone())?;
            }
            // Deliberately after Map publication, including every throwing path.
            let nodes = read(store, &call.receiver, "nodes")?;
            let id = collection(store, &nodes, "push", "this.nodes.push", false)?;
            let position = push_index(store, id)?;
            room(store, 0, &[&position.key], 1)?;
            push(store, id, position, node.clone())?;
            Ok(node)
        }
        2 => {
            let links = read(store, &call.receiver, "links")?;
            let id = collection(store, &links, "push", "this.links.push", false)?;
            let position = push_index(store, id)?;
            room(store, 1, &["source", "target", "value", &position.key], 4)?;
            let link = record(
                store,
                vec![("source", arg(0)), ("target", arg(1)), ("value", arg(2))],
            )?;
            push(store, id, position, link)?;
            Ok(Atom::Undefined)
        }
        _ => Err(invalid("Sankey public method is unavailable").into()),
    }
}
