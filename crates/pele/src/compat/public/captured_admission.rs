//! Ordered admission over immutable captured storage. Arena slices stay borrowed.
use super::{Atom, ErrorObjectKind, MethodId, ModelGraph, Node, Property, PrototypeId};
use crate::compat::{ErrorCode, PeleError};
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub struct Extent {
    pub version: u32,
    pub nodes: usize,
    pub strings: usize,
}
#[derive(Clone, Copy)]
// Auxiliary payloads are consumed by the opt-in canonical frame counter.
#[cfg_attr(not(feature = "web-host-sankey"), allow(dead_code))]
pub enum AdmissionKind {
    Object,
    Instance(PrototypeId),
    Array(u32),
    Map(usize),
    Set(usize),
    Date(u64),
    Error(ErrorObjectKind),
    /// Observed scalar encoding markers: size counting must precede rejection.
    InvalidArray,
    InvalidError,
}
#[derive(Clone, Copy)]
pub struct Header {
    pub properties: usize,
    pub kind: AdmissionKind,
}
/// A borrowed arena atom or one decoded stack scalar; neither variant allocates.
pub struct ScalarAtomRef<'a>(ScalarAtom<'a>);
enum ScalarAtom<'a> {
    Borrowed(&'a Atom),
    #[cfg(feature = "web-host-sankey")]
    Owned(Atom),
}
impl<'a> ScalarAtomRef<'a> {
    fn borrowed(value: &'a Atom) -> Self {
        Self(ScalarAtom::Borrowed(value))
    }
    #[cfg(feature = "web-host-sankey")]
    pub(crate) fn owned(value: Atom) -> Self {
        Self(ScalarAtom::Owned(value))
    }
    pub fn with_atom<R>(&self, visit: impl FnOnce(&Atom) -> R) -> R {
        match &self.0 {
            ScalarAtom::Borrowed(v) => visit(v),
            #[cfg(feature = "web-host-sankey")]
            ScalarAtom::Owned(v) => visit(v),
        }
    }
}
pub struct ScalarPropertyRef<'a> {
    pub key: u32,
    pub flags: u8,
    pub value: ScalarAtomRef<'a>,
}
// Static dispatch is intentional. The caller's closure owns validation policy;
// storage only lends records. Flat adapters may lend bounded scratch batches,
// while the arena lends its entire remaining slice with no copy or allocation.
pub trait AdmissionRead {
    fn extent(&mut self) -> Result<Extent, PeleError>;
    fn with_root<R>(
        &mut self,
        visit: impl FnOnce(&Atom) -> Result<R, PeleError>,
    ) -> Result<R, PeleError>;
    fn visit_string_lengths(
        &mut self,
        visit: impl FnMut(usize) -> Result<(), PeleError>,
    ) -> Result<(), PeleError>;
    fn header(&mut self, node: usize) -> Result<Header, PeleError>;
    fn with_properties<R>(
        &mut self,
        node: usize,
        start: usize,
        visit: impl FnOnce(&[Property]) -> Result<R, PeleError>,
    ) -> Result<R, PeleError>;
    fn with_map_entries<R>(
        &mut self,
        node: usize,
        start: usize,
        visit: impl FnOnce(&[(Atom, Atom)]) -> Result<R, PeleError>,
    ) -> Result<R, PeleError>;
    fn with_set_values<R>(
        &mut self,
        node: usize,
        start: usize,
        visit: impl FnOnce(&[Atom]) -> Result<R, PeleError>,
    ) -> Result<R, PeleError>;
    fn visit_properties(
        &mut self,
        node: usize,
        start: usize,
        remaining: usize,
        mut visit: impl FnMut(ScalarPropertyRef<'_>) -> Result<(), PeleError>,
    ) -> Result<usize, PeleError> {
        self.with_properties(node, start, |values| {
            next(0, values.len(), remaining)?;
            for p in values {
                visit(ScalarPropertyRef {
                    key: p.key,
                    flags: p.flags,
                    value: ScalarAtomRef::borrowed(&p.value),
                })?;
            }
            Ok(values.len())
        })
    }
    fn visit_map_entries(
        &mut self,
        node: usize,
        start: usize,
        remaining: usize,
        mut visit: impl FnMut(ScalarAtomRef<'_>, ScalarAtomRef<'_>) -> Result<(), PeleError>,
    ) -> Result<usize, PeleError> {
        self.with_map_entries(node, start, |values| {
            next(0, values.len(), remaining)?;
            for (k, v) in values {
                visit(ScalarAtomRef::borrowed(k), ScalarAtomRef::borrowed(v))?;
            }
            Ok(values.len())
        })
    }
    fn visit_set_values(
        &mut self,
        node: usize,
        start: usize,
        remaining: usize,
        mut visit: impl FnMut(ScalarAtomRef<'_>) -> Result<(), PeleError>,
    ) -> Result<usize, PeleError> {
        self.with_set_values(node, start, |values| {
            next(0, values.len(), remaining)?;
            for v in values {
                visit(ScalarAtomRef::borrowed(v))?;
            }
            Ok(values.len())
        })
    }
}

pub(super) struct ArenaAdmission<'g>(pub(super) &'g ModelGraph);
impl AdmissionRead for ArenaAdmission<'_> {
    fn extent(&mut self) -> Result<Extent, PeleError> {
        Ok(Extent {
            version: self.0.version,
            nodes: self.0.nodes.len(),
            strings: self.0.strings.len(),
        })
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
        for string in &self.0.strings {
            visit(string.0.len())?;
        }
        Ok(())
    }
    fn header(&mut self, node: usize) -> Result<Header, PeleError> {
        let node = self.0.nodes.get(node).ok_or_else(protocol)?;
        Ok(Header {
            properties: node.properties().len(),
            kind: match node {
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
    fn with_properties<R>(
        &mut self,
        node: usize,
        start: usize,
        visit: impl FnOnce(&[Property]) -> Result<R, PeleError>,
    ) -> Result<R, PeleError> {
        let node = self.0.nodes.get(node).ok_or_else(protocol)?;
        visit(node.properties().get(start..).ok_or_else(protocol)?)
    }
    fn with_map_entries<R>(
        &mut self,
        node: usize,
        start: usize,
        visit: impl FnOnce(&[(Atom, Atom)]) -> Result<R, PeleError>,
    ) -> Result<R, PeleError> {
        match self.0.nodes.get(node) {
            Some(Node::Map { entries, .. }) => visit(entries.get(start..).ok_or_else(protocol)?),
            _ => Err(protocol()),
        }
    }
    fn with_set_values<R>(
        &mut self,
        node: usize,
        start: usize,
        visit: impl FnOnce(&[Atom]) -> Result<R, PeleError>,
    ) -> Result<R, PeleError> {
        match self.0.nodes.get(node) {
            Some(Node::Set { values, .. }) => visit(values.get(start..).ok_or_else(protocol)?),
            _ => Err(protocol()),
        }
    }
}
fn invalid(message: &str) -> PeleError {
    PeleError::new(message, ErrorCode::Semantic)
}
fn protocol() -> PeleError {
    invalid("Invalid captured admission storage")
}
fn next(start: usize, count: usize, total: usize) -> Result<usize, PeleError> {
    if count == 0 {
        return Err(protocol());
    }
    start
        .checked_add(count)
        .filter(|end| *end <= total)
        .ok_or_else(protocol)
}
pub(super) fn atom(extent: Extent, atom: &Atom) -> Result<(), PeleError> {
    match atom {
        Atom::Ref(id) if *id as usize >= extent.nodes => {
            Err(invalid("Invalid public object reference"))
        }
        Atom::String(id) if *id as usize >= extent.strings => {
            Err(invalid("Invalid public string reference"))
        }
        _ => Ok(()),
    }
}
pub(super) fn properties<S: AdmissionRead>(
    store: &mut S,
    node: usize,
    total: usize,
    mut check: impl FnMut(ScalarPropertyRef<'_>) -> Result<(), PeleError>,
) -> Result<(), PeleError> {
    let mut start = 0;
    while start < total {
        let count = store.visit_properties(node, start, total - start, &mut check)?;
        start = next(start, count, total)?;
    }
    Ok(())
}
pub(super) fn items<S: AdmissionRead>(
    store: &mut S,
    node: usize,
    collection: AdmissionKind,
    mut check: impl FnMut(&Atom) -> Result<(), PeleError>,
) -> Result<(), PeleError> {
    match collection {
        AdmissionKind::Map(total) => {
            let mut start = 0;
            while start < total {
                let count = store.visit_map_entries(node, start, total - start, |key, value| {
                    key.with_atom(&mut check)?;
                    value.with_atom(&mut check)
                })?;
                start = next(start, count, total)?;
            }
        }
        AdmissionKind::Set(total) => {
            let mut start = 0;
            while start < total {
                let count = store.visit_set_values(node, start, total - start, |value| {
                    value.with_atom(&mut check)
                })?;
                start = next(start, count, total)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub(crate) fn validate<S: AdmissionRead>(store: &mut S) -> Result<(), PeleError> {
    let extent = store.extent()?;
    if extent.version != 1 {
        return Err(invalid("Unsupported public graph version"));
    }
    let mut units = 0usize;
    let mut strings = 0usize;
    store.visit_string_lengths(|length| {
        if strings >= extent.strings {
            return Err(protocol());
        }
        strings += 1;
        units = units
            .checked_add(length)
            .ok_or_else(|| invalid("Public graph size overflow"))?;
        Ok(())
    })?;
    if strings != extent.strings {
        return Err(protocol());
    }
    if extent.nodes > 1_000_000 || extent.strings > 1_000_000 || units > 16_000_000 {
        return Err(PeleError::new(
            "Public model graph exceeds work budget",
            ErrorCode::Limit,
        ));
    }
    store.with_root(|value| atom(extent, value))?;
    let mut work = 0usize;
    for node in 0..extent.nodes {
        let header = store.header(node)?;
        if matches!(
            header.kind,
            AdmissionKind::InvalidArray | AdmissionKind::InvalidError
        ) {
            return Err(protocol());
        }
        let references = match header.kind {
            AdmissionKind::Map(count) => count.saturating_mul(2),
            AdmissionKind::Set(count) => count,
            _ => 0,
        };
        work = work
            .saturating_add(header.properties)
            .saturating_add(references);
        if work > 8_000_000 {
            return Err(PeleError::new(
                "Public graph exceeds work budget",
                ErrorCode::Limit,
            ));
        }
        properties(store, node, header.properties, |property| {
            if property.key as usize >= extent.strings || property.flags & !7 != 0 {
                return Err(invalid("Invalid public property"));
            }
            property.value.with_atom(|value| atom(extent, value))
        })?;
        items(store, node, header.kind, |value| atom(extent, value))?;
    }
    Ok(())
}

pub(crate) fn validate_schema_ids<S: AdmissionRead>(
    store: &mut S,
    methods: &HashSet<MethodId>,
    prototypes: &HashSet<PrototypeId>,
) -> Result<(), PeleError> {
    let check = |value: &Atom| -> Result<(), PeleError> {
        if let Atom::BuiltinFunction(id) = value {
            if !methods.contains(&MethodId(*id)) {
                return Err(invalid("Unknown public builtin function"));
            }
        }
        Ok(())
    };
    let extent = store.extent()?;
    store.with_root(check)?;
    for node in 0..extent.nodes {
        let header = store.header(node)?;
        if let AdmissionKind::Instance(prototype) = header.kind {
            if !prototypes.contains(&prototype) {
                return Err(invalid("Unknown public instance prototype"));
            }
        }
        properties(store, node, header.properties, |property| {
            property.value.with_atom(check)
        })?;
        items(store, node, header.kind, check)?;
    }
    Ok(())
}
