//! Operation-scoped host storage. No model externref is retained by a policy.
#[path = "er_entry_frame.rs"]
mod entry_frame;
use super::wasm_registry::NativeRegistry;
use entry_frame::EntryFrame;
use js_sys::{Array, Function, Uint16Array, Uint8Array};
use pele::compat::{
    host_er::{MethodStore, RenderStore},
    public::{Atom, JavaScriptErrorKind, JsString, PublicError},
    ErrorCode, PeleError,
};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    rc::Rc,
};
use wasm_bindgen::prelude::*;

// Raw relative module preserves the existing package layout and avoids a
// wasm-bindgen snippet copy of the host adapter. Both bindings/web/pkg and
// packed package/pkg resolve the same sibling host-er-store.js.
#[wasm_bindgen(raw_module = "../host-er-store.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = hostErRenderResult)]
    fn render_result_packet(
        raw: bool,
        diagram_type: &str,
        svg: &str,
        width_low: u32,
        width_high: u32,
        height_low: u32,
        height_high: u32,
        links: &str,
        metadata_bytes: u32,
    ) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = hostErReadOwn)]
    fn read_own(context: &JsValue, object: u32, key: &JsValue) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, js_name = hostErReadOwnCount)]
    fn read_own_count(context: &JsValue, object: u32) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, js_name = hostErReadIndex)]
    fn read_index(context: &JsValue, object: u32, index: u32) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, js_name = hostErReadKind)]
    fn read_kind(context: &JsValue, object: u32) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, js_name = hostErReadNumberBits)]
    fn read_number_bits(context: &JsValue) -> Result<u64, JsValue>;
    #[wasm_bindgen(catch, js_name = hostErReadUnits)]
    fn read_units(context: &JsValue, string: u32) -> Result<Uint16Array, JsValue>;
    #[wasm_bindgen(catch, js_name = hostErReadMapText)]
    fn read_map_text(context: &JsValue, object: u32, key: &JsValue) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, js_name = hostErReadMapValue)]
    fn read_map_value(
        context: &JsValue,
        object: u32,
        tag: u32,
        payload: u32,
        numeric: f64,
    ) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, js_name = hostErReadMapIterator)]
    fn read_map_iterator(context: &JsValue, object: u32) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = hostErReadMapChunk)]
    fn read_map_chunk(
        context: &JsValue,
        cursor: &JsValue,
        maximum: u32,
    ) -> Result<Uint8Array, JsValue>;
}

fn invalid(text: &str) -> PublicError {
    draw_invalid(text).into()
}
fn draw_invalid(text: &str) -> PeleError {
    let mut error = PeleError::new(text, ErrorCode::Semantic);
    error.diagram_type = Some("er".into());
    error
}
fn limit() -> PublicError {
    PeleError::new(
        "ER native graph operation exceeds work budget",
        ErrorCode::Limit,
    )
    .into()
}
fn type_error(text: &str) -> PublicError {
    PublicError::JavaScript {
        name: JavaScriptErrorKind::TypeError,
        message: JsString::from(text),
    }
}
fn number(v: u32) -> JsValue {
    JsValue::from_f64(v as f64)
}
fn atom_js(a: &Atom) -> JsValue {
    let out = Array::new();
    let (tag, value) = match a {
        Atom::Undefined => (0, None),
        Atom::Null => (1, None),
        Atom::Bool(false) => (2, None),
        Atom::Bool(true) => (3, None),
        Atom::Number(v) => (4, Some(JsValue::from_str(&format!("{:016x}", v.to_bits())))),
        Atom::String(v) => (5, Some(number(*v))),
        Atom::Ref(v) => (6, Some(number(*v))),
        Atom::BuiltinFunction(v) => (7, Some(number(*v))),
        Atom::HostReference(v) => (8, Some(number(*v))),
    };
    out.push(&number(tag));
    if let Some(value) = value {
        out.push(&value);
    }
    out.into()
}
fn unsigned(value: &JsValue) -> Result<u32, PublicError> {
    let n = value
        .as_f64()
        .ok_or_else(|| invalid("Invalid host storage number"))?;
    if n.is_finite() && n >= 0. && n <= u32::MAX as f64 && n.fract() == 0. {
        Ok(n as u32)
    } else {
        Err(invalid("Invalid host storage number"))
    }
}
fn atom(value: JsValue) -> Result<Atom, PublicError> {
    if !Array::is_array(&value) {
        return Err(invalid("Invalid host storage atom"));
    }
    let a = Array::from(&value);
    Ok(match unsigned(&a.get(0))? {
        0 => Atom::Undefined,
        1 => Atom::Null,
        2 => Atom::Bool(false),
        3 => Atom::Bool(true),
        4 => {
            let text = a
                .get(1)
                .as_string()
                .ok_or_else(|| invalid("Invalid host number bits"))?;
            let bits =
                u64::from_str_radix(&text, 16).map_err(|_| invalid("Invalid host number bits"))?;
            Atom::Number(f64::from_bits(bits))
        }
        5 => Atom::String(unsigned(&a.get(1))?),
        6 => Atom::Ref(unsigned(&a.get(1))?),
        7 => Atom::BuiltinFunction(unsigned(&a.get(1))?),
        8 => Atom::HostReference(unsigned(&a.get(1))?),
        _ => return Err(invalid("Invalid host atom tag")),
    })
}

// Private map frames carry exact numeric bits and canonical owner IDs. They
// exist for one bounded transfer, never as a retained graph or decoded mirror.
fn map_frame_atom(bytes: &[u8]) -> Result<Atom, PublicError> {
    if bytes.len() != 9 {
        return Err(invalid("Invalid host map atom length"));
    }
    let payload = u64::from_le_bytes(bytes[1..9].try_into().expect("eight-byte map payload"));
    if bytes[0] != 4 && payload > u64::from(u32::MAX) {
        return Err(invalid("Invalid host map atom payload"));
    }
    if bytes[0] <= 3 && payload != 0 {
        return Err(invalid("Invalid host map primitive payload"));
    }
    Ok(match bytes[0] {
        0 => Atom::Undefined,
        1 => Atom::Null,
        2 => Atom::Bool(false),
        3 => Atom::Bool(true),
        4 => Atom::Number(f64::from_bits(payload)),
        5 => Atom::String(payload as u32),
        6 => Atom::Ref(payload as u32),
        7 => Atom::BuiltinFunction(payload as u32),
        8 => Atom::HostReference(payload as u32),
        _ => return Err(invalid("Invalid host map atom tag")),
    })
}
fn map_key_arguments(atom: &Atom) -> (u32, u32, f64) {
    match atom {
        Atom::Undefined => (0, 0, 0.0),
        Atom::Null => (1, 0, 0.0),
        Atom::Bool(false) => (2, 0, 0.0),
        Atom::Bool(true) => (3, 0, 0.0),
        Atom::Number(value) => (4, 0, *value),
        Atom::String(id) => (5, *id, 0.0),
        Atom::Ref(id) => (6, *id, 0.0),
        Atom::BuiltinFunction(id) => (7, *id, 0.0),
        Atom::HostReference(id) => (8, *id, 0.0),
    }
}

fn pele(error: PublicError) -> PeleError {
    match error {
        PublicError::Pele(e) => e,
        _ => PeleError::new("Host ER operation failed", ErrorCode::Semantic),
    }
}
struct Own {
    count: usize,
    value: Option<Atom>,
    flags: u8,
}
pub struct HostStore {
    primitive: Function,
    capture: Function,
    // Owns host identity only during this operation, never in a policy/core model.
    context: JsValue,
    key_cache: RefCell<HashMap<String, Rc<JsValue>>>,
    cached_key_units: Cell<usize>,
    root: Atom,
    work: usize,
    refs: usize,
    maps: HashSet<u32>,
    sets: HashSet<u32>,
    live_work: Cell<usize>,
    live_units: Cell<usize>,
    // Optional operation-local transfer cache; mutable fields and conversion
    // results are never cached. A full cache changes cost, never admission.
    unit_cache: RefCell<HashMap<u32, Rc<Vec<u16>>>>,
    cached_units: Cell<usize>,
    entry_frame: Option<Rc<RefCell<Option<EntryFrame>>>>,
}
pub enum Cursor {
    Array(u32, u32),
    Map(JsValue),
    Set(JsValue),
    String(u32, usize),
}
impl HostStore {
    fn new(primitive: Function, capture: Function) -> Result<Self, PublicError> {
        let mut value = Self {
            primitive,
            capture,
            context: JsValue::UNDEFINED,
            key_cache: RefCell::new(HashMap::new()),
            cached_key_units: Cell::new(0),
            root: Atom::Undefined,
            work: 0,
            refs: 0,
            maps: HashSet::new(),
            sets: HashSet::new(),
            live_work: Cell::new(0),
            live_units: Cell::new(0),
            unit_cache: RefCell::new(HashMap::new()),
            cached_units: Cell::new(0),
            entry_frame: None,
        };
        value.root = atom(value.call("root", &[])?)?;
        value.refs = unsigned(&Array::from(&value.call("counts", &[])?).get(3))? as usize;
        value.context = value.call("context", &[])?;
        Ok(value)
    }
    fn attach_frame(&mut self, frame: Option<Uint8Array>) {
        self.entry_frame = frame
            .filter(|bytes| bytes.length() as usize <= entry_frame::MAX_BYTES)
            .and_then(|bytes| EntryFrame::decode(bytes.to_vec()))
            .map(|value| Rc::new(RefCell::new(Some(value))));
    }
    fn frame<T>(&self, read: impl FnOnce(&EntryFrame) -> Option<T>) -> Option<T> {
        self.entry_frame
            .as_ref()
            .and_then(|frame| frame.borrow().as_ref().and_then(read))
    }
    fn call(&self, op: &str, args: &[JsValue]) -> Result<JsValue, PublicError> {
        if matches!(op, "joinUnits" | "coerceUnits" | "stringFailure") {
            discard_frame(&self.entry_frame);
        }
        let arguments = Array::new();
        for value in args {
            arguments.push(value);
        }
        self.primitive
            .call2(&JsValue::UNDEFINED, &JsValue::from_str(op), &arguments)
            .map_err(|error| {
                // Captured in the current JS operation scope; caller restores identity.
                let _ = self.capture.call1(&JsValue::UNDEFINED, &error);
                invalid("Host storage callback failed")
            })
    }
    fn captured<T>(&self, result: Result<T, JsValue>) -> Result<T, PublicError> {
        result.map_err(|error| {
            // Preserve the same first-thrown-value path as generic imports.
            let _ = self.capture.call1(&JsValue::UNDEFINED, &error);
            invalid("Host storage callback failed")
        })
    }
    fn key(&self, key: &str) -> Rc<JsValue> {
        // Rc cloning needs no host import; never clone a JsValue under a borrow.
        let known = { self.key_cache.borrow().get(key).cloned() };
        if let Some(value) = known {
            return value;
        }
        let value = Rc::new(JsValue::from_str(key));
        let units = key.encode_utf16().count();
        let mut cache = self.key_cache.borrow_mut();
        // Optional immutable transfer cache, independent of admission limits.
        if cache.len() < 4096 && units <= 1_000_000 - self.cached_key_units.get() {
            self.cached_key_units
                .set(self.cached_key_units.get() + units);
            cache.insert(key.to_owned(), Rc::clone(&value));
        }
        value
    }
    fn read_atom(&self, word: f64) -> Result<Option<Atom>, PublicError> {
        if word == -1.0 {
            return Ok(None);
        }
        if !word.is_finite() || word < 0.0 || word >= (1_u64 << 39) as f64 || word.fract() != 0.0 {
            return Err(invalid("Invalid host read atom word"));
        }
        let word = word as u64;
        let payload = word as u32;
        let tag = ((word >> 32) & 15) as u8;
        if tag <= 4 && payload != 0 {
            return Err(invalid("Invalid host read atom payload"));
        }
        Ok(Some(match tag {
            0 => Atom::Undefined,
            1 => Atom::Null,
            2 => Atom::Bool(false),
            3 => Atom::Bool(true),
            4 => {
                // Immediate, no callback or other import between read and bits.
                let bits = self.captured(read_number_bits(&self.context))?;
                Atom::Number(f64::from_bits(bits))
            }
            5 => Atom::String(payload),
            6 => Atom::Ref(payload),
            7 => Atom::BuiltinFunction(payload),
            8 => Atom::HostReference(payload),
            _ => return Err(invalid("Invalid host read atom tag")),
        }))
    }
    fn kind(&self, id: u32) -> Result<(u32, u32), PublicError> {
        if let Some(value) = self.frame(|frame| frame.kind(id)) {
            return Ok(value);
        }
        let word = self.captured(read_kind(&self.context, id))?;
        if !word.is_finite() || word < 0.0 || word >= (6_u64 << 32) as f64 || word.fract() != 0.0 {
            return Err(invalid("Invalid host kind word"));
        }
        let word = word as u64;
        Ok(((word >> 32) as u32, word as u32))
    }
    fn own_info(&self, id: u32, key: &str) -> Result<Own, PublicError> {
        let a = Array::from(&self.call("own", &[number(id), JsValue::from_str(key)])?);
        Ok(Own {
            count: unsigned(&a.get(0))? as usize,
            value: if a.get(1).as_bool() == Some(true) {
                Some(atom(a.get(3))?)
            } else {
                None
            },
            flags: unsigned(&a.get(2))? as u8,
        })
    }
    fn own_count(&self, id: u32) -> Result<usize, PublicError> {
        if let Some(value) = self.frame(|frame| frame.own_count(id)) {
            return Ok(value);
        }
        let count = self.captured(read_own_count(&self.context, id))?;
        // Match unsigned() validation without another JsValue conversion.
        if count.is_finite() && count >= 0.0 && count <= u32::MAX as f64 && count.fract() == 0.0 {
            Ok(count as u32 as usize)
        } else {
            Err(invalid("Invalid host storage number"))
        }
    }
    fn peek_own(&self, id: u32, key: &str) -> Result<Option<Atom>, PublicError> {
        if let Some(value) = self.frame(|frame| frame.own(id, key)) {
            return Ok(value);
        }
        let key = self.key(key);
        let word = self.captured(read_own(&self.context, id, key.as_ref()))?;
        self.read_atom(word)
    }
    fn peek_index(&self, id: u32, index: usize) -> Result<Option<Atom>, PublicError> {
        let Ok(index) = u32::try_from(index) else {
            return self.peek_own(id, &index.to_string());
        };
        if let Some(value) = self.frame(|frame| frame.index(id, index)) {
            return Ok(value);
        }
        let word = self.captured(read_index(&self.context, id, index))?;
        self.read_atom(word)
    }
    fn units(&self, id: u32) -> Result<Rc<Vec<u16>>, PublicError> {
        // End the RefCell borrow before any import: a callback/nested operation
        // must never encounter a retained Rust borrow guard.
        let known = { self.unit_cache.borrow().get(&id).cloned() };
        if let Some(units) = known {
            return Ok(units);
        }
        let units = if let Some(units) = self.frame(|frame| frame.units(id)) {
            units
        } else {
            let data = self.captured(read_units(&self.context, id))?;
            if data.length() > 16_000_000 {
                return Err(limit());
            }
            data.to_vec()
        };
        let units = Rc::new(units);
        let mut cache = self.unit_cache.borrow_mut();
        // These are optional storage caps, not graph/string limits. Permanent
        // and temporary string histories retain their existing independent
        // admission rules; uncached IDs follow the exact original import path.
        if cache.len() < 4096 && units.len() <= 1_000_000 - self.cached_units.get() {
            self.cached_units.set(self.cached_units.get() + units.len());
            cache.insert(id, Rc::clone(&units));
        }
        Ok(units)
    }
    fn counts(&self) -> Result<(usize, usize, usize), PublicError> {
        let a = Array::from(&self.call("counts", &[])?);
        Ok((
            unsigned(&a.get(0))? as usize,
            unsigned(&a.get(1))? as usize,
            unsigned(&a.get(2))? as usize,
        ))
    }
    fn room(&mut self, refs: usize) -> Result<(), PublicError> {
        self.refs = self.refs.saturating_add(refs);
        if self.refs > 8_000_000 || self.counts()?.0 >= 1_000_000 {
            Err(limit())
        } else {
            Ok(())
        }
    }
    fn define(&self, id: u32, key: &str, value: &Atom, flags: u8) -> Result<(), PublicError> {
        if self
            .call(
                "define",
                &[
                    number(id),
                    JsValue::from_str(key),
                    atom_js(value),
                    number(flags as u32),
                ],
            )?
            .as_bool()
            != Some(true)
        {
            return Err(type_error(&format!(
                "Cannot define property {key}, object is not extensible"
            )));
        }
        Ok(())
    }
    fn allocated(&mut self, kind: &str) -> Result<Atom, PublicError> {
        self.room(0)?;
        Ok(Atom::Ref(unsigned(
            &self.call("allocate", &[JsValue::from_str(kind)])?,
        )?))
    }
    fn indexed_map(&mut self, id: u32) -> Result<(), PublicError> {
        if !self.maps.contains(&id) {
            let (_, length) = self.kind(id)?;
            self.tick(length as usize)?;
            self.maps.insert(id);
        }
        Ok(())
    }
    fn indexed_set(&mut self, id: u32) -> Result<(), PublicError> {
        if !self.sets.contains(&id) {
            let (_, length) = self.kind(id)?;
            self.tick(length as usize)?;
            self.sets.insert(id);
        }
        Ok(())
    }
}
impl MethodStore for HostStore {
    type Cursor = Cursor;
    fn tick(&mut self, n: usize) -> Result<(), PublicError> {
        self.work = self.work.saturating_add(n);
        if self.work > 1_000_000 {
            Err(limit())
        } else {
            Ok(())
        }
    }
    fn own(&mut self, id: u32, k: &str) -> Result<Option<Atom>, PublicError> {
        let own = self.own_info(id, k)?;
        self.tick(own.count)?;
        Ok(own.value)
    }
    fn read(&mut self, a: &Atom, k: &str) -> Result<Atom, PublicError> {
        match a {
            Atom::Undefined | Atom::Null => Err(type_error(&format!(
                "Cannot read properties of {} (reading '{k}')",
                if matches!(a, Atom::Null) {
                    "null"
                } else {
                    "undefined"
                }
            ))),
            Atom::Ref(id) => {
                let (kind, n) = self.kind(*id)?;
                if k == "length" && kind == 1 {
                    return Ok(Atom::Number(n as f64));
                }
                if let Some(value) = self.own(*id, k)? {
                    return Ok(value);
                }
                if k == "size" && kind == 2 {
                    return Ok(Atom::Number(n as f64));
                }
                Ok(Atom::Undefined)
            }
            Atom::String(id) => {
                let units = self.units(*id)?;
                if k == "length" {
                    return Ok(Atom::Number(units.len() as f64));
                }
                if let Some(n) = k.parse::<usize>().ok().filter(|n| n.to_string() == k) {
                    if let Some(c) = units.get(n) {
                        return self.intern(JsString(vec![*c]));
                    }
                }
                Ok(Atom::Undefined)
            }
            _ => Ok(Atom::Undefined),
        }
    }
    fn set(&mut self, a: &Atom, k: &str, v: Atom) -> Result<(), PublicError> {
        let Atom::Ref(id) = a else {
            if matches!(a, Atom::Null | Atom::Undefined) {
                return Err(type_error(&format!(
                    "Cannot set properties of {} (setting '{k}')",
                    if matches!(a, Atom::Null) {
                        "null"
                    } else {
                        "undefined"
                    }
                )));
            }
            let kind = match a {
                Atom::String(_) => "string",
                Atom::Bool(_) => "boolean",
                Atom::Number(_) => "number",
                _ => return Err(invalid("ER host property receiver dispatch is unavailable")),
            };
            let mut text = format!("Cannot create property '{k}' on {kind} '")
                .encode_utf16()
                .collect::<Vec<_>>();
            text.extend(self.coercion(a)?);
            text.push(39);
            return Err(PublicError::JavaScript {
                name: JavaScriptErrorKind::TypeError,
                message: JsString(text),
            });
        };
        let own = self.own_info(*id, k)?;
        self.tick(own.count)?;
        let flags = if own.value.is_some() {
            if own.flags & 2 == 0 {
                return Err(type_error(&format!(
                    "Cannot assign to read only property '{k}' of object '#<Object>'"
                )));
            }
            own.flags
        } else {
            self.room(1)?;
            self.text(k)?;
            7
        };
        self.define(*id, k, &v, flags)
    }
    fn intern(&mut self, s: JsString) -> Result<Atom, PublicError> {
        let data: JsValue = Uint16Array::from(s.0.as_slice()).into();
        let known = self.call("findString", &[data.clone()])?;
        if !known.is_undefined() {
            return Ok(Atom::String(unsigned(&known)?));
        }
        let (_, strings, units) = self.counts()?;
        if strings >= 1_000_000 || units.saturating_add(s.0.len()) > 16_000_000 {
            return Err(limit());
        }
        Ok(Atom::String(unsigned(&self.call("intern", &[data])?)?))
    }
    fn text(&mut self, s: &str) -> Result<Atom, PublicError> {
        self.intern(JsString::from(s))
    }
    fn record(&mut self, fields: Vec<(&str, Atom)>) -> Result<Atom, PublicError> {
        let a = self.allocated("object")?;
        for (k, v) in fields {
            self.set(&a, k, v)?;
        }
        Ok(a)
    }
    fn array(&mut self, values: Vec<Atom>) -> Result<Atom, PublicError> {
        let a = self.allocated("array")?;
        for value in values {
            self.push(&a, value, "array.push")?;
        }
        Ok(a)
    }
    fn native(&mut self, a: &Atom, p: &str, expr: &str, kind: u8) -> Result<u32, PublicError> {
        let found = self.read(a, p)?;
        if matches!(found, Atom::BuiltinFunction(_) | Atom::HostReference(_)) {
            return Err(invalid(&format!(
                "ER overridden {expr} callback dispatch is unavailable"
            )));
        }
        let Atom::Ref(id) = a else {
            return Err(type_error(&format!("{expr} is not a function")));
        };
        let actual = self.kind(*id)?.0;
        let good = matches!((actual, kind), (2, 0) | (1, 1) | (3, 2));
        if !good || self.own(*id, p)?.is_some() {
            return Err(type_error(&format!("{expr} is not a function")));
        }
        Ok(*id)
    }
    fn map_get(&mut self, a: &Atom, k: &Atom, has: bool, expr: &str) -> Result<Atom, PublicError> {
        let id = self.native(a, if has { "has" } else { "get" }, expr, 0)?;
        self.indexed_map(id)?;
        let value = self.call(
            if has { "mapHas" } else { "mapGet" },
            &[number(id), atom_js(k)],
        )?;
        if has {
            Ok(Atom::Bool(value.as_bool() == Some(true)))
        } else {
            atom(value)
        }
    }
    fn map_set(&mut self, a: &Atom, k: Atom, v: Atom, expr: &str) -> Result<(), PublicError> {
        let id = self.native(a, "set", expr, 0)?;
        self.indexed_map(id)?;
        if self.call("mapHas", &[number(id), atom_js(&k)])?.as_bool() != Some(true) {
            self.room(2)?;
        }
        self.call("mapSet", &[number(id), atom_js(&k), atom_js(&v)])?;
        Ok(())
    }
    fn set_has(&mut self, a: &Atom, k: &Atom) -> Result<bool, PublicError> {
        let id = self.native(a, "has", "this.claimed.has", 2)?;
        self.indexed_set(id)?;
        Ok(self.call("setHas", &[number(id), atom_js(k)])?.as_bool() == Some(true))
    }
    fn set_add(&mut self, a: &Atom, k: Atom) -> Result<(), PublicError> {
        let id = self.native(a, "add", "this.claimed.add", 2)?;
        self.indexed_set(id)?;
        if self.call("setHas", &[number(id), atom_js(&k)])?.as_bool() != Some(true) {
            self.room(1)?;
            self.call("setAdd", &[number(id), atom_js(&k)])?;
        }
        Ok(())
    }
    fn push(&mut self, a: &Atom, v: Atom, expr: &str) -> Result<(), PublicError> {
        let id = self.native(a, "push", expr, 1)?;
        let length = self.kind(id)?.1;
        if let Some(Atom::Number(_)) = self.own(id, "length")? {
            if self.own_info(id, "length")?.flags & 2 == 0 {
                return Err(type_error(
                    "Cannot assign to read only property 'length' of object '[object Array]'",
                ));
            }
        }
        if length >= 1_000_000 {
            return Err(limit());
        }
        self.set(a, &length.to_string(), v)?;
        if self
            .call("length", &[number(id), number(length + 1)])?
            .as_bool()
            != Some(true)
        {
            return Err(type_error(
                "Cannot assign to read only property 'length' of object '[object Array]'",
            ));
        }
        Ok(())
    }
    fn iter(&mut self, a: &Atom) -> Result<Cursor, PublicError> {
        match a {
            Atom::Ref(id) => match self.kind(*id)?.0 {
                1 => Ok(Cursor::Array(*id, 0)),
                2 => Ok(Cursor::Map(self.call("mapIterator", &[number(*id)])?)),
                3 => Ok(Cursor::Set(self.call("setIterator", &[number(*id)])?)),
                _ => Err(type_error("value is not iterable")),
            },
            Atom::String(id) => Ok(Cursor::String(*id, 0)),
            _ => Err(type_error("value is not iterable")),
        }
    }
    fn next(&mut self, it: &mut Cursor) -> Result<Option<Atom>, PublicError> {
        self.tick(1)?;
        match it {
            Cursor::Array(id, n) => {
                if *n >= self.kind(*id)?.1 {
                    return Ok(None);
                }
                let a = self.read(&Atom::Ref(*id), &n.to_string())?;
                *n += 1;
                Ok(Some(a))
            }
            Cursor::Map(cursor) => {
                let pair = Array::from(&self.call("nextMap", &[cursor.clone()])?);
                if pair.length() == 0 {
                    Ok(None)
                } else {
                    self.array(vec![atom(pair.get(0))?, atom(pair.get(1))?])
                        .map(Some)
                }
            }
            Cursor::Set(cursor) => {
                let item = Array::from(&self.call("nextSet", &[cursor.clone()])?);
                if item.length() == 0 {
                    Ok(None)
                } else {
                    atom(item.get(0)).map(Some)
                }
            }
            Cursor::String(id, n) => {
                let units = self.units(*id)?;
                if *n >= units.len() {
                    return Ok(None);
                }
                let count = if (0xd800..=0xdbff).contains(&units[*n])
                    && units
                        .get(*n + 1)
                        .is_some_and(|c| (0xdc00..=0xdfff).contains(c))
                {
                    2
                } else {
                    1
                };
                let text = JsString(units[*n..*n + count].to_vec());
                *n += count;
                self.intern(text).map(Some)
            }
        }
    }
    fn string_units(&mut self, a: &Atom, expr: &str) -> Result<Vec<u16>, PublicError> {
        if let Atom::String(id) = a {
            let units = self.units(*id)?;
            self.tick(units.len())?;
            Ok(units.as_ref().clone())
        } else {
            self.read(a, expr.rsplit('.').next().unwrap_or(expr))?;
            Err(type_error(&format!("{expr} is not a function")))
        }
    }
    fn includes_color(&mut self, a: &Atom) -> Result<bool, PublicError> {
        if let Atom::String(id) = a {
            let units = self.units(*id)?;
            self.tick(units.len())?;
            return Ok(units.windows(5).any(|w| w == [99, 111, 108, 111, 114]));
        }
        let id = self.native(a, "includes", "style.includes", 1)?;
        let length = self.kind(id)?.1;
        for n in 0..length {
            self.tick(1)?;
            if matches!(self.read(a,&n.to_string())?,Atom::String(id) if self.string_is(id,"color")?)
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn coercion(&mut self, a: &Atom) -> Result<Vec<u16>, PublicError> {
        match a {
            Atom::String(id) => {
                let units = self.units(*id)?;
                self.tick(units.len())?;
                Ok(units.as_ref().clone())
            }
            Atom::Undefined => Ok("undefined".encode_utf16().collect()),
            Atom::Null => Ok("null".encode_utf16().collect()),
            Atom::Bool(b) => Ok(if *b { "true" } else { "false" }.encode_utf16().collect()),
            Atom::Number(n) => Ok(pele::compat::host_er::number_string(*n)
                .encode_utf16()
                .collect()),
            _ => Err(invalid(
                "ER object primitive-conversion dispatch is unavailable",
            )),
        }
    }
    fn truth(&self, value: &Atom) -> Result<bool, PublicError> {
        Ok(match value {
            Atom::Undefined | Atom::Null => false,
            Atom::Bool(v) => *v,
            Atom::Number(n) => *n != 0. && !n.is_nan(),
            Atom::String(id) => !self.units(*id)?.is_empty(),
            _ => true,
        })
    }
    fn is_er_instance(&self, id: u32) -> Result<bool, PublicError> {
        Ok(self.kind(id)? == (5, 1))
    }
    fn array_length(&self, id: u32) -> Result<Option<u32>, PublicError> {
        let (kind, n) = self.kind(id)?;
        Ok((kind == 1).then_some(n))
    }
    fn string_is(&self, id: u32, text: &str) -> Result<bool, PublicError> {
        Ok(self.units(id)?.iter().copied().eq(text.encode_utf16()))
    }
}
impl pele::compat::host_er::AdmissionStore for HostStore {
    fn root(&self) -> Atom {
        self.root.clone()
    }
    fn own(&self, value: &Atom, key: &str) -> Result<Option<Atom>, PeleError> {
        let Atom::Ref(id) = value else {
            return Err(draw_invalid("ER drawable records must be objects"));
        };
        self.peek_own(*id, key).map_err(pele)
    }
    fn utf16(&self, value: &Atom) -> Result<JsString, PeleError> {
        let Atom::String(id) = value else {
            return Err(PeleError::new(
                "Expected a public string",
                ErrorCode::Semantic,
            ));
        };
        self.units(*id)
            .map(|units| JsString(units.as_ref().clone()))
            .map_err(pele)
    }
    fn array(&self, value: &Atom) -> Result<(usize, usize), PeleError> {
        let Atom::Ref(id) = value else {
            return Err(draw_invalid("ER drawable list must be an Array"));
        };
        let (kind, length) = self.kind(*id).map_err(pele)?;
        if kind != 1 {
            return Err(draw_invalid("ER drawable list must be an Array"));
        }
        let count = self.own_count(*id).map_err(pele)?;
        Ok((length as usize, count))
    }
    fn index(&self, value: &Atom, index: usize) -> Result<Atom, PeleError> {
        let Atom::Ref(id) = value else {
            return Err(draw_invalid("ER drawable list must be an Array"));
        };
        self.peek_index(*id, index)
            .map(|v| v.unwrap_or(Atom::Undefined))
            .map_err(pele)
    }
    fn map_len(&self, value: &Atom) -> Result<usize, PeleError> {
        let Atom::Ref(id) = value else {
            return Err(draw_invalid("ER drawable lookup must be a Map"));
        };
        let (kind, length) = self.kind(*id).map_err(pele)?;
        if kind != 2 {
            return Err(draw_invalid("ER drawable lookup must be a Map"));
        }
        Ok(length as usize)
    }
    fn map(&self, value: &Atom) -> Result<Vec<(Atom, Atom)>, PeleError> {
        RenderStore::map_entries(self, value)
    }
}
impl RenderStore for HostStore {
    fn root(&self) -> Atom {
        self.root.clone()
    }
    fn field(&self, a: &Atom, key: &str) -> Result<Atom, PeleError> {
        match a {
            Atom::Ref(id) => self
                .peek_own(*id, key)
                .map(|v| v.unwrap_or(Atom::Undefined))
                .map_err(pele),
            Atom::Null | Atom::Undefined => {
                self.call(
                    "typeError",
                    &[JsValue::from_str(&format!(
                        "Cannot read properties of {} (reading '{key}')",
                        if matches!(a, Atom::Null) {
                            "null"
                        } else {
                            "undefined"
                        }
                    ))],
                )
                .map_err(pele)?;
                Err(pele(invalid(
                    "Host exception primitive returned unexpectedly",
                )))
            }
            _ => Ok(Atom::Undefined),
        }
    }
    fn array_length(&self, a: &Atom) -> Result<usize, PeleError> {
        let Atom::Ref(id) = a else {
            return Err(draw_invalid("ER drawable list must be an Array"));
        };
        let (kind, length) = self.kind(*id).map_err(pele)?;
        if kind != 1 {
            return Err(draw_invalid("ER drawable list must be an Array"));
        }
        if length > 1_000_000 {
            return Err(pele(limit()));
        }
        Ok(length as usize)
    }
    fn array_has(&self, a: &Atom, index: usize) -> Result<bool, PeleError> {
        let Atom::Ref(id) = a else {
            return Err(draw_invalid("ER drawable list must be an Array"));
        };
        self.peek_index(*id, index)
            .map(|v| v.is_some())
            .map_err(pele)
    }
    fn array_get(&self, a: &Atom, index: usize) -> Result<Atom, PeleError> {
        if let Atom::Ref(id) = a {
            self.peek_index(*id, index)
                .map(|value| value.unwrap_or(Atom::Undefined))
                .map_err(pele)
        } else {
            RenderStore::field(self, a, &index.to_string())
        }
    }
    fn map_entries(&self, a: &Atom) -> Result<Vec<(Atom, Atom)>, PeleError> {
        let Atom::Ref(id) = a else {
            return Err(draw_invalid("ER drawable lookup must be a Map"));
        };
        let (kind, length) = self.kind(*id).map_err(pele)?;
        if kind != 2 {
            return Err(draw_invalid("ER drawable lookup must be a Map"));
        }
        if length > 1_000_000 {
            return Err(pele(limit()));
        }
        if let Some(entries) = self.frame(|frame| frame.map_entries(*id)) {
            return Ok(entries);
        }
        let cursor = self
            .captured(read_map_iterator(&self.context, *id))
            .map_err(pele)?;
        let mut out = Vec::with_capacity(length as usize);
        loop {
            // The old loop calls next/classifies one excess pair before its
            // limit error. Never prefetch beyond that same failure boundary.
            let maximum = (1_000_000_usize - out.len() + 1).min(128) as u32;
            let frame = self
                .captured(read_map_chunk(&self.context, &cursor, maximum))
                .map_err(pele)?;
            if frame.length() < 5 || frame.length() > 5 + 18 * maximum {
                return Err(pele(invalid("Invalid host map chunk length")));
            }
            let bytes = frame.to_vec();
            let count = u32::from_le_bytes(bytes[..4].try_into().expect("four-byte map count"));
            let done = bytes[4];
            if count > maximum
                || done > 1
                || bytes.len() != 5 + 18 * count as usize
                || (done == 0 && count != maximum)
            {
                return Err(pele(invalid("Invalid host map chunk header")));
            }
            for pair in bytes[5..].chunks_exact(18) {
                if out.len() >= 1_000_000 {
                    return Err(pele(limit()));
                }
                out.push((
                    map_frame_atom(&pair[..9]).map_err(pele)?,
                    map_frame_atom(&pair[9..]).map_err(pele)?,
                ));
            }
            if done == 1 {
                break;
            }
        }
        Ok(out)
    }
    fn map_get(&self, a: &Atom, key: &str) -> Result<Atom, PeleError> {
        let Atom::Ref(id) = a else {
            return Err(draw_invalid("ER drawable lookup must be a Map"));
        };
        if self.kind(*id).map_err(pele)?.0 != 2 {
            return Err(draw_invalid("ER drawable lookup must be a Map"));
        }
        let key = self.key(key);
        let word = self
            .captured(read_map_text(&self.context, *id, key.as_ref()))
            .map_err(pele)?;
        self.read_atom(word)
            .map_err(pele)?
            .ok_or_else(|| draw_invalid("Invalid host map lookup atom"))
    }
    fn map_get_value(&self, a: &Atom, key: &Atom) -> Result<Atom, PeleError> {
        let Atom::Ref(id) = a else {
            return Err(draw_invalid("ER drawable lookup must be a Map"));
        };
        if self.kind(*id).map_err(pele)?.0 != 2 {
            return Err(draw_invalid("ER drawable lookup must be a Map"));
        }
        let (tag, payload, numeric) = map_key_arguments(key);
        let word = self
            .captured(read_map_value(&self.context, *id, tag, payload, numeric))
            .map_err(pele)?;
        self.read_atom(word)
            .map_err(pele)?
            .ok_or_else(|| draw_invalid("Invalid host map lookup atom"))
    }
    fn string_units(&self, value: &Atom) -> Result<JsString, PeleError> {
        let Atom::String(id) = value else {
            return Err(PeleError::new(
                "Expected a public string",
                ErrorCode::Semantic,
            ));
        };
        self.units(*id)
            .map(|units| JsString(units.as_ref().clone()))
            .map_err(pele)
    }
    fn coerce_string(&self, value: &Atom) -> Result<String, PeleError> {
        // Primitive ToString cannot run caller code. Reuse immutable UTF16 and
        // the existing JS-compatible formatter without a host round trip.
        match value {
            Atom::String(id) => {
                return String::from_utf16(&self.units(*id).map_err(pele)?).map_err(|_| {
                    pele(invalid(
                        "ER rendering of lone UTF-16 surrogates is not implemented",
                    ))
                });
            }
            Atom::Undefined => return Ok("undefined".into()),
            Atom::Null => return Ok("null".into()),
            Atom::Bool(value) => return Ok(if *value { "true" } else { "false" }.into()),
            Atom::Number(value) => return Ok(pele::compat::host_er::number_string(*value)),
            // Registered functions are objects too and may carry conversion
            // hooks. Symbols and other opaque values retain host error identity.
            Atom::Ref(_) | Atom::BuiltinFunction(_) | Atom::HostReference(_) => {}
        }
        let value = self.call("coerceUnits", &[atom_js(value)]).map_err(pele)?;
        if !value.is_instance_of::<Uint16Array>() {
            return Err(pele(invalid("Invalid host string units")));
        }
        let units = Uint16Array::new(&value);
        if units.length() > 16_000_000 {
            return Err(pele(limit()));
        }
        String::from_utf16(&units.to_vec()).map_err(|_| {
            pele(invalid(
                "ER rendering of lone UTF-16 surrogates is not implemented",
            ))
        })
    }
    fn string(&self, value: &Atom, operation: &'static str) -> Result<String, PeleError> {
        if let Atom::String(id) = value {
            return String::from_utf16(&self.units(*id).map_err(pele)?).map_err(|_| {
                pele(invalid(
                    "ER rendering of lone UTF-16 surrogates is not implemented",
                ))
            });
        }
        // Label layout tests its markup regex before calling string methods.
        // RegExp ToString can throw (notably Symbols) or invoke caller coercion.
        // Other text helpers reach includes directly without this conversion.
        if operation == "label" {
            RenderStore::coerce_string(self, value)?;
        }
        self.call(
            "stringFailure",
            &[atom_js(value), JsValue::from_str(operation)],
        )
        .map_err(pele)?;
        Err(pele(invalid(
            "Host exception primitive returned unexpectedly",
        )))
    }
    fn join(&self, array: &Atom, separator: &str) -> Result<String, PeleError> {
        RenderStore::array_length(self, array)?;
        let value = self
            .call("joinUnits", &[atom_js(array), JsValue::from_str(separator)])
            .map_err(pele)?;
        let units = Uint16Array::new(&value);
        if units.length() > 16_000_000 {
            return Err(pele(limit()));
        }
        String::from_utf16(&units.to_vec()).map_err(|_| {
            pele(invalid(
                "ER rendering of lone UTF-16 surrogates is not implemented",
            ))
        })
    }
    fn truthy(&self, value: &Atom) -> Result<bool, PeleError> {
        MethodStore::truth(self, value).map_err(pele)
    }
    fn charge(&self, work: usize, units: usize) -> Result<(), PeleError> {
        let n = self.live_work.get().saturating_add(work);
        let u = self.live_units.get().saturating_add(units);
        self.live_work.set(n);
        self.live_units.set(u);
        if n > 1_000_000 || u.saturating_mul(3) > 128 * 1024 * 1024 {
            Err(pele(limit()))
        } else {
            Ok(())
        }
    }
}

fn discard_frame(frame: &Option<Rc<RefCell<Option<EntryFrame>>>>) {
    if let Some(frame) = frame {
        frame.borrow_mut().take();
    }
}
struct FrameResources<R> {
    frame: Option<Rc<RefCell<Option<EntryFrame>>>>,
    inner: R,
}
impl<R: pele::compat::RenderResources> pele::compat::RenderResources for FrameResources<R> {
    fn host_function(
        &mut self,
        reference: u32,
    ) -> Result<Rc<dyn pele::compat::public::HostFunction>, PublicError> {
        discard_frame(&self.frame);
        self.inner.host_function(reference)
    }
    fn presentation_budget(&self) -> Option<pele::compat::PresentationBudget> {
        self.inner.presentation_budget()
    }
    fn capture_presentation(&mut self, outcome: pele::compat::PresentationOutcome) {
        discard_frame(&self.frame);
        self.inner.capture_presentation(outcome);
    }
    fn image(&mut self, uri: &str) -> Result<pele::compat::ImageResource, PeleError> {
        discard_frame(&self.frame);
        self.inner.image(uri)
    }
    fn text_width(&mut self, request: pele::compat::TextRequest<'_>) -> Result<f64, PeleError> {
        discard_frame(&self.frame);
        self.inner.text_width(request)
    }
    fn icon(&mut self, name: &str) -> Result<Option<pele::compat::TrustedSvg>, PeleError> {
        discard_frame(&self.frame);
        self.inner.icon(name)
    }
    fn raw_inline_icon(
        &mut self,
        name: &str,
    ) -> Result<Option<pele::compat::TrustedSvg>, PeleError> {
        discard_frame(&self.frame);
        self.inner.raw_inline_icon(name)
    }
    fn direct_icon(&mut self, name: &str) -> Result<Option<pele::compat::TrustedSvg>, PeleError> {
        discard_frame(&self.frame);
        self.inner.direct_icon(name)
    }
    fn parse_date(&mut self, value: &str) -> Result<Option<f64>, PeleError> {
        discard_frame(&self.frame);
        self.inner.parse_date(value)
    }
    fn calendar_fields(
        &mut self,
        value: f64,
    ) -> Result<Option<pele::compat::CalendarFields>, PeleError> {
        discard_frame(&self.frame);
        self.inner.calendar_fields(value)
    }
    fn calendar_instant(
        &mut self,
        value: pele::compat::CivilDateTime,
    ) -> Result<Option<f64>, PeleError> {
        discard_frame(&self.frame);
        self.inner.calendar_instant(value)
    }
    fn direct_now_millis(&mut self) -> Result<f64, PeleError> {
        discard_frame(&self.frame);
        self.inner.direct_now_millis()
    }
    fn now_millis(&mut self) -> Result<f64, PeleError> {
        discard_frame(&self.frame);
        self.inner.now_millis()
    }
}

/// Temporary facade selection: captured policy plus originating registry.
/// Released after adoption; it never owns the caller root or a graph.
#[wasm_bindgen(js_name=HostErEntrySelection)]
pub struct HostErEntrySelection {
    inner: pele::compat::public::HostErEntrySelection,
    origin: Rc<()>,
}
#[wasm_bindgen]
impl HostErEntrySelection {
    pub fn schema(&self) -> String {
        super::public_result(&self.inner.schema())
    }
}

#[wasm_bindgen(js_name=HostErDocument)]
pub struct HostErDocument {
    policy: pele::compat::host_er::Policy,
}
#[wasm_bindgen]
impl HostErDocument {
    pub fn invoke_host(
        &self,
        method: u32,
        primitive: Function,
        capture: Function,
        finish: Function,
        values: Array,
    ) -> Result<String, JsValue> {
        let result = (|| {
            if values.length() == 0 {
                return Err(invalid("Public method receiver missing"));
            }
            if values.length() > 1_000_001 {
                return Err(limit());
            }
            let mut store = HostStore::new(primitive, capture)?;
            let receiver = atom(values.get(0))?;
            let arguments = (1..values.length())
                .map(|i| atom(values.get(i)))
                .collect::<Result<Vec<_>, _>>()?;
            pele::compat::host_er::invoke(&mut store, method, receiver, arguments)
        })();
        finish.call0(&JsValue::UNDEFINED)?;
        metadata(&result)
    }
    pub fn live_provider_view(&self) -> Result<Array, JsValue> {
        let config_json = super::foreign_provider::config_json(self.policy.config())
            .map_err(|e| JsValue::from_str(&pele::compat::error_envelope(&e)))?;
        let values = Array::new();
        values.push(&JsValue::from_str("er"));
        values.push(&js_sys::JSON::parse(&config_json)?);
        values
            .push(&super::foreign_provider::retained_policy(pele::compat::DiagramType::Er).into());
        Ok(values)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn render_host(
        &self,
        primitive: Function,
        capture: Function,
        finish: Function,
        json: &str,
        mask: u32,
        numbers: &[f64],
        output_kind: u32,
        output: f64,
        width: Option<Function>,
        icons: Option<Function>,
        clock: Option<Function>,
    ) -> Result<String, JsValue> {
        let result = self.render_host_body(
            primitive,
            capture,
            finish,
            json,
            mask,
            numbers,
            output_kind,
            output,
            width,
            icons,
            clock,
            None,
        )?;
        metadata(&result.as_ref().map(pele::compat::render_result_transport))
    }
    #[allow(clippy::too_many_arguments)]
    pub fn render_host_packet(
        &self,
        primitive: Function,
        capture: Function,
        finish: Function,
        json: &str,
        mask: u32,
        numbers: &[f64],
        output_kind: u32,
        output: f64,
        width: Option<Function>,
        icons: Option<Function>,
        clock: Option<Function>,
    ) -> Result<JsValue, JsValue> {
        let result = self.render_host_body(
            primitive,
            capture,
            finish,
            json,
            mask,
            numbers,
            output_kind,
            output,
            width,
            icons,
            clock,
            None,
        )?;
        let transport = result.as_ref().map(pele::compat::render_result_transport);
        match &result {
            Ok(value) => successful_packet(
                false,
                value.diagram_type.as_str(),
                &value.rendered,
                &transport,
            ),
            Err(_) => metadata(&transport).map(JsValue::from),
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn render_host_frame_packet(
        &self,
        primitive: Function,
        capture: Function,
        finish: Function,
        json: &str,
        mask: u32,
        numbers: &[f64],
        output_kind: u32,
        output: f64,
        width: Option<Function>,
        icons: Option<Function>,
        clock: Option<Function>,
        frame: Option<Uint8Array>,
    ) -> Result<JsValue, JsValue> {
        let result = self.render_host_body(
            primitive,
            capture,
            finish,
            json,
            mask,
            numbers,
            output_kind,
            output,
            width,
            icons,
            clock,
            frame,
        )?;
        let transport = result.as_ref().map(pele::compat::render_result_transport);
        match &result {
            Ok(value) => successful_packet(
                false,
                value.diagram_type.as_str(),
                &value.rendered,
                &transport,
            ),
            Err(_) => metadata(&transport).map(JsValue::from),
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn render_raw_host(
        &self,
        primitive: Function,
        capture: Function,
        finish: Function,
        config_json: &str,
        json: &str,
        mask: u32,
        numbers: &[f64],
        width: Option<Function>,
        icons: Option<Function>,
        clock: Option<Function>,
        inline: Option<Function>,
    ) -> Result<String, JsValue> {
        let result = self.render_raw_host_body(
            primitive,
            capture,
            finish,
            config_json,
            json,
            mask,
            numbers,
            width,
            icons,
            clock,
            inline,
        )?;
        metadata(&result.as_ref().map(super::prepared_entries::transport))
    }
    #[allow(clippy::too_many_arguments)]
    pub fn render_raw_host_packet(
        &self,
        primitive: Function,
        capture: Function,
        finish: Function,
        config_json: &str,
        json: &str,
        mask: u32,
        numbers: &[f64],
        width: Option<Function>,
        icons: Option<Function>,
        clock: Option<Function>,
        inline: Option<Function>,
    ) -> Result<JsValue, JsValue> {
        let result = self.render_raw_host_body(
            primitive,
            capture,
            finish,
            config_json,
            json,
            mask,
            numbers,
            width,
            icons,
            clock,
            inline,
        )?;
        let transport = result.as_ref().map(super::prepared_entries::transport);
        match &result {
            Ok(value) => successful_packet(true, "", value, &transport),
            Err(_) => metadata(&transport).map(JsValue::from),
        }
    }
}
impl HostErDocument {
    #[allow(clippy::too_many_arguments)]
    fn render_host_body(
        &self,
        primitive: Function,
        capture: Function,
        finish: Function,
        json: &str,
        mask: u32,
        numbers: &[f64],
        output_kind: u32,
        output: f64,
        width: Option<Function>,
        icons: Option<Function>,
        clock: Option<Function>,
        frame: Option<Uint8Array>,
    ) -> Result<Result<pele::compat::RenderResult, PublicError>, JsValue> {
        let (options, limit) =
            super::render_numbers::options(json, mask, numbers, output_kind, output)
                .map_err(native_failure)?;
        let result = (|| {
            let mut store = HostStore::new(primitive, capture).map_err(pele)?;
            store.attach_frame(frame);
            pele::compat::host_er::validate_render(&store)?;
            self.policy.render(
                &store,
                &options,
                limit,
                &mut FrameResources {
                    frame: store.entry_frame.clone(),
                    inner: super::render_numbers::NumericResources {
                        browser: super::BrowserResources { width, icons },
                        clock,
                    },
                },
            )
        })()
        .map_err(PublicError::Pele);
        finish.call0(&JsValue::UNDEFINED)?;
        Ok(result)
    }
    #[allow(clippy::too_many_arguments)]
    fn render_raw_host_body(
        &self,
        primitive: Function,
        capture: Function,
        finish: Function,
        config_json: &str,
        json: &str,
        mask: u32,
        numbers: &[f64],
        width: Option<Function>,
        icons: Option<Function>,
        clock: Option<Function>,
        inline: Option<Function>,
    ) -> Result<Result<pele::compat::Rendered, PublicError>, JsValue> {
        let config = super::prepared_entries::config(config_json).map_err(native_failure)?;
        let (options, _) =
            super::render_numbers::options(json, mask, numbers, 3, 0.).map_err(native_failure)?;
        let result = (|| {
            let store = HostStore::new(primitive, capture).map_err(pele)?;
            pele::compat::host_er::validate_render(&store)?;
            self.policy.render_raw(
                &store,
                &config,
                &options.render,
                &mut super::prepared_entries::host_er_raw_resources(width, icons, clock, inline),
            )
        })()
        .map_err(PublicError::Pele);
        finish.call0(&JsValue::UNDEFINED)?;
        Ok(result)
    }
}
fn successful_packet(
    raw: bool,
    diagram_type: &str,
    rendered: &pele::compat::Rendered,
    transport: &impl serde::Serialize,
) -> Result<JsValue, JsValue> {
    // Audit already finished. Count the exact legacy envelope, including any
    // $numbers fields, before allocating the smaller links payload or JS result.
    let metadata_bytes =
        super::render_numbers::encoded_length(transport).map_err(native_failure)?;
    let links = super::render_numbers::encode(&rendered.links).map_err(native_failure)?;
    let links = std::str::from_utf8(&links).expect("JSON UTF8");
    let width = rendered.width.to_bits();
    let height = rendered.height.to_bits();
    render_result_packet(
        raw,
        diagram_type,
        &rendered.svg,
        width as u32,
        (width >> 32) as u32,
        height as u32,
        (height >> 32) as u32,
        links,
        metadata_bytes as u32,
    )
}

fn native_failure(error: PeleError) -> JsValue {
    JsValue::from_str(&super::render_numbers::failure(&error))
}
fn metadata(value: &impl serde::Serialize) -> Result<String, JsValue> {
    super::render_numbers::encode(value)
        .map(|bytes| String::from_utf8(bytes).expect("JSON UTF8"))
        .map_err(native_failure)
}

/// Pure unpublished construction emits bounded storage events, never a graph.
struct HostRecordLayout {
    identity: usize,
    length: usize,
}
struct HostConstruction {
    store: HostStore,
    record_layouts: Vec<HostRecordLayout>,
    strings: HashMap<String, u32>,
    string_units: usize,
    nodes: u32,
    frame: Vec<u8>,
    events: u16,
}
const CONSTRUCTION_BATCH_BYTES: usize = 64 * 1024;
const CONSTRUCTION_BATCH_EVENTS: u16 = 128;
fn construction_frame_limit() -> PeleError {
    PeleError::new(
        "Public binary graph exceeds transport budget",
        ErrorCode::Limit,
    )
}
fn construction_atom_len(atom: &Atom) -> usize {
    match atom {
        Atom::Undefined | Atom::Null | Atom::Bool(_) => 1,
        Atom::Number(_) => 9,
        _ => 5,
    }
}
// Count only until the bounded inline event no longer fits. Large nodes keep
// the existing incremental stream; this never allocates a second node/frame.
fn construction_inline_len(node: &pele::compat::public::Node) -> Option<usize> {
    use pele::compat::public::Node;
    let mut bytes = 18usize;
    let mut add = |amount: usize| -> Option<()> {
        bytes = bytes.checked_add(amount)?;
        (bytes <= CONSTRUCTION_BATCH_BYTES - 2).then_some(())
    };
    match node {
        Node::Map { entries, .. } => {
            for (key, value) in entries {
                add(construction_atom_len(key) + construction_atom_len(value))?;
            }
        }
        Node::Set { values, .. } => {
            for value in values {
                add(construction_atom_len(value))?;
            }
        }
        _ => {}
    }
    for property in node.properties() {
        add(5 + construction_atom_len(&property.value))?;
    }
    Some(bytes)
}
fn construction_atom(out: &mut Vec<u8>, atom: &Atom) {
    let (tag, payload) = match atom {
        Atom::Undefined => (0, 0),
        Atom::Null => (1, 0),
        Atom::Bool(false) => (2, 0),
        Atom::Bool(true) => (3, 0),
        Atom::Number(value) => (4, value.to_bits()),
        Atom::String(id) => (5, u64::from(*id)),
        Atom::Ref(id) => (6, u64::from(*id)),
        Atom::BuiltinFunction(id) => (7, u64::from(*id)),
        Atom::HostReference(id) => (8, u64::from(*id)),
    };
    out.push(tag);
    out.extend_from_slice(&payload.to_le_bytes()[..construction_atom_len(atom) - 1]);
}
impl HostConstruction {
    fn flush(&mut self) -> Result<(), PeleError> {
        if self.events == 0 {
            return Ok(());
        }
        self.frame[..2].copy_from_slice(&self.events.to_le_bytes());
        // An owned copy avoids a borrowed WASM view across arbitrary imports.
        let bytes = Uint8Array::from(self.frame.as_slice());
        let result = self
            .store
            .call("publishBatch", &[bytes.into()])
            .map_err(pele)?;
        match result.as_f64() {
            Some(0.0) => {}
            Some(-1.0) => {
                return Err(pele(invalid(
                    "Fresh host constructor property was rejected",
                )))
            }
            Some(-2.0) => {
                return Err(pele(invalid(
                    "Fresh host constructor array length was rejected",
                )))
            }
            _ => return Err(pele(invalid("Invalid host constructor batch result"))),
        }
        self.frame.clear();
        self.events = 0;
        Ok(())
    }
    fn event(&mut self, tag: u8, bytes: usize) -> Result<(), PeleError> {
        // Every event, including a large-string fragment, is bounded before
        // reserving scratch or allocating the owned JS transfer buffer.
        if bytes == 0 || bytes > CONSTRUCTION_BATCH_BYTES - 2 {
            return Err(construction_frame_limit());
        }
        if self.events == CONSTRUCTION_BATCH_EVENTS
            || self.frame.len().max(2) + bytes > CONSTRUCTION_BATCH_BYTES
        {
            self.flush()?;
        }
        let needed = self.frame.len().max(2) + bytes;
        if self.frame.capacity() < needed {
            let capacity = needed
                .max(512)
                .next_power_of_two()
                .min(CONSTRUCTION_BATCH_BYTES);
            self.frame.reserve_exact(capacity - self.frame.len());
        }
        if self.frame.is_empty() {
            self.frame.extend_from_slice(&[0, 0]);
        }
        self.frame.push(tag);
        self.events += 1;
        Ok(())
    }
}
impl pele::compat::host_er::InitialWriter for HostConstruction {
    type Output = ();
    fn string(&mut self, value: &str) -> Result<u32, PeleError> {
        if let Some(id) = self.strings.get(value) {
            return Ok(*id);
        }
        let units = value.encode_utf16().count();
        let total = self
            .string_units
            .checked_add(units)
            .filter(|n| *n <= 16_000_000)
            .ok_or_else(|| {
                PeleError::new("Public model graph exceeds work budget", ErrorCode::Limit)
            })?;
        if self.strings.len() >= 1_000_000 {
            return Err(PeleError::new(
                "Public model graph exceeds work budget",
                ErrorCode::Limit,
            ));
        }
        // Fresh owner + pure construction gives the same first-intern sequence
        // as GraphWriter and the host pool. No callback can rewrite that pool.
        // Legacy fallback completed exact counting admission before this sink.
        let id = self.strings.len() as u32;
        if units <= (CONSTRUCTION_BATCH_BYTES - 2 - 9) / 2 {
            self.event(0, 9 + units * 2)?;
            self.frame.extend_from_slice(&id.to_le_bytes());
            self.frame.extend_from_slice(&(units as u32).to_le_bytes());
            for unit in value.encode_utf16() {
                self.frame.extend_from_slice(&unit.to_le_bytes());
            }
        } else {
            self.event(1, 9)?;
            self.frame.extend_from_slice(&id.to_le_bytes());
            self.frame.extend_from_slice(&(units as u32).to_le_bytes());
            let mut remaining = units;
            let mut input = value.encode_utf16();
            while remaining > 0 {
                let count = remaining.min((CONSTRUCTION_BATCH_BYTES - 2 - 5) / 2);
                self.event(2, 5 + count * 2)?;
                self.frame.extend_from_slice(&(count as u32).to_le_bytes());
                for unit in input.by_ref().take(count) {
                    self.frame.extend_from_slice(&unit.to_le_bytes());
                }
                remaining -= count;
            }
            self.event(3, 1)?;
        }
        self.string_units = total;
        self.strings.insert(value.to_owned(), id);
        Ok(id)
    }
    fn record<const N: usize>(
        &mut self,
        keys: &'static [&'static str; N],
        values: [Atom; N],
    ) -> Result<u32, PeleError> {
        // Private identity of immutable layout storage, never sent to JS. A
        // compiler may merge identical constants; their field order is equal.
        let identity = keys.as_ptr() as usize;
        let known = self
            .record_layouts
            .iter()
            .position(|layout| layout.identity == identity && layout.length == N);
        let layout = if let Some(id) = known {
            id
        } else {
            // Optional bounded specialization. Unusual future layouts use the
            // exact generic property path; this is not a new model limit.
            let eligible = N <= 64
                && self.record_layouts.len() < 32
                && keys.iter().enumerate().all(|(index, key)| {
                    *key != "__proto__" && *key != "length" && !keys[..index].contains(key)
                });
            if !eligible {
                let properties = keys
                    .iter()
                    .zip(values)
                    .map(|(key, value)| self.property(key, value))
                    .collect::<Result<_, _>>()?;
                return self.node(pele::compat::public::Node::Object { properties });
            }
            // First layout occurrence interns keys at the original rec() stage,
            // after all values. Later occurrences are guaranteed interner hits;
            // this sink charges text/table storage only on first intern.
            let mut ids = Vec::with_capacity(N);
            for key in keys {
                ids.push(self.string(key)?);
            }
            let id = self.record_layouts.len();
            self.event(10, 3 + 4 * N)?;
            self.frame.push(id as u8);
            self.frame.push(N as u8);
            for key in ids {
                self.frame.extend_from_slice(&key.to_le_bytes());
            }
            self.record_layouts.push(HostRecordLayout {
                identity,
                length: N,
            });
            id
        };
        // Match node() admission after key interning. All fixed-layout records
        // fit a single bounded event, independently of their referenced strings.
        if self.nodes >= 1_000_000 {
            return Err(PeleError::new(
                "Public model graph exceeds work budget",
                ErrorCode::Limit,
            ));
        }
        let id = self.nodes;
        let bytes = 6 + values.iter().map(construction_atom_len).sum::<usize>();
        self.event(11, bytes)?;
        self.frame.extend_from_slice(&id.to_le_bytes());
        self.frame.push(layout as u8);
        for value in values {
            construction_atom(&mut self.frame, &value);
        }
        self.nodes += 1;
        Ok(id)
    }
    fn dense_array(&mut self, values: &[Atom]) -> Result<u32, PeleError> {
        // Eligibility examines only existing Atom tags. Oversized arrays keep
        // the original property/node path without an extra value collection.
        let bytes = values.iter().try_fold(9usize, |bytes, value| {
            bytes
                .checked_add(construction_atom_len(value))
                .filter(|bytes| *bytes <= CONSTRUCTION_BATCH_BYTES - 2)
        });
        let Some(bytes) = bytes else {
            let mut properties = Vec::with_capacity(values.len());
            for (index, value) in values.iter().enumerate() {
                properties.push(self.property(&index.to_string(), value.clone())?);
            }
            return self.node(pele::compat::public::Node::Array {
                length: properties.len() as u32,
                properties,
            });
        };
        // These keys remain part of the canonical string table even though
        // this event reconstructs their names. Preserve all IDs and budgets.
        for index in 0..values.len() {
            self.string(&index.to_string())?;
        }
        if self.nodes >= 1_000_000 {
            return Err(PeleError::new(
                "Public model graph exceeds work budget",
                ErrorCode::Limit,
            ));
        }
        let id = self.nodes;
        self.event(12, bytes)?;
        self.frame.extend_from_slice(&id.to_le_bytes());
        // Event fit bounds the number of atoms below u32::MAX.
        self.frame
            .extend_from_slice(&(values.len() as u32).to_le_bytes());
        for value in values {
            construction_atom(&mut self.frame, value);
        }
        self.nodes += 1;
        Ok(id)
    }
    fn node(&mut self, node: pele::compat::public::Node) -> Result<u32, PeleError> {
        use pele::compat::public::Node;
        let (kind, length) = match &node {
            Node::Object { .. } => (0u8, 0),
            Node::Array { length, .. } => (1, *length),
            Node::Map { .. } => (2, 0),
            Node::Set { .. } => (3, 0),
            Node::Instance { prototype, .. } if prototype.0 == 1 => (4, 0),
            _ => return Err(pele(invalid("Unsupported host constructor node"))),
        };
        if self.nodes >= 1_000_000 {
            return Err(PeleError::new(
                "Public model graph exceeds work budget",
                ErrorCode::Limit,
            ));
        }
        let id = self.nodes;
        if let Some(bytes) = construction_inline_len(&node) {
            let payload_count = match &node {
                Node::Map { entries, .. } => entries.len(),
                Node::Set { values, .. } => values.len(),
                _ => 0,
            };
            // The length check bounds both counts below u32::MAX before casts.
            self.event(9, bytes)?;
            self.frame.extend_from_slice(&id.to_le_bytes());
            self.frame.push(kind);
            self.frame.extend_from_slice(&length.to_le_bytes());
            self.frame
                .extend_from_slice(&(payload_count as u32).to_le_bytes());
            self.frame
                .extend_from_slice(&(node.properties().len() as u32).to_le_bytes());
            match &node {
                Node::Map { entries, .. } => {
                    for (key, value) in entries {
                        construction_atom(&mut self.frame, key);
                        construction_atom(&mut self.frame, value);
                    }
                }
                Node::Set { values, .. } => {
                    for value in values {
                        construction_atom(&mut self.frame, value);
                    }
                }
                _ => {}
            }
            for property in node.properties() {
                self.frame.extend_from_slice(&property.key.to_le_bytes());
                self.frame.push(property.flags);
                construction_atom(&mut self.frame, &property.value);
            }
            self.nodes += 1;
            return Ok(id);
        }
        self.event(4, 10)?;
        self.frame.extend_from_slice(&id.to_le_bytes());
        self.frame.push(kind);
        self.frame.extend_from_slice(&length.to_le_bytes());
        match &node {
            Node::Map { entries, .. } => {
                for (key, value) in entries {
                    self.event(
                        5,
                        1 + construction_atom_len(key) + construction_atom_len(value),
                    )?;
                    construction_atom(&mut self.frame, key);
                    construction_atom(&mut self.frame, value);
                }
            }
            Node::Set { values, .. } => {
                for value in values {
                    self.event(6, 1 + construction_atom_len(value))?;
                    construction_atom(&mut self.frame, value);
                }
            }
            _ => {}
        }
        for property in node.properties() {
            self.event(7, 6 + construction_atom_len(&property.value))?;
            self.frame.extend_from_slice(&property.key.to_le_bytes());
            self.frame.push(property.flags);
            construction_atom(&mut self.frame, &property.value);
        }
        self.event(8, 1)?;
        self.nodes += 1;
        Ok(id)
    }
    fn finish(mut self, root: Atom) -> Result<(), PeleError> {
        self.flush()?;
        self.store
            .call("setRoot", &[atom_js(&root)])
            .map_err(pele)?;
        self.store.call("constructed", &[]).map_err(pele)?;
        Ok(())
    }
    fn abort(self) -> Result<(), PeleError> {
        // Do not publish pending events after a grammar/eligibility failure.
        self.store.call("discard", &[]).map_err(pele)?;
        Ok(())
    }
}
fn host_construction(
    schema: pele::compat::public::ModelSchemaView,
    make: &Function,
    capture: &Function,
) -> Result<HostConstruction, PeleError> {
    // Reuse the facade's existing validated immutable-schema cache.
    // On wasm these are static linear-memory offsets, not host addresses.
    #[cfg(target_arch = "wasm32")]
    let key = format!(
        "1:{:x}:{:x}:{}:{:x}:{}",
        std::ptr::from_ref(schema.model) as usize,
        schema.shared_functions.as_ptr() as usize,
        schema.shared_functions.len(),
        schema.function_shapes.as_ptr() as usize,
        schema.function_shapes.len()
    );
    #[cfg(not(target_arch = "wasm32"))]
    let key = String::from("native-host-er");
    let factory = |json: &JsValue| {
        make.call2(&JsValue::UNDEFINED, &JsValue::from_str(&key), json)
            .map_err(|error| {
                let _ = capture.call1(&JsValue::UNDEFINED, &error);
                pele(invalid("Host construction factory failed"))
            })
    };
    let cached = factory(&JsValue::UNDEFINED)?;
    let primitive = if cached.is_undefined() {
        factory(&JsValue::from_str(&super::public_result(&schema)))?
    } else {
        cached
    };
    let primitive = primitive
        .dyn_into::<Function>()
        .map_err(|_| pele(invalid("Invalid host construction factory")))?;
    Ok(HostConstruction {
        store: HostStore::new(primitive, capture.clone()).map_err(pele)?,
        record_layouts: Vec::new(),
        strings: HashMap::new(),
        string_units: 0,
        nodes: 0,
        frame: Vec::new(),
        events: 0,
    })
}
fn host_parsed(parsed: pele::compat::host_er::Parsed<()>) -> Array {
    let result = Array::new();
    match parsed {
        pele::compat::host_er::Parsed::Host { policy, .. } => {
            result.push(&number(1));
            result.push(&HostErDocument { policy }.into());
        }
        pele::compat::host_er::Parsed::Arena(inner) => {
            result.push(&number(0));
            result.push(&super::MutableDocument { inner }.into());
        }
    }
    result
}
#[wasm_bindgen]
impl NativeRegistry {
    /// Raw caller selection does not parse, import or publish any model.
    pub fn select_prepared_host_er(
        &self,
        id: &str,
    ) -> Result<Option<HostErEntrySelection>, JsValue> {
        let engine = self.inner.as_ref().ok_or_else(|| {
            native_failure(PeleError::new("Registry is closed", ErrorCode::Semantic))
        })?;
        let id = super::prepared_entries::builtin_id(id).map_err(native_failure)?;
        let selected = engine.select_host_er_entry(&id).map_err(native_failure)?;
        Ok(selected.map(|inner| HostErEntrySelection {
            inner,
            origin: self.raw_entry_origin.get_or_init(|| Rc::new(())).clone(),
        }))
    }
    /// Recheck open after caller traversal, before decoding config. The
    /// captured registration survives replacement; it is never selected again.
    pub fn bind_prepared_host_er(
        &self,
        selection: &HostErEntrySelection,
        config_json: &str,
    ) -> Result<HostErDocument, JsValue> {
        self.inner.as_ref().ok_or_else(|| {
            native_failure(PeleError::new("Registry is closed", ErrorCode::Semantic))
        })?;
        if !self
            .raw_entry_origin
            .get()
            .is_some_and(|origin| Rc::ptr_eq(origin, &selection.origin))
        {
            return Err(native_failure(PeleError::new(
                "Raw entry selection belongs to another registry",
                ErrorCode::Semantic,
            )));
        }
        let config = super::prepared_entries::config(config_json).map_err(native_failure)?;
        Ok(HostErDocument {
            policy: selection.inner.bind(config).map_err(native_failure)?,
        })
    }
    /// Optional facade-only path. The low-level parse method keeps returning a
    /// MutableDocument with its existing graph, revision and snapshot contract.
    #[allow(clippy::too_many_arguments)]
    pub fn parse_with_host_er(
        &self,
        source: &str,
        json: &str,
        source_kind: u32,
        source_value: f64,
        edge_kind: u32,
        edge_value: f64,
        make: Function,
        capture: Function,
    ) -> Result<Array, JsValue> {
        let engine = self.inner.as_ref().ok_or_else(|| {
            native_failure(PeleError::new("Registry is closed", ErrorCode::Semantic))
        })?;
        let (options, limits) = super::parse_limits::decode_parse_options(
            source,
            json,
            source_kind,
            source_value,
            edge_kind,
            edge_value,
        )
        .map_err(native_failure)?;
        let parsed = engine
            .parse_host_er_with_limits(source, &options, limits, |schema| {
                host_construction(schema, &make, &capture)
            })
            .map_err(native_failure)?;
        Ok(host_parsed(parsed))
    }
    /// Additive facade entry; the public parse_prepared export remains arena-based.
    #[allow(clippy::too_many_arguments)]
    pub fn parse_prepared_with_host_er(
        &self,
        id: &str,
        source: &str,
        json: &str,
        title: Option<String>,
        line_offset: f64,
        edge_kind: u32,
        edge_value: f64,
        make: Function,
        capture: Function,
    ) -> Result<Array, JsValue> {
        let input = super::prepared_entries::prepared_input(
            self,
            id,
            source,
            json,
            title.as_deref(),
            line_offset,
            edge_kind,
            edge_value,
        )
        .map_err(native_failure)?;
        let parsed = input
            .engine
            .parse_prepared_host_er_as(
                &input.id,
                pele::compat::ParseContext {
                    source,
                    config: &input.config,
                    title: title.as_deref(),
                    line_offset: input.line_offset,
                    max_edges: pele::compat::DEFAULT_MAX_EDGES,
                },
                input.limit,
                |schema| host_construction(schema, &make, &capture),
            )
            .map_err(native_failure)?;
        Ok(host_parsed(parsed))
    }
}
