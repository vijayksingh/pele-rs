//! Bounded storage-only initial graph encoder. No diagram lifecycle, JS owner,
//! persistent graph, callback capture or public error mapping lives here.
use pele::compat::{
    public::{Atom, InitialWriter, PrototypeId},
    ErrorCode, PeleError,
};
use std::collections::HashMap;

/// The family facade owns callback/throw capture, final admission and output.
/// A transport must copy bytes before any import can reenter/retain the buffer.
pub(crate) trait ConstructionBatchTransport {
    fn publish_batch(&mut self, bytes: &[u8]) -> Result<(), PeleError>;
    fn invalid(&self, message: &str) -> PeleError;
}
pub(crate) trait ConstructionTransport: ConstructionBatchTransport {
    type Output;
    fn finish(self, root: Atom) -> Result<Self::Output, PeleError>;
    fn abort(self) -> Result<(), PeleError>;
}

/// Pure unpublished construction emits bounded storage events, never a graph.
struct HostRecordLayout {
    identity: usize,
    length: usize,
}
pub(crate) struct HostConstruction<T> {
    emitter: ConstructionEmitter<T>,
    record_layouts: Vec<HostRecordLayout>,
    strings: HashMap<String, u32>,
    string_units: usize,
    nodes: u32,
}
/// Emits assigned IDs into the existing bounded wire. No interner, model-limit
/// admission, ID assignment or source/helper lifecycle is implied.
pub(crate) struct ConstructionEmitter<T> {
    transport: T,
    instance_prototype: PrototypeId,
    frame: Vec<u8>,
    events: u16,
}
const CONSTRUCTION_BATCH_BYTES: usize = 64 * 1024;
const CONSTRUCTION_BATCH_EVENTS: u16 = 128;
fn construction_frame_limit() -> PeleError {
    PeleError::new(
        "Public binary graph exceeds transport budget",
        ErrorCode::Limit,
    )
}
fn construction_atom_len(atom: &Atom) -> usize {
    match atom {
        Atom::Undefined | Atom::Null | Atom::Bool(_) => 1,
        Atom::Number(_) => 9,
        _ => 5,
    }
}
// Count only until the bounded inline event no longer fits. Large nodes keep
// the existing incremental stream; this never allocates a second node/frame.
fn construction_inline_len(node: &pele::compat::public::Node) -> Option<usize> {
    use pele::compat::public::Node;
    let mut bytes = 18usize;
    let mut add = |amount: usize| -> Option<()> {
        bytes = bytes.checked_add(amount)?;
        (bytes <= CONSTRUCTION_BATCH_BYTES - 2).then_some(())
    };
    match node {
        Node::Map { entries, .. } => {
            for (key, value) in entries {
                add(construction_atom_len(key) + construction_atom_len(value))?;
            }
        }
        Node::Set { values, .. } => {
            for value in values {
                add(construction_atom_len(value))?;
            }
        }
        _ => {}
    }
    for property in node.properties() {
        add(5 + construction_atom_len(&property.value))?;
    }
    Some(bytes)
}
fn construction_atom(out: &mut Vec<u8>, atom: &Atom) {
    let (tag, payload) = match atom {
        Atom::Undefined => (0, 0),
        Atom::Null => (1, 0),
        Atom::Bool(false) => (2, 0),
        Atom::Bool(true) => (3, 0),
        Atom::Number(value) => (4, value.to_bits()),
        Atom::String(id) => (5, u64::from(*id)),
        Atom::Ref(id) => (6, u64::from(*id)),
        Atom::BuiltinFunction(id) => (7, u64::from(*id)),
        Atom::HostReference(id) => (8, u64::from(*id)),
    };
    out.push(tag);
    out.extend_from_slice(&payload.to_le_bytes()[..construction_atom_len(atom) - 1]);
}
impl<T: ConstructionBatchTransport> ConstructionEmitter<T> {
    pub(crate) fn new(transport: T, instance_prototype: PrototypeId) -> Self {
        Self {
            transport,
            instance_prototype,
            frame: Vec::new(),
            events: 0,
        }
    }
    /// Does not flush. The operation decides whether pending writes must commit
    /// before a read/error outcome or must be discarded with an unpublished root.
    pub(crate) fn into_transport(self) -> T {
        self.transport
    }
    pub(crate) fn flush(&mut self) -> Result<(), PeleError> {
        if self.events == 0 {
            return Ok(());
        }
        self.frame[..2].copy_from_slice(&self.events.to_le_bytes());
        self.transport.publish_batch(&self.frame)?;
        self.frame.clear();
        self.events = 0;
        Ok(())
    }
    fn event(&mut self, tag: u8, bytes: usize) -> Result<(), PeleError> {
        // Every event, including a large-string fragment, is bounded before
        // reserving scratch or allocating the owned JS transfer buffer.
        if bytes == 0 || bytes > CONSTRUCTION_BATCH_BYTES - 2 {
            return Err(construction_frame_limit());
        }
        if self.events == CONSTRUCTION_BATCH_EVENTS
            || self.frame.len().max(2) + bytes > CONSTRUCTION_BATCH_BYTES
        {
            self.flush()?;
        }
        let needed = self.frame.len().max(2) + bytes;
        if self.frame.capacity() < needed {
            let capacity = needed
                .max(512)
                .next_power_of_two()
                .min(CONSTRUCTION_BATCH_BYTES);
            self.frame.reserve_exact(capacity - self.frame.len());
        }
        if self.frame.is_empty() {
            self.frame.extend_from_slice(&[0, 0]);
        }
        self.frame.push(tag);
        self.events += 1;
        Ok(())
    }
}
impl<T: ConstructionBatchTransport> ConstructionEmitter<T> {
    /// Assigned string ID comes from the caller's current canonical storage.
    /// Source uses the private pre-counted form so it does not scan UTF16 twice.
    #[cfg(feature = "captured-sankey")]
    pub(crate) fn append_string(&mut self, id: u32, value: &str) -> Result<(), PeleError> {
        let units = value.encode_utf16().count();
        u32::try_from(units).map_err(|_| construction_frame_limit())?;
        self.append_string_units(id, value, units)
    }
    fn append_string_units(&mut self, id: u32, value: &str, units: usize) -> Result<(), PeleError> {
        if units <= (CONSTRUCTION_BATCH_BYTES - 2 - 9) / 2 {
            self.event(0, 9 + units * 2)?;
            self.frame.extend_from_slice(&id.to_le_bytes());
            self.frame.extend_from_slice(&(units as u32).to_le_bytes());
            for unit in value.encode_utf16() {
                self.frame.extend_from_slice(&unit.to_le_bytes());
            }
        } else {
            self.event(1, 9)?;
            self.frame.extend_from_slice(&id.to_le_bytes());
            self.frame.extend_from_slice(&(units as u32).to_le_bytes());
            let mut remaining = units;
            let mut input = value.encode_utf16();
            while remaining > 0 {
                let count = remaining.min((CONSTRUCTION_BATCH_BYTES - 2 - 5) / 2);
                self.event(2, 5 + count * 2)?;
                self.frame.extend_from_slice(&(count as u32).to_le_bytes());
                for unit in input.by_ref().take(count) {
                    self.frame.extend_from_slice(&unit.to_le_bytes());
                }
                remaining -= count;
            }
            self.event(3, 1)?;
        }
        Ok(())
    }
    fn node_kind(&self, node: &pele::compat::public::Node) -> Result<(u8, u32), PeleError> {
        use pele::compat::public::Node;
        Ok(match node {
            Node::Object { .. } => (0u8, 0),
            Node::Array { length, .. } => (1, *length),
            Node::Map { .. } => (2, 0),
            Node::Set { .. } => (3, 0),
            Node::Instance { prototype, .. } if prototype.0 == self.instance_prototype.0 => (4, 0),
            _ => return Err(self.transport.invalid("Unsupported host constructor node")),
        })
    }
    #[cfg(feature = "captured-sankey")]
    pub(crate) fn append_node(
        &mut self,
        id: u32,
        node: pele::compat::public::Node,
    ) -> Result<(), PeleError> {
        let (kind, length) = self.node_kind(&node)?;
        self.append_node_kind(id, node, kind, length)
    }
    fn append_node_kind(
        &mut self,
        id: u32,
        node: pele::compat::public::Node,
        kind: u8,
        length: u32,
    ) -> Result<(), PeleError> {
        use pele::compat::public::Node;
        if let Some(bytes) = construction_inline_len(&node) {
            let payload_count = match &node {
                Node::Map { entries, .. } => entries.len(),
                Node::Set { values, .. } => values.len(),
                _ => 0,
            };
            // The length check bounds both counts below u32::MAX before casts.
            self.event(9, bytes)?;
            self.frame.extend_from_slice(&id.to_le_bytes());
            self.frame.push(kind);
            self.frame.extend_from_slice(&length.to_le_bytes());
            self.frame
                .extend_from_slice(&(payload_count as u32).to_le_bytes());
            self.frame
                .extend_from_slice(&(node.properties().len() as u32).to_le_bytes());
            match &node {
                Node::Map { entries, .. } => {
                    for (key, value) in entries {
                        construction_atom(&mut self.frame, key);
                        construction_atom(&mut self.frame, value);
                    }
                }
                Node::Set { values, .. } => {
                    for value in values {
                        construction_atom(&mut self.frame, value);
                    }
                }
                _ => {}
            }
            for property in node.properties() {
                self.frame.extend_from_slice(&property.key.to_le_bytes());
                self.frame.push(property.flags);
                construction_atom(&mut self.frame, &property.value);
            }
            return Ok(());
        }
        self.event(4, 10)?;
        self.frame.extend_from_slice(&id.to_le_bytes());
        self.frame.push(kind);
        self.frame.extend_from_slice(&length.to_le_bytes());
        match &node {
            Node::Map { entries, .. } => {
                for (key, value) in entries {
                    self.event(
                        5,
                        1 + construction_atom_len(key) + construction_atom_len(value),
                    )?;
                    construction_atom(&mut self.frame, key);
                    construction_atom(&mut self.frame, value);
                }
            }
            Node::Set { values, .. } => {
                for value in values {
                    self.event(6, 1 + construction_atom_len(value))?;
                    construction_atom(&mut self.frame, value);
                }
            }
            _ => {}
        }
        for property in node.properties() {
            self.event(7, 6 + construction_atom_len(&property.value))?;
            self.frame.extend_from_slice(&property.key.to_le_bytes());
            self.frame.push(property.flags);
            construction_atom(&mut self.frame, &property.value);
        }
        self.event(8, 1)?;
        Ok(())
    }
}
impl<T: ConstructionTransport> HostConstruction<T> {
    pub(crate) fn new(transport: T, instance_prototype: PrototypeId) -> Self {
        Self {
            emitter: ConstructionEmitter::new(transport, instance_prototype),
            record_layouts: Vec::new(),
            strings: HashMap::new(),
            string_units: 0,
            nodes: 0,
        }
    }
}
impl<T: ConstructionTransport> InitialWriter for HostConstruction<T> {
    type Output = T::Output;
    fn string(&mut self, value: &str) -> Result<u32, PeleError> {
        if let Some(id) = self.strings.get(value) {
            return Ok(*id);
        }
        let units = value.encode_utf16().count();
        let total = self
            .string_units
            .checked_add(units)
            .filter(|n| *n <= 16_000_000)
            .ok_or_else(|| {
                PeleError::new("Public model graph exceeds work budget", ErrorCode::Limit)
            })?;
        if self.strings.len() >= 1_000_000 {
            return Err(PeleError::new(
                "Public model graph exceeds work budget",
                ErrorCode::Limit,
            ));
        }
        // Fresh owner + pure construction gives the same first-intern sequence
        // as GraphWriter and the host pool. No callback can rewrite that pool.
        // Legacy fallback completed exact counting admission before this sink.
        let id = self.strings.len() as u32;
        self.emitter.append_string_units(id, value, units)?;
        self.string_units = total;
        self.strings.insert(value.to_owned(), id);
        Ok(id)
    }
    fn record<const N: usize>(
        &mut self,
        keys: &'static [&'static str; N],
        values: [Atom; N],
    ) -> Result<u32, PeleError> {
        // Private identity of immutable layout storage, never sent to JS. A
        // compiler may merge identical constants; their field order is equal.
        let identity = keys.as_ptr() as usize;
        let known = self
            .record_layouts
            .iter()
            .position(|layout| layout.identity == identity && layout.length == N);
        let layout = if let Some(id) = known {
            id
        } else {
            // Optional bounded specialization. Unusual future layouts use the
            // exact generic property path; this is not a new model limit.
            let eligible = N <= 64
                && self.record_layouts.len() < 32
                && keys.iter().enumerate().all(|(index, key)| {
                    *key != "__proto__" && *key != "length" && !keys[..index].contains(key)
                });
            if !eligible {
                let properties = keys
                    .iter()
                    .zip(values)
                    .map(|(key, value)| self.property(key, value))
                    .collect::<Result<_, _>>()?;
                return self.node(pele::compat::public::Node::Object { properties });
            }
            // First layout occurrence interns keys at the original rec() stage,
            // after all values. Later occurrences are guaranteed interner hits;
            // this sink charges text/table storage only on first intern.
            let mut ids = Vec::with_capacity(N);
            for key in keys {
                ids.push(self.string(key)?);
            }
            let id = self.record_layouts.len();
            self.emitter.event(10, 3 + 4 * N)?;
            self.emitter.frame.push(id as u8);
            self.emitter.frame.push(N as u8);
            for key in ids {
                self.emitter.frame.extend_from_slice(&key.to_le_bytes());
            }
            self.record_layouts.push(HostRecordLayout {
                identity,
                length: N,
            });
            id
        };
        // Match node() admission after key interning. All fixed-layout records
        // fit a single bounded event, independently of their referenced strings.
        if self.nodes >= 1_000_000 {
            return Err(PeleError::new(
                "Public model graph exceeds work budget",
                ErrorCode::Limit,
            ));
        }
        let id = self.nodes;
        let bytes = 6 + values.iter().map(construction_atom_len).sum::<usize>();
        self.emitter.event(11, bytes)?;
        self.emitter.frame.extend_from_slice(&id.to_le_bytes());
        self.emitter.frame.push(layout as u8);
        for value in values {
            construction_atom(&mut self.emitter.frame, &value);
        }
        self.nodes += 1;
        Ok(id)
    }
    fn dense_array(&mut self, values: &[Atom]) -> Result<u32, PeleError> {
        // Eligibility examines only existing Atom tags. Oversized arrays keep
        // the original property/node path without an extra value collection.
        let bytes = values.iter().try_fold(9usize, |bytes, value| {
            bytes
                .checked_add(construction_atom_len(value))
                .filter(|bytes| *bytes <= CONSTRUCTION_BATCH_BYTES - 2)
        });
        let Some(bytes) = bytes else {
            let mut properties = Vec::with_capacity(values.len());
            for (index, value) in values.iter().enumerate() {
                properties.push(self.property(&index.to_string(), value.clone())?);
            }
            return self.node(pele::compat::public::Node::Array {
                length: properties.len() as u32,
                properties,
            });
        };
        // These keys remain part of the canonical string table even though
        // this event reconstructs their names. Preserve all IDs and budgets.
        for index in 0..values.len() {
            self.string(&index.to_string())?;
        }
        if self.nodes >= 1_000_000 {
            return Err(PeleError::new(
                "Public model graph exceeds work budget",
                ErrorCode::Limit,
            ));
        }
        let id = self.nodes;
        self.emitter.event(12, bytes)?;
        self.emitter.frame.extend_from_slice(&id.to_le_bytes());
        // Event fit bounds the number of atoms below u32::MAX.
        self.emitter
            .frame
            .extend_from_slice(&(values.len() as u32).to_le_bytes());
        for value in values {
            construction_atom(&mut self.emitter.frame, value);
        }
        self.nodes += 1;
        Ok(id)
    }
    fn node(&mut self, node: pele::compat::public::Node) -> Result<u32, PeleError> {
        let (kind, length) = self.emitter.node_kind(&node)?;
        if self.nodes >= 1_000_000 {
            return Err(PeleError::new(
                "Public model graph exceeds work budget",
                ErrorCode::Limit,
            ));
        }
        let id = self.nodes;
        self.emitter.append_node_kind(id, node, kind, length)?;
        self.nodes += 1;
        Ok(id)
    }
    fn finish(mut self, root: Atom) -> Result<Self::Output, PeleError> {
        self.emitter.flush()?;
        self.emitter.into_transport().finish(root)
    }
    fn abort(self) -> Result<(), PeleError> {
        // Do not publish pending events after a grammar/eligibility failure.
        self.emitter.into_transport().abort()
    }
}
