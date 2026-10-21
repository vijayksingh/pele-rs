//! Optional bounded read view of one completed host entry audit. No model owner.
use pele::compat::public::Atom;

pub(super) const MAX_BYTES: usize = 8 * 1024 * 1024;
const HEADER: usize = 17;
const PROPERTY: usize = 14;

pub(super) struct EntryFrame {
    bytes: Vec<u8>,
    nodes: Vec<u32>,
    strings: Vec<u32>,
}
impl EntryFrame {
    // Invalid/private unsupported frames simply fall back to live reads. Public
    // model admission still runs through the unchanged AdmissionStore contract.
    pub(super) fn decode(bytes: Vec<u8>) -> Option<Self> {
        fn word(bytes: &[u8], at: usize) -> Option<u32> {
            Some(u32::from_le_bytes(
                bytes.get(at..at.checked_add(4)?)?.try_into().ok()?,
            ))
        }
        if bytes.len() < 12 || bytes.len() > MAX_BYTES {
            return None;
        }
        let nodes = word(&bytes, 0)? as usize;
        let strings = word(&bytes, 4)? as usize;
        let records = word(&bytes, 8)? as usize;
        if nodes > 1_000_000
            || strings > 1_000_000
            || records > nodes
            || nodes
                .checked_add(strings)?
                .checked_mul(4)?
                .checked_add(bytes.len())?
                > MAX_BYTES
        {
            return None;
        }
        let mut value = Self {
            bytes,
            nodes: vec![0; nodes],
            strings: vec![0; strings],
        };
        let mut at = 12usize;
        for _ in 0..records {
            let id = word(&value.bytes, at)? as usize;
            let kind = *value.bytes.get(at + 4)?;
            let properties = word(&value.bytes, at + 9)? as usize;
            let entries = word(&value.bytes, at + 13)? as usize;
            if id >= nodes
                || value.nodes[id] != 0
                || kind > 5
                || (kind != 2 && kind != 3 && entries != 0)
            {
                return None;
            }
            let size = HEADER
                .checked_add(properties.checked_mul(PROPERTY)?)?
                .checked_add(entries.checked_mul(if kind == 2 { 18 } else { 9 })?)?;
            let end = at.checked_add(size)?;
            if end > value.bytes.len() {
                return None;
            }
            value.nodes[id] = u32::try_from(at).ok()?;
            at = end;
        }
        let mut units = 0usize;
        for id in 0..strings {
            let length = word(&value.bytes, at)? as usize;
            units = units.checked_add(length)?;
            if units > 16_000_000 {
                return None;
            }
            value.strings[id] = u32::try_from(at).ok()?;
            at = at.checked_add(4)?.checked_add(length.checked_mul(2)?)?;
            if at > value.bytes.len() {
                return None;
            }
        }
        if at != value.bytes.len() {
            return None;
        }
        // Framing/reference validation only. Never decode/validate UTF16 or ER
        // semantics before the original requested field/read.
        for offset in value.nodes.iter().copied().filter(|n| *n != 0) {
            let at = offset as usize;
            let count = value.word(at + 9) as usize;
            for p in 0..count {
                let property = at + HEADER + p * PROPERTY;
                if value.word(property) as usize >= strings
                    || value.bytes[property + 4] > 7
                    || !value.valid_atom(property + 5)
                {
                    return None;
                }
            }
            let mut cursor = at + HEADER + count * PROPERTY;
            let entries = value.word(at + 13) as usize;
            let atoms = entries * if value.bytes[at + 4] == 2 { 2 } else { 1 };
            for _ in 0..atoms {
                if !value.valid_atom(cursor) {
                    return None;
                }
                cursor += 9;
            }
            if matches!(value.bytes[at + 4], 2 | 3) && value.word(at + 5) as usize != entries {
                return None;
            }
        }
        Some(value)
    }
    fn word(&self, at: usize) -> u32 {
        u32::from_le_bytes(self.bytes[at..at + 4].try_into().expect("validated word"))
    }
    fn payload(&self, at: usize) -> u64 {
        u64::from_le_bytes(
            self.bytes[at + 1..at + 9]
                .try_into()
                .expect("validated atom"),
        )
    }
    fn valid_atom(&self, at: usize) -> bool {
        let payload = self.payload(at);
        match self.bytes[at] {
            0..=3 => payload == 0,
            4 => true,
            5 => payload < self.strings.len() as u64,
            6 => payload < self.nodes.len() as u64,
            7 | 8 => payload <= u32::MAX as u64,
            _ => false,
        }
    }
    fn atom(&self, at: usize) -> Atom {
        let payload = self.payload(at);
        match self.bytes[at] {
            0 => Atom::Undefined,
            1 => Atom::Null,
            2 => Atom::Bool(false),
            3 => Atom::Bool(true),
            4 => Atom::Number(f64::from_bits(payload)),
            5 => Atom::String(payload as u32),
            6 => Atom::Ref(payload as u32),
            7 => Atom::BuiltinFunction(payload as u32),
            8 => Atom::HostReference(payload as u32),
            _ => unreachable!("validated atom"),
        }
    }
    fn node(&self, id: u32) -> Option<usize> {
        self.nodes
            .get(id as usize)
            .copied()
            .filter(|n| *n != 0)
            .map(|n| n as usize)
    }
    fn units_at(&self, id: u32) -> Option<impl Iterator<Item = u16> + '_> {
        let at = *self.strings.get(id as usize)? as usize;
        let length = self.word(at) as usize;
        Some(
            self.bytes[at + 4..at + 4 + length * 2]
                .chunks_exact(2)
                .map(|v| u16::from_le_bytes([v[0], v[1]])),
        )
    }
    fn text_eq(&self, id: u32, text: &str) -> bool {
        self.units_at(id)
            .is_some_and(|units| units.eq(text.encode_utf16()))
    }
    pub(super) fn units(&self, id: u32) -> Option<Vec<u16>> {
        Some(self.units_at(id)?.collect())
    }
    pub(super) fn kind(&self, id: u32) -> Option<(u32, u32)> {
        let at = self.node(id)?;
        let kind = self.bytes[at + 4];
        // Native Date kind currently calls intrinsic getTime, including its
        // brand rejection. Do not suppress that read with audit information.
        (kind != 4).then(|| (u32::from(kind), self.word(at + 5)))
    }
    pub(super) fn own_count(&self, id: u32) -> Option<usize> {
        Some(self.word(self.node(id)? + 9) as usize)
    }
    pub(super) fn own(&self, id: u32, key: &str) -> Option<Option<Atom>> {
        let at = self.node(id)?;
        for p in 0..self.word(at + 9) as usize {
            let property = at + HEADER + p * PROPERTY;
            if self.text_eq(self.word(property), key) {
                return Some(Some(self.atom(property + 5)));
            }
        }
        if self.bytes[at + 4] == 1 && key == "length" {
            return Some(Some(Atom::Number(self.word(at + 5) as f64)));
        }
        Some(None)
    }
    fn index_key(&self, id: u32) -> Option<u32> {
        let mut units = self.units_at(id)?;
        let first = units.next()?;
        if !(48..=57).contains(&first) {
            return None;
        }
        let mut number = u32::from(first - 48);
        let mut length = 1;
        for unit in units {
            if first == 48 || !(48..=57).contains(&unit) || length == 10 {
                return None;
            }
            number = number.checked_mul(10)?.checked_add(u32::from(unit - 48))?;
            length += 1;
        }
        (number < u32::MAX).then_some(number)
    }
    pub(super) fn index(&self, id: u32, index: u32) -> Option<Option<Atom>> {
        let at = self.node(id)?;
        // Own names place canonical integer indices first in ascending order.
        // Fixed records allow binary lookup without a second property index.
        let mut lo = 0usize;
        let mut hi = self.word(at + 9) as usize;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let p = at + HEADER + mid * PROPERTY;
            match self.index_key(self.word(p)).unwrap_or(u32::MAX).cmp(&index) {
                std::cmp::Ordering::Less => lo = mid + 1,
                std::cmp::Ordering::Greater => hi = mid,
                std::cmp::Ordering::Equal if index != u32::MAX => {
                    return Some(Some(self.atom(p + 5)))
                }
                _ => return self.own(id, &index.to_string()),
            }
        }
        Some(None)
    }
    pub(super) fn map_entries(&self, id: u32) -> Option<Vec<(Atom, Atom)>> {
        let at = self.node(id)?;
        if self.bytes[at + 4] != 2 {
            return None;
        }
        let count = self.word(at + 13) as usize;
        if count > 1_000_000 {
            return None;
        } // Existing caller retains its error.
        let start = at + HEADER + self.word(at + 9) as usize * PROPERTY;
        Some(
            (0..count)
                .map(|i| (self.atom(start + i * 18), self.atom(start + i * 18 + 9)))
                .collect(),
        )
    }
}
