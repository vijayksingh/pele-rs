//! Private operation-scoped captured storage imports. Not activated by lib.rs.
//! Native Document never owns this adapter, its context, or an exception value.
use js_sys::{Array, Uint16Array, Uint32Array, Uint8Array};
use pele::compat::{
    host_sankey::{
        AdmissionHeader, AdmissionKind, AdmissionRead, CapturedRead, CapturedWrite, Extent,
        GraphCounts, MapMatch, NodeHeader, NodeKind, OwnQuery, PropertyPrefix,
    },
    public::{Atom, ErrorObjectKind, Node, Property},
    ErrorCode, PeleError,
};
use std::{cell::RefCell, collections::HashMap};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(raw_module = "../captured-store.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = capturedRoot)]
    fn root(context: &JsValue) -> Result<Uint32Array, JsValue>;
    #[wasm_bindgen(catch, js_name = capturedNodeHeaders)]
    fn headers(context: &JsValue, start: u32, count: u32) -> Result<Uint32Array, JsValue>;
    #[wasm_bindgen(catch, js_name = capturedOwnFields)]
    fn fields(context: &JsValue, id: u32, keys: &Uint32Array) -> Result<Uint32Array, JsValue>;
    #[wasm_bindgen(catch, js_name = capturedOwnDescriptors)]
    fn descriptors(context: &JsValue, id: u32, keys: &Uint32Array) -> Result<Uint32Array, JsValue>;
    #[wasm_bindgen(catch, js_name = capturedPropertyPrefixes)]
    fn prefixes(context: &JsValue, id: u32, start: u32, count: u32)
        -> Result<Uint32Array, JsValue>;
    #[wasm_bindgen(catch, js_name = capturedStringLength)]
    fn string_length(context: &JsValue, id: u32) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, js_name = capturedStringUnits)]
    fn string_units(
        context: &JsValue,
        id: u32,
        start: u32,
        count: u32,
    ) -> Result<Uint16Array, JsValue>;
    #[wasm_bindgen(catch, js_name = capturedStringLengths)]
    fn string_lengths(context: &JsValue, start: u32, count: u32) -> Result<Uint32Array, JsValue>;
    #[wasm_bindgen(catch, js_name = capturedCounts)]
    fn counts(context: &JsValue) -> Result<Uint32Array, JsValue>;
    #[wasm_bindgen(catch, js_name = capturedFindStrings)]
    fn find_strings(context: &JsValue, keys: &Uint32Array) -> Result<Uint32Array, JsValue>;
    #[wasm_bindgen(catch, js_name = capturedMapFind)]
    fn map_find(
        context: &JsValue,
        id: u32,
        tag: u32,
        lo: u32,
        hi: u32,
    ) -> Result<Uint32Array, JsValue>;
    #[wasm_bindgen(catch, js_name = capturedProperties)]
    fn properties(
        context: &JsValue,
        id: u32,
        start: u32,
        count: u32,
    ) -> Result<Uint32Array, JsValue>;
    #[wasm_bindgen(catch, js_name = capturedMapEntries)]
    fn map_entries(
        context: &JsValue,
        id: u32,
        start: u32,
        count: u32,
    ) -> Result<Uint32Array, JsValue>;
    #[wasm_bindgen(catch, js_name = capturedSetValues)]
    fn set_values(
        context: &JsValue,
        id: u32,
        start: u32,
        count: u32,
    ) -> Result<Uint32Array, JsValue>;
    #[wasm_bindgen(catch, js_name = capturedWriteBatch)]
    fn write_batch(context: &JsValue, bytes: &Uint8Array) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, js_name = capturedQueryKeys)]
    fn query_keys(context: &JsValue, names: &Array) -> Result<Uint32Array, JsValue>;
}
fn invalid() -> PeleError {
    PeleError::new("Invalid captured storage response", ErrorCode::Semantic)
}
fn word(value: usize) -> Result<u32, PeleError> {
    u32::try_from(value).map_err(|_| invalid())
}
fn words(value: Uint32Array, maximum: usize, multiple: usize) -> Result<Vec<u32>, PeleError> {
    let count = value.length() as usize;
    if count > maximum || count % multiple != 0 {
        return Err(invalid());
    }
    Ok(value.to_vec())
}
fn exact(value: Uint32Array, count: usize) -> Result<Vec<u32>, PeleError> {
    if value.length() as usize != count {
        return Err(invalid());
    }
    Ok(value.to_vec())
}
fn atom(value: &[u32]) -> Result<Atom, PeleError> {
    if value.len() != 3 || (value[0] != 4 && value[2] != 0) || (value[0] <= 3 && value[1] != 0) {
        return Err(invalid());
    }
    Ok(match value[0] {
        0 => Atom::Undefined,
        1 => Atom::Null,
        2 => Atom::Bool(false),
        3 => Atom::Bool(true),
        4 => Atom::Number(f64::from_bits(
            u64::from(value[1]) | (u64::from(value[2]) << 32),
        )),
        5 => Atom::String(value[1]),
        6 => Atom::Ref(value[1]),
        7 => Atom::BuiltinFunction(value[1]),
        8 => Atom::HostReference(value[1]),
        _ => return Err(invalid()),
    })
}
fn atom_words(value: &Atom) -> [u32; 3] {
    let (tag, bits) = match value {
        Atom::Undefined => (0, 0),
        Atom::Null => (1, 0),
        Atom::Bool(false) => (2, 0),
        Atom::Bool(true) => (3, 0),
        Atom::Number(v) => (4, v.to_bits()),
        Atom::String(id) => (5, u64::from(*id)),
        Atom::Ref(id) => (6, u64::from(*id)),
        Atom::BuiltinFunction(id) => (7, u64::from(*id)),
        Atom::HostReference(id) => (8, u64::from(*id)),
    };
    [tag, bits as u32, (bits >> 32) as u32]
}
fn write_atom(bytes: &mut Vec<u8>, value: &Atom) {
    let [tag, lo, hi] = atom_words(value);
    bytes.push(tag as u8);
    bytes.extend_from_slice(&lo.to_le_bytes());
    bytes.extend_from_slice(&hi.to_le_bytes());
}

/// Scope owns the original thrown value; no borrow is held during an import.
#[derive(Default)]
pub(crate) struct ErrorLane {
    first: RefCell<Option<JsValue>>,
}
impl ErrorLane {
    pub(crate) fn take(&self) -> Option<JsValue> {
        self.first.borrow_mut().take()
    }
    fn capture<T>(&self, value: Result<T, JsValue>) -> Result<T, PeleError> {
        value.map_err(|error| {
            let mut first = self.first.borrow_mut();
            if first.is_none() {
                *first = Some(error);
            }
            PeleError::new("Captured storage operation failed", ErrorCode::Semantic)
        })
    }
}

pub(crate) struct CapturedStore<'op> {
    context: JsValue,
    errors: &'op ErrorLane,
    root: Atom,
    prototype: PrototypeId,
    keys: HashMap<String, u32>,
    header_start: u32,
    header_words: Vec<u32>, // <=64 headers, invalidated by every write/append.
}
impl<'op> CapturedStore<'op> {
    pub(crate) fn into_context(self) -> JsValue {
        self.context
    }

    pub(crate) fn new(
        context: JsValue,
        prototype: PrototypeId,
        errors: &'op ErrorLane,
    ) -> Result<Self, PeleError> {
        let root = atom(&exact(errors.capture(root(&context))?, 3)?)?;
        Ok(Self {
            context,
            errors,
            root,
            prototype,
            keys: HashMap::new(),
            header_start: 0,
            header_words: Vec::new(),
        })
    }
    fn raw_header(&mut self, id: u32) -> Result<Option<[u32; 7]>, PeleError> {
        let offset = id.checked_sub(self.header_start).map(|v| v as usize);
        if !offset.is_some_and(|at| at < self.header_words.len() / 7) {
            if id as usize >= self.raw_counts()?.nodes {
                return Ok(None);
            }
            self.header_words =
                words(self.errors.capture(headers(&self.context, id, 64))?, 448, 7)?;
            self.header_start = id;
        }
        let Some(offset) = id.checked_sub(self.header_start).map(|v| v as usize * 7) else {
            return Err(invalid());
        };
        let Some(source) = self.header_words.get(offset..offset + 7) else {
            return Ok(None);
        };
        if source[0] == u32::MAX {
            return Ok(None);
        }
        let mut result = [0; 7];
        result.copy_from_slice(source);
        Ok(Some(result))
    }
    fn raw_counts(&self) -> Result<GraphCounts, PeleError> {
        let data = exact(self.errors.capture(counts(&self.context))?, 4)?;
        Ok(GraphCounts {
            nodes: data[0] as usize,
            strings: data[1] as usize,
            units: data[2] as usize,
            work: data[3] as usize,
        })
    }
}
impl CapturedRead for CapturedStore<'_> {
    type Value = Atom;
    fn root(&self) -> Atom {
        self.root.clone()
    }
    fn header(&mut self, id: u32) -> Result<Option<NodeHeader>, PeleError> {
        let Some(h) = self.raw_header(id)? else {
            return Ok(None);
        };
        let kind = match h[0] {
            0 => NodeKind::Object,
            1 => NodeKind::Instance,
            2 => NodeKind::Array,
            3 => NodeKind::Map,
            4 => NodeKind::Set,
            5 => NodeKind::Date,
            6 => NodeKind::Error,
            _ => return Err(invalid()),
        };
        // Invalid scalar markers are rejected by observed admission before the
        // importer is called, never silently normalized into a valid header.
        if matches!(h[0], 2 | 6) && h[2] != 0 {
            return Err(invalid());
        }
        Ok(Some(NodeHeader {
            kind,
            property_count: h[4],
            array_length: (h[0] == 2).then_some(h[1]),
        }))
    }
    fn own_fields(
        &mut self,
        id: u32,
        names: &[&str],
        out: &mut [Option<Atom>],
    ) -> Result<(), PeleError> {
        if names.len() != out.len() {
            return Err(invalid());
        }
        out.fill(None);
        for (names, out) in names.chunks(128).zip(out.chunks_mut(128)) {
            let keys = self.key_handles(names)?;
            let data = exact(
                self.errors.capture(fields(&self.context, id, &keys))?,
                names.len() * 4,
            )?;
            for (target, record) in out.iter_mut().zip(data.chunks_exact(4)) {
                *target = match record[0] {
                    0 => None,
                    1 => Some(atom(&record[1..])?),
                    _ => return Err(invalid()),
                };
            }
        }
        Ok(())
    }
    fn property_prefixes(
        &mut self,
        id: u32,
        start: u32,
        out: &mut [PropertyPrefix<Atom>],
    ) -> Result<usize, PeleError> {
        for target in out.iter_mut() {
            *target = PropertyPrefix::default();
        }
        let capacity = out.len().min(128);
        let data = words(
            self.errors
                .capture(prefixes(&self.context, id, start, word(capacity)?))?,
            capacity * 9,
            9,
        )?;
        for (target, row) in out.iter_mut().zip(data.chunks_exact(9)) {
            target.key_units = row[0];
            for i in 0..10 {
                target.prefix[i] = (row[1 + i / 2] >> ((i % 2) * 16)) as u16;
            }
            target.value = Some(atom(&row[6..9])?);
        }
        Ok(data.len() / 9)
    }
    fn string_length(&mut self, id: u32) -> Result<u32, PeleError> {
        self.errors.capture(string_length(&self.context, id))
    }
    fn with_string_chunks(
        &mut self,
        id: u32,
        mut visit: impl FnMut(&[u16]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        let length = CapturedRead::string_length(self, id)?;
        let mut start = 0;
        while start < length {
            let count = (length - start).min(1024);
            let reply = self
                .errors
                .capture(string_units(&self.context, id, start, count))?;
            if reply.length() != count {
                return Err(invalid());
            }
            let data = reply.to_vec();
            visit(&data)?;
            start += count;
        }
        Ok(())
    }
}
impl AdmissionRead for CapturedStore<'_> {
    fn extent(&mut self) -> Result<Extent, PeleError> {
        let c = self.raw_counts()?;
        Ok(Extent {
            version: 1,
            nodes: c.nodes,
            strings: c.strings,
        })
    }
    fn with_root<R>(
        &mut self,
        visit: impl FnOnce(&Atom) -> Result<R, PeleError>,
    ) -> Result<R, PeleError> {
        visit(&self.root)
    }
    fn visit_string_lengths(
        &mut self,
        mut visit: impl FnMut(usize) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        let count = self.raw_counts()?.strings;
        let mut start = 0;
        while start < count {
            let size = (count - start).min(256);
            let data = exact(
                self.errors
                    .capture(string_lengths(&self.context, word(start)?, word(size)?))?,
                size,
            )?;
            for length in data {
                visit(length as usize)?;
            }
            start += size;
        }
        Ok(())
    }
    fn header(&mut self, id: usize) -> Result<AdmissionHeader, PeleError> {
        let h = self.raw_header(word(id)?)?.ok_or_else(invalid)?;
        let kind = match h[0] {
            0 => AdmissionKind::Object,
            1 => AdmissionKind::Instance(PrototypeId(h[1])),
            2 => {
                if h[2] == 0 {
                    AdmissionKind::Array(h[1])
                } else {
                    AdmissionKind::InvalidArray
                }
            }
            3 => AdmissionKind::Map(h[6] as usize),
            4 => AdmissionKind::Set(h[6] as usize),
            5 => AdmissionKind::Date(u64::from(h[1]) | (u64::from(h[2]) << 32)),
            6 => {
                if h[2] != 0 {
                    AdmissionKind::InvalidError
                } else {
                    AdmissionKind::Error(match h[1] {
                        0 => ErrorObjectKind::Error,
                        1 => ErrorObjectKind::Pele,
                        2 => ErrorObjectKind::Type,
                        3 => ErrorObjectKind::Range,
                        4 => ErrorObjectKind::Syntax,
                        5 => ErrorObjectKind::Reference,
                        6 => ErrorObjectKind::Eval,
                        7 => ErrorObjectKind::Uri,
                        _ => return Err(invalid()),
                    })
                }
            }
            _ => return Err(invalid()),
        };
        Ok(AdmissionHeader {
            properties: h[4] as usize,
            kind,
        })
    }
    fn with_properties<R>(
        &mut self,
        id: usize,
        start: usize,
        visit: impl FnOnce(&[Property]) -> Result<R, PeleError>,
    ) -> Result<R, PeleError> {
        let data = words(
            self.errors
                .capture(properties(&self.context, word(id)?, word(start)?, 128))?,
            640,
            5,
        )?;
        let mut out = Vec::with_capacity(data.len() / 5);
        for row in data.chunks_exact(5) {
            if row[1] > 7 {
                return Err(invalid());
            }
            out.push(Property {
                key: row[0],
                flags: row[1] as u8,
                value: atom(&row[2..])?,
            });
        }
        visit(&out)
    }
    fn with_map_entries<R>(
        &mut self,
        id: usize,
        start: usize,
        visit: impl FnOnce(&[(Atom, Atom)]) -> Result<R, PeleError>,
    ) -> Result<R, PeleError> {
        let data = words(
            self.errors
                .capture(map_entries(&self.context, word(id)?, word(start)?, 128))?,
            768,
            6,
        )?;
        let mut out = Vec::with_capacity(data.len() / 6);
        for row in data.chunks_exact(6) {
            out.push((atom(&row[..3])?, atom(&row[3..])?));
        }
        visit(&out)
    }
    fn with_set_values<R>(
        &mut self,
        id: usize,
        start: usize,
        visit: impl FnOnce(&[Atom]) -> Result<R, PeleError>,
    ) -> Result<R, PeleError> {
        let data = words(
            self.errors
                .capture(set_values(&self.context, word(id)?, word(start)?, 256))?,
            768,
            3,
        )?;
        let mut out = Vec::with_capacity(data.len() / 3);
        for row in data.chunks_exact(3) {
            out.push(atom(row)?);
        }
        visit(&out)
    }
}

#[wasm_bindgen(raw_module = "../captured-store.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = capturedBeginQueryKey)]
    fn begin_key(context: &JsValue, length: u32) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = capturedQueryKeyChunk)]
    fn key_chunk(context: &JsValue, units: &Uint16Array) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = capturedFinishQueryKey)]
    fn finish_key(context: &JsValue) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, js_name = createCapturedConstructionContext)]
    fn construction_context(owner: &JsValue, prototype: u32) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = capturedPublishConstruction)]
    fn publish_construction(
        context: &JsValue,
        bytes: &Uint8Array,
        prototype: u32,
    ) -> Result<i32, JsValue>;
    #[wasm_bindgen(catch, js_name = capturedFinishAppend)]
    fn finish_append(context: &JsValue) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, js_name = capturedFinishConstruction)]
    fn finish_construction(context: &JsValue, tag: u32, lo: u32, hi: u32) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, js_name = capturedAbortConstruction)]
    fn abort_construction(context: &JsValue) -> Result<u32, JsValue>;
}
use super::host_construction::{
    ConstructionBatchTransport, ConstructionEmitter, ConstructionTransport, HostConstruction,
};
use pele::compat::public::PrototypeId;

impl CapturedStore<'_> {
    fn key_handles(&mut self, names: &[&str]) -> Result<Uint32Array, PeleError> {
        if names.len() > 128 {
            return Err(invalid());
        }
        let mut handles = Vec::with_capacity(names.len());
        // Only queried operation keys are cached. No native model-string index
        // or canonical history is seeded or reconstructed.
        for name in names {
            let id = if let Some(id) = self.keys.get(*name) {
                *id
            } else {
                let units = name.encode_utf16().count();
                let id = if units <= 32768 {
                    let input = Array::new();
                    input.push(&JsValue::from_str(name));
                    exact(self.errors.capture(query_keys(&self.context, &input))?, 1)?[0]
                } else {
                    self.errors
                        .capture(begin_key(&self.context, word(units)?))?;
                    let mut input = name.encode_utf16();
                    let mut chunk = [0u16; 1024];
                    loop {
                        let mut count = 0;
                        for target in &mut chunk {
                            match input.next() {
                                Some(unit) => {
                                    *target = unit;
                                    count += 1;
                                }
                                None => break,
                            }
                        }
                        if count == 0 {
                            break;
                        }
                        let copied = Uint16Array::from(&chunk[..count]);
                        self.errors.capture(key_chunk(&self.context, &copied))?;
                    }
                    self.errors.capture(finish_key(&self.context))?
                };
                self.keys.insert((*name).to_owned(), id);
                id
            };
            handles.push(id);
        }
        Ok(Uint32Array::from(handles.as_slice()))
    }
    fn write(&mut self, opcode: u8, payload: &[u8]) -> Result<(), PeleError> {
        self.header_words.clear();
        let mut bytes = Vec::with_capacity(9 + payload.len());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.push(opcode);
        bytes.extend_from_slice(&word(payload.len())?.to_le_bytes());
        bytes.extend_from_slice(payload);
        // Every fallible trait write is committed before returning. We never
        // defer a prior write past a later family error or counter check.
        let copied = Uint8Array::from(bytes.as_slice());
        if self.errors.capture(write_batch(&self.context, &copied))? != 0 {
            return Err(invalid());
        }
        Ok(())
    }
    fn slot_value(
        &mut self,
        opcode: u8,
        id: u32,
        slot: u32,
        value: &Atom,
    ) -> Result<(), PeleError> {
        let mut payload = Vec::with_capacity(17);
        payload.extend_from_slice(&id.to_le_bytes());
        payload.extend_from_slice(&slot.to_le_bytes());
        write_atom(&mut payload, value);
        self.write(opcode, &payload)
    }
}
impl CapturedWrite for CapturedStore<'_> {
    fn counts(&mut self) -> Result<GraphCounts, PeleError> {
        self.raw_counts()
    }
    fn find_strings(&mut self, names: &[&str], out: &mut [Option<u32>]) -> Result<(), PeleError> {
        if names.len() != out.len() {
            return Err(invalid());
        }
        out.fill(None);
        for (names, target) in names.chunks(128).zip(out.chunks_mut(128)) {
            let keys = self.key_handles(names)?;
            let data = exact(
                self.errors.capture(find_strings(&self.context, &keys))?,
                names.len(),
            )?;
            for (out, id) in target.iter_mut().zip(data) {
                *out = (id != u32::MAX).then_some(id);
            }
        }
        Ok(())
    }
    fn intern(&mut self, text: &str) -> Result<u32, PeleError> {
        let mut found = [None];
        self.find_strings(&[text], &mut found)?;
        if let Some(id) = found[0] {
            return Ok(id);
        }
        let id = word(self.raw_counts()?.strings)?;
        self.header_words.clear();
        let transport = AppendTransport {
            context: &self.context,
            errors: self.errors,
            prototype: self.prototype,
        };
        let mut emitter = ConstructionEmitter::new(transport, self.prototype);
        emitter.append_string(id, text)?;
        emitter.flush()?;
        if self.errors.capture(finish_append(&self.context))? != 0 {
            return Err(invalid());
        }
        Ok(id)
    }
    fn append_node(&mut self, node: Node) -> Result<u32, PeleError> {
        let id = word(self.raw_counts()?.nodes)?;
        self.header_words.clear();
        let transport = AppendTransport {
            context: &self.context,
            errors: self.errors,
            prototype: self.prototype,
        };
        let mut emitter = ConstructionEmitter::new(transport, self.prototype);
        emitter.append_node(id, node)?;
        emitter.flush()?;
        if self.errors.capture(finish_append(&self.context))? != 0 {
            return Err(invalid());
        }
        Ok(id)
    }
    fn own_descriptors(
        &mut self,
        id: u32,
        names: &[&str],
        out: &mut [OwnQuery],
    ) -> Result<(), PeleError> {
        if names.len() != out.len() {
            return Err(invalid());
        }
        for item in out.iter_mut() {
            *item = OwnQuery::default();
        }
        for (names, target) in names.chunks(128).zip(out.chunks_mut(128)) {
            let keys = self.key_handles(names)?;
            let data = exact(
                self.errors.capture(descriptors(&self.context, id, &keys))?,
                names.len() * 3,
            )?;
            for (out, row) in target.iter_mut().zip(data.chunks_exact(3)) {
                if row[0] > 1 || row[2] > 1 {
                    return Err(invalid());
                }
                out.first_slot = (row[0] == 1).then_some(row[1]);
                out.any_nonwritable = row[2] == 1;
            }
        }
        Ok(())
    }
    fn map_find(&mut self, id: u32, key: &Atom) -> Result<Option<MapMatch>, PeleError> {
        let [tag, lo, hi] = atom_words(key);
        let data = exact(
            self.errors
                .capture(map_find(&self.context, id, tag, lo, hi))?,
            5,
        )?;
        match data[0] {
            0 => Ok(None),
            1 => Ok(Some(MapMatch {
                slot: data[1],
                value: atom(&data[2..])?,
            })),
            _ => Err(invalid()),
        }
    }
    fn replace_map_value(&mut self, id: u32, slot: u32, value: Atom) -> Result<(), PeleError> {
        self.slot_value(1, id, slot, &value)
    }
    fn append_map_entry(&mut self, id: u32, key: Atom, value: Atom) -> Result<(), PeleError> {
        let mut payload = Vec::with_capacity(22);
        payload.extend_from_slice(&id.to_le_bytes());
        write_atom(&mut payload, &key);
        write_atom(&mut payload, &value);
        self.write(2, &payload)
    }
    fn replace_property_value(&mut self, id: u32, slot: u32, value: Atom) -> Result<(), PeleError> {
        self.slot_value(3, id, slot, &value)
    }
    fn append_property(&mut self, id: u32, property: Property) -> Result<(), PeleError> {
        let mut payload = Vec::with_capacity(18);
        payload.extend_from_slice(&id.to_le_bytes());
        payload.extend_from_slice(&property.key.to_le_bytes());
        payload.push(property.flags);
        write_atom(&mut payload, &property.value);
        self.write(4, &payload)
    }
    fn replace_matching_property_values(
        &mut self,
        id: u32,
        key: &str,
        value: Atom,
    ) -> Result<(), PeleError> {
        let handle = self.key_handles(&[key])?.get_index(0);
        self.slot_value(5, id, handle, &value)
    }
    fn set_array_length(&mut self, id: u32, length: u32) -> Result<(), PeleError> {
        let mut payload = Vec::with_capacity(8);
        payload.extend_from_slice(&id.to_le_bytes());
        payload.extend_from_slice(&length.to_le_bytes());
        self.write(6, &payload)
    }
}
struct AppendTransport<'a> {
    context: &'a JsValue,
    errors: &'a ErrorLane,
    prototype: PrototypeId,
}
impl ConstructionBatchTransport for AppendTransport<'_> {
    fn publish_batch(&mut self, bytes: &[u8]) -> Result<(), PeleError> {
        let copied = Uint8Array::from(bytes);
        let status = self.errors.capture(publish_construction(
            self.context,
            &copied,
            self.prototype.0,
        ))?;
        if status != 0 {
            return Err(invalid());
        }
        Ok(())
    }
    fn invalid(&self, message: &str) -> PeleError {
        PeleError::new(message, ErrorCode::Semantic)
    }
}

/// Pure fresh source transport. Its context stays private until the core admits
/// output. Schema selection, caller model publication and revision policy live
/// outside this transport and are deliberately not implemented here.
pub(crate) struct InitialTransport<'op> {
    context: JsValue,
    errors: &'op ErrorLane,
    prototype: PrototypeId,
}
impl<'op> InitialTransport<'op> {
    pub(crate) fn writer(
        owner: &JsValue,
        prototype: PrototypeId,
        errors: &'op ErrorLane,
    ) -> Result<HostConstruction<Self>, PeleError> {
        let context = errors.capture(construction_context(owner, prototype.0))?;
        Ok(HostConstruction::new(
            Self {
                context,
                errors,
                prototype,
            },
            prototype,
        ))
    }
}
impl ConstructionBatchTransport for InitialTransport<'_> {
    fn publish_batch(&mut self, bytes: &[u8]) -> Result<(), PeleError> {
        let copied = Uint8Array::from(bytes);
        if self.errors.capture(publish_construction(
            &self.context,
            &copied,
            self.prototype.0,
        ))? != 0
        {
            return Err(invalid());
        }
        Ok(())
    }
    fn invalid(&self, message: &str) -> PeleError {
        PeleError::new(message, ErrorCode::Semantic)
    }
}
impl<'op> ConstructionTransport for InitialTransport<'op> {
    type Output = CapturedStore<'op>;
    fn finish(self, root: Atom) -> Result<Self::Output, PeleError> {
        let [tag, lo, hi] = atom_words(&root);
        if self
            .errors
            .capture(finish_construction(&self.context, tag, lo, hi))?
            != 0
        {
            return Err(invalid());
        }
        CapturedStore::new(self.context, self.prototype, self.errors)
    }
    fn abort(self) -> Result<(), PeleError> {
        if self.errors.capture(abort_construction(&self.context))? != 0 {
            return Err(invalid());
        }
        Ok(())
    }
}
