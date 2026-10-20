//! Sole generic canonical storage for the opt-in native captured backend.
//! All exported tables are POD. Native algorithms never reconstruct ModelGraph.
mod observation;
mod pages;
mod read;
mod write;
use super::{Atom, ErrorObjectKind, Node, Property, PrototypeId};
use crate::compat::{ErrorCode, PeleError};
pub use observation::{NativeObservation, ObservedCapture};
use pages::Pages;
use std::{
    collections::{hash_map::RandomState, HashMap},
    hash::{BuildHasher, Hash, Hasher},
    sync::Arc,
};
pub use write::NativeSourceWriter;

fn fault() -> PeleError {
    PeleError::new("Invalid native captured storage", ErrorCode::Semantic)
}
fn word(n: usize) -> Result<u32, PeleError> {
    u32::try_from(n).map_err(|_| fault())
}
fn capacity(n: usize) -> Result<usize, PeleError> {
    if n == 0 {
        Ok(0)
    } else {
        n.checked_next_power_of_two().ok_or_else(fault)
    }
}
const ABSENT: [u32; 7] = [u32::MAX, 0, 0, 0, 0, 0, 0];
fn words(atom: &Atom) -> [u32; 3] {
    match atom {
        Atom::Undefined => [0, 0, 0],
        Atom::Null => [1, 0, 0],
        Atom::Bool(false) => [2, 0, 0],
        Atom::Bool(true) => [3, 0, 0],
        Atom::Number(n) => {
            let bits = n.to_bits();
            [4, bits as u32, (bits >> 32) as u32]
        }
        Atom::String(id) => [5, *id, 0],
        Atom::Ref(id) => [6, *id, 0],
        Atom::BuiltinFunction(id) => [7, *id, 0],
        Atom::HostReference(id) => [8, *id, 0],
    }
}
fn atom(value: [u32; 3]) -> Result<Atom, PeleError> {
    let [tag, lo, hi] = value;
    Ok(match tag {
        0 if lo == 0 && hi == 0 => Atom::Undefined,
        1 if lo == 0 && hi == 0 => Atom::Null,
        2 if lo == 0 && hi == 0 => Atom::Bool(false),
        3 if lo == 0 && hi == 0 => Atom::Bool(true),
        4 => Atom::Number(f64::from_bits(lo as u64 | ((hi as u64) << 32))),
        5 if hi == 0 => Atom::String(lo),
        6 if hi == 0 => Atom::Ref(lo),
        7 if hi == 0 => Atom::BuiltinFunction(lo),
        8 if hi == 0 => Atom::HostReference(lo),
        _ => return Err(fault()),
    })
}
fn property_words(p: &Property) -> [u32; 5] {
    let a = words(&p.value);
    [p.key, p.flags as u32, a[0], a[1], a[2]]
}
fn property(p: [u32; 5]) -> Result<Property, PeleError> {
    Ok(Property {
        key: p[0],
        flags: u8::try_from(p[1]).map_err(|_| fault())?,
        value: atom([p[2], p[3], p[4]])?,
    })
}
fn error_word(kind: ErrorObjectKind) -> u32 {
    match kind {
        ErrorObjectKind::Error => 0,
        ErrorObjectKind::Pele => 1,
        ErrorObjectKind::Type => 2,
        ErrorObjectKind::Range => 3,
        ErrorObjectKind::Syntax => 4,
        ErrorObjectKind::Reference => 5,
        ErrorObjectKind::Eval => 6,
        ErrorObjectKind::Uri => 7,
    }
}
fn error_kind(word: u32) -> Result<ErrorObjectKind, PeleError> {
    Ok(match word {
        0 => ErrorObjectKind::Error,
        1 => ErrorObjectKind::Pele,
        2 => ErrorObjectKind::Type,
        3 => ErrorObjectKind::Range,
        4 => ErrorObjectKind::Syntax,
        5 => ErrorObjectKind::Reference,
        6 => ErrorObjectKind::Eval,
        7 => ErrorObjectKind::Uri,
        _ => return Err(fault()),
    })
}

/// String payload exists once. Index stores keyed hashes/IDs and exact collision chains.
#[derive(Clone)]
pub struct FlatStringHistory {
    descriptors: Pages<[u32; 2]>,
    units: Pages<u16>,
    hash: RandomState,
    index: Arc<Vec<Arc<HashMap<u64, Vec<u32>>>>>,
}
impl Default for FlatStringHistory {
    fn default() -> Self {
        Self {
            descriptors: Pages::default(),
            units: Pages::default(),
            hash: RandomState::new(),
            index: Arc::new(vec![Arc::new(HashMap::new()); 256]),
        }
    }
}
impl FlatStringHistory {
    pub fn len(&self) -> usize {
        self.descriptors.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn common_prefix_len(&self, other: &Self) -> usize {
        let n = self.len().min(other.len());
        for i in 0..n {
            let Ok([a, la]) = self.descriptors.get(i) else {
                return i;
            };
            let Ok([b, lb]) = other.descriptors.get(i) else {
                return i;
            };
            if la != lb {
                return i;
            }
            if !self
                .units
                .equal_range(a as usize, &other.units, b as usize, la as usize)
                .unwrap_or(false)
            {
                return i;
            }
        }
        n
    }

    fn digest(&self, units: impl Iterator<Item = u16>, length: usize) -> u64 {
        let mut hash = self.hash.build_hasher();
        length.hash(&mut hash);
        for unit in units {
            unit.hash(&mut hash);
        }
        hash.finish()
    }
    fn is_text(&self, id: u32, text: &str) -> Result<bool, PeleError> {
        let [start, len] = self.descriptors.get(id as usize)?;
        let mut expected = text.encode_utf16();
        for i in 0..len as usize {
            if expected.next() != Some(self.units.get(start as usize + i)?) {
                return Ok(false);
            }
        }
        Ok(expected.next().is_none())
    }
    fn equal(&self, a: u32, b: u32) -> Result<bool, PeleError> {
        if a == b {
            return Ok(true);
        }
        let [sa, la] = self.descriptors.get(a as usize)?;
        let [sb, lb] = self.descriptors.get(b as usize)?;
        if la != lb {
            return Ok(false);
        }
        for i in 0..la as usize {
            if self.units.get(sa as usize + i)? != self.units.get(sb as usize + i)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn first(&self, text: &str) -> Result<Option<u32>, PeleError> {
        let digest = self.digest(text.encode_utf16(), text.encode_utf16().count());
        self.first_hashed(text, digest)
    }
    fn first_hashed(&self, text: &str, digest: u64) -> Result<Option<u32>, PeleError> {
        if let Some(ids) = self.index[digest as usize % 256].get(&digest) {
            for id in ids {
                if self.is_text(*id, text)? {
                    return Ok(Some(*id));
                }
            }
        }
        Ok(None)
    }
    fn finish_string(&mut self, start: usize, length: usize) -> Result<u32, PeleError> {
        let id = word(self.descriptors.len())?;
        let mut hash = self.hash.build_hasher();
        length.hash(&mut hash);
        self.units.chunks(start, length, |chunk| {
            for unit in chunk {
                unit.hash(&mut hash);
            }
            Ok(())
        })?;
        let digest = hash.finish();
        self.commit_string(id, start, length, digest)
    }
    fn commit_string(
        &mut self,
        id: u32,
        start: usize,
        length: usize,
        digest: u64,
    ) -> Result<u32, PeleError> {
        self.descriptors.push([word(start)?, word(length)?])?;
        Arc::make_mut(&mut Arc::make_mut(&mut self.index)[digest as usize % 256])
            .entry(digest)
            .or_default()
            .push(id);
        Ok(id)
    }
    fn append(&mut self, units: &[u16]) -> Result<u32, PeleError> {
        let start = self.units.len();
        word(start.checked_add(units.len()).ok_or_else(fault)?)?;
        for unit in units {
            self.units.push(*unit)?;
        }
        self.finish_string(start, units.len())
    }
    fn intern(&mut self, text: &str) -> Result<u32, PeleError> {
        if let Some(id) = self.first(text)? {
            return Ok(id);
        }
        let start = self.units.len();
        let length = text.encode_utf16().count();
        word(start.checked_add(length).ok_or_else(fault)?)?;
        for unit in text.encode_utf16() {
            self.units.push(unit)?;
        }
        self.finish_string(start, length)
    }
}
#[derive(Clone)]
pub struct FlatEpoch {
    data: Data,
}
#[derive(Clone)]
struct Data {
    version: u32,
    root: [u32; 3],
    headers: Pages<[u32; 7]>,
    properties: Pages<[u32; 5]>,
    items: Pages<[u32; 3]>,
    strings: FlatStringHistory,
}
impl Default for Data {
    fn default() -> Self {
        Self {
            version: 1,
            root: [0, 0, 0],
            headers: Pages::default(),
            properties: Pages::default(),
            items: Pages::default(),
            strings: FlatStringHistory::default(),
        }
    }
}
pub struct FlatWorking {
    data: Data,
}
impl FlatEpoch {
    pub fn empty() -> Self {
        Self {
            data: Data::default(),
        }
    }
    pub fn compact(&mut self) -> Result<(), PeleError> {
        let mut working = self.fork();
        working.compact()?;
        *self = working.seal();
        Ok(())
    }
    pub fn fork(&self) -> FlatWorking {
        FlatWorking {
            data: self.data.clone(),
        }
    }
    pub fn string_history(&self) -> FlatStringHistory {
        self.data.strings.clone()
    }
    pub fn root_words(&self) -> [u32; 3] {
        self.data.root
    }
    pub fn logical_counts(&self) -> (usize, usize) {
        (self.data.headers.len(), self.data.strings.len())
    }
    pub fn version(&self) -> u32 {
        self.data.version
    }
    pub fn visit_header_pages(&self, mut visit: impl FnMut(usize, &[u32])) {
        self.data.headers.visit(|i, p| visit(i, p.as_flattened()))
    }
    pub fn visit_property_pages(&self, mut visit: impl FnMut(usize, &[u32])) {
        self.data
            .properties
            .visit(|i, p| visit(i, p.as_flattened()))
    }
    pub fn visit_item_pages(&self, mut visit: impl FnMut(usize, &[u32])) {
        self.data.items.visit(|i, p| visit(i, p.as_flattened()))
    }
    pub fn visit_string_pages(&self, mut visit: impl FnMut(usize, &[u32])) {
        self.data
            .strings
            .descriptors
            .visit(|i, p| visit(i, p.as_flattened()))
    }
    pub fn visit_unit_pages(&self, visit: impl FnMut(usize, &[u16])) {
        self.data.strings.units.visit(visit)
    }
    /// Compare logical records, never slack or physical span addresses.
    pub fn exact_eq(&self, other: &Self) -> bool {
        self.data.exact_eq(&other.data).unwrap_or(false)
    }
}
impl Data {
    fn exact_eq(&self, other: &Self) -> Result<bool, PeleError> {
        if self.version != other.version
            || self.root != other.root
            || self.headers.len() != other.headers.len()
            || self.strings.len() != other.strings.len()
        {
            return Ok(false);
        }
        for i in 0..self.strings.len() {
            let [a, n] = self.strings.descriptors.get(i)?;
            let [b, m] = other.strings.descriptors.get(i)?;
            if n != m {
                return Ok(false);
            }
            for j in 0..n as usize {
                if self.strings.units.get(a as usize + j)?
                    != other.strings.units.get(b as usize + j)?
                {
                    return Ok(false);
                }
            }
        }
        for i in 0..self.headers.len() {
            let a = self.headers.get(i)?;
            let b = other.headers.get(i)?;
            if a[0..3] != b[0..3] || a[4] != b[4] || a[6] != b[6] {
                return Ok(false);
            }
            for j in 0..a[4] as usize {
                if self.properties.get(a[3] as usize + j)?
                    != other.properties.get(b[3] as usize + j)?
                {
                    return Ok(false);
                }
            }
            let n = a[6] as usize * if a[0] == 3 { 2 } else { 1 };
            for j in 0..n {
                if self.items.get(a[5] as usize + j)? != other.items.get(b[5] as usize + j)? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
    fn header(&self, id: u32) -> Result<[u32; 7], PeleError> {
        let h = self.headers.get(id as usize)?;
        if h[0] == u32::MAX {
            Err(fault())
        } else {
            Ok(h)
        }
    }
}
impl FlatWorking {
    pub fn set_root(&mut self, root: Atom) {
        self.data.root = words(&root)
    }
    pub fn set_version(&mut self, version: u32) {
        self.data.version = version
    }
    pub fn append_string_units(&mut self, units: &[u16]) -> Result<u32, PeleError> {
        self.data.strings.append(units)
    }
    /// Commit already-written physical state without a fallible maintenance step.
    pub fn seal(self) -> FlatEpoch {
        FlatEpoch { data: self.data }
    }
    pub fn freeze(mut self) -> Result<FlatEpoch, PeleError> {
        self.compact()?;
        Ok(FlatEpoch { data: self.data })
    }
    fn compact(&mut self) -> Result<(), PeleError> {
        let mut properties = 0usize;
        let mut items = 0usize;
        for i in 0..self.data.headers.len() {
            let h = self.data.headers.get(i)?;
            properties = properties.checked_add(h[4] as usize).ok_or_else(fault)?;
            items = items
                .checked_add(h[6] as usize * if h[0] == 3 { 2 } else { 1 })
                .ok_or_else(fault)?;
        }
        let compact_properties =
            self.data.properties.len() > properties.saturating_mul(4).saturating_add(2048);
        let compact_items = self.data.items.len() > items.saturating_mul(4).saturating_add(2048);
        if !compact_properties && !compact_items {
            return Ok(());
        }
        let mut pp = Pages::default();
        let mut ip = Pages::default();
        for i in 0..self.data.headers.len() {
            let mut h = self.data.headers.get(i)?;
            if compact_properties {
                let start = pp.reserve_span(capacity(h[4] as usize)?)?;
                for j in 0..h[4] as usize {
                    pp.set(start + j, self.data.properties.get(h[3] as usize + j)?)?;
                }
                h[3] = word(start)?;
            }
            if compact_items {
                let n = h[6] as usize * if h[0] == 3 { 2 } else { 1 };
                let start = ip.reserve_span(capacity(n)?)?;
                for j in 0..n {
                    ip.set(start + j, self.data.items.get(h[5] as usize + j)?)?;
                }
                h[5] = word(start)?;
            }
            self.data.headers.set(i, h)?;
        }
        if compact_properties {
            self.data.properties = pp
        }
        if compact_items {
            self.data.items = ip
        }
        Ok(())
    }
}
