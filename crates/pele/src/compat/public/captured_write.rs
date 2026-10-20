//! Mutable captured storage. Family policy and admission stay in the caller.
use super::{
    captured_read::{ArenaRead, CapturedRead, NodeHeader, PropertyPrefix, PROPERTY_CHUNK},
    Atom, JsString, ModelGraph, Node, Property,
};
use crate::compat::PeleError;

pub(crate) struct GraphCounts {
    pub nodes: usize,
    pub strings: usize,
    pub units: usize,
    pub work: usize,
}
#[derive(Default)]
pub(crate) struct OwnQuery {
    pub first_slot: Option<u32>,
    pub any_nonwritable: bool,
}
pub(crate) struct MapMatch {
    pub slot: u32,
    pub value: Atom,
}

pub(crate) trait CapturedWrite: CapturedRead<Value = Atom> {
    fn counts(&mut self) -> Result<GraphCounts, PeleError>;
    fn find_strings(&mut self, names: &[&str], out: &mut [Option<u32>]) -> Result<(), PeleError>;
    fn intern(&mut self, text: &str) -> Result<u32, PeleError>;
    fn append_node(&mut self, node: Node) -> Result<u32, PeleError>;
    fn own_descriptors(
        &mut self,
        id: u32,
        names: &[&str],
        out: &mut [OwnQuery],
    ) -> Result<(), PeleError>;
    fn map_find(&mut self, id: u32, key: &Atom) -> Result<Option<MapMatch>, PeleError>;
    fn replace_map_value(&mut self, id: u32, slot: u32, value: Atom) -> Result<(), PeleError>;
    fn append_map_entry(&mut self, id: u32, key: Atom, value: Atom) -> Result<(), PeleError>;
    fn replace_property_value(&mut self, id: u32, slot: u32, value: Atom) -> Result<(), PeleError>;
    fn append_property(&mut self, id: u32, property: Property) -> Result<(), PeleError>;
    fn replace_matching_property_values(
        &mut self,
        id: u32,
        key: &str,
        value: Atom,
    ) -> Result<(), PeleError>;
    fn set_array_length(&mut self, id: u32, length: u32) -> Result<(), PeleError>;
}

pub(crate) struct ArenaWrite<'g> {
    graph: &'g mut ModelGraph,
}
impl<'g> ArenaWrite<'g> {
    pub fn new(graph: &'g mut ModelGraph) -> Self {
        Self { graph }
    }
}
impl CapturedRead for ArenaWrite<'_> {
    type Value = Atom;
    fn root(&self) -> Atom {
        self.graph.root.clone()
    }
    fn header(&mut self, id: u32) -> Result<Option<NodeHeader>, PeleError> {
        ArenaRead::new(self.graph).header(id)
    }
    fn own_fields(
        &mut self,
        id: u32,
        names: &[&str],
        out: &mut [Option<Atom>],
    ) -> Result<(), PeleError> {
        assert_eq!(names.len(), out.len());
        out.fill(None);
        let mut remaining = names.len();
        if remaining == 0 {
            return Ok(());
        }
        if let Some(node) = self.graph.nodes.get(id as usize) {
            for property in node.properties() {
                for (name, value) in names.iter().zip(out.iter_mut()) {
                    if value.is_none() && self.graph.strings[property.key as usize].is(name) {
                        *value = Some(property.value.clone());
                        remaining -= 1;
                    }
                }
                if remaining == 0 {
                    break;
                }
            }
        }
        Ok(())
    }
    fn property_prefixes(
        &mut self,
        id: u32,
        start: u32,
        out: &mut [PropertyPrefix<Atom>],
    ) -> Result<usize, PeleError> {
        for slot in out.iter_mut() {
            *slot = PropertyPrefix::default();
        }
        let mut borrowed: [PropertyPrefix<&Atom>; PROPERTY_CHUNK] =
            std::array::from_fn(|_| PropertyPrefix::default());
        let capacity = out.len().min(PROPERTY_CHUNK);
        let count =
            ArenaRead::new(self.graph).property_prefixes(id, start, &mut borrowed[..capacity])?;
        for (target, source) in out.iter_mut().zip(&borrowed[..count]) {
            target.key_units = source.key_units;
            target.prefix = source.prefix;
            target.value = source.value.cloned();
        }
        Ok(count)
    }
    fn string_length(&mut self, id: u32) -> Result<u32, PeleError> {
        ArenaRead::new(self.graph).string_length(id)
    }
    fn with_string_chunks(
        &mut self,
        id: u32,
        visit: impl FnMut(&[u16]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        ArenaRead::new(self.graph).with_string_chunks(id, visit)
    }
}

fn same_key(graph: &ModelGraph, a: &Atom, b: &Atom) -> bool {
    match (a, b) {
        (Atom::Undefined, Atom::Undefined) | (Atom::Null, Atom::Null) => true,
        (Atom::Bool(a), Atom::Bool(b)) => a == b,
        (Atom::Number(a), Atom::Number(b)) => a == b || (a.is_nan() && b.is_nan()),
        (Atom::String(a), Atom::String(b)) => {
            graph.strings[*a as usize] == graph.strings[*b as usize]
        }
        (Atom::Ref(a), Atom::Ref(b))
        | (Atom::BuiltinFunction(a), Atom::BuiltinFunction(b))
        | (Atom::HostReference(a), Atom::HostReference(b)) => a == b,
        _ => false,
    }
}
impl CapturedWrite for ArenaWrite<'_> {
    fn counts(&mut self) -> Result<GraphCounts, PeleError> {
        let units = self
            .graph
            .strings
            .iter()
            .fold(0usize, |n, s| n.saturating_add(s.0.len()));
        let work = self.graph.nodes.iter().fold(0usize, |n, node| {
            n.saturating_add(node.properties().len())
                .saturating_add(match node {
                    Node::Map { entries, .. } => entries.len().saturating_mul(2),
                    Node::Set { values, .. } => values.len(),
                    _ => 0,
                })
        });
        Ok(GraphCounts {
            nodes: self.graph.nodes.len(),
            strings: self.graph.strings.len(),
            units,
            work,
        })
    }
    fn find_strings(&mut self, names: &[&str], out: &mut [Option<u32>]) -> Result<(), PeleError> {
        assert_eq!(names.len(), out.len());
        out.fill(None);
        let mut remaining = names.len();
        if remaining == 0 {
            return Ok(());
        }
        for (id, text) in self.graph.strings.iter().enumerate() {
            for (name, value) in names.iter().zip(out.iter_mut()) {
                if value.is_none() && text.is(name) {
                    *value = Some(id as u32);
                    remaining -= 1;
                }
            }
            if remaining == 0 {
                break;
            }
        }
        Ok(())
    }
    fn intern(&mut self, text: &str) -> Result<u32, PeleError> {
        if let Some(id) = self.graph.strings.iter().position(|s| s.is(text)) {
            return Ok(id as u32);
        }
        let id = self.graph.strings.len() as u32;
        self.graph.strings.push(JsString::from(text));
        Ok(id)
    }
    fn append_node(&mut self, node: Node) -> Result<u32, PeleError> {
        let id = self.graph.nodes.len() as u32;
        self.graph.nodes.push(node);
        Ok(id)
    }
    fn own_descriptors(
        &mut self,
        id: u32,
        names: &[&str],
        out: &mut [OwnQuery],
    ) -> Result<(), PeleError> {
        assert_eq!(names.len(), out.len());
        for value in out.iter_mut() {
            *value = OwnQuery::default();
        }
        if names.is_empty() {
            return Ok(());
        }
        if let Some(node) = self.graph.nodes.get(id as usize) {
            for (slot, property) in node.properties().iter().enumerate() {
                for (name, query) in names.iter().zip(out.iter_mut()) {
                    if self.graph.strings[property.key as usize].is(name) {
                        if query.first_slot.is_none() {
                            query.first_slot = Some(slot as u32);
                        }
                        query.any_nonwritable |= property.flags & 2 == 0;
                    }
                }
            }
        }
        Ok(())
    }
    fn map_find(&mut self, id: u32, key: &Atom) -> Result<Option<MapMatch>, PeleError> {
        let Node::Map { entries, .. } = &self.graph.nodes[id as usize] else {
            unreachable!()
        };
        Ok(entries
            .iter()
            .enumerate()
            .find(|(_, (stored, _))| same_key(self.graph, stored, key))
            .map(|(slot, (_, value))| MapMatch {
                slot: slot as u32,
                value: value.clone(),
            }))
    }
    fn replace_map_value(&mut self, id: u32, slot: u32, value: Atom) -> Result<(), PeleError> {
        let Node::Map { entries, .. } = &mut self.graph.nodes[id as usize] else {
            unreachable!()
        };
        entries[slot as usize].1 = value;
        Ok(())
    }
    fn append_map_entry(&mut self, id: u32, key: Atom, value: Atom) -> Result<(), PeleError> {
        let Node::Map { entries, .. } = &mut self.graph.nodes[id as usize] else {
            unreachable!()
        };
        entries.push((key, value));
        Ok(())
    }
    fn replace_property_value(&mut self, id: u32, slot: u32, value: Atom) -> Result<(), PeleError> {
        self.graph.nodes[id as usize].properties_mut()[slot as usize].value = value;
        Ok(())
    }
    fn append_property(&mut self, id: u32, property: Property) -> Result<(), PeleError> {
        self.graph.nodes[id as usize]
            .properties_mut()
            .push(property);
        Ok(())
    }
    fn replace_matching_property_values(
        &mut self,
        id: u32,
        key: &str,
        value: Atom,
    ) -> Result<(), PeleError> {
        for property in self.graph.nodes[id as usize].properties_mut() {
            if self.graph.strings[property.key as usize].is(key) {
                property.value = value.clone();
            }
        }
        Ok(())
    }
    fn set_array_length(&mut self, id: u32, length: u32) -> Result<(), PeleError> {
        let Node::Array { length: target, .. } = &mut self.graph.nodes[id as usize] else {
            unreachable!()
        };
        *target = length;
        Ok(())
    }
}
