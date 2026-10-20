//! Count canonical captured slots without a graph mirror or encoded byte buffer.
use super::{binary, captured_admission as admission, Atom};
use crate::compat::PeleError;
use admission::{AdmissionKind, AdmissionRead};

struct Counter {
    bytes: usize,
    budget: usize,
}
impl Counter {
    fn put(&mut self, count: usize) -> Result<(), PeleError> {
        binary::check_bytes(self.bytes, count, self.budget)?;
        self.bytes = self.bytes.checked_add(count).ok_or_else(binary::limit)?;
        Ok(())
    }
    fn count(&mut self, count: usize) -> Result<(), PeleError> {
        u32::try_from(count).map_err(|_| binary::limit())?;
        self.put(4)
    }
    fn atom(&mut self, atom: &Atom) -> Result<(), PeleError> {
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
    let mut out = Counter { bytes: 0, budget };
    out.put(8)?;
    out.put(4)?;
    out.put(8)?;
    out.count(metadata_bytes)?;
    store.with_root(|atom| out.atom(atom))?;
    out.count(extent.strings)?;
    let mut seen = 0usize;
    store.visit_string_lengths(|units| {
        if seen >= extent.strings {
            return Err(binary::invalid());
        }
        seen += 1;
        out.count(units)?;
        out.put(units.checked_mul(2).ok_or_else(binary::limit)?)
    })?;
    if seen != extent.strings {
        return Err(binary::invalid());
    }
    out.count(extent.nodes)?;
    for node in 0..extent.nodes {
        let header = store.header(node)?;
        out.put(1)?;
        match header.kind {
            AdmissionKind::Object => {}
            AdmissionKind::Instance(_) | AdmissionKind::Array(_) | AdmissionKind::InvalidArray => {
                out.put(4)?
            }
            AdmissionKind::Date(_) => out.put(8)?,
            AdmissionKind::Error(_) | AdmissionKind::InvalidError => out.put(1)?,
            AdmissionKind::Map(count) | AdmissionKind::Set(count) => {
                out.count(count)?;
                admission::items(store, node, header.kind, |atom| out.atom(atom))?;
            }
        }
        out.count(header.properties)?;
        admission::properties(store, node, header.properties, |property| {
            out.put(5)?;
            out.atom(&property.value)
        })?;
    }
    out.put(metadata_bytes)?;
    Ok(out.bytes)
}
/// Canonical observed slots have no malformed byte encodings. Keep the original
/// binary API's parser and its physical-length checks for actual byte input.
pub fn admit_observed<S: AdmissionRead>(
    store: &mut S,
    metadata_bytes: usize,
    budget: usize,
) -> Result<(), PeleError> {
    canonical_frame_bytes(store, metadata_bytes, budget)?;
    let extent = store.extent()?;
    if extent.version != 1 {
        return Err(binary::invalid());
    }
    binary::check_count(extent.strings, binary::MAX_TABLE)?;
    let mut units = 0usize;
    store.visit_string_lengths(|count| {
        binary::check_count(count, binary::MAX_UNITS)?;
        binary::add_bounded(&mut units, count, binary::MAX_UNITS)
    })?;
    binary::check_count(extent.nodes, binary::MAX_TABLE)?;
    let mut work = 0usize;
    for node in 0..extent.nodes {
        let header = store.header(node)?;
        match header.kind {
            AdmissionKind::InvalidArray | AdmissionKind::InvalidError => {
                return Err(binary::invalid())
            }
            AdmissionKind::Map(count) => {
                binary::check_count(count, binary::MAX_ENTRIES / 2)?;
                binary::add_bounded(&mut work, count * 2, binary::MAX_ENTRIES)?;
            }
            AdmissionKind::Set(count) => {
                binary::check_count(count, binary::MAX_ENTRIES)?;
                binary::add_bounded(&mut work, count, binary::MAX_ENTRIES)?;
            }
            _ => {}
        }
        binary::check_count(header.properties, binary::MAX_ENTRIES)?;
        binary::add_bounded(&mut work, header.properties, binary::MAX_ENTRIES)?;
    }
    admission::validate(store)
}
pub fn admit_outbound<S: AdmissionRead>(
    store: &mut S,
    metadata_bytes: usize,
    budget: usize,
) -> Result<(), PeleError> {
    admission::validate(store)?;
    canonical_frame_bytes(store, metadata_bytes, budget).map(|_| ())
}
