//! Immutable captured storage reads. Diagram interpretation stays in Rust.
use super::{Atom, JsString, ModelGraph, Node};
use crate::compat::{ErrorCode, PeleError};
use std::borrow::Borrow;

pub(crate) const PROPERTY_CHUNK: usize = 128;
pub(crate) const STRING_CHUNK: usize = 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum NodeKind {
    Object,
    Instance,
    Array,
    Map,
    Set,
    Date,
    Error,
}
pub(crate) struct NodeHeader {
    pub kind: NodeKind,
    pub property_count: u32,
    pub array_length: Option<u32>,
}
pub(crate) struct PropertyPrefix<V> {
    pub key_units: u32,
    pub prefix: [u16; 10],
    pub value: Option<V>,
}
impl<V> Default for PropertyPrefix<V> {
    fn default() -> Self {
        Self {
            key_units: 0,
            prefix: [0; 10],
            value: None,
        }
    }
}

pub(crate) trait CapturedRead {
    // Arena values borrow the immutable graph, not this reader's mutable query
    // borrow. A host implementation can return owned scalar Atoms instead.
    type Value: Borrow<Atom> + Clone;
    fn root(&self) -> Atom;
    fn header(&mut self, id: u32) -> Result<Option<NodeHeader>, PeleError>;
    fn own_fields(
        &mut self,
        id: u32,
        names: &[&str],
        out: &mut [Option<Self::Value>],
    ) -> Result<(), PeleError>;
    fn property_prefixes(
        &mut self,
        id: u32,
        start: u32,
        out: &mut [PropertyPrefix<Self::Value>],
    ) -> Result<usize, PeleError>;
    fn string_length(&mut self, id: u32) -> Result<u32, PeleError>;
    fn with_string_chunks(
        &mut self,
        id: u32,
        visit: impl FnMut(&[u16]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError>;
}

pub(crate) struct ArenaRead<'g> {
    graph: &'g ModelGraph,
}
impl<'g> ArenaRead<'g> {
    pub fn new(graph: &'g ModelGraph) -> Self {
        Self { graph }
    }
    fn text(&self, id: u32) -> Result<&'g JsString, PeleError> {
        self.graph
            .strings
            .get(id as usize)
            .ok_or_else(|| PeleError::new("Invalid public string reference", ErrorCode::Semantic))
    }
}
impl<'g> CapturedRead for ArenaRead<'g> {
    type Value = &'g Atom;
    fn root(&self) -> Atom {
        self.graph.root.clone()
    }
    fn header(&mut self, id: u32) -> Result<Option<NodeHeader>, PeleError> {
        Ok(self.graph.nodes.get(id as usize).map(|node| NodeHeader {
            kind: match node {
                Node::Object { .. } => NodeKind::Object,
                Node::Instance { .. } => NodeKind::Instance,
                Node::Array { .. } => NodeKind::Array,
                Node::Map { .. } => NodeKind::Map,
                Node::Set { .. } => NodeKind::Set,
                Node::Date { .. } => NodeKind::Date,
                Node::Error { .. } => NodeKind::Error,
            },
            property_count: node.properties().len() as u32,
            array_length: match node {
                Node::Array { length, .. } => Some(*length),
                _ => None,
            },
        }))
    }
    fn own_fields(
        &mut self,
        id: u32,
        names: &[&str],
        out: &mut [Option<Self::Value>],
    ) -> Result<(), PeleError> {
        assert_eq!(names.len(), out.len());
        out.fill(None);
        let mut remaining = names.len();
        if remaining == 0 {
            return Ok(());
        }
        let graph: &'g ModelGraph = self.graph;
        if let Some(node) = graph.nodes.get(id as usize) {
            for property in node.properties() {
                for (name, value) in names.iter().zip(out.iter_mut()) {
                    if value.is_none() && graph.strings[property.key as usize].is(name) {
                        *value = Some(&property.value);
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
        out: &mut [PropertyPrefix<Self::Value>],
    ) -> Result<usize, PeleError> {
        for slot in out.iter_mut() {
            *slot = PropertyPrefix::default();
        }
        let graph: &'g ModelGraph = self.graph;
        let properties = graph
            .nodes
            .get(id as usize)
            .ok_or_else(|| {
                PeleError::new("Invalid captured object reference", ErrorCode::Semantic)
            })?
            .properties();
        let start = start as usize;
        if start > properties.len() {
            return Err(PeleError::new(
                "Invalid captured property cursor",
                ErrorCode::Semantic,
            ));
        }
        let count = out.len().min(PROPERTY_CHUNK).min(properties.len() - start);
        for (slot, property) in out.iter_mut().zip(&properties[start..start + count]) {
            let key = &graph.strings[property.key as usize].0;
            slot.key_units = key.len() as u32;
            let length = key.len().min(slot.prefix.len());
            slot.prefix[..length].copy_from_slice(&key[..length]);
            slot.value = Some(&property.value);
        }
        Ok(count)
    }
    fn string_length(&mut self, id: u32) -> Result<u32, PeleError> {
        Ok(self.text(id)?.0.len() as u32)
    }
    fn with_string_chunks(
        &mut self,
        id: u32,
        mut visit: impl FnMut(&[u16]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        for chunk in self.text(id)?.0.chunks(STRING_CHUNK) {
            visit(chunk)?;
        }
        Ok(())
    }
}
