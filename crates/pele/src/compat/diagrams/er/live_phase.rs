//! Bridge generic owned host values into the existing ER phase reader.
//! Private Atom IDs name this job's ledger, never a document graph coordinate.
use super::live_read::RenderStore;
use crate::compat::{
    public::{Atom, JsString, LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    sync::Arc,
};

pub(crate) struct ErDetachedInput {
    limit: Option<PeleError>,
}
impl ErDetachedInput {
    pub(super) fn new(error: Option<PeleError>) -> Self {
        Self {
            limit: error.filter(|e| e.code == ErrorCode::Limit),
        }
    }
}
fn invalid(text: &str) -> PeleError {
    let mut e = PeleError::new(text, ErrorCode::Semantic);
    e.diagram_type = Some("er".into());
    e
}
fn limit() -> PeleError {
    let mut e = PeleError::new(
        "ER native graph operation exceeds work budget",
        ErrorCode::Limit,
    );
    e.diagram_type = Some("er".into());
    e
}
#[derive(Hash, Eq, PartialEq)]
enum Key {
    Undefined,
    Null,
    Bool(bool),
    Number(u64),
    Text(Arc<JsString>),
    Object(u64),
}
struct Bound<V> {
    value: V,
    text: Option<Arc<JsString>>,
}
struct Ledger<V> {
    values: Vec<Bound<V>>,
    keys: HashMap<Key, u32>,
    units: usize,
}
struct Store<'a, R: LiveRenderRead> {
    read: RefCell<&'a mut R>,
    ledger: RefCell<Ledger<R::Value>>,
    root: Atom,
    original: RefCell<Option<PublicError>>,
    work: Cell<usize>,
    units: Cell<usize>,
}
impl<R: LiveRenderRead> Store<'_, R> {
    fn result<T>(&self, result: Result<T, PublicError>) -> Result<T, PeleError> {
        result.map_err(|e| {
            let mut original = self.original.borrow_mut();
            if original.is_none() {
                *original = Some(e);
            }
            invalid("ER live read failed")
        })
    }
    fn bind(&self, value: R::Value) -> Result<Atom, PeleError> {
        let (key, text, scalar) = {
            let mut read = self.read.borrow_mut();
            match read.kind(&value) {
                LiveValueKind::Undefined => (Key::Undefined, None, Some(Atom::Undefined)),
                LiveValueKind::Null => (Key::Null, None, Some(Atom::Null)),
                LiveValueKind::Boolean => {
                    let b = read.truthy(&value);
                    (Key::Bool(b), None, Some(Atom::Bool(b)))
                }
                LiveValueKind::Number => {
                    let n = read
                        .number(&value)
                        .ok_or_else(|| invalid("Invalid live number"))?;
                    (Key::Number(n.to_bits()), None, Some(Atom::Number(n)))
                }
                LiveValueKind::String => {
                    let s = self
                        .result(read.string_units(&value))?
                        .ok_or_else(|| invalid("Invalid live string"))?;
                    if s.0.len() > 16_000_000 {
                        return Err(limit());
                    }
                    let text = Arc::new(s);
                    (Key::Text(Arc::clone(&text)), Some(text), None)
                }
                LiveValueKind::Object | LiveValueKind::Function => {
                    let id = self
                        .result(read.object_identity(&value))?
                        .ok_or_else(|| invalid("Missing live object identity"))?;
                    (Key::Object(id), None, None)
                }
                _ => {
                    return Err(invalid(
                        "Native ER opaque primitive rendering is not implemented",
                    ))
                }
            }
        };
        let mut ledger = self.ledger.borrow_mut();
        let id = if let Some(id) = ledger.keys.get(&key) {
            *id
        } else {
            if ledger.values.len() >= 1_000_000 {
                return Err(limit());
            }
            let added = text.as_ref().map_or(0, |s| s.0.len());
            let units = ledger.units.saturating_add(added);
            if units.saturating_mul(3) > 128 * 1024 * 1024 {
                return Err(limit());
            }
            let id = ledger.values.len() as u32;
            ledger.values.push(Bound {
                value,
                text: text.clone(),
            });
            ledger.keys.insert(key, id);
            ledger.units = units;
            id
        };
        Ok(scalar.unwrap_or_else(|| {
            if text.is_some() {
                Atom::String(id)
            } else {
                Atom::HostReference(id)
            }
        }))
    }
    fn value(&self, atom: &Atom) -> Result<R::Value, PeleError> {
        let ledger = self.ledger.borrow();
        let id = match atom {
            Atom::String(id) | Atom::HostReference(id) => Some(*id),
            Atom::Undefined => ledger.keys.get(&Key::Undefined).copied(),
            Atom::Null => ledger.keys.get(&Key::Null).copied(),
            Atom::Bool(b) => ledger.keys.get(&Key::Bool(*b)).copied(),
            Atom::Number(n) => ledger.keys.get(&Key::Number(n.to_bits())).copied(),
            _ => None,
        }
        .ok_or_else(|| invalid("Invalid private ER value"))?;
        ledger
            .values
            .get(id as usize)
            .map(|v| v.value.clone())
            .ok_or_else(|| invalid("Invalid private ER value"))
    }
    fn read_value(
        &self,
        call: impl FnOnce(&mut R) -> Result<R::Value, PublicError>,
    ) -> Result<Atom, PeleError> {
        let result = { call(&mut **self.read.borrow_mut()) };
        self.bind(self.result(result)?)
    }
    fn utf8(&self, text: JsString) -> Result<String, PeleError> {
        if text.0.len() > 16_000_000 {
            return Err(limit());
        }
        text.well_formed()
            .ok_or_else(|| invalid("ER rendering of lone UTF-16 surrogates is not implemented"))
    }
}
impl<R: LiveRenderRead> RenderStore for Store<'_, R> {
    fn root(&self) -> Atom {
        self.root.clone()
    }
    fn field(&self, receiver: &Atom, key: &str) -> Result<Atom, PeleError> {
        let value = self.value(receiver)?;
        self.read_value(|r| r.get(&value, key))
    }
    fn array_length(&self, array: &Atom) -> Result<usize, PeleError> {
        let length = self.field(array, "length")?;
        let Atom::Number(n) = length else {
            return Err(invalid("Invalid live array length"));
        };
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(invalid("Invalid live array length"));
        }
        if n > 1_000_000. {
            return Err(limit());
        }
        Ok(n as usize)
    }
    fn array_get(&self, array: &Atom, index: usize) -> Result<Atom, PeleError> {
        let value = self.value(array)?;
        self.read_value(|r| r.index(&value, index))
    }
    fn array_has(&self, array: &Atom, index: usize) -> Result<bool, PeleError> {
        let value = self.value(array)?;
        let result = self.read.borrow_mut().has_index(&value, index);
        self.result(result)
    }
    fn map_entries(&self, map: &Atom) -> Result<Vec<(Atom, Atom)>, PeleError> {
        let value = self.value(map)?;
        let captured = self.result(self.read.borrow_mut().snapshot_iterable(&value, 1_000_000))?;
        if captured.len() > 1_000_000 {
            return Err(limit());
        }
        let mut entries = Vec::with_capacity(captured.len());
        for pair in captured {
            let key = self.read_value(|r| r.index(&pair, 0))?;
            let value = self.read_value(|r| r.index(&pair, 1))?;
            entries.push((key, value));
        }
        Ok(entries)
    }
    fn map_get(&self, map: &Atom, key: &str) -> Result<Atom, PeleError> {
        let value = self.value(map)?;
        self.read_value(|r| r.map_get_text(&value, key))
    }
    fn map_get_value(&self, map: &Atom, key: &Atom) -> Result<Atom, PeleError> {
        let (value, key) = (self.value(map)?, self.value(key)?);
        self.read_value(|r| r.map_get(&value, &key))
    }
    fn string_units(&self, atom: &Atom) -> Result<JsString, PeleError> {
        let Atom::String(id) = atom else {
            return Err(invalid("Invalid private ER string"));
        };
        self.ledger
            .borrow()
            .values
            .get(*id as usize)
            .and_then(|v| v.text.as_ref())
            .map(|s| s.as_ref().clone())
            .ok_or_else(|| invalid("Invalid private ER string"))
    }
    fn coerce_string(&self, atom: &Atom) -> Result<String, PeleError> {
        let value = self.value(atom)?;
        let result = self.read.borrow_mut().to_string(&value);
        self.utf8(self.result(result)?)
    }
    fn string(&self, atom: &Atom, operation: &'static str) -> Result<String, PeleError> {
        if matches!(atom, Atom::String(_)) {
            return self.utf8(self.string_units(atom)?);
        }
        if operation == "label" {
            self.coerce_string(atom)?;
        }
        let value = self.value(atom)?;
        let method = match operation {
            "split" | "toLowerCase" | "indexOf" | "replace" => operation,
            "includes" | "label" | "accessibility" => "includes",
            _ => "trim",
        };
        let result = self.read.borrow_mut().require_string(
            &value,
            LiveStringUse {
                method,
                expression_receiver: "e",
            },
        );
        self.utf8(self.result(result)?)
    }
    fn join(&self, atom: &Atom, separator: &str) -> Result<String, PeleError> {
        self.array_length(atom)?;
        let value = self.value(atom)?;
        let result = self.read.borrow_mut().array_join(&value, separator);
        self.utf8(self.result(result)?)
    }
    fn truthy(&self, atom: &Atom) -> Result<bool, PeleError> {
        Ok(self.read.borrow().truthy(&self.value(atom)?))
    }
    fn charge(&self, work: usize, units: usize) -> Result<(), PeleError> {
        let work = self.work.get().saturating_add(work);
        let units = self.units.get().saturating_add(units);
        self.work.set(work);
        self.units.set(units);
        if work > 1_000_000 || units.saturating_mul(3) > 128 * 1024 * 1024 {
            Err(limit())
        } else {
            Ok(())
        }
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    seed: &ErDetachedInput,
    config: &Object,
    settings: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    if let Some(e) = &seed.limit {
        return Err(e.clone().into());
    }
    let root = read.root();
    let mut store = Store {
        read: RefCell::new(read),
        ledger: RefCell::new(Ledger {
            values: vec![],
            keys: HashMap::new(),
            units: 0,
        }),
        root: Atom::Undefined,
        original: RefCell::new(None),
        work: Cell::new(0),
        units: Cell::new(0),
    };
    store.root = match store.bind(root) {
        Ok(root) => root,
        Err(e) => return Err(store.original.take().unwrap_or_else(|| e.into())),
    };
    let result = super::live_read::render_owned_store(&store, config, settings, resources);
    if let Some(original) = store.original.take() {
        return Err(original);
    }
    result.map_err(Into::into)
}
