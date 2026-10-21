//! Private projection for audited, pure builtin serializers. Serde JSON remains
//! the serialization authority; we observe its formatter events and exact writes.
use super::{engine::project_model, ErrorCode, Object, PeleError, Value, MAX_MODEL_JSON_BYTES};
use serde::Serialize;
use serde_json::ser::{CharEscape, CompactFormatter, Formatter};
use std::io::{self, Write};

#[derive(Default)]
struct Count {
    bytes: usize,
    exceeded: bool,
}
impl Write for Count {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > MAX_MODEL_JSON_BYTES - self.bytes {
            self.exceeded = true;
            return Err(io::Error::other("model projection limit"));
        }
        self.bytes += bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

enum Frame {
    Array(Vec<Value>),
    Object { value: Object, key: Option<String> },
}
#[derive(Default)]
struct Projection {
    frames: Vec<Frame>,
    string: Option<String>,
    key: bool,
    root: Option<Value>,
    depth: usize,
    too_deep: bool,
    unsupported: bool,
}
impl Projection {
    fn unsupported<T>(&mut self) -> io::Result<T> {
        self.unsupported = true;
        Err(io::Error::other(
            "builtin projection compatibility fallback",
        ))
    }
    fn value(&mut self, value: Value) -> io::Result<()> {
        if self.too_deep {
            return Ok(());
        }
        if self.string.is_some() || self.key {
            return self.unsupported();
        }
        match self.frames.last_mut() {
            Some(Frame::Array(values)) => values.push(value),
            Some(Frame::Object { value: object, key }) => {
                let Some(key) = key.take() else {
                    return self.unsupported();
                };
                object.insert(key, value);
            }
            None if self.root.is_none() => self.root = Some(value),
            None => return self.unsupported(),
        }
        Ok(())
    }
    fn begin(&mut self, object: bool) -> io::Result<()> {
        self.depth = match self.depth.checked_add(1) {
            Some(depth) => depth,
            None => return self.unsupported(),
        };
        if self.depth > 128 {
            // Depth is checked after complete serialization in the legacy path.
            // Stop retaining values, but let later byte/serializer errors win.
            self.too_deep = true;
            self.frames.clear();
            self.string = None;
            self.root = None;
            self.key = false;
        }
        if self.too_deep {
            return Ok(());
        }
        if self.string.is_some() || self.key {
            return self.unsupported();
        }
        self.frames.push(if object {
            Frame::Object {
                value: Object::default(),
                key: None,
            }
        } else {
            Frame::Array(Vec::new())
        });
        Ok(())
    }
    fn end(&mut self, object: bool) -> io::Result<()> {
        let Some(depth) = self.depth.checked_sub(1) else {
            return self.unsupported();
        };
        self.depth = depth;
        if self.too_deep {
            return Ok(());
        }
        let value = match self.frames.pop() {
            Some(Frame::Array(values)) if !object => Value::Array(values),
            Some(Frame::Object { value, key: None }) if object => Value::Object(value),
            _ => return self.unsupported(),
        };
        self.value(value)
    }
    fn number(&mut self, bytes: &[u8]) -> io::Result<()> {
        if self.too_deep {
            return Ok(());
        }
        let Ok(text) = std::str::from_utf8(bytes) else {
            return self.unsupported();
        };
        if let Some(string) = &mut self.string {
            // serde_json quotes scalar numeric keys through the same formatter.
            string.push_str(text);
            Ok(())
        } else {
            match text.parse::<f64>() {
                Ok(number) => self.value(Value::Number(number)),
                Err(_) => self.unsupported(),
            }
        }
    }
}

// Primitive JSON numbers fit comfortably; unknown future number fragments fall
// back rather than adding a new public limit. Forward original writer chunks
// first so byte admission is identical to CompactFormatter + ModelWriter.
struct NumberWrite<'a, W: ?Sized> {
    writer: &'a mut W,
    bytes: [u8; 128],
    len: usize,
    overflow: bool,
}
impl<W: ?Sized + Write> Write for NumberWrite<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.writer.write_all(bytes)?;
        if bytes.len() > self.bytes.len() - self.len {
            self.overflow = true;
            return Err(io::Error::other("builtin numeric fragment fallback"));
        }
        self.bytes[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

macro_rules! number {
    ($method:ident, $ty:ty) => {
        fn $method<W: ?Sized + Write>(&mut self, writer: &mut W, value: $ty) -> io::Result<()> {
            if self.too_deep {
                return CompactFormatter.$method(writer, value);
            }
            let mut output = NumberWrite {
                writer,
                bytes: [0; 128],
                len: 0,
                overflow: false,
            };
            let result = CompactFormatter.$method(&mut output, value);
            if output.overflow {
                return self.unsupported();
            }
            result?;
            self.number(&output.bytes[..output.len])
        }
    };
}
impl Formatter for &mut Projection {
    number!(write_i8, i8);
    number!(write_i16, i16);
    number!(write_i32, i32);
    number!(write_i64, i64);
    number!(write_i128, i128);
    number!(write_u8, u8);
    number!(write_u16, u16);
    number!(write_u32, u32);
    number!(write_u64, u64);
    number!(write_u128, u128);
    number!(write_f32, f32);
    number!(write_f64, f64);
    number!(write_number_str, &str);

    fn write_null<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        CompactFormatter.write_null(writer)?;
        self.value(Value::Null)
    }
    fn write_bool<W: ?Sized + Write>(&mut self, writer: &mut W, value: bool) -> io::Result<()> {
        CompactFormatter.write_bool(writer, value)?;
        if self.too_deep {
            return Ok(());
        }
        if let Some(string) = &mut self.string {
            string.push_str(if value { "true" } else { "false" });
            Ok(())
        } else {
            self.value(Value::Bool(value))
        }
    }
    fn begin_string<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        CompactFormatter.begin_string(writer)?;
        if self.too_deep {
            return Ok(());
        }
        if self.string.is_some() {
            return self.unsupported();
        }
        self.string = Some(String::new());
        Ok(())
    }
    fn end_string<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        CompactFormatter.end_string(writer)?;
        if self.too_deep {
            return Ok(());
        }
        let Some(string) = self.string.take() else {
            return self.unsupported();
        };
        if self.key {
            match self.frames.last_mut() {
                Some(Frame::Object { key, .. }) if key.is_none() => *key = Some(string),
                _ => return self.unsupported(),
            }
            self.key = false;
            Ok(())
        } else {
            self.value(Value::String(string))
        }
    }
    fn write_string_fragment<W: ?Sized + Write>(
        &mut self,
        writer: &mut W,
        fragment: &str,
    ) -> io::Result<()> {
        CompactFormatter.write_string_fragment(writer, fragment)?;
        if self.too_deep {
            return Ok(());
        }
        match &mut self.string {
            Some(string) => {
                string.push_str(fragment);
                Ok(())
            }
            None => self.unsupported(),
        }
    }
    fn write_char_escape<W: ?Sized + Write>(
        &mut self,
        writer: &mut W,
        escape: CharEscape,
    ) -> io::Result<()> {
        let value = match &escape {
            CharEscape::Quote => '"',
            CharEscape::ReverseSolidus => '\\',
            CharEscape::Solidus => '/',
            CharEscape::Backspace => '\u{8}',
            CharEscape::FormFeed => '\u{c}',
            CharEscape::LineFeed => '\n',
            CharEscape::CarriageReturn => '\r',
            CharEscape::Tab => '\t',
            CharEscape::AsciiControl(byte) => char::from(*byte),
        };
        CompactFormatter.write_char_escape(writer, escape)?;
        if self.too_deep {
            return Ok(());
        }
        match &mut self.string {
            Some(string) => {
                string.push(value);
                Ok(())
            }
            None => self.unsupported(),
        }
    }
    fn begin_array<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        CompactFormatter.begin_array(writer)?;
        self.begin(false)
    }
    fn end_array<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        CompactFormatter.end_array(writer)?;
        self.end(false)
    }
    fn begin_object<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        CompactFormatter.begin_object(writer)?;
        self.begin(true)
    }
    fn end_object<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        CompactFormatter.end_object(writer)?;
        self.end(true)
    }
    fn begin_object_key<W: ?Sized + Write>(
        &mut self,
        writer: &mut W,
        first: bool,
    ) -> io::Result<()> {
        CompactFormatter.begin_object_key(writer, first)?;
        if self.too_deep {
            return Ok(());
        }
        if self.key || self.string.is_some() {
            return self.unsupported();
        }
        match self.frames.last() {
            Some(Frame::Object { key: None, .. }) => {
                self.key = true;
                Ok(())
            }
            _ => self.unsupported(),
        }
    }
    fn begin_object_value<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        CompactFormatter.begin_object_value(writer)?;
        if self.too_deep {
            return Ok(());
        }
        match self.frames.last() {
            Some(Frame::Object { key: Some(_), .. }) if !self.key && self.string.is_none() => {
                Ok(())
            }
            _ => self.unsupported(),
        }
    }
    fn write_raw_fragment<W: ?Sized + Write>(
        &mut self,
        _writer: &mut W,
        _fragment: &str,
    ) -> io::Result<()> {
        // Raw fragments can change structural depth, duplicate/error semantics
        // and whole-document error positions. Never parse them speculatively.
        self.unsupported()
    }
    // Other punctuation methods use Formatter's identical Compact defaults.
    // In particular write_byte_array must dispatch through self, not through a
    // separate CompactFormatter, so its element events remain visible.
}

pub(super) fn project(model: &impl Serialize) -> Result<Value, PeleError> {
    let mut count = Count::default();
    let mut projection = Projection::default();
    let result = model.serialize(&mut serde_json::Serializer::with_formatter(
        &mut count,
        &mut projection,
    ));
    if projection.unsupported {
        // Only called for a privately selected pure builtin. Public providers
        // retain the original one-call serializer and never reach this retry.
        drop(projection);
        return project_model(model);
    }
    if count.exceeded {
        return Err(PeleError::new(
            format!("Parsed model JSON exceeds {MAX_MODEL_JSON_BYTES} bytes"),
            ErrorCode::Limit,
        ));
    }
    result.map_err(|error| {
        PeleError::new(
            format!("Module model cannot be serialized: {error}"),
            ErrorCode::Semantic,
        )
    })?;
    if projection.too_deep {
        return Err(PeleError::new(
            "Module model cannot be serialized: model nesting is too deep",
            ErrorCode::Semantic,
        ));
    }
    if projection.depth != 0
        || !projection.frames.is_empty()
        || projection.string.is_some()
        || projection.key
        || projection.root.is_none()
    {
        drop(projection);
        return project_model(model);
    }
    Ok(projection.root.take().expect("complete projection"))
}
