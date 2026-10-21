//! Mutable Packet objects, Pele 458322a5 (MIT, kepano).
//!
//! The graph owns object identity and arbitrary host fields. Only renderer reads
//! are projected into PacketModel; edits never rebuild source or normalize rows.
use super::{Packet, PacketBlock, PacketModel};
use crate::compat::{
    public::{Atom, GraphWriter, ModelGraph, Node, PublicDiagramModule},
    DiagramType, ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
use std::{collections::HashMap, sync::Arc};

// Independent of Pele's user source limits. Shared row aliases must not turn a
// bounded arena into an unbounded typed projection.
const MAX_ROWS: usize = 1_000_000;
const MAX_BLOCKS: usize = 8_000_000;
struct Imported {
    model: PacketModel,
    holes: Vec<usize>,
}
pub struct PacketPublicState {
    graph: ModelGraph,
    imported: Result<Imported, PeleError>,
}
fn invalid(message: impl Into<String>) -> PeleError {
    let mut error = PeleError::new(message, ErrorCode::Semantic);
    error.diagram_type = Some("packet".into());
    error
}
fn budget() -> PeleError {
    let mut error = invalid("Packet public model expansion exceeds the work budget");
    error.code = ErrorCode::Limit;
    error
}
fn field<'a>(graph: &'a ModelGraph, id: u32, name: &str) -> Option<&'a Atom> {
    graph.nodes[id as usize]
        .properties()
        .iter()
        .find_map(|p| graph.strings[p.key as usize].is(name).then_some(&p.value))
}
fn text(graph: &ModelGraph, atom: &Atom) -> Result<String, PeleError> {
    graph
        .text(atom)?
        .well_formed()
        .ok_or_else(|| invalid("Packet rendering of lone UTF-16 surrogates is not implemented"))
}
fn optional_text(graph: &ModelGraph, key: &str) -> Result<Option<String>, PeleError> {
    match graph.field(key)? {
        None | Some(Atom::Undefined | Atom::Null | Atom::Bool(false)) => Ok(None),
        Some(Atom::Number(n)) if *n == 0. || n.is_nan() => Ok(None),
        Some(atom) => text(graph, atom).map(Some),
    }
}
fn number(atom: Option<&Atom>) -> Result<f64, PeleError> {
    match atom {
        Some(Atom::Number(n)) => Ok(*n),
        None | Some(Atom::Undefined) => Ok(f64::NAN),
        Some(Atom::Null) => Ok(0.),
        Some(Atom::Bool(v)) => Ok(u8::from(*v) as f64),
        _ => Err(invalid("Packet numeric renderer fields must be numbers")),
    }
}
// Canonical indexes only: array string properties and a property such as "01"
// do not become elements. Sparse length is checked before allocating.
fn array<'a>(
    graph: &'a ModelGraph,
    atom: &Atom,
) -> Result<(usize, HashMap<usize, &'a Atom>), PeleError> {
    let Atom::Ref(id) = atom else {
        return Err(invalid("Packet rows and row entries must be Arrays"));
    };
    let Node::Array { length, properties } = &graph.nodes[*id as usize] else {
        return Err(invalid("Packet rows and row entries must be Arrays"));
    };
    let length = *length as usize;
    if length > MAX_BLOCKS {
        return Err(budget());
    }
    let mut indexes = HashMap::new();
    for property in properties {
        let key = &graph.strings[property.key as usize].0;
        if key.is_empty() || key.len() > 10 || (key.len() > 1 && key[0] == 48) {
            continue;
        }
        let mut index = 0u64;
        let mut valid = true;
        for &unit in key {
            if !(48..=57).contains(&unit) {
                valid = false;
                break;
            }
            index = index * 10 + u64::from(unit - 48);
        }
        if valid && index < length as u64 {
            indexes.entry(index as usize).or_insert(&property.value);
        }
    }
    Ok((length, indexes))
}
fn imported(graph: &ModelGraph) -> Result<Imported, PeleError> {
    let rows = graph
        .field("rows")?
        .ok_or_else(|| invalid("Packet rows must be an Array"))?;
    let (length, entries) = array(graph, rows)?;
    if length > MAX_ROWS {
        return Err(budget());
    }
    let mut rows = Vec::with_capacity(length);
    let mut holes = Vec::new();
    let mut blocks = HashMap::<u32, PacketBlock>::new();
    let mut labels = HashMap::<u32, Arc<str>>::new();
    let mut work = 0usize;
    for i in 0..length {
        let Some(row) = entries.get(&i) else {
            holes.push(i);
            rows.push(Vec::new());
            continue;
        };
        let (length, entries) = array(graph, row)?;
        work = work.checked_add(length).ok_or_else(budget)?;
        if work > MAX_BLOCKS {
            return Err(budget());
        }
        let mut row = Vec::with_capacity(length);
        for i in 0..length {
            let atom = entries.get(&i).copied().unwrap_or(&Atom::Undefined);
            let Atom::Ref(id) = atom else {
                return Err(invalid(match atom {
                    Atom::Null => "Cannot read properties of null (reading 'end')",
                    Atom::Undefined => "Cannot read properties of undefined (reading 'end')",
                    _ => "Packet block records must be objects",
                }));
            };
            if let Some(block) = blocks.get(id) {
                row.push(block.clone());
                continue;
            }
            let label = match field(graph, *id, "label") {
                Some(Atom::String(id)) => {
                    if let Some(label) = labels.get(id) {
                        label.clone()
                    } else {
                        let label: Arc<str> = text(graph, &Atom::String(*id))?.into();
                        labels.insert(*id, label.clone());
                        label
                    }
                }
                Some(Atom::Null) => {
                    return Err(invalid(
                        "Cannot read properties of null (reading 'includes')",
                    ))
                }
                None | Some(Atom::Undefined) => {
                    return Err(invalid(
                        "Cannot read properties of undefined (reading 'includes')",
                    ))
                }
                _ => return Err(invalid("Packet block labels must be strings")),
            };
            let block = PacketBlock {
                start: number(field(graph, *id, "start"))?,
                end: number(field(graph, *id, "end"))?,
                // Strict piece equality never matches absent/non-number bits.
                bits: match field(graph, *id, "bits") {
                    Some(Atom::Number(n)) => *n,
                    _ => f64::NAN,
                },
                label,
            };
            blocks.insert(*id, block.clone());
            row.push(block);
        }
        rows.push(row);
    }
    Ok(Imported {
        model: PacketModel {
            diagram_type: DiagramType::Packet,
            title: optional_text(graph, "title")?,
            acc_title: optional_text(graph, "accTitle")?,
            acc_descr: optional_text(graph, "accDescr")?,
            bits_per_row: number(graph.field("bitsPerRow")?)?,
            rows,
        },
        holes,
    })
}
fn array_node(out: &mut GraphWriter, values: impl Iterator<Item = Atom>) -> u32 {
    let properties: Vec<_> = values
        .enumerate()
        .map(|(i, value)| out.property(&i.to_string(), value))
        .collect();
    out.node(Node::Array {
        length: properties.len() as u32,
        properties,
    })
}
impl PublicDiagramModule for Packet {
    type PublicState = PacketPublicState;
    fn detached_public_render_capability(
        &self,
    ) -> Option<crate::compat::public::DetachedRenderCapability> {
        Some(crate::compat::public::DetachedRenderCapability::packet())
    }
    fn detached_public_render(
        &self,
        state: &Self::PublicState,
    ) -> Option<crate::compat::public::DetachedPublicRender> {
        Some(crate::compat::public::DetachedPublicRender::packet(
            super::PacketDetachedInput::new(state.imported.as_ref().err()),
        ))
    }

    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let imported = imported(&graph);
        Ok(PacketPublicState { graph, imported })
    }
    fn create_public(&self, model: &PacketModel) -> Result<Self::PublicState, PeleError> {
        let mut out = GraphWriter::default();
        let mut label_ids = HashMap::new();
        let mut row_ids = Vec::with_capacity(model.rows.len());
        for row in &model.rows {
            let mut block_ids = Vec::with_capacity(row.len());
            for block in row {
                let key = (block.label.as_ptr() as usize, block.label.len());
                let label = *label_ids
                    .entry(key)
                    .or_insert_with(|| out.string(block.label.as_ref()));
                let properties = vec![
                    out.property("start", Atom::Number(block.start)),
                    out.property("end", Atom::Number(block.end)),
                    out.property("bits", Atom::Number(block.bits)),
                    out.property("label", Atom::String(label)),
                ];
                block_ids.push(out.node(Node::Object { properties }));
            }
            row_ids.push(array_node(&mut out, block_ids.into_iter().map(Atom::Ref)));
        }
        let rows = array_node(&mut out, row_ids.into_iter().map(Atom::Ref));
        let kind = out.string("packet");
        let mut properties = vec![out.property("type", Atom::String(kind))];
        for (key, value) in [
            ("title", &model.title),
            ("accTitle", &model.acc_title),
            ("accDescr", &model.acc_descr),
        ] {
            let atom = value
                .as_deref()
                .map_or(Atom::Undefined, |v| Atom::String(out.string(v)));
            properties.push(out.property(key, atom));
        }
        properties.push(out.property("bitsPerRow", Atom::Number(model.bits_per_row)));
        properties.push(out.property("rows", Atom::Ref(rows)));
        let root = out.node(Node::Object { properties });
        let graph = out.finish(Atom::Ref(root))?;
        let imported = imported(&graph);
        Ok(PacketPublicState { graph, imported })
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
        state.imported = imported(&graph);
        state.graph = graph;
        Ok(())
    }
    fn render_public(
        &self,
        state: &mut Self::PublicState,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        let data = state.imported.as_ref().map_err(Clone::clone)?;
        super::render::render_with_row_holes(&data.model, config, options, host, &data.holes)
    }
}
