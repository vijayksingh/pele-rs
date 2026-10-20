use super::super::captured_admission::AdmissionKind;
use super::write::Span;
use super::*;
struct PendingString {
    id: u32,
    start: usize,
    total: usize,
    written: usize,
}
struct PendingNode {
    id: u32,
    properties: Span,
    items: Span,
    item_kind: Option<u32>,
}
/// Pure physical observation transaction. Completed string history survives abort;
/// changed node/header spans remain private until the complete transaction returns.
pub struct NativeObservation {
    working: FlatWorking,
    pending_string: Option<PendingString>,
    pending_node: Option<PendingNode>,
    stream_root: Option<Atom>,
    argument_length: Option<u32>,
    arguments: Vec<(u32, [u32; 3])>,
}
impl NativeObservation {
    pub fn new(baseline: &FlatEpoch, history: &FlatStringHistory) -> Self {
        let mut working = baseline.fork();
        working.data.strings = history.clone();
        Self {
            working,
            pending_string: None,
            pending_node: None,
            stream_root: None,
            argument_length: None,
            arguments: Vec::new(),
        }
    }
    pub fn history(&self) -> FlatStringHistory {
        let mut history = self.working.data.strings.clone();
        if let Some(pending) = &self.pending_string {
            history.units.truncate(pending.start);
        }
        history
    }
    pub fn abort(self) -> FlatStringHistory {
        let mut history = self.working.data.strings;
        if let Some(pending) = self.pending_string {
            history.units.truncate(pending.start);
        }
        history
    }
    pub fn begin_string(&mut self, id: u32, total_units: usize) -> Result<(), PeleError> {
        if self.pending_string.is_some() || id as usize != self.working.data.strings.len() {
            return Err(fault());
        }
        let start = self.working.data.strings.units.len();
        word(start.checked_add(total_units).ok_or_else(fault)?)?;
        self.pending_string = Some(PendingString {
            id,
            start,
            total: total_units,
            written: 0,
        });
        Ok(())
    }
    pub fn string_chunk(&mut self, units: &[u16]) -> Result<(), PeleError> {
        let pending = self.pending_string.as_mut().ok_or_else(fault)?;
        let next = pending
            .written
            .checked_add(units.len())
            .filter(|n| *n <= pending.total)
            .ok_or_else(fault)?;
        for unit in units {
            self.working.data.strings.units.push(*unit)?;
        }
        pending.written = next;
        Ok(())
    }
    pub fn end_string(&mut self) -> Result<(), PeleError> {
        let pending = self.pending_string.as_ref().ok_or_else(fault)?;
        if pending.written != pending.total {
            return Err(fault());
        }
        if pending.id as usize != self.working.data.strings.len() {
            return Err(fault());
        }
        self.working
            .data
            .strings
            .finish_string(pending.start, pending.total)?;
        self.pending_string = None;
        Ok(())
    }
    pub fn begin_node(&mut self, id: u32) -> Result<(), PeleError> {
        // Private framing guard: shared observation never discovers >=1M IDs.
        // Keep exactly 1M admissible physically for the normal ordered limit error.
        if id > 1_000_000 {
            return Err(fault());
        }
        if self.pending_node.is_some() {
            return Err(fault());
        }
        let old = if (id as usize) < self.working.data.headers.len() {
            self.working.data.headers.get(id as usize)?
        } else {
            ABSENT
        };
        let item_count = old[6] as usize * if old[0] == 3 { 2 } else { 1 };
        // Reuse private W's original spans. Equal scalar writes never clone pages.
        // Counts/header remain unchanged until all node reads have completed.
        self.pending_node = Some(PendingNode {
            id,
            properties: Span {
                start: old[3] as usize,
                len: 0,
                capacity: capacity(old[4] as usize)?,
            },
            items: Span {
                start: old[5] as usize,
                len: 0,
                capacity: capacity(item_count)?,
            },
            item_kind: None,
        });
        Ok(())
    }
    pub fn property(&mut self, key: u32, flags: u8, value: Atom) -> Result<(), PeleError> {
        let pending = self.pending_node.as_mut().ok_or_else(fault)?;
        let a = words(&value);
        pending.properties.append(
            &mut self.working.data.properties,
            [key, flags as u32, a[0], a[1], a[2]],
        )
    }
    pub fn map_pair(&mut self, key: Atom, value: Atom) -> Result<(), PeleError> {
        let pending = self.pending_node.as_mut().ok_or_else(fault)?;
        if pending.item_kind.is_some_and(|k| k != 3) {
            return Err(fault());
        }
        pending.item_kind = Some(3);
        pending
            .items
            .append(&mut self.working.data.items, words(&key))?;
        pending
            .items
            .append(&mut self.working.data.items, words(&value))
    }
    pub fn set_value(&mut self, value: Atom) -> Result<(), PeleError> {
        let pending = self.pending_node.as_mut().ok_or_else(fault)?;
        if pending.item_kind.is_some_and(|k| k != 4) {
            return Err(fault());
        }
        pending.item_kind = Some(4);
        pending
            .items
            .append(&mut self.working.data.items, words(&value))
    }
    pub fn end_node(&mut self, kind: AdmissionKind) -> Result<(), PeleError> {
        let p = self.pending_node.as_ref().ok_or_else(fault)?;
        let mut h = [
            0,
            0,
            0,
            word(p.properties.start)?,
            word(p.properties.len)?,
            word(p.items.start)?,
            0,
        ];
        match kind {
            AdmissionKind::Object => {}
            AdmissionKind::Instance(proto) => {
                h[0] = 1;
                h[1] = proto.0
            }
            AdmissionKind::Array(n) => {
                h[0] = 2;
                h[1] = n
            }
            AdmissionKind::InvalidArray => {
                h[0] = 2;
                h[2] = 1
            }
            AdmissionKind::Map(n) => {
                if p.items.len != n.checked_mul(2).ok_or_else(fault)?
                    || p.item_kind.is_some_and(|k| k != 3)
                {
                    return Err(fault());
                }
                h[0] = 3;
                h[6] = word(n)?
            }
            AdmissionKind::Set(n) => {
                if p.items.len != n || p.item_kind.is_some_and(|k| k != 4) {
                    return Err(fault());
                }
                h[0] = 4;
                h[6] = word(n)?
            }
            AdmissionKind::Date(bits) => {
                h[0] = 5;
                h[1] = bits as u32;
                h[2] = (bits >> 32) as u32
            }
            AdmissionKind::Error(kind) => {
                h[0] = 6;
                h[1] = error_word(kind)
            }
            AdmissionKind::InvalidError => {
                h[0] = 6;
                h[2] = 1
            }
        }
        if h[0] != 3 && h[0] != 4 && p.items.len != 0 {
            return Err(fault());
        }
        self.working.commit_node(p.id, h)?;
        self.pending_node = None;
        Ok(())
    }
    pub fn finish(mut self, root: Atom) -> Result<FlatEpoch, PeleError> {
        if self.pending_node.is_some() || self.pending_string.is_some() {
            return Err(fault());
        }
        self.working.set_root(root);
        self.working.freeze()
    }
}

/// Arguments are sparse physical records. A declared length alone never allocates
/// a giant table or moves existing method-call admission ahead of graph admission.
pub struct ObservedCapture {
    pub epoch: FlatEpoch,
    pub argument_length: u32,
    pub arguments: Vec<(u32, [u32; 3])>,
}
struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], PeleError> {
        let end = self
            .at
            .checked_add(n)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(fault)?;
        let value = &self.bytes[self.at..end];
        self.at = end;
        Ok(value)
    }
    fn u8(&mut self) -> Result<u8, PeleError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, PeleError> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    fn u32(&mut self) -> Result<u32, PeleError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn atom(&mut self) -> Result<Atom, PeleError> {
        let tag = self.u8()? as u32;
        let lo = self.u32()?;
        let hi = self.u32()?;
        atom([tag, lo, hi])
    }
    fn done(&self) -> bool {
        self.at == self.bytes.len()
    }
}
impl NativeObservation {
    pub fn history_count(&self) -> usize {
        self.working.data.strings.len()
    }
    pub fn append_batch(&mut self, bytes: &[u8]) -> Result<(), PeleError> {
        if bytes.len() < 8 || bytes.len() > 65_536 {
            return Err(fault());
        }
        let mut frame = Cursor { bytes, at: 0 };
        if frame.u16()? != 1 {
            return Err(fault());
        }
        let count = frame.u16()?;
        if count > 128 || frame.u32()? as usize != bytes.len() {
            return Err(fault());
        }
        for _ in 0..count {
            let op = frame.u8()?;
            let size = frame.u32()? as usize;
            let payload = frame.take(size)?;
            let mut p = Cursor {
                bytes: payload,
                at: 0,
            };
            match op {
                1 => {
                    if size != 8 {
                        return Err(fault());
                    }
                    let id = p.u32()?;
                    let total = p.u32()?;
                    self.begin_string(id, total as usize)?;
                }
                2 => {
                    if size % 2 != 0 || size > 2048 {
                        return Err(fault());
                    }
                    let mut units = [0u16; 1024];
                    for unit in units.iter_mut().take(size / 2) {
                        *unit = p.u16()?;
                    }
                    self.string_chunk(&units[..size / 2])?;
                }
                3 => {
                    if size != 0 {
                        return Err(fault());
                    }
                    self.end_string()?;
                }
                4 => {
                    if size != 4 {
                        return Err(fault());
                    }
                    self.begin_node(p.u32()?)?;
                }
                5 => {
                    if size != 14 {
                        return Err(fault());
                    }
                    let key = p.u32()?;
                    let flags = p.u8()?;
                    let value = p.atom()?;
                    self.property(key, flags, value)?;
                }
                6 => {
                    if size != 18 {
                        return Err(fault());
                    }
                    let key = p.atom()?;
                    let value = p.atom()?;
                    self.map_pair(key, value)?;
                }
                7 => {
                    if size != 9 {
                        return Err(fault());
                    }
                    self.set_value(p.atom()?)?;
                }
                8 => {
                    if size != 12 {
                        return Err(fault());
                    }
                    let tag = p.u32()?;
                    let a = p.u32()?;
                    let b = p.u32()?;
                    let kind = match (tag, a, b) {
                        (0, 0, 0) => AdmissionKind::Object,
                        (1, id, 0) => AdmissionKind::Instance(PrototypeId(id)),
                        (2, n, 0) => AdmissionKind::Array(n),
                        (2, 0, 1) => AdmissionKind::InvalidArray,
                        (3, n, 0) => AdmissionKind::Map(n as usize),
                        (4, n, 0) => AdmissionKind::Set(n as usize),
                        (5, lo, hi) => AdmissionKind::Date(lo as u64 | ((hi as u64) << 32)),
                        (6, k, 0) => AdmissionKind::Error(error_kind(k)?),
                        (6, 0, 1) => AdmissionKind::InvalidError,
                        _ => return Err(fault()),
                    };
                    self.end_node(kind)?;
                }
                9 => {
                    if size != 9 || self.stream_root.is_some() {
                        return Err(fault());
                    }
                    self.stream_root = Some(p.atom()?);
                }
                10 => {
                    if size != 4 || self.argument_length.is_some() {
                        return Err(fault());
                    }
                    self.argument_length = Some(p.u32()?);
                }
                11 => {
                    if size != 13 {
                        return Err(fault());
                    }
                    let index = p.u32()?;
                    let value = p.atom()?;
                    let length = self.argument_length.ok_or_else(fault)?;
                    if index >= length
                        || self
                            .arguments
                            .last()
                            .is_some_and(|(previous, _)| *previous >= index)
                    {
                        return Err(fault());
                    }
                    self.arguments.push((index, words(&value)));
                }
                _ => return Err(fault()),
            }
            if !p.done() {
                return Err(fault());
            }
        }
        if !frame.done() {
            return Err(fault());
        }
        Ok(())
    }
    pub fn finish_stream(mut self) -> Result<ObservedCapture, PeleError> {
        let root = self.stream_root.take().ok_or_else(fault)?;
        let argument_length = self.argument_length.unwrap_or(0);
        let arguments = std::mem::take(&mut self.arguments);
        let epoch = self.finish(root)?;
        Ok(ObservedCapture {
            epoch,
            argument_length,
            arguments,
        })
    }
}
