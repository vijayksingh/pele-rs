//! Append SVG fragments into an existing paint layer without owned formatting
//! intermediates. This sink has no resource callbacks or output-limit policy.
use crate::compat::{source::decode_entities, PeleError};

pub(crate) struct SvgSink<'a> {
    output: &'a mut String,
}
impl<'a> SvgSink<'a> {
    pub(crate) fn new(output: &'a mut String) -> Self {
        Self { output }
    }
    pub(crate) fn raw(&mut self, fragment: &str) {
        self.output.push_str(fragment);
    }
    pub(crate) fn coordinate(&mut self, value: f64) {
        coordinate(value, |formatted| self.output.push_str(formatted));
    }
    pub(crate) fn js_number(&mut self, value: f64) {
        js_number(value, |formatted| self.output.push_str(formatted));
    }
    pub(crate) fn escaped(&mut self, text: &str) {
        escaped(self.output, text);
    }
    pub(crate) fn decoded_text(&mut self, text: &str) -> Result<(), PeleError> {
        let decoded = decode_entities(text)?;
        self.escaped(&decoded);
        Ok(())
    }
}

pub(super) fn needs_escape(text: &str) -> bool {
    text.bytes()
        .any(|byte| matches!(byte, b'&' | b'<' | b'>' | b'"' | 0..=8 | 11 | 12 | 14..=31 | 127))
}
pub(super) fn escaped(output: &mut String, text: &str) {
    if !needs_escape(text) {
        output.push_str(text);
        return;
    }
    escaped_slow(output, text);
}
pub(super) fn escaped_slow(output: &mut String, text: &str) {
    for c in text.chars() {
        match c {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\0'..='\x08' | '\x0b' | '\x0c' | '\x0e'..='\x1f' | '\x7f' => {}
            _ => output.push(c),
        }
    }
}

pub(super) fn js_number<R>(value: f64, emit: impl FnOnce(&str) -> R) -> R {
    let mut buffer = ryu_js::Buffer::new();
    emit(buffer.format(value))
}

pub(super) fn coordinate<R>(value: f64, emit: impl FnOnce(&str) -> R) -> R {
    let scaled = value * 100.;
    let lower = scaled.floor();
    // Preserve Math.round's ties toward positive infinity without rounding
    // a value just below a tie through an intermediate addition of 0.5.
    let n = if scaled - lower >= 0.5 {
        lower + 1.
    } else {
        lower
    };
    if n == 0. {
        return emit("0");
    }
    if n.abs() < 1e11 {
        let m = n.abs() as u64;
        let q = m / 100;
        let r = m % 100;
        let mut bytes = [0u8; 16];
        let mut at = 10;
        let mut integer = q;
        loop {
            at -= 1;
            bytes[at] = b'0' + (integer % 10) as u8;
            integer /= 10;
            if integer == 0 {
                break;
            }
        }
        if n < 0. {
            at -= 1;
            bytes[at] = b'-';
        }
        let mut end = 10;
        if r != 0 {
            bytes[end] = b'.';
            bytes[end + 1] = b'0' + (r / 10) as u8;
            end += 2;
            if r % 10 != 0 {
                bytes[end] = b'0' + (r % 10) as u8;
                end += 1;
            }
        }
        emit(
            std::str::from_utf8(&bytes[at..end])
                .expect("SVG coordinates contain only ASCII digits"),
        )
    } else {
        js_number(n / 100., emit)
    }
}
