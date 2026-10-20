//! Source-owned logical graph. Static records and collection slots are derived;
//! only actual mutations own generic records. No source Property/Node arena.
use super::indexed::{key_slot, IndexedSankey};
use crate::compat::public::captured_admission::{
    AdmissionKind, AdmissionRead, Extent, Header, ScalarAtomRef, ScalarPropertyRef,
};
use crate::compat::public::captured_read::{CapturedRead, NodeHeader, NodeKind, PropertyPrefix};
use crate::compat::public::captured_write::{CapturedWrite, GraphCounts, MapMatch, OwnQuery};
use crate::compat::public::graph_read::{
    self, AdmissionState, PublicGraphRead, ReadAdapter, TrustedGraphOwner, TrustedGraphRead,
    TrustedInitialGraphOwner,
};
use crate::compat::public::{Atom, JsString, ModelGraph, Node, Property, PrototypeId};
use crate::compat::PeleError;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

// Identity is process-local, never a pointer or content hash. Once exhausted,
// new owners retain the old self-contained recipe protocol without new errors.
static NEXT_LAYOUT_ID: AtomicU64 = AtomicU64::new(1);
fn layout_id() -> Option<u64> {
    NEXT_LAYOUT_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .ok()
}

#[derive(Default)]
struct Patch {
    values: HashMap<usize, Atom>,
    properties: Vec<Property>,
    map_values: HashMap<usize, Atom>,
    entries: Vec<(Atom, Atom)>,
    length: Option<u32>,
}
pub(super) struct SourceCanonical {
    pub source: Arc<IndexedSankey>,
    admission: AdmissionState,
    layout_id: Option<u64>,
    version: u32,
    root: Atom,
    node_count: usize,
    string_count: usize,
    strings: HashMap<usize, JsString>,
    nodes: HashMap<usize, Node>,
    patches: HashMap<usize, Patch>,
}
impl SourceCanonical {
    pub fn new(source: IndexedSankey) -> Result<Self, PeleError> {
        let node_count = source.labels.len() + source.links.len() + 4;
        let string_count = source.catalog.len();
        let s = Self {
            source: Arc::new(source),
            admission: AdmissionState::default(),
            layout_id: layout_id(),
            version: 1,
            root: Atom::Ref((node_count - 1) as u32),
            node_count,
            string_count,
            strings: HashMap::new(),
            nodes: HashMap::new(),
            patches: HashMap::new(),
        };
        graph_read::validate_initial_structure(&s)?;
        Ok(s)
    }
    fn key(&self, key: &str) -> u32 {
        self.source.keys[key_slot(key)] as u32
    }
    fn base_header(&self, id: usize) -> Result<Header, PeleError> {
        let n = self.source.labels.len();
        let m = self.source.links.len();
        Ok(if id < n {
            Header {
                properties: 2,
                kind: AdmissionKind::Object,
            }
        } else if id == n {
            Header {
                properties: n,
                kind: AdmissionKind::Array(n as u32),
            }
        } else if id < n + m + 1 {
            Header {
                properties: 3,
                kind: AdmissionKind::Object,
            }
        } else if id == n + m + 1 {
            Header {
                properties: m,
                kind: AdmissionKind::Array(m as u32),
            }
        } else if id == n + m + 2 {
            Header {
                properties: 0,
                kind: AdmissionKind::Map(n),
            }
        } else if id == n + m + 3 {
            Header {
                properties: 7,
                kind: AdmissionKind::Instance(PrototypeId(1)),
            }
        } else {
            return Err(graph_read::invalid());
        })
    }
    fn base_property(&self, id: usize, slot: usize) -> Property {
        let n = self.source.labels.len();
        let m = self.source.links.len();
        let (key, value) = if id < n {
            if slot == 0 {
                (
                    self.source.keys[0] as u32,
                    Atom::String(self.source.labels[id] as u32),
                )
            } else {
                (self.source.keys[1] as u32, Atom::Number(id as f64))
            }
        } else if id == n {
            (self.source.indices[slot] as u32, Atom::Ref(slot as u32))
        } else if id < n + m + 1 {
            let link = &self.source.links[id - n - 1];
            match slot {
                0 => (self.source.keys[2] as u32, Atom::Ref(link.source as u32)),
                1 => (self.source.keys[3] as u32, Atom::Ref(link.target as u32)),
                _ => (self.source.keys[4] as u32, Atom::Number(link.value)),
            }
        } else if id == n + m + 1 {
            (
                self.source.indices[slot] as u32,
                Atom::Ref((n + 1 + slot) as u32),
            )
        } else {
            match slot {
                0 => (
                    self.source.keys[6] as u32,
                    Atom::String(self.source.keys[5] as u32),
                ),
                1 => (self.source.keys[7] as u32, Atom::Ref(n as u32)),
                2 => (self.source.keys[8] as u32, Atom::Ref((n + m + 1) as u32)),
                3 => (
                    self.source.keys[9] as u32,
                    self.source
                        .title
                        .map(|id| Atom::String(id as u32))
                        .unwrap_or(Atom::Undefined),
                ),
                4 => (self.source.keys[10] as u32, Atom::Undefined),
                5 => (self.source.keys[11] as u32, Atom::Undefined),
                _ => (self.source.keys[12] as u32, Atom::Ref((n + m + 2) as u32)),
            }
        };
        Property {
            key,
            value,
            flags: 7,
        }
    }
    fn property_at(&self, id: usize, slot: usize) -> Result<Property, PeleError> {
        if let Some(node) = self.nodes.get(&id) {
            return node
                .properties()
                .get(slot)
                .cloned()
                .ok_or_else(graph_read::invalid);
        }
        let base = self.base_header(id)?.properties;
        let patch = self.patches.get(&id);
        if slot >= base {
            return patch
                .and_then(|p| p.properties.get(slot - base))
                .cloned()
                .ok_or_else(graph_read::invalid);
        }
        let mut value = self.base_property(id, slot);
        if let Some(v) = patch.and_then(|p| p.values.get(&slot)) {
            value.value = v.clone();
        }
        Ok(value)
    }
    fn base_entry(&self, slot: usize) -> (Atom, Atom) {
        (
            Atom::String(self.source.labels[slot] as u32),
            Atom::Ref(slot as u32),
        )
    }
    fn entry_at(&self, id: usize, slot: usize) -> Result<(Atom, Atom), PeleError> {
        if let Some(Node::Map { entries, .. }) = self.nodes.get(&id) {
            return entries.get(slot).cloned().ok_or_else(graph_read::invalid);
        }
        let AdmissionKind::Map(base) = self.base_header(id)?.kind else {
            return Err(graph_read::invalid());
        };
        let patch = self.patches.get(&id);
        if slot >= base {
            return patch
                .and_then(|p| p.entries.get(slot - base))
                .cloned()
                .ok_or_else(graph_read::invalid);
        }
        let (key, value) = self.base_entry(slot);
        Ok((
            key,
            patch
                .and_then(|p| p.map_values.get(&slot))
                .cloned()
                .unwrap_or(value),
        ))
    }
    fn string_is(&self, id: usize, text: &str) -> bool {
        if let Some(s) = self.strings.get(&id) {
            return s.is(text);
        }
        id < self.string_count
            && self
                .source
                .catalog
                .get(id)
                .is_some_and(|e| e.text.as_str() == text)
    }
    fn same_string(&self, a: usize, b: usize) -> Result<bool, PeleError> {
        if a >= self.string_count || b >= self.string_count {
            return Err(graph_read::invalid());
        }
        Ok(match (self.strings.get(&a), self.strings.get(&b)) {
            (Some(a), Some(b)) => a == b,
            (Some(a), None) => a.is(self
                .source
                .catalog
                .get(b)
                .ok_or_else(graph_read::invalid)?
                .text
                .as_str()),
            (None, Some(b)) => b.is(self
                .source
                .catalog
                .get(a)
                .ok_or_else(graph_read::invalid)?
                .text
                .as_str()),
            (None, None) => {
                self.source
                    .catalog
                    .get(a)
                    .ok_or_else(graph_read::invalid)?
                    .text
                    .as_str()
                    == self
                        .source
                        .catalog
                        .get(b)
                        .ok_or_else(graph_read::invalid)?
                        .text
                        .as_str()
            }
        })
    }
    pub fn adopt(&mut self, graph: ModelGraph) -> Result<(), PeleError> {
        self.admission.invalidate();
        // Input is already structurally/schema admitted by the document. Compare
        // against immutable source, not the last observed overlay; retain only
        // actual differences and every appended/detached ID.
        let string_count = graph.strings.len();
        let mut strings = HashMap::new();
        for (id, text) in graph.strings.into_iter().enumerate() {
            let equal = self
                .source
                .catalog
                .get(id)
                .is_some_and(|e| text.0.iter().copied().eq(e.text.as_str().encode_utf16()));
            if !equal {
                strings.insert(id, text);
            }
        }
        let mut nodes = HashMap::new();
        let node_count = graph.nodes.len();
        for (id, node) in graph.nodes.into_iter().enumerate() {
            if !self.base_node_equal(id, &node) {
                nodes.insert(id, node);
            }
        }
        self.version = graph.version;
        self.root = graph.root;
        self.node_count = node_count;
        self.string_count = string_count;
        self.strings = strings;
        self.nodes = nodes;
        self.patches.clear();
        Ok(())
    }
    fn base_node_equal(&self, id: usize, node: &Node) -> bool {
        let Ok(h) = self.base_header(id) else {
            return false;
        };
        if !same_kind(h.kind, node) || node.properties().len() != h.properties {
            return false;
        }
        for (slot, p) in node.properties().iter().enumerate() {
            if !same_property(p, &self.base_property(id, slot)) {
                return false;
            }
        }
        if let Node::Map { entries, .. } = node {
            if entries.len() != self.source.labels.len() {
                return false;
            }
            for (i, (k, v)) in entries.iter().enumerate() {
                if !same_atom(k, &Atom::String(self.source.labels[i] as u32))
                    || !same_atom(v, &Atom::Ref(i as u32))
                {
                    return false;
                }
            }
        }
        true
    }
}
fn same_atom(a: &Atom, b: &Atom) -> bool {
    match (a, b) {
        (Atom::Number(a), Atom::Number(b)) => a.to_bits() == b.to_bits(),
        (Atom::Undefined, Atom::Undefined) | (Atom::Null, Atom::Null) => true,
        (Atom::Bool(a), Atom::Bool(b)) => a == b,
        (Atom::String(a), Atom::String(b))
        | (Atom::Ref(a), Atom::Ref(b))
        | (Atom::BuiltinFunction(a), Atom::BuiltinFunction(b))
        | (Atom::HostReference(a), Atom::HostReference(b)) => a == b,
        _ => false,
    }
}
fn same_property(a: &Property, b: &Property) -> bool {
    a.key == b.key && a.flags == b.flags && same_atom(&a.value, &b.value)
}
fn same_kind(k: AdmissionKind, n: &Node) -> bool {
    match (k, n) {
        (AdmissionKind::Object, Node::Object { .. }) => true,
        (AdmissionKind::Instance(a), Node::Instance { prototype: b, .. }) => a == *b,
        (AdmissionKind::Array(a), Node::Array { length: b, .. }) => a == *b,
        (AdmissionKind::Map(a), Node::Map { entries, .. }) => a == entries.len(),
        _ => false,
    }
}
impl TrustedGraphOwner for SourceCanonical {
    fn admission_state(&self) -> &AdmissionState {
        &self.admission
    }
}
impl TrustedInitialGraphOwner for SourceCanonical {
    type InitialRead<'a> = InitialRead<'a>;
    fn initial_read(&self) -> Self::InitialRead<'_> {
        InitialRead(self)
    }
}
pub(super) struct InitialRead<'a>(&'a SourceCanonical);
impl AdmissionRead for InitialRead<'_> {
    fn extent(&mut self) -> Result<Extent, PeleError> {
        Ok(PublicGraphRead::extent(self.0))
    }
    fn with_root<R>(
        &mut self,
        visit: impl FnOnce(&Atom) -> Result<R, PeleError>,
    ) -> Result<R, PeleError> {
        visit(&self.0.root)
    }
    fn visit_string_lengths(
        &mut self,
        mut visit: impl FnMut(usize) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        PublicGraphRead::string_lengths(self.0, &mut visit)
    }
    fn header(&mut self, node: usize) -> Result<Header, PeleError> {
        PublicGraphRead::header(self.0, node)
    }
    fn with_properties<R>(
        &mut self,
        node: usize,
        start: usize,
        visit: impl FnOnce(&[Property]) -> Result<R, PeleError>,
    ) -> Result<R, PeleError> {
        ReadAdapter(self.0).with_properties(node, start, visit)
    }
    fn with_map_entries<R>(
        &mut self,
        node: usize,
        start: usize,
        visit: impl FnOnce(&[(Atom, Atom)]) -> Result<R, PeleError>,
    ) -> Result<R, PeleError> {
        ReadAdapter(self.0).with_map_entries(node, start, visit)
    }
    fn with_set_values<R>(
        &mut self,
        node: usize,
        start: usize,
        visit: impl FnOnce(&[Atom]) -> Result<R, PeleError>,
    ) -> Result<R, PeleError> {
        ReadAdapter(self.0).with_set_values(node, start, visit)
    }
    fn visit_properties(
        &mut self,
        node: usize,
        start: usize,
        remaining: usize,
        mut visit: impl FnMut(ScalarPropertyRef<'_>) -> Result<(), PeleError>,
    ) -> Result<usize, PeleError> {
        // Defensive fallback keeps this reader exact even if reused after an
        // edit. Ordinary mutation paths do not issue initial-owner evidence.
        if self.0.nodes.contains_key(&node) || self.0.patches.contains_key(&node) {
            return ReadAdapter(self.0).visit_properties(node, start, remaining, visit);
        }
        let total = PublicGraphRead::header(self.0, node)?.properties;
        let end = start
            .checked_add(remaining)
            .filter(|end| *end <= total)
            .ok_or_else(graph_read::invalid)?;
        for slot in start..end {
            let property = self.0.base_property(node, slot);
            visit(ScalarPropertyRef {
                key: property.key,
                flags: property.flags,
                value: ScalarAtomRef::borrowed(&property.value),
            })?;
        }
        Ok(remaining)
    }
    fn visit_map_entries(
        &mut self,
        node: usize,
        start: usize,
        remaining: usize,
        mut visit: impl FnMut(ScalarAtomRef<'_>, ScalarAtomRef<'_>) -> Result<(), PeleError>,
    ) -> Result<usize, PeleError> {
        if self.0.nodes.contains_key(&node) || self.0.patches.contains_key(&node) {
            return ReadAdapter(self.0).visit_map_entries(node, start, remaining, visit);
        }
        let AdmissionKind::Map(total) = PublicGraphRead::header(self.0, node)?.kind else {
            return Err(graph_read::invalid());
        };
        let end = start
            .checked_add(remaining)
            .filter(|end| *end <= total)
            .ok_or_else(graph_read::invalid)?;
        for slot in start..end {
            let (key, value) = self.0.base_entry(slot);
            visit(
                ScalarAtomRef::borrowed(&key),
                ScalarAtomRef::borrowed(&value),
            )?;
        }
        Ok(remaining)
    }
}
impl PublicGraphRead for SourceCanonical {
    fn trusted_read(&self) -> Option<TrustedGraphRead<'_>> {
        Some(TrustedGraphRead::new(self))
    }
    fn source_layout(&self, revision: u64) -> Result<Option<Vec<u8>>, PeleError> {
        self.layout_packet(revision, false)
    }
    fn source_layout_update(&self, revision: u64) -> Result<Option<Vec<u8>>, PeleError> {
        self.layout_packet(revision, true)
    }
    fn extent(&self) -> Extent {
        Extent {
            version: self.version,
            nodes: self.node_count,
            strings: self.string_count,
        }
    }
    fn root(&self) -> Atom {
        self.root.clone()
    }
    fn header(&self, id: usize) -> Result<Header, PeleError> {
        if id >= self.node_count {
            return Err(graph_read::invalid());
        }
        if let Some(node) = self.nodes.get(&id) {
            return node_header(node);
        }
        let mut h = self.base_header(id)?;
        if let Some(p) = self.patches.get(&id) {
            h.properties += p.properties.len();
            if let AdmissionKind::Map(n) = &mut h.kind {
                *n += p.entries.len()
            }
            if let Some(n) = p.length {
                h.kind = AdmissionKind::Array(n)
            }
        }
        Ok(h)
    }
    fn string_length(&self, id: usize) -> Result<usize, PeleError> {
        if id >= self.string_count {
            return Err(graph_read::invalid());
        }
        self.strings
            .get(&id)
            .map(|s| s.0.len())
            .or_else(|| self.source.catalog.get(id).map(|e| e.units))
            .ok_or_else(graph_read::invalid)
    }
    fn string_lengths(
        &self,
        v: &mut dyn FnMut(usize) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        for id in 0..self.string_count {
            v(PublicGraphRead::string_length(self, id)?)?;
        }
        Ok(())
    }
    fn string_chunks(
        &self,
        id: usize,
        v: &mut dyn FnMut(&[u16]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        if id >= self.string_count {
            return Err(graph_read::invalid());
        }
        if let Some(s) = self.strings.get(&id) {
            for c in s.0.chunks(1024) {
                v(c)?;
            }
            return Ok(());
        }
        let text = self
            .source
            .catalog
            .get(id)
            .ok_or_else(graph_read::invalid)?
            .text
            .as_str();
        let mut chunk = [0u16; 1024];
        let mut n = 0;
        for unit in text.encode_utf16() {
            chunk[n] = unit;
            n += 1;
            if n == chunk.len() {
                v(&chunk)?;
                n = 0
            }
        }
        if n != 0 {
            v(&chunk[..n])?;
        }
        Ok(())
    }
    fn properties(
        &self,
        id: usize,
        start: usize,
        v: &mut dyn FnMut(&[Property]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        if let Some(n) = self.nodes.get(&id) {
            return v(n
                .properties()
                .get(start..)
                .ok_or_else(graph_read::invalid)?);
        }
        if id >= self.node_count {
            return Err(graph_read::invalid());
        }
        let base = self.base_header(id)?.properties;
        let patch = self.patches.get(&id);
        let total = base + patch.map_or(0, |p| p.properties.len());
        if start > total {
            return Err(graph_read::invalid());
        }
        let count = (total - start).min(128);
        if count == 0 {
            return v(&[]);
        }
        if count <= 8 {
            self.property_batch::<8>(id, base, patch, start, count, v)
        } else {
            self.property_batch::<128>(id, base, patch, start, count, v)
        }
    }
    fn map_entries(
        &self,
        id: usize,
        start: usize,
        v: &mut dyn FnMut(&[(Atom, Atom)]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        if id >= self.node_count {
            return Err(graph_read::invalid());
        }
        if let Some(node) = self.nodes.get(&id) {
            let Node::Map { entries, .. } = node else {
                return Err(graph_read::invalid());
            };
            return v(entries.get(start..).ok_or_else(graph_read::invalid)?);
        }
        let AdmissionKind::Map(base) = self.base_header(id)?.kind else {
            return Err(graph_read::invalid());
        };
        let patch = self.patches.get(&id);
        let total = base + patch.map_or(0, |p| p.entries.len());
        if start > total {
            return Err(graph_read::invalid());
        }
        let count = (total - start).min(128);
        if count == 0 {
            return v(&[]);
        }
        if count <= 8 {
            self.map_batch::<8>(base, patch, start, count, v)
        } else {
            self.map_batch::<128>(base, patch, start, count, v)
        }
    }
    fn set_values(
        &self,
        id: usize,
        start: usize,
        v: &mut dyn FnMut(&[Atom]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        let Some(Node::Set { values, .. }) = self.nodes.get(&id) else {
            return Err(graph_read::invalid());
        };
        v(values.get(start..).ok_or_else(graph_read::invalid)?)
    }
}
impl SourceCanonical {
    fn property_batch<const CAPACITY: usize>(
        &self,
        id: usize,
        base: usize,
        patch: Option<&Patch>,
        start: usize,
        count: usize,
        visit: &mut dyn FnMut(&[Property]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        let mut batch: [Property; CAPACITY] = std::array::from_fn(|_| Property {
            key: 0,
            flags: 0,
            value: Atom::Undefined,
        });
        for (offset, property) in batch[..count].iter_mut().enumerate() {
            let slot = start + offset;
            if slot < base {
                *property = self.base_property(id, slot);
                if let Some(value) = patch.and_then(|p| p.values.get(&slot)) {
                    property.value = value.clone();
                }
            } else {
                *property = patch
                    .and_then(|p| p.properties.get(slot - base))
                    .cloned()
                    .ok_or_else(graph_read::invalid)?;
            }
        }
        visit(&batch[..count])
    }
    fn map_batch<const CAPACITY: usize>(
        &self,
        base: usize,
        patch: Option<&Patch>,
        start: usize,
        count: usize,
        visit: &mut dyn FnMut(&[(Atom, Atom)]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        let mut batch: [(Atom, Atom); CAPACITY] =
            std::array::from_fn(|_| (Atom::Undefined, Atom::Undefined));
        for (offset, entry) in batch[..count].iter_mut().enumerate() {
            let slot = start + offset;
            *entry = if slot < base {
                let (key, value) = self.base_entry(slot);
                (
                    key,
                    patch
                        .and_then(|p| p.map_values.get(&slot))
                        .cloned()
                        .unwrap_or(value),
                )
            } else {
                patch
                    .and_then(|p| p.entries.get(slot - base))
                    .cloned()
                    .ok_or_else(graph_read::invalid)?
            };
        }
        visit(&batch[..count])
    }
}
fn node_header(n: &Node) -> Result<Header, PeleError> {
    Ok(Header {
        properties: n.properties().len(),
        kind: match n {
            Node::Object { .. } => AdmissionKind::Object,
            Node::Instance { prototype, .. } => AdmissionKind::Instance(*prototype),
            Node::Array { length, .. } => AdmissionKind::Array(*length),
            Node::Map { entries, .. } => AdmissionKind::Map(entries.len()),
            Node::Set { values, .. } => AdmissionKind::Set(values.len()),
            Node::Date { milliseconds, .. } => AdmissionKind::Date(milliseconds.to_bits()),
            Node::Error { error_type, .. } => AdmissionKind::Error(*error_type),
        },
    })
}
impl CapturedRead for SourceCanonical {
    type Value = Atom;
    fn root(&self) -> Atom {
        self.root.clone()
    }
    fn header(&mut self, id: u32) -> Result<Option<NodeHeader>, PeleError> {
        if id as usize >= self.node_count {
            return Ok(None);
        }
        let h = PublicGraphRead::header(self, id as usize)?;
        let (kind, array_length) = match h.kind {
            AdmissionKind::Object => (NodeKind::Object, None),
            AdmissionKind::Instance(_) => (NodeKind::Instance, None),
            AdmissionKind::Array(n) => (NodeKind::Array, Some(n)),
            AdmissionKind::Map(_) => (NodeKind::Map, None),
            AdmissionKind::Set(_) => (NodeKind::Set, None),
            AdmissionKind::Date(_) => (NodeKind::Date, None),
            AdmissionKind::Error(_) => (NodeKind::Error, None),
            _ => return Err(graph_read::invalid()),
        };
        Ok(Some(NodeHeader {
            kind,
            array_length,
            property_count: h.properties as u32,
        }))
    }
    fn own_fields(
        &mut self,
        id: u32,
        names: &[&str],
        out: &mut [Option<Atom>],
    ) -> Result<(), PeleError> {
        assert_eq!(names.len(), out.len());
        out.fill(None);
        if id as usize >= self.node_count {
            return Ok(());
        }
        let mut remaining = names.len();
        if remaining == 0 {
            return Ok(());
        }
        let total = PublicGraphRead::header(self, id as usize)?.properties;
        for slot in 0..total {
            let p = self.property_at(id as usize, slot)?;
            for (name, value) in names.iter().zip(out.iter_mut()) {
                if value.is_none() && self.string_is(p.key as usize, name) {
                    *value = Some(p.value.clone());
                    remaining -= 1
                }
            }
            if remaining == 0 {
                break;
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
        for p in out.iter_mut() {
            *p = PropertyPrefix::default()
        }
        let h = PublicGraphRead::header(self, id as usize)?;
        let start = start as usize;
        if start > h.properties {
            return Err(graph_read::invalid());
        }
        let count = (h.properties - start).min(out.len()).min(128);
        for (i, p) in out[..count].iter_mut().enumerate() {
            let raw = self.property_at(id as usize, start + i)?;
            p.key_units = PublicGraphRead::string_length(self, raw.key as usize)? as u32;
            let mut cursor = 0;
            self.string_chunks(raw.key as usize, &mut |chunk| {
                for unit in chunk {
                    if cursor < 10 {
                        p.prefix[cursor] = *unit;
                        cursor += 1
                    } else {
                        break;
                    }
                }
                Ok(())
            })?;
            p.value = Some(raw.value);
        }
        Ok(count)
    }
    fn string_length(&mut self, id: u32) -> Result<u32, PeleError> {
        Ok(PublicGraphRead::string_length(self, id as usize)? as u32)
    }
    fn with_string_chunks(
        &mut self,
        id: u32,
        mut v: impl FnMut(&[u16]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        self.string_chunks(id as usize, &mut v)
    }
}
impl CapturedWrite for SourceCanonical {
    fn counts(&mut self) -> Result<GraphCounts, PeleError> {
        let mut units = 0usize;
        self.string_lengths(&mut |n| {
            units = units.saturating_add(n);
            Ok(())
        })?;
        let mut work = 0usize;
        for id in 0..self.node_count {
            let h = PublicGraphRead::header(self, id)?;
            work = work
                .saturating_add(h.properties)
                .saturating_add(match h.kind {
                    AdmissionKind::Map(n) => n.saturating_mul(2),
                    AdmissionKind::Set(n) => n,
                    _ => 0,
                });
        }
        Ok(GraphCounts {
            nodes: self.node_count,
            strings: self.string_count,
            units,
            work,
        })
    }
    fn find_strings(&mut self, names: &[&str], out: &mut [Option<u32>]) -> Result<(), PeleError> {
        assert_eq!(names.len(), out.len());
        out.fill(None);
        for id in 0..self.string_count {
            for (name, value) in names.iter().zip(out.iter_mut()) {
                if value.is_none() && self.string_is(id, name) {
                    *value = Some(id as u32)
                }
            }
            if out.iter().all(Option::is_some) {
                break;
            }
        }
        Ok(())
    }
    fn intern(&mut self, text: &str) -> Result<u32, PeleError> {
        for id in 0..self.string_count {
            if self.string_is(id, text) {
                return Ok(id as u32);
            }
        }
        let id = self.string_count;
        self.admission.invalidate();
        self.strings.insert(id, JsString::from(text));
        self.string_count += 1;
        Ok(id as u32)
    }
    fn append_node(&mut self, node: Node) -> Result<u32, PeleError> {
        let id = self.node_count;
        self.admission.invalidate();
        self.nodes.insert(id, node);
        self.node_count += 1;
        Ok(id as u32)
    }
    fn own_descriptors(
        &mut self,
        id: u32,
        names: &[&str],
        out: &mut [OwnQuery],
    ) -> Result<(), PeleError> {
        assert_eq!(names.len(), out.len());
        for o in out.iter_mut() {
            *o = OwnQuery::default()
        }
        if id as usize >= self.node_count {
            return Ok(());
        }
        for slot in 0..PublicGraphRead::header(self, id as usize)?.properties {
            let p = self.property_at(id as usize, slot)?;
            for (name, o) in names.iter().zip(out.iter_mut()) {
                if self.string_is(p.key as usize, name) {
                    o.first_slot.get_or_insert(slot as u32);
                    o.any_nonwritable |= p.flags & 2 == 0;
                }
            }
        }
        Ok(())
    }
    fn map_find(&mut self, id: u32, key: &Atom) -> Result<Option<MapMatch>, PeleError> {
        let AdmissionKind::Map(n) = PublicGraphRead::header(self, id as usize)?.kind else {
            return Err(graph_read::invalid());
        };
        for slot in 0..n {
            let (k, v) = self.entry_at(id as usize, slot)?;
            let same = match (&k, key) {
                (Atom::Number(a), Atom::Number(b)) => a == b || (a.is_nan() && b.is_nan()),
                (Atom::String(a), Atom::String(b)) => self.same_string(*a as usize, *b as usize)?,
                _ => same_atom(&k, key),
            };
            if same {
                return Ok(Some(MapMatch {
                    slot: slot as u32,
                    value: v,
                }));
            }
        }
        Ok(None)
    }
    fn replace_map_value(&mut self, id: u32, slot: u32, value: Atom) -> Result<(), PeleError> {
        self.admission.invalidate();
        let id = id as usize;
        let slot = slot as usize;
        if let Some(Node::Map { entries, .. }) = self.nodes.get_mut(&id) {
            entries[slot].1 = value;
            return Ok(());
        }
        let AdmissionKind::Map(base) = self.base_header(id)?.kind else {
            return Err(graph_read::invalid());
        };
        let p = self.patches.entry(id).or_default();
        if slot < base {
            p.map_values.insert(slot, value);
        } else {
            p.entries[slot - base].1 = value;
        }
        Ok(())
    }
    fn append_map_entry(&mut self, id: u32, key: Atom, value: Atom) -> Result<(), PeleError> {
        self.admission.invalidate();
        if let Some(Node::Map { entries, .. }) = self.nodes.get_mut(&(id as usize)) {
            entries.push((key, value))
        } else {
            self.patches
                .entry(id as usize)
                .or_default()
                .entries
                .push((key, value))
        }
        Ok(())
    }
    fn replace_property_value(&mut self, id: u32, slot: u32, value: Atom) -> Result<(), PeleError> {
        self.admission.invalidate();
        let id = id as usize;
        let slot = slot as usize;
        if let Some(n) = self.nodes.get_mut(&id) {
            n.properties_mut()[slot].value = value;
            return Ok(());
        }
        let base = self.base_header(id)?.properties;
        let p = self.patches.entry(id).or_default();
        if slot < base {
            p.values.insert(slot, value);
        } else {
            p.properties[slot - base].value = value;
        }
        Ok(())
    }
    fn append_property(&mut self, id: u32, p: Property) -> Result<(), PeleError> {
        self.admission.invalidate();
        if let Some(n) = self.nodes.get_mut(&(id as usize)) {
            n.properties_mut().push(p)
        } else {
            self.patches
                .entry(id as usize)
                .or_default()
                .properties
                .push(p)
        }
        Ok(())
    }
    fn replace_matching_property_values(
        &mut self,
        id: u32,
        key: &str,
        value: Atom,
    ) -> Result<(), PeleError> {
        for slot in 0..PublicGraphRead::header(self, id as usize)?.properties {
            let p = self.property_at(id as usize, slot)?;
            if self.string_is(p.key as usize, key) {
                self.replace_property_value(id, slot as u32, value.clone())?
            }
        }
        Ok(())
    }
    fn set_array_length(&mut self, id: u32, length: u32) -> Result<(), PeleError> {
        self.admission.invalidate();
        if let Some(Node::Array { length: n, .. }) = self.nodes.get_mut(&(id as usize)) {
            *n = length
        } else {
            self.patches.entry(id as usize).or_default().length = Some(length)
        }
        Ok(())
    }
}

// Generic declarative recipe. The decoder knows only record layouts, affine
// indexes and scalar columns; no diagram grammar or helper policy crosses it.
#[derive(Clone, Copy)]
struct Expr {
    op: u8,
    tag: u8,
    a: u32,
    b: u32,
}
impl Expr {
    fn literal(a: &Atom) -> Self {
        let (tag, a, b) = words(a);
        Self { op: 0, tag, a, b }
    }
    fn column(column: u32, item: bool) -> Self {
        Self {
            op: if item { 4 } else { 3 },
            tag: 0,
            a: column,
            b: 0,
        }
    }
    fn affine(tag: u8, base: u32, item: bool) -> Self {
        Self {
            op: if item { 2 } else { 1 },
            tag,
            a: base,
            b: 1,
        }
    }
}
fn words(a: &Atom) -> (u8, u32, u32) {
    match a {
        Atom::Undefined => (0, 0, 0),
        Atom::Null => (1, 0, 0),
        Atom::Bool(false) => (2, 0, 0),
        Atom::Bool(true) => (3, 0, 0),
        Atom::Number(n) => {
            let b = n.to_bits();
            (4, b as u32, (b >> 32) as u32)
        }
        Atom::String(n) => (5, *n, 0),
        Atom::Ref(n) => (6, *n, 0),
        Atom::BuiltinFunction(n) => (7, *n, 0),
        Atom::HostReference(n) => (8, *n, 0),
    }
}
fn atom9(w: &mut crate::compat::public::binary::Writer, a: &Atom) -> Result<(), PeleError> {
    let (t, lo, hi) = words(a);
    w.byte(t)?;
    w.u32(lo)?;
    w.u32(hi)
}
// Homogeneous scalar transport, independent of record/group semantics. IDs
// are already u32; numbers arrive as raw IEEE bits and never round-trip through
// an intermediate floating-point value here. Legacy fixed atoms share this
// exact source iterator, retaining the identity-exhaustion wire unchanged.
#[derive(Clone, Copy)]
enum ColumnKind {
    String,
    Reference,
    Number,
}
impl ColumnKind {
    fn tag(self) -> u8 {
        match self {
            Self::String => 5,
            Self::Reference => 6,
            Self::Number => 4,
        }
    }
    fn width(self) -> usize {
        if matches!(self, Self::Number) {
            8
        } else {
            4
        }
    }
}
fn column(
    w: &mut crate::compat::public::binary::Writer,
    typed: bool,
    kind: ColumnKind,
    values: impl ExactSizeIterator<Item = u64>,
) -> Result<(), PeleError> {
    let count = values.len();
    if typed {
        w.byte(kind.tag())?;
    }
    w.count(count)?;
    if typed {
        let padding = (4 - w.bytes.len() % 4) % 4;
        w.put(&[0; 3][..padding])?;
    }
    let width = if typed { kind.width() } else { 9 };
    // The recipe writer is capped at 64MiB. Overflow maps to a size guaranteed
    // to fail the same existing byte-limit predicate, without allocating it.
    w.check_additional(count.checked_mul(width).unwrap_or(usize::MAX))?;
    for value in values {
        if !typed {
            w.byte(kind.tag())?;
        }
        w.u32(value as u32)?;
        if !typed || matches!(kind, ColumnKind::Number) {
            w.u32((value >> 32) as u32)?;
        }
    }
    Ok(())
}
fn expr(w: &mut crate::compat::public::binary::Writer, e: Expr) -> Result<(), PeleError> {
    w.byte(e.op)?;
    w.byte(e.tag)?;
    w.u32(e.a)?;
    w.u32(e.b)
}
fn group(
    w: &mut crate::compat::public::binary::Writer,
    first: usize,
    count: usize,
    kind: u8,
    aux: u32,
    props: &[(Expr, Expr)],
    dynamic: Option<(usize, Expr, Expr)>,
    map: Option<(usize, Expr, Expr)>,
) -> Result<(), PeleError> {
    w.count(first)?;
    w.count(count)?;
    w.byte(kind)?;
    w.u32(aux)?;
    w.u32(0)?;
    w.count(props.len())?;
    for (k, v) in props {
        expr(w, *k)?;
        w.byte(7)?;
        expr(w, *v)?;
    }
    w.count(usize::from(dynamic.is_some()))?;
    if let Some((n, k, v)) = dynamic {
        w.count(n)?;
        expr(w, k)?;
        w.byte(7)?;
        expr(w, v)?;
    }
    w.count(usize::from(map.is_some()))?;
    if let Some((n, k, v)) = map {
        w.count(n)?;
        expr(w, k)?;
        expr(w, v)?;
    }
    Ok(())
}
impl SourceCanonical {
    fn layout_packet(&self, revision: u64, update: bool) -> Result<Option<Vec<u8>>, PeleError> {
        match self.recipe(revision, update) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.code == crate::compat::ErrorCode::Limit => Ok(None),
            Err(e) => Err(e),
        }
    }
    fn recipe(&self, revision: u64, update: bool) -> Result<Vec<u8>, PeleError> {
        use crate::compat::public::binary::{write_read_node, Writer, DEFAULT_BINARY_BUDGET};
        let mut w = Writer {
            bytes: Vec::new(),
            budget: DEFAULT_BINARY_BUDGET,
        };
        let update = update && self.layout_id.is_some();
        if let Some(id) = self.layout_id {
            w.put(if update { b"PLCSRU01" } else { b"PLCSRC03" })?;
            w.put(&id.to_le_bytes())?;
        } else {
            w.put(b"PLCSRC01")?;
        }
        w.put(&revision.to_le_bytes())?;
        w.u32(self.version)?;
        atom9(&mut w, &self.root)?;
        w.count(self.string_count)?;
        w.count(self.node_count)?;
        if !update {
            if self.layout_id.is_some() {
                // A full frame carries the original base even after replacement
                // or truncation. Current changes follow as complete patches.
                w.count(self.source.catalog.len())?;
                for entry in &self.source.catalog {
                    w.count(entry.units)?;
                    for unit in entry.text.as_str().encode_utf16() {
                        w.put(&unit.to_le_bytes())?;
                    }
                }
            } else {
                // Exhausted identity space retains the prior self-contained ABI.
                w.count(self.string_count)?;
                for id in 0..self.string_count {
                    w.count(PublicGraphRead::string_length(self, id)?)?;
                    self.string_chunks(id, &mut |units| {
                        for unit in units {
                            w.put(&unit.to_le_bytes())?;
                        }
                        Ok(())
                    })?;
                }
            }
            // Five scalar columns: labels, source, target, value, decimal-key IDs.
            // Only full packets with a layout identity use the typed03 ABI.
            let typed = self.layout_id.is_some();
            w.count(5)?;
            column(
                &mut w,
                typed,
                ColumnKind::String,
                self.source.labels.iter().map(|id| u64::from(*id as u32)),
            )?;
            column(
                &mut w,
                typed,
                ColumnKind::Reference,
                self.source
                    .links
                    .iter()
                    .map(|link| u64::from(link.source as u32)),
            )?;
            column(
                &mut w,
                typed,
                ColumnKind::Reference,
                self.source
                    .links
                    .iter()
                    .map(|link| u64::from(link.target as u32)),
            )?;
            column(
                &mut w,
                typed,
                ColumnKind::Number,
                self.source.links.iter().map(|link| link.value.to_bits()),
            )?;
            column(
                &mut w,
                typed,
                ColumnKind::String,
                self.source.indices.iter().map(|id| u64::from(*id as u32)),
            )?;
            let n = self.source.labels.len();
            let m = self.source.links.len();
            let key = |k: &str| Expr::literal(&Atom::String(self.key(k)));
            w.count(6)?;
            let node_props = if n == 0 {
                vec![]
            } else {
                vec![
                    (key("id"), Expr::column(0, false)),
                    (
                        key("index"),
                        Expr {
                            op: 5,
                            tag: 4,
                            a: 0,
                            b: 1,
                        },
                    ),
                ]
            };
            group(&mut w, 0, n, 0, 0, &node_props, None, None)?;
            group(
                &mut w,
                n,
                1,
                2,
                n as u32,
                &[],
                Some((n, Expr::column(4, true), Expr::affine(6, 0, true))),
                None,
            )?;
            // Empty link groups do not name absent source/target/value string keys.
            let link_props = if m == 0 {
                vec![]
            } else {
                vec![
                    (key("source"), Expr::column(1, false)),
                    (key("target"), Expr::column(2, false)),
                    (key("value"), Expr::column(3, false)),
                ]
            };
            group(&mut w, n + 1, m, 0, 0, &link_props, None, None)?;
            group(
                &mut w,
                n + m + 1,
                1,
                2,
                m as u32,
                &[],
                Some((
                    m,
                    Expr::column(4, true),
                    Expr::affine(6, (n + 1) as u32, true),
                )),
                None,
            )?;
            group(
                &mut w,
                n + m + 2,
                1,
                3,
                0,
                &[],
                None,
                Some((n, Expr::column(0, true), Expr::affine(6, 0, true))),
            )?;
            let mut root_props = Vec::with_capacity(7);
            for slot in 0..7 {
                let p = self.base_property(n + m + 3, slot);
                root_props.push((Expr::literal(&Atom::String(p.key)), Expr::literal(&p.value)));
            }
            group(&mut w, n + m + 3, 1, 1, 1, &root_props, None, None)?;
        }
        if self.layout_id.is_some() {
            let mut changed_strings: Vec<_> = self
                .strings
                .keys()
                .copied()
                .filter(|id| *id < self.string_count)
                .collect();
            changed_strings.sort_unstable();
            w.count(changed_strings.len())?;
            for id in changed_strings {
                let value = &self.strings[&id];
                w.count(id)?;
                w.count(value.0.len())?;
                for unit in &value.0 {
                    w.put(&unit.to_le_bytes())?;
                }
            }
        }
        let mut changed: Vec<_> = self
            .nodes
            .keys()
            .chain(self.patches.keys())
            .copied()
            .filter(|id| *id < self.node_count)
            .collect();
        changed.sort_unstable();
        changed.dedup();
        w.count(changed.len())?;
        for id in changed {
            w.count(id)?;
            write_read_node(&mut w, self, id)?;
        }
        Ok(w.bytes)
    }
}

/// Incoming replacements shadow the ORIGINAL source, not current accepted
/// patches. A rejected observation can therefore be repaired without replaying
/// an observed-baseline delta against a different native state.
pub(super) struct SourceCandidate<'a> {
    base: &'a mut SourceCanonical,
    input: &'a crate::compat::public::source_input::SourceOverlay,
}
impl SourceCanonical {
    pub(super) fn ingress_id(&self) -> Option<u64> {
        self.layout_id
    }
    pub(super) fn candidate<'a>(
        &'a mut self,
        input: &'a crate::compat::public::source_input::SourceOverlay,
    ) -> Result<SourceCandidate<'a>, PeleError> {
        if self.layout_id != Some(input.layout_id) {
            return Err(graph_read::invalid());
        }
        Ok(SourceCandidate { base: self, input })
    }
    /// Scan borrowed replacements twice without any count-proportional
    /// allocation: complete canonical width first, then the decoder's ordered
    /// table/UTF16/work gates. Only bounded, admitted payloads are expanded.
    pub(super) fn prepare_overlay(
        &self,
        input: crate::compat::public::source_input::BorrowedOverlay<'_>,
    ) -> Result<crate::compat::public::source_input::SourceOverlay, PeleError> {
        use crate::compat::public::{
            binary,
            captured_transport::{Counter, DecoderBudget},
        };
        if self.layout_id != Some(input.layout_id) {
            return Err(graph_read::invalid());
        }
        let mut size = Counter::new(binary::DEFAULT_BINARY_BUDGET);
        size.prefix(0)?;
        size.atom(&input.root)?;
        size.count(input.string_count)?;
        self.overlay_string_lengths(&input, |units| size.string(units))?;
        size.count(input.node_count)?;
        self.overlay_node_headers(&input, |id, header, raw| {
            if let Some(raw) = raw {
                // This is the already-scanned canonical body, with no metadata
                // or sparse directory wrapper. Counting it never expands atoms.
                return size.put(raw.len());
            }
            size.node_prefix(header.kind)?;
            if let AdmissionKind::Map(count) = header.kind {
                for slot in 0..count {
                    let (key, value) = self.base_entry(slot);
                    size.atom(&key)?;
                    size.atom(&value)?;
                }
            }
            size.count(header.properties)?;
            for slot in 0..header.properties {
                size.property(&self.base_property(id, slot).value)?;
            }
            Ok(())
        })?;
        let mut limits = DecoderBudget::new(Extent {
            version: input.version,
            strings: input.string_count,
            nodes: input.node_count,
        })?;
        self.overlay_string_lengths(&input, |units| limits.string(units))?;
        limits.begin_nodes()?;
        self.overlay_node_headers(&input, |_, header, _| limits.node(header))?;
        input.materialize()
    }
    fn overlay_string_lengths(
        &self,
        input: &crate::compat::public::source_input::BorrowedOverlay<'_>,
        mut visit: impl FnMut(usize) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        let mut entries = input.string_entries();
        let mut next = entries.next().transpose()?;
        for id in 0..input.string_count {
            let units = if next.as_ref().is_some_and(|(key, _)| *key == id) {
                let (_, raw) = next.take().expect("matched replacement");
                next = entries.next().transpose()?;
                raw.len() / 2
            } else {
                self.source
                    .catalog
                    .get(id)
                    .ok_or_else(graph_read::invalid)?
                    .units
            };
            visit(units)?;
        }
        if next.is_some() {
            return Err(graph_read::invalid());
        }
        Ok(())
    }
    fn overlay_node_headers(
        &self,
        input: &crate::compat::public::source_input::BorrowedOverlay<'_>,
        mut visit: impl FnMut(usize, Header, Option<&[u8]>) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        let mut entries = input.node_entries();
        let mut next = entries.next().transpose()?;
        for id in 0..input.node_count {
            if next.as_ref().is_some_and(|(key, _)| *key == id) {
                let (_, node) = next.take().expect("matched replacement");
                visit(id, node.header, Some(node.raw))?;
                next = entries.next().transpose()?;
            } else {
                visit(id, self.base_header(id)?, None)?;
            }
        }
        if next.is_some() {
            return Err(graph_read::invalid());
        }
        Ok(())
    }
    pub(super) fn adopt_overlay(
        &mut self,
        input: crate::compat::public::source_input::SourceOverlay,
    ) {
        self.admission.invalidate();
        self.version = input.version;
        self.root = input.root;
        self.string_count = input.string_count;
        self.node_count = input.node_count;
        self.strings = input.strings;
        self.nodes = input.nodes;
        self.patches.clear();
    }
}
impl crate::compat::public::source_input::LogicalCapturedText for SourceCandidate<'_> {
    fn string_equals(&self, id: u32, text: &str) -> Result<bool, PeleError> {
        let id = id as usize;
        if id >= self.input.string_count {
            return Err(graph_read::invalid());
        }
        if let Some(value) = self.input.strings.get(&id) {
            return Ok(value.is(text));
        }
        let entry = self
            .base
            .source
            .catalog
            .get(id)
            .ok_or_else(graph_read::invalid)?;
        Ok(entry.text.as_str() == text)
    }
    fn original_utf8(&self, id: u32) -> Result<Option<&str>, PeleError> {
        let id = id as usize;
        if id >= self.input.string_count {
            return Err(graph_read::invalid());
        }
        if self.input.strings.contains_key(&id) {
            return Ok(None);
        }
        let entry = self
            .base
            .source
            .catalog
            .get(id)
            .ok_or_else(graph_read::invalid)?;
        Ok(Some(entry.text.as_str()))
    }
}
impl PublicGraphRead for SourceCandidate<'_> {
    fn extent(&self) -> Extent {
        Extent {
            version: self.input.version,
            strings: self.input.string_count,
            nodes: self.input.node_count,
        }
    }
    fn root(&self) -> Atom {
        self.input.root.clone()
    }
    fn header(&self, id: usize) -> Result<Header, PeleError> {
        if id >= self.input.node_count {
            return Err(graph_read::invalid());
        }
        match self.input.nodes.get(&id) {
            Some(node) => node_header(node),
            None => self.base.base_header(id),
        }
    }
    fn string_length(&self, id: usize) -> Result<usize, PeleError> {
        if id >= self.input.string_count {
            return Err(graph_read::invalid());
        }
        self.input
            .strings
            .get(&id)
            .map(|s| s.0.len())
            .or_else(|| self.base.source.catalog.get(id).map(|e| e.units))
            .ok_or_else(graph_read::invalid)
    }
    fn string_lengths(
        &self,
        visit: &mut dyn FnMut(usize) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        for id in 0..self.input.string_count {
            visit(self.string_length(id)?)?;
        }
        Ok(())
    }
    fn string_chunks(
        &self,
        id: usize,
        visit: &mut dyn FnMut(&[u16]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        if id >= self.input.string_count {
            return Err(graph_read::invalid());
        }
        if let Some(text) = self.input.strings.get(&id) {
            for chunk in text.0.chunks(1024) {
                visit(chunk)?;
            }
            return Ok(());
        }
        let text = self
            .base
            .source
            .catalog
            .get(id)
            .ok_or_else(graph_read::invalid)?
            .text
            .as_str();
        let mut chunk = [0u16; 1024];
        let mut used = 0;
        for unit in text.encode_utf16() {
            chunk[used] = unit;
            used += 1;
            if used == chunk.len() {
                visit(&chunk)?;
                used = 0;
            }
        }
        if used != 0 {
            visit(&chunk[..used])?;
        }
        Ok(())
    }
    fn properties(
        &self,
        id: usize,
        start: usize,
        visit: &mut dyn FnMut(&[Property]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        if id >= self.input.node_count {
            return Err(graph_read::invalid());
        }
        if let Some(node) = self.input.nodes.get(&id) {
            return visit(
                node.properties()
                    .get(start..)
                    .ok_or_else(graph_read::invalid)?,
            );
        }
        let total = self.base.base_header(id)?.properties;
        if start > total {
            return Err(graph_read::invalid());
        }
        let count = (total - start).min(128);
        if count == 0 {
            return visit(&[]);
        }
        if count <= 8 {
            self.base
                .property_batch::<8>(id, total, None, start, count, visit)
        } else {
            self.base
                .property_batch::<128>(id, total, None, start, count, visit)
        }
    }
    fn map_entries(
        &self,
        id: usize,
        start: usize,
        visit: &mut dyn FnMut(&[(Atom, Atom)]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        if id >= self.input.node_count {
            return Err(graph_read::invalid());
        }
        if let Some(node) = self.input.nodes.get(&id) {
            let Node::Map { entries, .. } = node else {
                return Err(graph_read::invalid());
            };
            return visit(entries.get(start..).ok_or_else(graph_read::invalid)?);
        }
        let AdmissionKind::Map(total) = self.base.base_header(id)?.kind else {
            return Err(graph_read::invalid());
        };
        if start > total {
            return Err(graph_read::invalid());
        }
        let count = (total - start).min(128);
        if count == 0 {
            return visit(&[]);
        }
        if count <= 8 {
            self.base.map_batch::<8>(total, None, start, count, visit)
        } else {
            self.base.map_batch::<128>(total, None, start, count, visit)
        }
    }
    fn set_values(
        &self,
        id: usize,
        start: usize,
        visit: &mut dyn FnMut(&[Atom]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        if id >= self.input.node_count {
            return Err(graph_read::invalid());
        }
        let Some(Node::Set { values, .. }) = self.input.nodes.get(&id) else {
            return Err(graph_read::invalid());
        };
        visit(values.get(start..).ok_or_else(graph_read::invalid)?)
    }
}
