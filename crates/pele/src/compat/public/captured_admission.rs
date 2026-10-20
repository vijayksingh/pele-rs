//! Ordered admission over immutable captured storage. Arena slices stay borrowed.
use super::{Atom, MethodId, ModelGraph, Node, Property, PrototypeId};
use crate::compat::{ErrorCode, PeleError};
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub(super) struct Extent {
    pub(super) version: u32,
    pub(super) nodes: usize,
    pub(super) strings: usize,
}
#[derive(Clone, Copy)]
pub(super) enum Collection {
    None,
    Map(usize),
    Set(usize),
}
#[derive(Clone, Copy)]
pub(super) struct Header {
    pub(super) properties: usize,
    pub(super) prototype: Option<PrototypeId>,
    pub(super) collection: Collection,
}

// Static dispatch is intentional. The caller's closure owns validation policy;
// storage only lends records. Flat adapters may lend bounded scratch batches,
// while the arena lends its entire remaining slice with no copy or allocation.
pub(super) trait AdmissionRead {
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
            prototype: match node {
                Node::Instance { prototype, .. } => Some(*prototype),
                _ => None,
            },
            collection: match node {
                Node::Map { entries, .. } => Collection::Map(entries.len()),
                Node::Set { values, .. } => Collection::Set(values.len()),
                _ => Collection::None,
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
fn atom(extent: Extent, atom: &Atom) -> Result<(), PeleError> {
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
fn properties<S: AdmissionRead>(
    store: &mut S,
    node: usize,
    total: usize,
    mut check: impl FnMut(&Property) -> Result<(), PeleError>,
) -> Result<(), PeleError> {
    let mut start = 0;
    while start < total {
        start = store.with_properties(node, start, |values| {
            let end = next(start, values.len(), total)?;
            for property in values {
                check(property)?;
            }
            Ok(end)
        })?;
    }
    Ok(())
}
fn items<S: AdmissionRead>(
    store: &mut S,
    node: usize,
    collection: Collection,
    mut check: impl FnMut(&Atom) -> Result<(), PeleError>,
) -> Result<(), PeleError> {
    match collection {
        Collection::None => {}
        Collection::Map(total) => {
            let mut start = 0;
            while start < total {
                start = store.with_map_entries(node, start, |entries| {
                    let end = next(start, entries.len(), total)?;
                    for (key, value) in entries {
                        check(key)?;
                        check(value)?;
                    }
                    Ok(end)
                })?;
            }
        }
        Collection::Set(total) => {
            let mut start = 0;
            while start < total {
                start = store.with_set_values(node, start, |values| {
                    let end = next(start, values.len(), total)?;
                    for value in values {
                        check(value)?;
                    }
                    Ok(end)
                })?;
            }
        }
    }
    Ok(())
}

pub(super) fn validate<S: AdmissionRead>(store: &mut S) -> Result<(), PeleError> {
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
        let references = match header.collection {
            Collection::None => 0,
            Collection::Map(count) => count.saturating_mul(2),
            Collection::Set(count) => count,
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
            atom(extent, &property.value)
        })?;
        items(store, node, header.collection, |value| atom(extent, value))?;
    }
    Ok(())
}

pub(super) fn validate_schema_ids<S: AdmissionRead>(
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
        if let Some(prototype) = header.prototype {
            if !prototypes.contains(&prototype) {
                return Err(invalid("Unknown public instance prototype"));
            }
        }
        properties(store, node, header.properties, |property| {
            check(&property.value)
        })?;
        items(store, node, header.collection, check)?;
    }
    Ok(())
}
