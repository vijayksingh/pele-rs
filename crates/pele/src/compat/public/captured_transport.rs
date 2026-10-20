//! Count canonical captured slots without a graph mirror or encoded byte buffer.
use super::{binary, captured_admission as admission, Atom};
use crate::compat::PeleError;
use admission::{AdmissionKind, AdmissionRead};

pub(crate) struct Counter {
    bytes: usize,
    budget: usize,
}
impl Counter {
    pub(crate) fn new(budget: usize) -> Self {
        Self { bytes: 0, budget }
    }
    pub(crate) fn bytes(&self) -> usize {
        self.bytes
    }
    pub(crate) fn prefix(&mut self, metadata: usize) -> Result<(), PeleError> {
        self.put(8)?;
        self.put(4)?;
        self.put(8)?;
        self.count(metadata)
    }
    pub(crate) fn string(&mut self, units: usize) -> Result<(), PeleError> {
        self.count(units)?;
        self.put(units.checked_mul(2).ok_or_else(binary::limit)?)
    }
    pub(crate) fn node_prefix(&mut self, kind: AdmissionKind) -> Result<(), PeleError> {
        self.put(1)?;
        match kind {
            AdmissionKind::Object => Ok(()),
            AdmissionKind::Instance(_) | AdmissionKind::Array(_) | AdmissionKind::InvalidArray => {
                self.put(4)
            }
            AdmissionKind::Date(_) => self.put(8),
            AdmissionKind::Error(_) | AdmissionKind::InvalidError => self.put(1),
            AdmissionKind::Map(count) | AdmissionKind::Set(count) => self.count(count),
        }
    }
    pub(crate) fn property(&mut self, value: &Atom) -> Result<(), PeleError> {
        self.put(5)?;
        self.atom(value)
    }
    pub(crate) fn put(&mut self, count: usize) -> Result<(), PeleError> {
        binary::check_bytes(self.bytes, count, self.budget)?;
        self.bytes = self.bytes.checked_add(count).ok_or_else(binary::limit)?;
        Ok(())
    }
    pub(crate) fn count(&mut self, count: usize) -> Result<(), PeleError> {
        u32::try_from(count).map_err(|_| binary::limit())?;
        self.put(4)
    }
    pub(crate) fn atom(&mut self, atom: &Atom) -> Result<(), PeleError> {
        self.put(match atom {
            Atom::Undefined | Atom::Null | Atom::Bool(_) => 1,
            Atom::Number(_) => 9,
            _ => 5,
        })
    }
}
/// Metadata length must come from the adapter's exact native Serialize result,
/// not a JS aggregate. The returned size includes all retained history.
pub fn canonical_frame_bytes<S: AdmissionRead>(
    store: &mut S,
    metadata_bytes: usize,
    budget: usize,
) -> Result<usize, PeleError> {
    let extent = store.extent()?;
    let mut out = Counter::new(budget);
    out.prefix(metadata_bytes)?;
    store.with_root(|atom| out.atom(atom))?;
    out.count(extent.strings)?;
    let mut seen = 0usize;
    store.visit_string_lengths(|units| {
        if seen >= extent.strings {
            return Err(binary::invalid());
        }
        seen += 1;
        out.string(units)
    })?;
    if seen != extent.strings {
        return Err(binary::invalid());
    }
    out.count(extent.nodes)?;
    for node in 0..extent.nodes {
        let header = store.header(node)?;
        out.node_prefix(header.kind)?;
        if matches!(header.kind, AdmissionKind::Map(_) | AdmissionKind::Set(_)) {
            admission::items(store, node, header.kind, |atom| out.atom(atom))?;
        }
        out.count(header.properties)?;
        admission::properties(store, node, header.properties, |property| {
            property.value.with_atom(|value| out.property(value))
        })?;
    }
    out.put(metadata_bytes)?;
    Ok(out.bytes)
}
/// The allocation gates of the canonical decoder, in its original order.
/// Borrowed sparse input runs these over the complete effective graph BEFORE
/// expanding replacement payloads. This is not structural/schema admission.
pub(crate) struct DecoderBudget {
    extent: admission::Extent,
    units: usize,
    work: usize,
}
impl DecoderBudget {
    pub(crate) fn new(extent: admission::Extent) -> Result<Self, PeleError> {
        if extent.version != 1 {
            return Err(binary::invalid());
        }
        binary::check_count(extent.strings, binary::MAX_TABLE)?;
        Ok(Self {
            extent,
            units: 0,
            work: 0,
        })
    }
    pub(crate) fn string(&mut self, count: usize) -> Result<(), PeleError> {
        binary::check_count(count, binary::MAX_UNITS)?;
        binary::add_bounded(&mut self.units, count, binary::MAX_UNITS)
    }
    pub(crate) fn begin_nodes(&mut self) -> Result<(), PeleError> {
        binary::check_count(self.extent.nodes, binary::MAX_TABLE)
    }
    pub(crate) fn node(&mut self, header: admission::Header) -> Result<(), PeleError> {
        match header.kind {
            AdmissionKind::InvalidArray | AdmissionKind::InvalidError => {
                return Err(binary::invalid())
            }
            AdmissionKind::Map(count) => {
                binary::check_count(count, binary::MAX_ENTRIES / 2)?;
                binary::add_bounded(&mut self.work, count * 2, binary::MAX_ENTRIES)?;
            }
            AdmissionKind::Set(count) => {
                binary::check_count(count, binary::MAX_ENTRIES)?;
                binary::add_bounded(&mut self.work, count, binary::MAX_ENTRIES)?;
            }
            _ => {}
        }
        binary::check_count(header.properties, binary::MAX_ENTRIES)?;
        binary::add_bounded(&mut self.work, header.properties, binary::MAX_ENTRIES)
    }
}
/// Canonical observed slots have no malformed byte encodings. Keep the original
/// binary API's parser and its physical-length checks for actual byte input.
#[cfg(feature = "web-host-sankey")]
pub fn admit_observed<S: AdmissionRead>(
    store: &mut S,
    metadata_bytes: usize,
    budget: usize,
) -> Result<(), PeleError> {
    canonical_frame_bytes(store, metadata_bytes, budget)?;
    let extent = store.extent()?;
    let mut limits = DecoderBudget::new(extent)?;
    store.visit_string_lengths(|count| limits.string(count))?;
    limits.begin_nodes()?;
    for node in 0..extent.nodes {
        limits.node(store.header(node)?)?;
    }
    admission::validate(store)
}
#[cfg(feature = "web-host-sankey")]
pub fn admit_outbound<S: AdmissionRead>(
    store: &mut S,
    metadata_bytes: usize,
    budget: usize,
) -> Result<(), PeleError> {
    admission::validate(store)?;
    canonical_frame_bytes(store, metadata_bytes, budget).map(|_| ())
}
