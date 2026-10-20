use super::super::{
    captured_write::{CapturedWrite, GraphCounts, MapMatch, OwnQuery},
    InitialWriter,
};
use super::*;
#[derive(Default)]
pub(super) struct Span {
    pub start: usize,
    pub len: usize,
    pub capacity: usize,
}
impl Span {
    pub fn append<T: Copy + Default + PartialEq>(
        &mut self,
        pages: &mut Pages<T>,
        value: T,
    ) -> Result<(), PeleError> {
        if self.len == self.capacity {
            let new_capacity = capacity(self.len.checked_add(1).ok_or_else(fault)?)?;
            let new_start = pages.reserve_span(new_capacity)?;
            for i in 0..self.len {
                pages.set(new_start + i, pages.get(self.start + i)?)?;
            }
            self.start = new_start;
            self.capacity = new_capacity;
        }
        pages.set(self.start + self.len, value)?;
        self.len += 1;
        Ok(())
    }
    pub fn existing(start: u32, len: usize) -> Result<Self, PeleError> {
        Ok(Self {
            start: start as usize,
            len,
            capacity: capacity(len)?,
        })
    }
}
impl FlatWorking {
    pub(super) fn commit_node(&mut self, id: u32, header: [u32; 7]) -> Result<(), PeleError> {
        while self.data.headers.len() <= id as usize {
            self.data.headers.push(ABSENT)?;
        }
        self.data.headers.set(id as usize, header)
    }
    fn add_node(&mut self, node: Node) -> Result<u32, PeleError> {
        self.add_node_with::<false>(node)
    }
    fn add_node_with<const FRESH: bool>(&mut self, node: Node) -> Result<u32, PeleError> {
        let id = word(self.data.headers.len())?;
        let props = node.properties();
        let pstart = if FRESH {
            self.data
                .properties
                .append_span_with(capacity(props.len())?, |i| {
                    Ok(props.get(i).map(property_words).unwrap_or_default())
                })?
        } else {
            let start = self.data.properties.reserve_span(capacity(props.len())?)?;
            for (i, p) in props.iter().enumerate() {
                self.data.properties.set(start + i, property_words(p))?;
            }
            start
        };
        let mut h = [0, 0, 0, word(pstart)?, word(props.len())?, 0, 0];
        match node {
            Node::Object { .. } => {}
            Node::Instance { prototype, .. } => {
                h[0] = 1;
                h[1] = prototype.0;
            }
            Node::Array { length, .. } => {
                h[0] = 2;
                h[1] = length;
            }
            Node::Date { milliseconds, .. } => {
                let bits = milliseconds.to_bits();
                h[0] = 5;
                h[1] = bits as u32;
                h[2] = (bits >> 32) as u32;
            }
            Node::Error { error_type, .. } => {
                h[0] = 6;
                h[1] = error_word(error_type);
            }
            Node::Map { entries, .. } => {
                h[0] = 3;
                h[6] = word(entries.len())?;
                let n = entries.len().checked_mul(2).ok_or_else(fault)?;
                let start = if FRESH {
                    self.data.items.append_span_with(capacity(n)?, |i| {
                        Ok(entries
                            .get(i / 2)
                            .map(|(k, v)| words(if i % 2 == 0 { k } else { v }))
                            .unwrap_or_default())
                    })?
                } else {
                    self.data.items.reserve_span(capacity(n)?)?
                };
                h[5] = word(start)?;
                if !FRESH {
                    for (i, (k, v)) in entries.iter().enumerate() {
                        self.data.items.set(start + 2 * i, words(k))?;
                        self.data.items.set(start + 2 * i + 1, words(v))?;
                    }
                }
            }
            Node::Set { values, .. } => {
                h[0] = 4;
                h[6] = word(values.len())?;
                let start = if FRESH {
                    self.data
                        .items
                        .append_span_with(capacity(values.len())?, |i| {
                            Ok(values.get(i).map(words).unwrap_or_default())
                        })?
                } else {
                    self.data.items.reserve_span(capacity(values.len())?)?
                };
                h[5] = word(start)?;
                if !FRESH {
                    for (i, v) in values.iter().enumerate() {
                        self.data.items.set(start + i, words(v))?;
                    }
                }
            }
        }
        if FRESH {
            self.data.headers.push(h)?;
        } else {
            self.commit_node(id, h)?;
        }
        Ok(id)
    }
    fn same_key(&self, a: &Atom, b: &Atom) -> Result<bool, PeleError> {
        Ok(match (a, b) {
            (Atom::Undefined, Atom::Undefined) | (Atom::Null, Atom::Null) => true,
            (Atom::Bool(a), Atom::Bool(b)) => a == b,
            (Atom::Number(a), Atom::Number(b)) => a == b || (a.is_nan() && b.is_nan()),
            (Atom::String(a), Atom::String(b)) => self.data.strings.equal(*a, *b)?,
            (Atom::Ref(a), Atom::Ref(b))
            | (Atom::BuiltinFunction(a), Atom::BuiltinFunction(b))
            | (Atom::HostReference(a), Atom::HostReference(b)) => a == b,
            _ => false,
        })
    }
}
impl CapturedWrite for FlatWorking {
    fn counts(&mut self) -> Result<GraphCounts, PeleError> {
        let mut work = 0usize;
        let mut units = 0usize;
        for i in 0..self.data.strings.len() {
            units = units.saturating_add(self.data.strings.descriptors.get(i)?[1] as usize);
        }
        for i in 0..self.data.headers.len() {
            let h = self.data.headers.get(i)?;
            if h[0] == u32::MAX {
                return Err(fault());
            }
            work = work
                .saturating_add(h[4] as usize)
                .saturating_add(match h[0] {
                    3 => (h[6] as usize).saturating_mul(2),
                    4 => h[6] as usize,
                    _ => 0,
                });
        }
        Ok(GraphCounts {
            nodes: self.data.headers.len(),
            strings: self.data.strings.len(),
            units,
            work,
        })
    }
    fn find_strings(&mut self, names: &[&str], out: &mut [Option<u32>]) -> Result<(), PeleError> {
        if names.len() != out.len() {
            return Err(fault());
        }
        for (name, id) in names.iter().zip(out.iter_mut()) {
            *id = self.data.strings.first(name)?;
        }
        Ok(())
    }
    fn intern(&mut self, text: &str) -> Result<u32, PeleError> {
        self.data.strings.intern(text)
    }
    fn append_node(&mut self, node: Node) -> Result<u32, PeleError> {
        self.add_node(node)
    }
    fn own_descriptors(
        &mut self,
        id: u32,
        names: &[&str],
        out: &mut [OwnQuery],
    ) -> Result<(), PeleError> {
        if names.len() != out.len() {
            return Err(fault());
        }
        for q in out.iter_mut() {
            *q = OwnQuery::default();
        }
        if names.is_empty() || id as usize >= self.data.headers.len() {
            return Ok(());
        }
        let h = self.data.headers.get(id as usize)?;
        if h[0] == u32::MAX {
            return Ok(());
        }
        for i in 0..h[4] as usize {
            let p = self.data.properties.get(h[3] as usize + i)?;
            for (name, q) in names.iter().zip(out.iter_mut()) {
                if self.data.strings.is_text(p[0], name)? {
                    if q.first_slot.is_none() {
                        q.first_slot = Some(word(i)?)
                    }
                    q.any_nonwritable |= p[1] & 2 == 0;
                }
            }
        }
        Ok(())
    }
    fn map_find(&mut self, id: u32, key: &Atom) -> Result<Option<MapMatch>, PeleError> {
        let h = self.data.header(id)?;
        if h[0] != 3 {
            return Err(fault());
        }
        for i in 0..h[6] as usize {
            let at = h[5] as usize + 2 * i;
            if self.same_key(&atom(self.data.items.get(at)?)?, key)? {
                return Ok(Some(MapMatch {
                    slot: word(i)?,
                    value: atom(self.data.items.get(at + 1)?)?,
                }));
            }
        }
        Ok(None)
    }
    fn replace_map_value(&mut self, id: u32, slot: u32, value: Atom) -> Result<(), PeleError> {
        let h = self.data.header(id)?;
        if h[0] != 3 || slot >= h[6] {
            return Err(fault());
        }
        self.data
            .items
            .set(h[5] as usize + slot as usize * 2 + 1, words(&value))
    }
    fn append_map_entry(&mut self, id: u32, key: Atom, value: Atom) -> Result<(), PeleError> {
        let mut h = self.data.header(id)?;
        if h[0] != 3 {
            return Err(fault());
        }
        let mut span = Span::existing(h[5], h[6] as usize * 2)?;
        span.append(&mut self.data.items, words(&key))?;
        span.append(&mut self.data.items, words(&value))?;
        h[5] = word(span.start)?;
        h[6] = word(span.len / 2)?;
        self.data.headers.set(id as usize, h)
    }
    fn replace_property_value(&mut self, id: u32, slot: u32, value: Atom) -> Result<(), PeleError> {
        let h = self.data.header(id)?;
        if slot >= h[4] {
            return Err(fault());
        }
        let at = h[3] as usize + slot as usize;
        let p = self.data.properties.get(at)?;
        let a = words(&value);
        self.data.properties.set(at, [p[0], p[1], a[0], a[1], a[2]])
    }
    fn append_property(&mut self, id: u32, p: Property) -> Result<(), PeleError> {
        let mut h = self.data.header(id)?;
        let mut span = Span::existing(h[3], h[4] as usize)?;
        span.append(&mut self.data.properties, property_words(&p))?;
        h[3] = word(span.start)?;
        h[4] = word(span.len)?;
        self.data.headers.set(id as usize, h)
    }
    fn replace_matching_property_values(
        &mut self,
        id: u32,
        key: &str,
        value: Atom,
    ) -> Result<(), PeleError> {
        let h = self.data.header(id)?;
        for i in 0..h[4] {
            let p = self.data.properties.get(h[3] as usize + i as usize)?;
            if self.data.strings.is_text(p[0], key)? {
                self.replace_property_value(id, i, value.clone())?;
            }
        }
        Ok(())
    }
    fn set_array_length(&mut self, id: u32, length: u32) -> Result<(), PeleError> {
        let mut h = self.data.header(id)?;
        if h[0] != 2 {
            return Err(fault());
        }
        h[1] = length;
        h[2] = 0;
        self.data.headers.set(id as usize, h)
    }
}
/// Fresh source-only writer. No JS calls, ModelGraph, or per-record property Vec.
pub struct NativeSourceWriter {
    working: FlatWorking,
}
impl FlatStringHistory {
    /// The source path has valid Rust text and needs no duplicate history row.
    /// Reuse the lookup digest when committing a newly appended UTF16 payload.
    fn intern_source(&mut self, text: &str) -> Result<u32, PeleError> {
        let length = text.encode_utf16().count();
        let digest = self.digest(text.encode_utf16(), length);
        if let Some(id) = self.first_hashed(text, digest)? {
            return Ok(id);
        }
        let start = self.units.len();
        word(start.checked_add(length).ok_or_else(fault)?)?;
        let mut units = text.encode_utf16();
        self.units.append_span_with(length, |_| {
            Ok(units.next().expect("validated UTF16 length"))
        })?;
        let id = word(self.descriptors.len())?;
        self.commit_string(id, start, length, digest)
    }
}
impl Default for NativeSourceWriter {
    fn default() -> Self {
        Self::new()
    }
}
impl NativeSourceWriter {
    pub fn new() -> Self {
        Self {
            working: FlatEpoch::empty().fork(),
        }
    }
}
impl InitialWriter for NativeSourceWriter {
    type Output = FlatEpoch;
    fn string(&mut self, text: &str) -> Result<u32, PeleError> {
        self.working.data.strings.intern_source(text)
    }
    fn node(&mut self, node: Node) -> Result<u32, PeleError> {
        self.working.add_node_with::<true>(node)
    }
    fn record<const N: usize>(
        &mut self,
        keys: &'static [&'static str; N],
        values: [Atom; N],
    ) -> Result<u32, PeleError> {
        let strings = &mut self.working.data.strings;
        let start = self
            .working
            .data
            .properties
            .append_span_with(capacity(N)?, |i| {
                if i >= N {
                    return Ok([0; 5]);
                }
                let key = strings.intern_source(keys[i])?;
                let a = words(&values[i]);
                Ok([key, 7, a[0], a[1], a[2]])
            })?;
        let id = word(self.working.data.headers.len())?;
        self.working
            .data
            .headers
            .push([0, 0, 0, word(start)?, word(N)?, 0, 0])?;
        Ok(id)
    }
    fn dense_array(&mut self, values: &[Atom]) -> Result<u32, PeleError> {
        let strings = &mut self.working.data.strings;
        let start =
            self.working
                .data
                .properties
                .append_span_with(capacity(values.len())?, |i| {
                    let Some(value) = values.get(i) else {
                        return Ok([0; 5]);
                    };
                    let key = strings.intern_source(&i.to_string())?;
                    let a = words(value);
                    Ok([key, 7, a[0], a[1], a[2]])
                })?;
        let id = word(self.working.data.headers.len())?;
        self.working.data.headers.push([
            2,
            word(values.len())?,
            0,
            word(start)?,
            word(values.len())?,
            0,
            0,
        ])?;
        Ok(id)
    }
    fn finish(mut self, root: Atom) -> Result<FlatEpoch, PeleError> {
        self.working.set_root(root);
        self.working.freeze()
    }
}
