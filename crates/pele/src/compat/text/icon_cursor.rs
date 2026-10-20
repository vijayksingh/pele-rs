//! Pele's inline-icon expression has one global UTF-16 lastIndex. A nested
//! render inside the icon resolver can reset it while the outer scan is paused.
use std::cell::Cell;
use std::ops::Range;
use std::sync::OnceLock;

thread_local! {
    static LAST_INDEX: Cell<usize> = const { Cell::new(0) };
}

pub(super) struct IconCursor {
    byte: usize,
    units: usize,
}
impl IconCursor {
    pub(super) fn new() -> Self {
        LAST_INDEX.set(0);
        Self { byte: 0, units: 0 }
    }

    pub(super) fn next(&mut self, text: &str) -> Option<Range<usize>> {
        static ICON: OnceLock<regex::Regex> = OnceLock::new();
        let re =
            ICON.get_or_init(|| regex::Regex::new(r"(fa[bklrs]?):fa-([A-Za-z0-9_-]+)").unwrap());
        let requested = LAST_INDEX.get();
        if requested < self.units {
            self.byte = 0;
            self.units = 0;
        }
        // A nested scanner can leave its index inside an astral character in
        // this string. No ASCII icon can start at either surrogate, so resume
        // at the following UTF-8 character boundary without lossy conversion.
        while self.units < requested {
            let Some(ch) = text[self.byte..].chars().next() else {
                LAST_INDEX.set(0);
                return None;
            };
            self.units += ch.len_utf16();
            self.byte += ch.len_utf8();
        }
        let Some(found) = re.find_at(text, self.byte) else {
            LAST_INDEX.set(0);
            return None;
        };
        self.units += text[self.byte..found.end()].encode_utf16().count();
        self.byte = found.end();
        // Set before the resolver (and entity decoding) can invoke/reenter or
        // throw. Never restore on return: that would hide nested regex state.
        LAST_INDEX.set(self.units);
        Some(found.range())
    }
}
