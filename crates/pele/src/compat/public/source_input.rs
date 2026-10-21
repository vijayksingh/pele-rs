//! Complete observed replacements against one immutable source layout.
//! Only crate-owned state can mint ingress; the erased codec also checks creator
//! provenance so forwarding a builtin capability cannot bypass a provider hook.
use super::{
    binary,
    captured_read::{CapturedRead, NodeHeader, NodeKind, PropertyPrefix},
    graph_read::PublicGraphRead,
    Atom, JsString, Node,
};
use crate::compat::PeleError;
use std::{any::TypeId, collections::HashMap};

pub(crate) struct SourceOverlay {
    pub layout_id: u64,
    pub revision: u64,
    pub version: u32,
    pub root: Atom,
    pub string_count: usize,
    pub node_count: usize,
    pub strings: HashMap<usize, JsString>,
    pub nodes: HashMap<usize, Node>,
}
/// Physical framing owns no decoded table or collection. A complete virtual
/// decoder preflight must succeed before materialize is called.
pub(crate) struct BorrowedOverlay<'a> {
    pub layout_id: u64,
    pub revision: u64,
    pub version: u32,
    pub root: Atom,
    pub string_count: usize,
    pub node_count: usize,
    strings: &'a [u8],
    nodes: &'a [u8],
    string_replacements: usize,
    node_replacements: usize,
}
pub(crate) struct BorrowedNode<'a> {
    pub raw: &'a [u8],
    pub header: super::captured_admission::Header,
}
impl<'a> BorrowedOverlay<'a> {
    pub(crate) fn decode(bytes: &'a [u8]) -> Result<Self, PeleError> {
        if bytes.len() > binary::DEFAULT_BINARY_BUDGET {
            return Err(binary::limit());
        }
        let mut r = binary::Reader::physical(bytes);
        if r.take(8)? != b"PLCSRI01" {
            return Err(binary::invalid());
        }
        let layout_id = r.u64()?;
        let revision = r.u64()?;
        let version = r.u32()?;
        let root = r.atom()?;
        let string_count = r.u32()? as usize;
        let node_count = r.u32()? as usize;
        let string_replacements = r.count(usize::MAX, 8)?;
        let strings_start = r.position();
        let mut previous = None;
        for _ in 0..string_replacements {
            let id = r.u32()? as usize;
            if id >= string_count || previous.is_some_and(|last| id <= last) {
                return Err(binary::invalid());
            }
            previous = Some(id);
            let units = r.count(usize::MAX, 2)?;
            r.take(units.checked_mul(2).ok_or_else(binary::limit)?)?;
        }
        let strings_end = r.position();
        let node_replacements = r.count(usize::MAX, 9)?;
        let nodes_start = r.position();
        previous = None;
        for _ in 0..node_replacements {
            let id = r.u32()? as usize;
            if id >= node_count || previous.is_some_and(|last| id <= last) {
                return Err(binary::invalid());
            }
            previous = Some(id);
            r.scan_node()?;
        }
        if !r.finished() {
            return Err(binary::invalid());
        }
        Ok(Self {
            layout_id,
            revision,
            version,
            root,
            string_count,
            node_count,
            strings: &bytes[strings_start..strings_end],
            nodes: &bytes[nodes_start..],
            string_replacements,
            node_replacements,
        })
    }
    pub(crate) fn string_entries(
        &self,
    ) -> impl Iterator<Item = Result<(usize, &'a [u8]), PeleError>> {
        StringEntries {
            reader: binary::Reader::physical(self.strings),
            remaining: self.string_replacements,
        }
    }
    pub(crate) fn node_entries(
        &self,
    ) -> impl Iterator<Item = Result<(usize, BorrowedNode<'a>), PeleError>> {
        NodeEntries {
            bytes: self.nodes,
            reader: binary::Reader::physical(self.nodes),
            remaining: self.node_replacements,
        }
    }
    // Caller has checked effective full table/unit/work limits, including base
    // records, in canonical decoder order. Only admitted replacements expand.
    pub(crate) fn materialize(self) -> Result<SourceOverlay, PeleError> {
        let mut strings = HashMap::with_capacity(self.string_replacements);
        for entry in self.string_entries() {
            let (id, raw) = entry?;
            strings.insert(
                id,
                JsString(
                    raw.chunks_exact(2)
                        .map(|p| u16::from_le_bytes([p[0], p[1]]))
                        .collect(),
                ),
            );
        }
        let mut nodes = HashMap::with_capacity(self.node_replacements);
        for entry in self.node_entries() {
            let (id, node) = entry?;
            let mut r = binary::Reader::physical(node.raw);
            let value = r.node()?;
            if !r.finished() {
                return Err(binary::invalid());
            }
            nodes.insert(id, value);
        }
        Ok(SourceOverlay {
            layout_id: self.layout_id,
            revision: self.revision,
            version: self.version,
            root: self.root,
            string_count: self.string_count,
            node_count: self.node_count,
            strings,
            nodes,
        })
    }
}
struct StringEntries<'a> {
    reader: binary::Reader<'a>,
    remaining: usize,
}
impl<'a> Iterator for StringEntries<'a> {
    type Item = Result<(usize, &'a [u8]), PeleError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        let result = (|| {
            let id = self.reader.u32()? as usize;
            let units = self.reader.count(usize::MAX, 2)?;
            Ok((
                id,
                self.reader
                    .take(units.checked_mul(2).ok_or_else(binary::limit)?)?,
            ))
        })();
        if result.is_err() {
            self.remaining = 0;
        }
        Some(result)
    }
}
struct NodeEntries<'a> {
    bytes: &'a [u8],
    reader: binary::Reader<'a>,
    remaining: usize,
}
impl<'a> Iterator for NodeEntries<'a> {
    type Item = Result<(usize, BorrowedNode<'a>), PeleError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        let result = (|| {
            let id = self.reader.u32()? as usize;
            let start = self.reader.position();
            let header = self.reader.scan_node()?;
            Ok((
                id,
                BorrowedNode {
                    raw: &self.bytes[start..self.reader.position()],
                    header,
                },
            ))
        })();
        if result.is_err() {
            self.remaining = 0;
        }
        Some(result)
    }
}

pub(crate) trait SourceIngressOwner {
    fn layout_id(&self) -> Option<u64>;
    fn prepare(&self, input: BorrowedOverlay<'_>) -> Result<SourceOverlay, PeleError>;
    fn with_candidate(
        &mut self,
        input: &SourceOverlay,
        visit: &mut dyn FnMut(&dyn PublicGraphRead) -> Result<(), PeleError>,
    ) -> Result<(), PeleError>;
    // Called only after ordered admission. Eager import may produce a retained
    // Err; it is not an apply failure. No user callbacks occur during commit.
    fn commit_admitted(&mut self, input: SourceOverlay);
}
#[doc(hidden)]
pub struct SourceIngress<'a> {
    creator: TypeId,
    owner: &'a mut dyn SourceIngressOwner,
}
impl<'a> SourceIngress<'a> {
    pub(crate) fn new<D: 'static>(owner: &'a mut dyn SourceIngressOwner) -> Self {
        Self {
            creator: TypeId::of::<D>(),
            owner,
        }
    }
    pub(crate) fn for_creator<D: 'static>(self) -> Option<Self> {
        (self.creator == TypeId::of::<D>()).then_some(self)
    }
    pub(crate) fn layout_id(&self) -> Option<u64> {
        self.owner.layout_id()
    }
    pub(crate) fn prepare(
        self,
        input: BorrowedOverlay<'_>,
    ) -> Result<PreparedSourceIngress<'a>, PeleError> {
        let input = self.owner.prepare(input)?;
        Ok(PreparedSourceIngress {
            ingress: self,
            input,
        })
    }
}

/// Allocation admission cannot be detached from its exact overlay or owner.
/// The owner borrow excludes accepted-state mutation until this value commits
/// or drops. Only the provenance-filtered ingress can construct it.
pub(crate) struct PreparedSourceIngress<'a> {
    ingress: SourceIngress<'a>,
    input: SourceOverlay,
}
impl<'a> PreparedSourceIngress<'a> {
    pub(crate) fn admit(
        self,
        visit: &mut dyn FnMut(&StructuredSourceInput<'_>) -> Result<(), PeleError>,
    ) -> Result<AdmittedSourceIngress<'a>, PeleError> {
        self.ingress
            .owner
            .with_candidate(&self.input, &mut |reader| {
                // prepare already performed exact canonical size and decoder
                // allocation gates. This remains the FIRST structural traversal,
                // before stale revision, schema compilation or schema IDs.
                super::captured_admission::validate(&mut super::graph_read::ReadAdapter(reader))?;
                visit(&StructuredSourceInput {
                    reader,
                    revision: self.input.revision,
                })
            })?;
        Ok(AdmittedSourceIngress(self))
    }
}
pub(crate) struct AdmittedSourceIngress<'a>(PreparedSourceIngress<'a>);
impl AdmittedSourceIngress<'_> {
    pub(crate) fn commit(self) {
        self.0.ingress.owner.commit_admitted(self.0.input);
    }
}

/// Scoped to one immutable candidate visit. No caller can replace the reader,
/// edit the overlay, mint this proof or persist it across the owner commit.
/// It authorizes only repeated structure checks, never schema-ID admission.
pub(crate) struct StructuredSourceInput<'a> {
    reader: &'a dyn PublicGraphRead,
    revision: u64,
}
impl StructuredSourceInput<'_> {
    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }
    pub(super) fn validate_schema(
        &self,
        compiled: &super::schema::CompiledSchema,
    ) -> Result<(), PeleError> {
        super::captured_admission::validate_schema_ids(
            &mut super::graph_read::ReadAdapter(self.reader),
            compiled.methods(),
            compiled.prototypes(),
        )
    }
}

/// Optional source-text access does not change logical field/import policy.
/// Implementations must return the same text as string_chunks. Original valid
/// UTF8 may be borrowed; replacement JsStrings retain strict UTF16 decoding.
pub(crate) trait LogicalCapturedText: PublicGraphRead {
    fn string_equals(&self, id: u32, text: &str) -> Result<bool, PeleError> {
        let mut expected = text.encode_utf16();
        let mut same = true;
        self.string_chunks(id as usize, &mut |chunk| {
            for &unit in chunk {
                if expected.next() != Some(unit) {
                    same = false;
                }
            }
            Ok(())
        })?;
        Ok(same && expected.next().is_none())
    }
    fn original_utf8(&self, _id: u32) -> Result<Option<&str>, PeleError> {
        Ok(None)
    }
}

/// Storage-only adapter for an admitted borrowed logical candidate. It keeps the
/// existing eager Sankey importer without constructing a ModelGraph.
pub(crate) struct CapturedLogical<'a, R: LogicalCapturedText + ?Sized>(pub &'a R);
impl<R: LogicalCapturedText + ?Sized> CapturedRead for CapturedLogical<'_, R> {
    type Value = Atom;
    fn root(&self) -> Atom {
        self.0.root()
    }
    fn header(&mut self, id: u32) -> Result<Option<NodeHeader>, PeleError> {
        use super::captured_admission::AdmissionKind as K;
        if id as usize >= self.0.extent().nodes {
            return Ok(None);
        }
        let h = self.0.header(id as usize)?;
        let (kind, array_length) = match h.kind {
            K::Object => (NodeKind::Object, None),
            K::Instance(_) => (NodeKind::Instance, None),
            K::Array(n) => (NodeKind::Array, Some(n)),
            K::Map(_) => (NodeKind::Map, None),
            K::Set(_) => (NodeKind::Set, None),
            K::Date(_) => (NodeKind::Date, None),
            K::Error(_) => (NodeKind::Error, None),
            _ => return Err(binary::invalid()),
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
        if names.is_empty() || id as usize >= self.0.extent().nodes {
            return Ok(());
        }
        let total = self.0.header(id as usize)?.properties;
        let mut start = 0;
        let mut remaining = names.len();
        while start < total && remaining != 0 {
            let mut used = 0;
            self.0.properties(id as usize, start, &mut |properties| {
                if properties.is_empty() || properties.len() > total - start {
                    return Err(binary::invalid());
                }
                used = properties.len();
                for p in properties {
                    for (name, value) in names.iter().zip(out.iter_mut()) {
                        if value.is_none() && self.0.string_equals(p.key, name)? {
                            *value = Some(p.value.clone());
                            remaining -= 1;
                        }
                    }
                    if remaining == 0 {
                        break;
                    }
                }
                Ok(())
            })?;
            start += used;
        }
        Ok(())
    }
    fn property_prefixes(
        &mut self,
        id: u32,
        start: u32,
        out: &mut [PropertyPrefix<Atom>],
    ) -> Result<usize, PeleError> {
        for value in out.iter_mut() {
            *value = PropertyPrefix::default();
        }
        let total = self.0.header(id as usize)?.properties;
        let start = start as usize;
        if start > total {
            return Err(binary::invalid());
        }
        let count = (total - start).min(out.len()).min(128);
        let mut filled = 0;
        while filled < count {
            let mut progress = 0;
            self.0
                .properties(id as usize, start + filled, &mut |properties| {
                    if properties.is_empty() || properties.len() > total - start - filled {
                        return Err(binary::invalid());
                    }
                    progress = properties.len().min(count - filled);
                    for (raw, p) in properties[..progress].iter().zip(out[filled..].iter_mut()) {
                        p.key_units = self.0.string_length(raw.key as usize)? as u32;
                        let mut cursor = 0;
                        self.0.string_chunks(raw.key as usize, &mut |chunk| {
                            for unit in chunk {
                                if cursor < 10 {
                                    p.prefix[cursor] = *unit;
                                    cursor += 1;
                                } else {
                                    break;
                                }
                            }
                            Ok(())
                        })?;
                        p.value = Some(raw.value.clone());
                    }
                    Ok(())
                })?;
            filled += progress;
        }
        Ok(count)
    }
    fn string_length(&mut self, id: u32) -> Result<u32, PeleError> {
        Ok(self.0.string_length(id as usize)? as u32)
    }
    fn with_string_chunks(
        &mut self,
        id: u32,
        mut visit: impl FnMut(&[u16]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        self.0.string_chunks(id as usize, &mut visit)
    }
}
