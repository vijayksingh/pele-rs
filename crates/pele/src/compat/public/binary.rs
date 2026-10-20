//! Bounded little-endian identity-graph transport, independent of diagram codecs.
//! JSON remains a diagnostic/legacy lane; strings and numbers retain exact bits.
use super::{Atom, ErrorObjectKind, JsString, ModelGraph, Node, Property, PrototypeId};
use crate::compat::{ErrorCode, PeleError};

const MAGIC: &[u8; 8] = b"PELGBIN1";
pub(super) const MAX_TABLE: usize = 1_000_000;
pub(super) const MAX_UNITS: usize = 16_000_000;
pub(super) const MAX_ENTRIES: usize = 8_000_000;
/// Per-exchange byte ceiling, separate from the engine's graph work ceilings.
pub const DEFAULT_BINARY_BUDGET: usize = 64 * 1024 * 1024;
/// Metadata is caller-owned UTF-8/JSON or another negotiated format; the graph
/// codec never interprets it. Revisions identify the snapshot/operation result.
#[derive(Debug)]
pub struct BinaryExchange {
    pub revision: u64,
    pub graph: ModelGraph,
    pub metadata: Vec<u8>,
}
pub(super) fn invalid() -> PeleError {
    PeleError::new("Invalid public binary graph transport", ErrorCode::Semantic)
}
pub(super) fn limit() -> PeleError {
    PeleError::new(
        "Public binary graph exceeds transport budget",
        ErrorCode::Limit,
    )
}
pub(super) fn check_count(n: usize, max: usize) -> Result<(), PeleError> {
    if n > max {
        Err(limit())
    } else {
        Ok(())
    }
}
pub(super) fn add_bounded(total: &mut usize, n: usize, max: usize) -> Result<(), PeleError> {
    *total = total.checked_add(n).ok_or_else(limit)?;
    check_count(*total, max)
}
pub(super) fn check_bytes(
    current: usize,
    additional: usize,
    budget: usize,
) -> Result<(), PeleError> {
    if additional > budget.saturating_sub(current) {
        Err(limit())
    } else {
        Ok(())
    }
}
pub(crate) struct Writer {
    pub(crate) bytes: Vec<u8>,
    pub(crate) budget: usize,
}
impl Writer {
    pub(crate) fn check_additional(&self, count: usize) -> Result<(), PeleError> {
        check_bytes(self.bytes.len(), count, self.budget)
    }
    pub(crate) fn put(&mut self, bytes: &[u8]) -> Result<(), PeleError> {
        self.check_additional(bytes.len())?;
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    pub(crate) fn byte(&mut self, value: u8) -> Result<(), PeleError> {
        self.put(&[value])
    }
    pub(crate) fn u32(&mut self, value: u32) -> Result<(), PeleError> {
        self.put(&value.to_le_bytes())
    }
    pub(crate) fn count(&mut self, value: usize) -> Result<(), PeleError> {
        self.u32(u32::try_from(value).map_err(|_| limit())?)
    }
    pub(crate) fn number(&mut self, value: f64) -> Result<(), PeleError> {
        self.put(&value.to_bits().to_le_bytes())
    }
    pub(crate) fn atom(&mut self, value: &Atom) -> Result<(), PeleError> {
        match value {
            Atom::Undefined => self.byte(0),
            Atom::Null => self.byte(1),
            Atom::Bool(false) => self.byte(2),
            Atom::Bool(true) => self.byte(3),
            Atom::Number(n) => {
                self.byte(4)?;
                self.number(*n)
            }
            Atom::String(id) => {
                self.byte(5)?;
                self.u32(*id)
            }
            Atom::Ref(id) => {
                self.byte(6)?;
                self.u32(*id)
            }
            Atom::BuiltinFunction(id) => {
                self.byte(7)?;
                self.u32(*id)
            }
            Atom::HostReference(id) => {
                self.byte(8)?;
                self.u32(*id)
            }
        }
    }
    pub(crate) fn properties(&mut self, values: &[Property]) -> Result<(), PeleError> {
        self.count(values.len())?;
        for p in values {
            self.u32(p.key)?;
            self.byte(p.flags)?;
            self.atom(&p.value)?;
        }
        Ok(())
    }
    pub(crate) fn node(&mut self, node: &Node) -> Result<(), PeleError> {
        match node {
            Node::Error { error_type, .. } => {
                self.byte(6)?;
                self.byte(match error_type {
                    ErrorObjectKind::Error => 0,
                    ErrorObjectKind::Pele => 1,
                    ErrorObjectKind::Type => 2,
                    ErrorObjectKind::Range => 3,
                    ErrorObjectKind::Syntax => 4,
                    ErrorObjectKind::Reference => 5,
                    ErrorObjectKind::Eval => 6,
                    ErrorObjectKind::Uri => 7,
                })?;
            }
            Node::Object { .. } => self.byte(0)?,
            Node::Instance { prototype, .. } => {
                self.byte(1)?;
                self.u32(prototype.0)?;
            }
            Node::Array { length, .. } => {
                self.byte(2)?;
                self.u32(*length)?;
            }
            Node::Map { entries, .. } => {
                self.byte(3)?;
                self.count(entries.len())?;
                for (k, v) in entries {
                    self.atom(k)?;
                    self.atom(v)?;
                }
            }
            Node::Set { values, .. } => {
                self.byte(4)?;
                self.count(values.len())?;
                for value in values {
                    self.atom(value)?;
                }
            }
            Node::Date { milliseconds, .. } => {
                self.byte(5)?;
                self.number(*milliseconds)?;
            }
        }
        self.properties(node.properties())
    }
}
/// Encode a validated graph without recursively expanding references or JSON
/// number/string projections. No partial bytes escape on failure.
pub fn encode_binary_exchange(
    revision: u64,
    graph: &ModelGraph,
    metadata: &[u8],
    byte_budget: usize,
) -> Result<Vec<u8>, PeleError> {
    graph.validate()?;
    let mut w = Writer {
        bytes: Vec::new(),
        budget: byte_budget,
    };
    w.put(MAGIC)?;
    w.u32(graph.version)?;
    w.put(&revision.to_le_bytes())?;
    w.count(metadata.len())?;
    w.atom(&graph.root)?;
    w.count(graph.strings.len())?;
    for s in &graph.strings {
        w.count(s.0.len())?;
        for unit in &s.0 {
            w.put(&unit.to_le_bytes())?;
        }
    }
    w.count(graph.nodes.len())?;
    for node in &graph.nodes {
        w.node(node)?;
    }
    w.put(metadata)?;
    Ok(w.bytes)
}
pub(super) struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
    units: usize,
    entries: usize,
    physical_only: bool,
}
impl<'a> Reader<'a> {
    pub(super) fn physical(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            offset: 0,
            units: 0,
            entries: 0,
            physical_only: true,
        }
    }
    pub(super) fn finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
    pub(super) fn take(&mut self, size: usize) -> Result<&'a [u8], PeleError> {
        let end = self.offset.checked_add(size).ok_or_else(invalid)?;
        let value = self.bytes.get(self.offset..end).ok_or_else(invalid)?;
        self.offset = end;
        Ok(value)
    }
    fn byte(&mut self) -> Result<u8, PeleError> {
        Ok(self.take(1)?[0])
    }
    pub(super) fn u32(&mut self) -> Result<u32, PeleError> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().map_err(|_| invalid())?,
        ))
    }
    pub(super) fn u64(&mut self) -> Result<u64, PeleError> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().map_err(|_| invalid())?,
        ))
    }
    // Check counts against both native ceilings and physical remaining input
    // before allocating. A malicious length cannot reserve unbacked memory.
    pub(super) fn count(&mut self, max: usize, min_bytes: usize) -> Result<usize, PeleError> {
        let n = self.u32()? as usize;
        if !self.physical_only {
            check_count(n, max)?;
        }
        if n > self.bytes.len().saturating_sub(self.offset) / min_bytes {
            return Err(invalid());
        }
        Ok(n)
    }
    fn work(&mut self, n: usize) -> Result<(), PeleError> {
        if self.physical_only {
            Ok(())
        } else {
            add_bounded(&mut self.entries, n, MAX_ENTRIES)
        }
    }
    pub(super) fn atom(&mut self) -> Result<Atom, PeleError> {
        Ok(match self.byte()? {
            0 => Atom::Undefined,
            1 => Atom::Null,
            2 => Atom::Bool(false),
            3 => Atom::Bool(true),
            4 => Atom::Number(f64::from_bits(self.u64()?)),
            5 => Atom::String(self.u32()?),
            6 => Atom::Ref(self.u32()?),
            7 => Atom::BuiltinFunction(self.u32()?),
            8 => Atom::HostReference(self.u32()?),
            _ => return Err(invalid()),
        })
    }
    pub(super) fn position(&self) -> usize {
        self.offset
    }
    pub(super) fn scan_node(&mut self) -> Result<super::captured_admission::Header, PeleError> {
        self.node_with::<ScanNode>()
    }
    pub(super) fn node(&mut self) -> Result<Node, PeleError> {
        self.node_with::<MaterializeNode>()
    }
    // One wire decoder owns kind/atom/count/error order for both physical
    // scanning and materialization. The scan sink never reserves collection
    // storage; the ordinary reader still checks semantic work before reserves.
    fn node_with<S: NodeSink>(&mut self) -> Result<S::Output, PeleError> {
        use super::captured_admission::AdmissionKind as K;
        let kind = match self.byte()? {
            0 => K::Object,
            1 => K::Instance(PrototypeId(self.u32()?)),
            2 => K::Array(self.u32()?),
            3 => {
                let n = self.count(MAX_ENTRIES / 2, 2)?;
                self.work(n * 2)?;
                K::Map(n)
            }
            4 => {
                let n = self.count(MAX_ENTRIES, 1)?;
                self.work(n)?;
                K::Set(n)
            }
            5 => K::Date(self.u64()?),
            6 => K::Error(match self.byte()? {
                0 => ErrorObjectKind::Error,
                1 => ErrorObjectKind::Pele,
                2 => ErrorObjectKind::Type,
                3 => ErrorObjectKind::Range,
                4 => ErrorObjectKind::Syntax,
                5 => ErrorObjectKind::Reference,
                6 => ErrorObjectKind::Eval,
                7 => ErrorObjectKind::Uri,
                _ => return Err(invalid()),
            }),
            _ => return Err(invalid()),
        };
        let mut sink = S::begin(kind)?;
        match kind {
            K::Map(n) => {
                for _ in 0..n {
                    let key = self.atom()?;
                    let value = self.atom()?;
                    sink.map_entry(key, value);
                }
            }
            K::Set(n) => {
                for _ in 0..n {
                    sink.set_item(self.atom()?);
                }
            }
            _ => {}
        }
        let n = self.count(MAX_ENTRIES, 6)?;
        self.work(n)?;
        sink.properties(n);
        for _ in 0..n {
            let key = self.u32()?;
            let flags = self.byte()?;
            let value = self.atom()?;
            sink.property(Property { key, flags, value });
        }
        Ok(sink.finish())
    }
}
trait NodeSink {
    type Output;
    fn begin(kind: super::captured_admission::AdmissionKind) -> Result<Self, PeleError>
    where
        Self: Sized;
    fn map_entry(&mut self, key: Atom, value: Atom);
    fn set_item(&mut self, value: Atom);
    fn properties(&mut self, count: usize);
    fn property(&mut self, value: Property);
    fn finish(self) -> Self::Output;
}
struct ScanNode(super::captured_admission::Header);
impl NodeSink for ScanNode {
    type Output = super::captured_admission::Header;
    fn begin(kind: super::captured_admission::AdmissionKind) -> Result<Self, PeleError> {
        Ok(Self(super::captured_admission::Header {
            kind,
            properties: 0,
        }))
    }
    fn map_entry(&mut self, _: Atom, _: Atom) {}
    fn set_item(&mut self, _: Atom) {}
    fn properties(&mut self, count: usize) {
        self.0.properties = count;
    }
    fn property(&mut self, _: Property) {}
    fn finish(self) -> Self::Output {
        self.0
    }
}
struct MaterializeNode(Node);
impl NodeSink for MaterializeNode {
    type Output = Node;
    fn begin(kind: super::captured_admission::AdmissionKind) -> Result<Self, PeleError> {
        use super::captured_admission::AdmissionKind as K;
        let properties = Vec::new();
        Ok(Self(match kind {
            K::Object => Node::Object { properties },
            K::Instance(prototype) => Node::Instance {
                prototype,
                properties,
            },
            K::Array(length) => Node::Array { length, properties },
            K::Map(count) => Node::Map {
                entries: Vec::with_capacity(count),
                properties,
            },
            K::Set(count) => Node::Set {
                values: Vec::with_capacity(count),
                properties,
            },
            K::Date(bits) => Node::Date {
                milliseconds: f64::from_bits(bits),
                properties,
            },
            K::Error(error_type) => Node::Error {
                error_type,
                properties,
            },
            K::InvalidArray | K::InvalidError => return Err(invalid()),
        }))
    }
    fn map_entry(&mut self, key: Atom, value: Atom) {
        let Node::Map { entries, .. } = &mut self.0 else {
            unreachable!("paired node sink")
        };
        entries.push((key, value));
    }
    fn set_item(&mut self, value: Atom) {
        let Node::Set { values, .. } = &mut self.0 else {
            unreachable!("paired node sink")
        };
        values.push(value);
    }
    fn properties(&mut self, count: usize) {
        *self.0.properties_mut() = Vec::with_capacity(count);
    }
    fn property(&mut self, value: Property) {
        self.0.properties_mut().push(value);
    }
    fn finish(self) -> Self::Output {
        self.0
    }
}
/// Decode one complete bounded exchange. All graph references/flags are checked;
/// apply/invoke still validate registration-specific method/prototype IDs.
pub fn decode_binary_exchange(
    bytes: &[u8],
    byte_budget: usize,
) -> Result<BinaryExchange, PeleError> {
    if bytes.len() > byte_budget {
        return Err(limit());
    }
    let mut r = Reader {
        bytes,
        offset: 0,
        units: 0,
        entries: 0,
        physical_only: false,
    };
    if r.take(MAGIC.len())? != MAGIC {
        return Err(invalid());
    }
    let version = r.u32()?;
    if version != 1 {
        return Err(invalid());
    }
    let revision = r.u64()?;
    let metadata_len = r.u32()? as usize;
    if metadata_len > bytes.len().saturating_sub(r.offset) {
        return Err(invalid());
    }
    // Exclude trailing metadata from all graph length checks.
    let graph_end = bytes.len() - metadata_len;
    r.bytes = bytes.get(..graph_end).ok_or_else(invalid)?;
    let root = r.atom()?;
    let count = r.count(MAX_TABLE, 4)?;
    let mut strings = Vec::with_capacity(count);
    for _ in 0..count {
        let n = r.count(MAX_UNITS, 2)?;
        add_bounded(&mut r.units, n, MAX_UNITS)?;
        let raw = r.take(n.checked_mul(2).ok_or_else(limit)?)?;
        strings.push(JsString(
            raw.chunks_exact(2)
                .map(|v| u16::from_le_bytes([v[0], v[1]]))
                .collect(),
        ));
    }
    let count = r.count(MAX_TABLE, 5)?;
    let mut nodes = Vec::with_capacity(count);
    for _ in 0..count {
        nodes.push(r.node()?);
    }
    if r.offset != graph_end {
        return Err(invalid());
    }
    let graph = ModelGraph {
        version,
        root,
        strings,
        nodes,
    };
    graph.validate()?;
    Ok(BinaryExchange {
        revision,
        graph,
        metadata: bytes[graph_end..].to_vec(),
    })
}

// Internal logical transport shares the same writer and ordered structural
// admission. Explicit byte decoders and ordinary arena encoding are unchanged.
pub(crate) fn encode_read_exchange(
    revision: u64,
    graph: &dyn super::graph_read::PublicGraphRead,
    metadata: &[u8],
    byte_budget: usize,
) -> Result<Vec<u8>, PeleError> {
    use super::captured_admission as admission;
    admission::validate(&mut super::graph_read::ReadAdapter(graph))?;
    let extent = graph.extent();
    let mut w = Writer {
        bytes: Vec::new(),
        budget: byte_budget,
    };
    w.put(MAGIC)?;
    w.u32(extent.version)?;
    w.put(&revision.to_le_bytes())?;
    w.count(metadata.len())?;
    w.atom(&graph.root())?;
    w.count(extent.strings)?;
    for id in 0..extent.strings {
        let length = graph.string_length(id)?;
        w.count(length)?;
        let mut seen = 0usize;
        graph.string_chunks(id, &mut |units| {
            seen = seen
                .checked_add(units.len())
                .filter(|n| *n <= length)
                .ok_or_else(invalid)?;
            for unit in units {
                w.put(&unit.to_le_bytes())?;
            }
            Ok(())
        })?;
        if seen != length {
            return Err(invalid());
        }
    }
    w.count(extent.nodes)?;
    for id in 0..extent.nodes {
        write_read_node(&mut w, graph, id)?;
    }
    w.put(metadata)?;
    Ok(w.bytes)
}

pub(crate) fn write_read_node(
    w: &mut Writer,
    graph: &dyn super::graph_read::PublicGraphRead,
    id: usize,
) -> Result<(), PeleError> {
    use super::captured_admission::AdmissionKind;
    let header = graph.header(id)?;
    match header.kind {
        AdmissionKind::Object => w.byte(0)?,
        AdmissionKind::Instance(p) => {
            w.byte(1)?;
            w.u32(p.0)?;
        }
        AdmissionKind::Array(n) => {
            w.byte(2)?;
            w.u32(n)?;
        }
        AdmissionKind::Map(n) => {
            w.byte(3)?;
            w.count(n)?;
            super::graph_read::visit_map(graph, id, |pairs| {
                for (k, v) in pairs {
                    w.atom(k)?;
                    w.atom(v)?;
                }
                Ok(())
            })?;
        }
        AdmissionKind::Set(n) => {
            w.byte(4)?;
            w.count(n)?;
            super::graph_read::visit_set(graph, id, |values| {
                for v in values {
                    w.atom(v)?;
                }
                Ok(())
            })?;
        }
        AdmissionKind::Date(bits) => {
            w.byte(5)?;
            w.put(&bits.to_le_bytes())?;
        }
        AdmissionKind::Error(kind) => {
            w.byte(6)?;
            w.byte(match kind {
                ErrorObjectKind::Error => 0,
                ErrorObjectKind::Pele => 1,
                ErrorObjectKind::Type => 2,
                ErrorObjectKind::Range => 3,
                ErrorObjectKind::Syntax => 4,
                ErrorObjectKind::Reference => 5,
                ErrorObjectKind::Eval => 6,
                ErrorObjectKind::Uri => 7,
            })?;
        }
        AdmissionKind::InvalidArray | AdmissionKind::InvalidError => return Err(invalid()),
    }
    w.count(header.properties)?;
    super::graph_read::visit_properties(graph, id, |properties| {
        for p in properties {
            w.u32(p.key)?;
            w.byte(p.flags)?;
            w.atom(&p.value)?;
        }
        Ok(())
    })?;

    Ok(())
}
