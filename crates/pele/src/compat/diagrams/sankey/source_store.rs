//! Source-owned logical graph. Static records and collection slots are derived;
//! only actual mutations own generic records. No source Property/Node arena.
use super::indexed::IndexedSankey;
use crate::compat::public::captured_admission::{AdmissionKind, Extent, Header};
use crate::compat::public::captured_read::{CapturedRead, NodeHeader, NodeKind, PropertyPrefix};
use crate::compat::public::captured_write::{CapturedWrite, GraphCounts, MapMatch, OwnQuery};
use crate::compat::public::graph_read::{
    self, AdmissionState, PublicGraphRead, TrustedGraphOwner, TrustedGraphRead,
};
use crate::compat::public::{Atom, JsString, ModelGraph, Node, Property, PrototypeId};
use crate::compat::PeleError;
use std::{collections::HashMap, sync::Arc};

enum Text {
    Label(usize),
    Title,
    Static(&'static str),
    Decimal(String),
}
struct Entry {
    text: Text,
    units: usize,
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
    catalog: Vec<Entry>,
    labels: Vec<u32>,
    indices: Vec<u32>,
    keys: [u32; 13],
    title: Atom,
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
        let mut s = Self {
            source: Arc::new(source),
            admission: AdmissionState::default(),
            catalog: vec![],
            labels: vec![],
            indices: vec![],
            keys: [0; 13],
            title: Atom::Undefined,
            version: 1,
            root: Atom::Undefined,
            node_count: 0,
            string_count: 0,
            strings: HashMap::new(),
            nodes: HashMap::new(),
            patches: HashMap::new(),
        };
        let mut ids = HashMap::<String, u32>::new();
        for i in 0..s.source.nodes.len() {
            let id = s.add_text(Text::Label(i), &mut ids);
            s.labels.push(id);
            s.add_key("id", &mut ids);
            s.add_key("index", &mut ids);
        }
        for i in 0..s.source.nodes.len() {
            let id = s.add_text(Text::Decimal(i.to_string()), &mut ids);
            s.indices.push(id);
        }
        if !s.source.links.is_empty() {
            for key in ["source", "target", "value"] {
                s.add_key(key, &mut ids);
            }
        }
        for i in s.indices.len()..s.source.links.len() {
            let id = s.add_text(Text::Decimal(i.to_string()), &mut ids);
            s.indices.push(id);
        }
        s.add_key("sankey", &mut ids);
        for key in ["type", "nodes", "links"] {
            s.add_key(key, &mut ids);
        }
        if s.source.title.is_some() {
            s.title = Atom::String(s.add_text(Text::Title, &mut ids));
        }
        for key in ["title", "accTitle", "accDescr", "lookup"] {
            s.add_key(key, &mut ids);
        }
        s.node_count = s.source.nodes.len() + s.source.links.len() + 4;
        s.string_count = s.catalog.len();
        s.root = Atom::Ref((s.node_count - 1) as u32);
        graph_read::validate_structure(&s)?;
        Ok(s)
    }
    fn text<'a>(&'a self, t: &'a Text) -> &'a str {
        match t {
            Text::Label(i) => &self.source.nodes[*i].id,
            Text::Title => self.source.title.as_deref().expect("source title"),
            Text::Static(s) => s,
            Text::Decimal(s) => s,
        }
    }
    fn add_text(&mut self, t: Text, ids: &mut HashMap<String, u32>) -> u32 {
        let text = self.text(&t);
        if let Some(id) = ids.get(text) {
            return *id;
        }
        let id = self.catalog.len() as u32;
        let units = text.encode_utf16().count();
        ids.insert(text.to_owned(), id);
        self.catalog.push(Entry { text: t, units });
        id
    }
    fn add_key(&mut self, key: &'static str, ids: &mut HashMap<String, u32>) {
        let id = self.add_text(Text::Static(key), ids);
        self.keys[key_slot(key)] = id;
    }
    fn key(&self, key: &str) -> u32 {
        self.keys[key_slot(key)]
    }
    fn base_header(&self, id: usize) -> Result<Header, PeleError> {
        let n = self.source.nodes.len();
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
        let n = self.source.nodes.len();
        let m = self.source.links.len();
        let (key, value) = if id < n {
            if slot == 0 {
                (self.keys[0], Atom::String(self.labels[id]))
            } else {
                (
                    self.keys[1],
                    Atom::Number(self.source.nodes[id].index as f64),
                )
            }
        } else if id == n {
            (self.indices[slot], Atom::Ref(slot as u32))
        } else if id < n + m + 1 {
            let link = &self.source.links[id - n - 1];
            match slot {
                0 => (self.keys[2], Atom::Ref(link.source as u32)),
                1 => (self.keys[3], Atom::Ref(link.target as u32)),
                _ => (self.keys[4], Atom::Number(link.value)),
            }
        } else if id == n + m + 1 {
            (self.indices[slot], Atom::Ref((n + 1 + slot) as u32))
        } else {
            match slot {
                0 => (self.keys[6], Atom::String(self.keys[5])),
                1 => (self.keys[7], Atom::Ref(n as u32)),
                2 => (self.keys[8], Atom::Ref((n + m + 1) as u32)),
                3 => (self.keys[9], self.title.clone()),
                4 => (self.keys[10], Atom::Undefined),
                5 => (self.keys[11], Atom::Undefined),
                _ => (self.keys[12], Atom::Ref((n + m + 2) as u32)),
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
        Ok((
            Atom::String(self.labels[slot]),
            patch
                .and_then(|p| p.map_values.get(&slot))
                .cloned()
                .unwrap_or(Atom::Ref(slot as u32)),
        ))
    }
    fn string_is(&self, id: usize, text: &str) -> bool {
        if let Some(s) = self.strings.get(&id) {
            return s.is(text);
        }
        id < self.string_count
            && self
                .catalog
                .get(id)
                .is_some_and(|e| self.text(&e.text) == text)
    }
    fn same_string(&self, a: usize, b: usize) -> Result<bool, PeleError> {
        if a >= self.string_count || b >= self.string_count {
            return Err(graph_read::invalid());
        }
        Ok(match (self.strings.get(&a), self.strings.get(&b)) {
            (Some(a), Some(b)) => a == b,
            (Some(a), None) => {
                a.is(self.text(&self.catalog.get(b).ok_or_else(graph_read::invalid)?.text))
            }
            (None, Some(b)) => {
                b.is(self.text(&self.catalog.get(a).ok_or_else(graph_read::invalid)?.text))
            }
            (None, None) => {
                self.text(&self.catalog.get(a).ok_or_else(graph_read::invalid)?.text)
                    == self.text(&self.catalog.get(b).ok_or_else(graph_read::invalid)?.text)
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
                .catalog
                .get(id)
                .is_some_and(|e| text.0.iter().copied().eq(self.text(&e.text).encode_utf16()));
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
            if entries.len() != self.labels.len() {
                return false;
            }
            for (i, (k, v)) in entries.iter().enumerate() {
                if !same_atom(k, &Atom::String(self.labels[i]))
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
impl PublicGraphRead for SourceCanonical {
    fn trusted_read(&self) -> Option<TrustedGraphRead<'_>> {
        Some(TrustedGraphRead::new(self))
    }
    fn source_layout(&self, revision: u64) -> Result<Option<Vec<u8>>, PeleError> {
        match self.recipe(revision) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.code == crate::compat::ErrorCode::Limit => Ok(None),
            Err(e) => Err(e),
        }
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
            .or_else(|| self.catalog.get(id).map(|e| e.units))
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
        let text = self.text(&self.catalog.get(id).ok_or_else(graph_read::invalid)?.text);
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
                (
                    Atom::String(self.labels[slot]),
                    patch
                        .and_then(|p| p.map_values.get(&slot))
                        .cloned()
                        .unwrap_or(Atom::Ref(slot as u32)),
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
    fn recipe(&self, revision: u64) -> Result<Vec<u8>, PeleError> {
        use crate::compat::public::binary::{write_read_node, Writer, DEFAULT_BINARY_BUDGET};
        let mut w = Writer {
            bytes: Vec::new(),
            budget: DEFAULT_BINARY_BUDGET,
        };
        w.put(b"PLCSRC01")?;
        w.put(&revision.to_le_bytes())?;
        w.u32(self.version)?;
        atom9(&mut w, &self.root)?;
        w.count(self.string_count)?;
        w.count(self.node_count)?;
        w.count(self.string_count)?;
        for id in 0..self.string_count {
            w.count(PublicGraphRead::string_length(self, id)?)?;
            self.string_chunks(id, &mut |units| {
                for u in units {
                    w.put(&u.to_le_bytes())?;
                }
                Ok(())
            })?;
        }
        // Five scalar columns: labels, source, target, value, decimal-key IDs.
        w.count(5)?;
        w.count(self.labels.len())?;
        for id in &self.labels {
            atom9(&mut w, &Atom::String(*id))?;
        }
        for col in 0..3 {
            w.count(self.source.links.len())?;
            for l in &self.source.links {
                atom9(
                    &mut w,
                    &match col {
                        0 => Atom::Ref(l.source as u32),
                        1 => Atom::Ref(l.target as u32),
                        _ => Atom::Number(l.value),
                    },
                )?;
            }
        }
        w.count(self.indices.len())?;
        for id in &self.indices {
            atom9(&mut w, &Atom::String(*id))?;
        }
        let n = self.source.nodes.len();
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

fn key_slot(key: &str) -> usize {
    match key {
        "id" => 0,
        "index" => 1,
        "source" => 2,
        "target" => 3,
        "value" => 4,
        "sankey" => 5,
        "type" => 6,
        "nodes" => 7,
        "links" => 8,
        "title" => 9,
        "accTitle" => 10,
        "accDescr" => 11,
        "lookup" => 12,
        _ => unreachable!("static Sankey field"),
    }
}
