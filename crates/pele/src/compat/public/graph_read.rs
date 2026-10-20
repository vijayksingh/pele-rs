//! Borrowed logical graph export. Source-backed records lend bounded stack
//! batches; arenas lend their actual slices. No per-field dynamic dispatch.
use super::captured_admission::{AdmissionKind, AdmissionRead, Extent, Header};
use super::{Atom, JsString, ModelGraph, Node, Property};
use crate::compat::{ErrorCode, PeleError};
use std::borrow::Cow;

#[doc(hidden)]
pub trait PublicGraphRead: Send {
    fn source_layout(&self, _revision: u64) -> Result<Option<Vec<u8>>, PeleError> {
        Ok(None)
    }
    fn extent(&self) -> Extent;
    fn root(&self) -> Atom;
    fn header(&self, node: usize) -> Result<Header, PeleError>;
    fn string_length(&self, id: usize) -> Result<usize, PeleError>;
    fn string_lengths(
        &self,
        visit: &mut dyn FnMut(usize) -> Result<(), PeleError>,
    ) -> Result<(), PeleError>;
    fn string_chunks(
        &self,
        id: usize,
        visit: &mut dyn FnMut(&[u16]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError>;
    // Exactly one callback per call, including an empty tail. A nonempty tail
    // must make progress. The borrowed slice never survives the callback.
    fn properties(
        &self,
        node: usize,
        start: usize,
        visit: &mut dyn FnMut(&[Property]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError>;
    fn map_entries(
        &self,
        node: usize,
        start: usize,
        visit: &mut dyn FnMut(&[(Atom, Atom)]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError>;
    fn set_values(
        &self,
        node: usize,
        start: usize,
        visit: &mut dyn FnMut(&[Atom]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError>;
}
pub(crate) fn invalid() -> PeleError {
    PeleError::new("Invalid captured admission storage", ErrorCode::Semantic)
}
impl PublicGraphRead for ModelGraph {
    fn extent(&self) -> Extent {
        Extent {
            version: self.version,
            nodes: self.nodes.len(),
            strings: self.strings.len(),
        }
    }
    fn root(&self) -> Atom {
        self.root.clone()
    }
    fn header(&self, id: usize) -> Result<Header, PeleError> {
        let n = self.nodes.get(id).ok_or_else(invalid)?;
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
    fn string_length(&self, id: usize) -> Result<usize, PeleError> {
        Ok(self.strings.get(id).ok_or_else(invalid)?.0.len())
    }
    fn string_lengths(
        &self,
        v: &mut dyn FnMut(usize) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        for s in &self.strings {
            v(s.0.len())?;
        }
        Ok(())
    }
    fn string_chunks(
        &self,
        id: usize,
        v: &mut dyn FnMut(&[u16]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        for chunk in self.strings.get(id).ok_or_else(invalid)?.0.chunks(1024) {
            v(chunk)?;
        }
        Ok(())
    }
    fn properties(
        &self,
        id: usize,
        start: usize,
        v: &mut dyn FnMut(&[Property]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        v(self
            .nodes
            .get(id)
            .ok_or_else(invalid)?
            .properties()
            .get(start..)
            .ok_or_else(invalid)?)
    }
    fn map_entries(
        &self,
        id: usize,
        start: usize,
        v: &mut dyn FnMut(&[(Atom, Atom)]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        let Some(Node::Map { entries, .. }) = self.nodes.get(id) else {
            return Err(invalid());
        };
        v(entries.get(start..).ok_or_else(invalid)?)
    }
    fn set_values(
        &self,
        id: usize,
        start: usize,
        v: &mut dyn FnMut(&[Atom]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        let Some(Node::Set { values, .. }) = self.nodes.get(id) else {
            return Err(invalid());
        };
        v(values.get(start..).ok_or_else(invalid)?)
    }
}
#[doc(hidden)]
pub enum GraphReadView<'a> {
    Arena(Cow<'a, ModelGraph>),
    Logical(&'a dyn PublicGraphRead),
}
impl std::fmt::Debug for GraphReadView<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Arena(g) => f.debug_tuple("Arena").field(g).finish(),
            Self::Logical(g) => f
                .debug_struct("Logical")
                .field("nodes", &g.extent().nodes)
                .field("strings", &g.extent().strings)
                .finish(),
        }
    }
}
impl<'a> GraphReadView<'a> {
    pub fn source_layout(&self, revision: u64) -> Result<Option<Vec<u8>>, PeleError> {
        self.reader().source_layout(revision)
    }
    pub fn canonical_binary_size(
        &self,
        metadata_bytes: usize,
        budget: usize,
    ) -> Result<usize, PeleError> {
        let mut read = ReadAdapter(self.reader());
        super::captured_admission::validate(&mut read)?;
        let total =
            super::captured_transport::canonical_frame_bytes(&mut read, metadata_bytes, budget)?;
        Ok(total - metadata_bytes)
    }
    pub fn reader(&self) -> &dyn PublicGraphRead {
        match self {
            Self::Arena(g) => g.as_ref(),
            Self::Logical(g) => *g,
        }
    }
    pub fn into_model(self) -> Result<Cow<'a, ModelGraph>, PeleError> {
        match self {
            Self::Arena(g) => Ok(g),
            Self::Logical(g) => materialize(g).map(Cow::Owned),
        }
    }
    pub fn encode_binary(
        &self,
        revision: u64,
        metadata: &[u8],
        budget: usize,
    ) -> Result<Vec<u8>, PeleError> {
        match self {
            Self::Arena(g) => super::binary::encode_binary_exchange(revision, g, metadata, budget),
            Self::Logical(g) => super::binary::encode_read_exchange(revision, *g, metadata, budget),
        }
    }
}
pub(crate) struct ReadAdapter<'a>(pub &'a dyn PublicGraphRead);
impl AdmissionRead for ReadAdapter<'_> {
    fn extent(&mut self) -> Result<Extent, PeleError> {
        Ok(self.0.extent())
    }
    fn with_root<R>(
        &mut self,
        v: impl FnOnce(&Atom) -> Result<R, PeleError>,
    ) -> Result<R, PeleError> {
        v(&self.0.root())
    }
    fn visit_string_lengths(
        &mut self,
        mut v: impl FnMut(usize) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        self.0.string_lengths(&mut v)
    }
    fn header(&mut self, id: usize) -> Result<Header, PeleError> {
        self.0.header(id)
    }
    fn with_properties<R>(
        &mut self,
        id: usize,
        start: usize,
        v: impl FnOnce(&[Property]) -> Result<R, PeleError>,
    ) -> Result<R, PeleError> {
        let mut visit = Some(v);
        let mut result = None;
        self.0.properties(id, start, &mut |values| {
            let f = visit.take().ok_or_else(invalid)?;
            result = Some(f(values)?);
            Ok(())
        })?;
        result.ok_or_else(invalid)
    }
    fn with_map_entries<R>(
        &mut self,
        id: usize,
        start: usize,
        v: impl FnOnce(&[(Atom, Atom)]) -> Result<R, PeleError>,
    ) -> Result<R, PeleError> {
        let mut visit = Some(v);
        let mut result = None;
        self.0.map_entries(id, start, &mut |values| {
            let f = visit.take().ok_or_else(invalid)?;
            result = Some(f(values)?);
            Ok(())
        })?;
        result.ok_or_else(invalid)
    }
    fn with_set_values<R>(
        &mut self,
        id: usize,
        start: usize,
        v: impl FnOnce(&[Atom]) -> Result<R, PeleError>,
    ) -> Result<R, PeleError> {
        let mut visit = Some(v);
        let mut result = None;
        self.0.set_values(id, start, &mut |values| {
            let f = visit.take().ok_or_else(invalid)?;
            result = Some(f(values)?);
            Ok(())
        })?;
        result.ok_or_else(invalid)
    }
}
pub(crate) fn visit_properties(
    g: &dyn PublicGraphRead,
    id: usize,
    mut v: impl FnMut(&[Property]) -> Result<(), PeleError>,
) -> Result<(), PeleError> {
    let total = g.header(id)?.properties;
    let mut at = 0;
    while at < total {
        let start = at;
        g.properties(id, start, &mut |p| {
            at = at
                .checked_add(p.len())
                .filter(|n| *n <= total)
                .ok_or_else(invalid)?;
            v(p)
        })?;
        if at == start {
            return Err(invalid());
        }
    }
    Ok(())
}
pub(crate) fn visit_map(
    g: &dyn PublicGraphRead,
    id: usize,
    mut v: impl FnMut(&[(Atom, Atom)]) -> Result<(), PeleError>,
) -> Result<(), PeleError> {
    let AdmissionKind::Map(total) = g.header(id)?.kind else {
        return Err(invalid());
    };
    let mut at = 0;
    while at < total {
        let start = at;
        g.map_entries(id, start, &mut |p| {
            at = at
                .checked_add(p.len())
                .filter(|n| *n <= total)
                .ok_or_else(invalid)?;
            v(p)
        })?;
        if at == start {
            return Err(invalid());
        }
    }
    Ok(())
}
pub(crate) fn visit_set(
    g: &dyn PublicGraphRead,
    id: usize,
    mut v: impl FnMut(&[Atom]) -> Result<(), PeleError>,
) -> Result<(), PeleError> {
    let AdmissionKind::Set(total) = g.header(id)?.kind else {
        return Err(invalid());
    };
    let mut at = 0;
    while at < total {
        let start = at;
        g.set_values(id, start, &mut |p| {
            at = at
                .checked_add(p.len())
                .filter(|n| *n <= total)
                .ok_or_else(invalid)?;
            v(p)
        })?;
        if at == start {
            return Err(invalid());
        }
    }
    Ok(())
}
pub(crate) fn materialize_node(g: &dyn PublicGraphRead, id: usize) -> Result<Node, PeleError> {
    let h = g.header(id)?;
    let mut properties = Vec::with_capacity(h.properties);
    visit_properties(g, id, |p| {
        properties.extend_from_slice(p);
        Ok(())
    })?;
    Ok(match h.kind {
        AdmissionKind::Object => Node::Object { properties },
        AdmissionKind::Instance(prototype) => Node::Instance {
            prototype,
            properties,
        },
        AdmissionKind::Array(length) => Node::Array { length, properties },
        AdmissionKind::Date(bits) => Node::Date {
            milliseconds: f64::from_bits(bits),
            properties,
        },
        AdmissionKind::Error(error_type) => Node::Error {
            error_type,
            properties,
        },
        AdmissionKind::Map(count) => {
            let mut entries = Vec::with_capacity(count);
            visit_map(g, id, |p| {
                entries.extend_from_slice(p);
                Ok(())
            })?;
            Node::Map {
                entries,
                properties,
            }
        }
        AdmissionKind::Set(count) => {
            let mut values = Vec::with_capacity(count);
            visit_set(g, id, |p| {
                values.extend_from_slice(p);
                Ok(())
            })?;
            Node::Set { values, properties }
        }
        AdmissionKind::InvalidArray | AdmissionKind::InvalidError => return Err(invalid()),
    })
}
pub(crate) fn materialize(g: &dyn PublicGraphRead) -> Result<ModelGraph, PeleError> {
    let e = g.extent();
    let mut strings = Vec::with_capacity(e.strings);
    for id in 0..e.strings {
        let mut units = Vec::with_capacity(g.string_length(id)?);
        g.string_chunks(id, &mut |c| {
            units.extend_from_slice(c);
            Ok(())
        })?;
        strings.push(JsString(units));
    }
    let mut nodes = Vec::with_capacity(e.nodes);
    for id in 0..e.nodes {
        nodes.push(materialize_node(g, id)?);
    }
    Ok(ModelGraph {
        version: e.version,
        root: g.root(),
        strings,
        nodes,
    })
}

use serde::{
    ser::{Error as _, SerializeSeq, SerializeStruct},
    Serialize, Serializer,
};
impl Serialize for GraphReadView<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Arena(g) => g.serialize(s),
            Self::Logical(g) => {
                let mut out = s.serialize_struct("ModelGraph", 4)?;
                out.serialize_field("version", &g.extent().version)?;
                out.serialize_field("root", &g.root())?;
                out.serialize_field("strings", &Strings(*g))?;
                out.serialize_field("nodes", &Nodes(*g))?;
                out.end()
            }
        }
    }
}
struct Strings<'a>(&'a dyn PublicGraphRead);
struct Units<'a>(&'a dyn PublicGraphRead, usize);
struct Nodes<'a>(&'a dyn PublicGraphRead);
struct NodeValue<'a>(&'a dyn PublicGraphRead, usize);
struct Properties<'a>(&'a dyn PublicGraphRead, usize);
struct MapValues<'a>(&'a dyn PublicGraphRead, usize);
struct SetValues<'a>(&'a dyn PublicGraphRead, usize);
impl Serialize for Strings<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(self.0.extent().strings))?;
        for id in 0..self.0.extent().strings {
            seq.serialize_element(&Units(self.0, id))?;
        }
        seq.end()
    }
}
impl Serialize for Units<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let n = self.0.string_length(self.1).map_err(S::Error::custom)?;
        let mut seq = s.serialize_seq(Some(n))?;
        let mut failure = None;
        let read = self.0.string_chunks(self.1, &mut |units| {
            for unit in units {
                if let Err(e) = seq.serialize_element(unit) {
                    failure = Some(e);
                    return Err(invalid());
                }
            }
            Ok(())
        });
        if let Some(e) = failure {
            return Err(e);
        }
        read.map_err(S::Error::custom)?;
        seq.end()
    }
}
impl Serialize for Nodes<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(self.0.extent().nodes))?;
        for id in 0..self.0.extent().nodes {
            seq.serialize_element(&NodeValue(self.0, id))?;
        }
        seq.end()
    }
}
impl Serialize for NodeValue<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let h = self.0.header(self.1).map_err(S::Error::custom)?;
        let fields = if matches!(h.kind, AdmissionKind::Object) {
            2
        } else {
            3
        };
        let mut out = s.serialize_struct("Node", fields)?;
        match h.kind {
            AdmissionKind::Object => out.serialize_field("kind", "object")?,
            AdmissionKind::Instance(p) => {
                out.serialize_field("kind", "instance")?;
                out.serialize_field("prototype", &p)?;
            }
            AdmissionKind::Array(n) => {
                out.serialize_field("kind", "array")?;
                out.serialize_field("length", &n)?;
            }
            AdmissionKind::Map(_) => {
                out.serialize_field("kind", "map")?;
                out.serialize_field("entries", &MapValues(self.0, self.1))?;
            }
            AdmissionKind::Set(_) => {
                out.serialize_field("kind", "set")?;
                out.serialize_field("values", &SetValues(self.0, self.1))?;
            }
            AdmissionKind::Date(bits) => {
                out.serialize_field("kind", "date")?;
                out.serialize_field("milliseconds", &format!("{bits:016x}"))?;
            }
            AdmissionKind::Error(kind) => {
                out.serialize_field("kind", "error")?;
                out.serialize_field("errorType", &kind)?;
            }
            _ => return Err(S::Error::custom(invalid())),
        };
        out.serialize_field("properties", &Properties(self.0, self.1))?;
        out.end()
    }
}
impl Serialize for Properties<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let n = self.0.header(self.1).map_err(S::Error::custom)?.properties;
        let mut seq = s.serialize_seq(Some(n))?;
        let mut failure = None;
        let read = visit_properties(self.0, self.1, |values| {
            for p in values {
                if let Err(e) = seq.serialize_element(p) {
                    failure = Some(e);
                    return Err(invalid());
                }
            }
            Ok(())
        });
        if let Some(e) = failure {
            return Err(e);
        }
        read.map_err(S::Error::custom)?;
        seq.end()
    }
}
impl Serialize for MapValues<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let AdmissionKind::Map(n) = self.0.header(self.1).map_err(S::Error::custom)?.kind else {
            return Err(S::Error::custom(invalid()));
        };
        let mut seq = s.serialize_seq(Some(n))?;
        let mut failure = None;
        let read = visit_map(self.0, self.1, |values| {
            for p in values {
                if let Err(e) = seq.serialize_element(p) {
                    failure = Some(e);
                    return Err(invalid());
                }
            }
            Ok(())
        });
        if let Some(e) = failure {
            return Err(e);
        }
        read.map_err(S::Error::custom)?;
        seq.end()
    }
}
impl Serialize for SetValues<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let AdmissionKind::Set(n) = self.0.header(self.1).map_err(S::Error::custom)?.kind else {
            return Err(S::Error::custom(invalid()));
        };
        let mut seq = s.serialize_seq(Some(n))?;
        let mut failure = None;
        let read = visit_set(self.0, self.1, |values| {
            for p in values {
                if let Err(e) = seq.serialize_element(p) {
                    failure = Some(e);
                    return Err(invalid());
                }
            }
            Ok(())
        });
        if let Some(e) = failure {
            return Err(e);
        }
        read.map_err(S::Error::custom)?;
        seq.end()
    }
}
