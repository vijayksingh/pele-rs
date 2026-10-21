//! Owned C jobs: no document or binding borrow crosses a resource callback.
mod call;
mod iterator;
use super::{
    host_functions, public, PeleHostResourcesV3, PelePublicDocument, PeleRenderNumbersV9,
    PeleResult,
};
use pele::compat::{
    public::{
        Atom, DetachedRenderCompletion, DetachedRenderJob, DetachedRenderRequest,
        DetachedRenderValue, GraphNodeKind, JavaScriptErrorKind, JsString, LiveRenderRead,
        LiveStringUse, LiveValueKind, PublicError, PublicGraphRead,
    },
    ErrorCode, PeleError,
};
use std::{
    collections::HashMap,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
enum Value {
    Atom(Atom),
    Iterator(Arc<Mutex<iterator::Cursor>>),
    String(Arc<JsString>),
    Sequence(Arc<[Value]>),
    Object(Arc<[(Arc<JsString>, Value)]>),
}
fn invalid(message: &str) -> PeleError {
    public::invalid(message)
}
fn type_error(message: String) -> PublicError {
    PublicError::JavaScript {
        name: JavaScriptErrorKind::TypeError,
        message: JsString::from(message.as_str()),
    }
}
fn string(reader: &dyn PublicGraphRead, id: u32) -> Result<JsString, PeleError> {
    let length = reader.string_length(id as usize)?;
    if length > 16_000_000 {
        return Err(PeleError::new(
            "Public model graph exceeds work budget",
            ErrorCode::Limit,
        ));
    }
    let mut units = Vec::with_capacity(length);
    reader.string_chunks(id as usize, &mut |chunk| {
        if chunk.len() > length - units.len() {
            return Err(invalid("Invalid captured string length"));
        }
        units.extend_from_slice(chunk);
        Ok(())
    })?;
    if units.len() != length {
        return Err(invalid("Invalid captured string length"));
    }
    Ok(JsString(units))
}
fn own(reader: &dyn PublicGraphRead, atom: Atom) -> Result<Value, PeleError> {
    match atom {
        Atom::String(id) => Ok(Value::String(Arc::new(string(reader, id)?))),
        atom => Ok(Value::Atom(atom)),
    }
}
fn capture_text(
    read: &dyn PublicGraphRead,
    id: u32,
    strings: &mut HashMap<u32, Arc<JsString>>,
) -> Result<Arc<JsString>, PeleError> {
    if let Some(text) = strings.get(&id) {
        return Ok(Arc::clone(text));
    }
    let text = Arc::new(string(read, id)?);
    strings.insert(id, Arc::clone(&text));
    Ok(text)
}
fn capture_value(
    read: &dyn PublicGraphRead,
    atom: &Atom,
    strings: &mut HashMap<u32, Arc<JsString>>,
) -> Result<Value, PeleError> {
    Ok(match atom {
        Atom::String(id) => Value::String(capture_text(read, *id, strings)?),
        atom => Value::Atom(atom.clone()),
    })
}
struct Reader {
    access: call::Access,
    iterator_handles: usize,
    iterator_advances: usize,
    owner: *const PelePublicDocument,
    root: Value,
    // Retain the allocation alongside its token: a dropped temporary's address
    // must never be reused as another object's identity during the same job.
    identities: HashMap<(u8, usize), (u64, Value)>,
}
impl Reader {
    fn field(&self, receiver: &Value, key: &str) -> Result<Value, PublicError> {
        if let Value::Object(properties) = receiver {
            return Ok(properties
                .iter()
                .find(|(name, _)| name.is(key))
                .map_or(Value::Atom(Atom::Undefined), |(_, value)| value.clone()));
        }
        if let Value::Sequence(values) = receiver {
            if key == "length" {
                return Ok(Value::Atom(Atom::Number(values.len() as f64)));
            }
            if let Ok(index) = key.parse::<usize>() {
                if index.to_string() == key {
                    return Ok(values
                        .get(index)
                        .cloned()
                        .unwrap_or(Value::Atom(Atom::Undefined)));
                }
            }
            return Ok(Value::Atom(Atom::Undefined));
        }
        if let Value::String(text) = receiver {
            if key == "length" {
                return Ok(Value::Atom(Atom::Number(text.0.len() as f64)));
            }
            if let Ok(index) = key.parse::<usize>() {
                if index.to_string() == key {
                    return Ok(text.0.get(index).map_or(Value::Atom(Atom::Undefined), |v| {
                        Value::String(Arc::new(JsString(vec![*v])))
                    }));
                }
            }
        }
        let id = match receiver {
            Value::Atom(Atom::Ref(id)) => *id as usize,
            Value::Atom(Atom::Undefined | Atom::Null) => {
                return Err(type_error(format!(
                    "Cannot read properties of {} (reading '{key}')",
                    if matches!(receiver, Value::Atom(Atom::Null)) {
                        "null"
                    } else {
                        "undefined"
                    }
                )))
            }
            _ => return Ok(Value::Atom(Atom::Undefined)),
        };
        // Native IDs are current graph coordinates, not JavaScript object leases.
        let document = unsafe { public::document(self.owner) }?;
        let graph = document.snapshot_read()?;
        let read = graph.reader();
        let header = read.header(id)?;
        let mut start = 0;
        while start < header.properties {
            let mut found = None;
            let mut count = 0;
            read.properties(id, start, &mut |properties| {
                count = properties.len();
                for property in properties {
                    if string(read, property.key)?.is(key) {
                        found = Some(own(read, property.value.clone())?);
                        break;
                    }
                }
                Ok(())
            })?;
            if let Some(value) = found {
                return Ok(value);
            }
            if count == 0 || count > header.properties - start {
                return Err(invalid("Invalid captured property batch").into());
            }
            start += count;
        }
        if key == "size" {
            if let GraphNodeKind::Map(count) = header.kind {
                return Ok(Value::Atom(Atom::Number(count as f64)));
            }
        }
        if key == "length" {
            if let GraphNodeKind::Array(length) = header.kind {
                return Ok(Value::Atom(Atom::Number(f64::from(length))));
            }
        }
        if let GraphNodeKind::Instance(mut prototype) = header.kind {
            let schema = document.model_schema();
            for _ in 0..schema.prototypes.len() {
                let Some(descriptor) = schema.prototype(prototype) else {
                    break;
                };
                if let Some(member) = descriptor.members.iter().find(|m| m.name == key) {
                    return match member.member {
                        pele::compat::public::MemberKind::Method { method } => {
                            Ok(Value::Atom(Atom::BuiltinFunction(method.0)))
                        }
                        _ => Err(invalid("Native live accessor dispatch is unavailable").into()),
                    };
                }
                match descriptor.parent {
                    pele::compat::public::PrototypeParent::Prototype(parent) => prototype = parent,
                    _ => break,
                }
            }
        }
        Ok(Value::Atom(Atom::Undefined))
    }
}
impl LiveRenderRead for Reader {
    type Value = Value;
    fn call(
        &mut self,
        function: &Value,
        receiver: &Value,
        arguments: &[pele::compat::public::LiveCallArgument<Value>],
        resources: &mut dyn pele::compat::RenderResources,
        not_callable_error: &'static str,
    ) -> Result<Value, PublicError> {
        if arguments.len() > 1_000_000 {
            return Err(
                PeleError::new("Live call argument budget exceeded", ErrorCode::Limit).into(),
            );
        }
        let units = arguments.iter().fold(0usize, |total, value| {
            total.saturating_add(match value {
                pele::compat::public::LiveCallArgument::Text(s) => s.0.len(),
                pele::compat::public::LiveCallArgument::Value(Value::String(s)) => s.0.len(),
                _ => 0,
            })
        });
        if units > 16_000_000 {
            return Err(PeleError::new("Live call text budget exceeded", ErrorCode::Limit).into());
        }
        let mut args = Vec::with_capacity(arguments.len());
        for argument in arguments {
            args.push(match argument {
                pele::compat::public::LiveCallArgument::Value(v) => v.clone(),
                pele::compat::public::LiveCallArgument::Text(s) => {
                    Value::String(Arc::new(s.clone()))
                }
                pele::compat::public::LiveCallArgument::Number(value) => {
                    Value::Atom(Atom::Number(*value))
                }
                pele::compat::public::LiveCallArgument::Date(value) => {
                    self.date_argument(*value)?
                }
            });
        }
        self.call_owned(function, receiver, &args, resources, not_callable_error)
    }
    fn to_number(
        &mut self,
        value: &Value,
        _: &mut dyn pele::compat::RenderResources,
    ) -> Result<f64, PublicError> {
        match value {
            Value::Atom(Atom::Undefined) => Ok(f64::NAN),
            Value::Atom(Atom::Null) => Ok(0.),
            Value::Atom(Atom::Bool(v)) => Ok(u8::from(*v) as f64),
            Value::Atom(Atom::Number(v)) => Ok(*v),
            Value::String(v) => Ok(super::constructor_host::string_number(&v.0)),
            Value::Atom(Atom::Ref(id)) => {
                let document = unsafe { public::document(self.owner) }?;
                let graph = document.snapshot_read()?;
                let reader = graph.reader();
                let header = reader.header(*id as usize)?;
                let GraphNodeKind::Date(bits) = header.kind else {
                    return Err(invalid(
                        "Native numeric coercion supports ordinary Dates and scalars",
                    )
                    .into());
                };
                let mut start = 0;
                while start < header.properties {
                    let mut count = 0;
                    reader.properties(*id as usize, start, &mut |properties| {
                        count = properties.len();
                        for property in properties {
                            let key = string(reader, property.key)?;
                            if key.is("valueOf") || key.is("toString") {
                                return Err(invalid("Native numeric coercion does not support overridden Date methods"));
                            }
                        }
                        Ok(())
                    })?;
                    if count == 0 || count > header.properties - start {
                        return Err(invalid("Invalid captured Date property batch").into());
                    }
                    start += count;
                }
                Ok(f64::from_bits(bits))
            }
            _ => Err(invalid("Native numeric coercion supports ordinary Dates and scalars").into()),
        }
    }
    fn iterator_begin(&mut self, v: &Value) -> Result<Value, PublicError> {
        self.begin_iterator(v)
    }
    fn iterator_next(&mut self, v: &Value) -> Result<Option<Value>, PublicError> {
        self.next_iterator(v)
    }
    fn iterator_close(&mut self, v: &Value) -> Result<(), PublicError> {
        self.close_iterator(v)
    }
    fn retained_graph_value(&mut self, node: u32) -> Result<Value, PublicError> {
        let document = unsafe { public::document(self.owner) }?;
        let graph = document.snapshot_read()?;
        graph.reader().header(node as usize)?;
        Ok(Value::Atom(Atom::Ref(node)))
    }
    fn root(&self) -> Value {
        self.root.clone()
    }
    fn get(&mut self, value: &Value, key: &str) -> Result<Value, PublicError> {
        self.field(value, key)
    }
    fn index(&mut self, value: &Value, index: usize) -> Result<Value, PublicError> {
        self.field(value, &index.to_string())
    }
    fn map_get_text(&mut self, value: &Value, key: &str) -> Result<Value, PublicError> {
        self.map_get(value, &Value::String(Arc::new(JsString::from(key))))
    }
    fn array_join(&mut self, value: &Value, separator: &str) -> Result<JsString, PublicError> {
        let length = match value {
            Value::Sequence(values) => values.len(),
            Value::Atom(Atom::Ref(id)) => {
                // The guard ends before scalar conversion/index operations.
                let document = unsafe { public::document(self.owner) }?;
                let graph = document.snapshot_read()?;
                let read = graph.reader();
                let header = read.header(*id as usize)?;
                let GraphNodeKind::Array(length) = header.kind else {
                    return Err(type_error("value.join is not a function".into()));
                };
                let mut start = 0;
                while start < header.properties {
                    let mut count = 0;
                    let mut shadowed = false;
                    read.properties(*id as usize, start, &mut |properties| {
                        count = properties.len();
                        for property in properties {
                            if string(read, property.key)?.is("join") {
                                shadowed = true;
                                break;
                            }
                        }
                        Ok(())
                    })?;
                    if shadowed {
                        return Err(
                            invalid("Native live Array join override is not implemented").into(),
                        );
                    }
                    if count == 0 || count > header.properties - start {
                        return Err(invalid("Invalid captured property batch").into());
                    }
                    start += count;
                }
                length as usize
            }
            _ => return Err(type_error("value.join is not a function".into())),
        };
        if length > 1_000_000 || separator.len() > 16_000_000 {
            return Err(
                PeleError::new("Live array join exceeds work budget", ErrorCode::Limit).into(),
            );
        }
        let separator = JsString::from(separator);
        let mut units = Vec::new();
        for index in 0..length {
            let value = self.index(value, index)?;
            let text = if matches!(value, Value::Atom(Atom::Undefined | Atom::Null)) {
                JsString(vec![])
            } else {
                self.to_string(&value)?
            };
            let sep = if index == 0 { 0 } else { separator.0.len() };
            if units.len().saturating_add(sep).saturating_add(text.0.len()) > 16_000_000 {
                return Err(PeleError::new(
                    "Live array join exceeds work budget",
                    ErrorCode::Limit,
                )
                .into());
            }
            if index != 0 {
                units.extend_from_slice(&separator.0);
            }
            units.extend_from_slice(&text.0);
        }
        Ok(JsString(units))
    }
    fn object_identity(&mut self, value: &Value) -> Result<Option<u64>, PublicError> {
        let key = match value {
            Value::Atom(Atom::Ref(id)) => return Ok(Some(u64::from(*id))),
            Value::Atom(Atom::BuiltinFunction(id)) => {
                return Ok(Some((1u64 << 32) + u64::from(*id)))
            }
            Value::Atom(Atom::HostReference(id)) => return Ok(Some((2u64 << 32) + u64::from(*id))),
            Value::Sequence(values) => (0, Arc::as_ptr(values) as *const () as usize),
            Value::Object(values) => (1, Arc::as_ptr(values) as *const () as usize),
            Value::Iterator(value) => (2, Arc::as_ptr(value) as usize),
            _ => return Ok(None),
        };
        if let Some((token, _)) = self.identities.get(&key) {
            return Ok(Some(*token));
        }
        if self.identities.len() >= 1_000_000 {
            return Err(PeleError::new(
                "Public model graph exceeds identity budget",
                ErrorCode::Limit,
            )
            .into());
        }
        let token = (3u64 << 32) + self.identities.len() as u64;
        self.identities.insert(key, (token, value.clone()));
        Ok(Some(token))
    }
    fn has_index(&mut self, receiver: &Value, index: usize) -> Result<bool, PublicError> {
        let key = index.to_string();
        match receiver {
            Value::Sequence(values) => return Ok(index < values.len()),
            Value::Object(properties) => {
                return Ok(properties.iter().any(|(name, _)| name.is(&key)))
            }
            Value::Atom(Atom::Ref(_)) => {}
            _ => {
                return Err(type_error(format!(
                    "Cannot use 'in' operator to search for '{index}' in a primitive value"
                )))
            }
        }
        let Value::Atom(Atom::Ref(id)) = receiver else {
            unreachable!()
        };
        let document = unsafe { public::document(self.owner) }?;
        let graph = document.snapshot_read()?;
        let read = graph.reader();
        let header = read.header(*id as usize)?;
        let mut start = 0;
        while start < header.properties {
            let mut found = false;
            let mut count = 0;
            read.properties(*id as usize, start, &mut |properties| {
                count = properties.len();
                for property in properties {
                    if string(read, property.key)?.is(&key) {
                        found = true;
                        break;
                    }
                }
                Ok(())
            })?;
            if found {
                return Ok(true);
            }
            if count == 0 || count > header.properties - start {
                return Err(invalid("Invalid captured property batch").into());
            }
            start += count;
        }
        // Native declared prototypes contain named methods, never indexed data.
        Ok(false)
    }
    fn shallow_copy(&mut self, value: &Value) -> Result<Value, PublicError> {
        let mut fields = Vec::new();
        match value {
            Value::Object(properties) => return Ok(Value::Object(Arc::from(properties.to_vec()))),
            Value::Sequence(values) => {
                for (index, value) in values.iter().enumerate() {
                    fields.push((
                        Arc::new(JsString::from(index.to_string().as_str())),
                        value.clone(),
                    ));
                }
            }
            Value::String(text) => {
                if text.0.len() > 8_000_000 {
                    return Err(PeleError::new(
                        "Public model graph exceeds work budget",
                        ErrorCode::Limit,
                    )
                    .into());
                }
                for (index, unit) in text.0.iter().enumerate() {
                    fields.push((
                        Arc::new(JsString::from(index.to_string().as_str())),
                        Value::String(Arc::new(JsString(vec![*unit]))),
                    ));
                }
            }
            Value::Atom(Atom::Ref(id)) => {
                let document = unsafe { public::document(self.owner) }?;
                let graph = document.snapshot_read()?;
                let read = graph.reader();
                let header = read.header(*id as usize)?;
                let mut strings = HashMap::<u32, Arc<JsString>>::new();
                let mut start = 0;
                while start < header.properties {
                    let mut count = 0;
                    read.properties(*id as usize, start, &mut |properties| {
                        count = properties.len();
                        if count == 0 || count > header.properties - start {
                            return Err(invalid("Invalid captured property batch"));
                        }
                        for property in properties.iter().filter(|p| p.flags & 1 != 0) {
                            let name = capture_text(read, property.key, &mut strings)?;
                            let value = capture_value(read, &property.value, &mut strings)?;
                            fields.push((name, value));
                        }
                        Ok(())
                    })?;
                    start += count;
                }
            }
            // Object spread boxes other primitives, which own no enumerable keys.
            _ => {}
        }
        Ok(Value::Object(Arc::from(fields)))
    }
    fn map_get(&mut self, value: &Value, key: &Value) -> Result<Value, PublicError> {
        let Value::Atom(Atom::Ref(id)) = value else {
            return Err(type_error("value.get is not a function".into()));
        };
        let document = unsafe { public::document(self.owner) }?;
        let graph = document.snapshot_read()?;
        let read = graph.reader();
        let header = read.header(*id as usize)?;
        let GraphNodeKind::Map(items) = header.kind else {
            return Err(type_error("value.get is not a function".into()));
        };
        let mut start = 0;
        while start < header.properties {
            let mut count = 0;
            let mut shadowed = false;
            read.properties(*id as usize, start, &mut |properties| {
                count = properties.len();
                for property in properties {
                    if string(read, property.key)?.is("get") {
                        shadowed = true;
                        break;
                    }
                }
                Ok(())
            })?;
            if shadowed {
                return Err(invalid("Native live Map get override is not implemented").into());
            }
            if count == 0 || count > header.properties - start {
                return Err(invalid("Invalid captured property batch").into());
            }
            start += count;
        }
        let mut start = 0;
        while start < items {
            let mut count = 0;
            let mut found = None;
            read.map_entries(*id as usize, start, &mut |entries| {
                count = entries.len();
                for (entry, value) in entries {
                    let matches = match (entry, key) {
                        (Atom::String(id), Value::String(text)) => string(read, *id)? == **text,
                        (Atom::Number(a), Value::Atom(Atom::Number(b))) => {
                            a == b || (a.is_nan() && b.is_nan())
                        }
                        (Atom::Bool(a), Value::Atom(Atom::Bool(b))) => a == b,
                        (Atom::Ref(a), Value::Atom(Atom::Ref(b)))
                        | (Atom::BuiltinFunction(a), Value::Atom(Atom::BuiltinFunction(b)))
                        | (Atom::HostReference(a), Value::Atom(Atom::HostReference(b))) => a == b,
                        (Atom::Undefined, Value::Atom(Atom::Undefined))
                        | (Atom::Null, Value::Atom(Atom::Null)) => true,
                        _ => false,
                    };
                    if matches {
                        found = Some(own(read, value.clone())?);
                        break;
                    }
                }
                Ok(())
            })?;
            if let Some(found) = found {
                return Ok(found);
            }
            if count == 0 || count > items - start {
                return Err(invalid("Invalid captured Map batch").into());
            }
            start += count;
        }
        Ok(Value::Atom(Atom::Undefined))
    }
    fn snapshot_iterable(
        &mut self,
        value: &Value,
        max_count: usize,
    ) -> Result<Vec<Value>, PublicError> {
        if let Value::Sequence(values) = value {
            if values.len() > max_count.min(8_000_000) {
                return Err(PeleError::new(
                    "Public model graph exceeds work budget",
                    ErrorCode::Limit,
                )
                .into());
            }
            return Ok(values.to_vec());
        }
        let Value::Atom(Atom::Ref(id)) = value else {
            return Err(invalid("Native live iterable must be an Array or Map").into());
        };
        // One short immutable snapshot. Own text before releasing it; pairs are
        // new identities just like ordinary Map iteration, never graph nodes.
        let document = unsafe { public::document(self.owner) }?;
        let graph = document.snapshot_read()?;
        let read = graph.reader();
        let header = read.header(*id as usize)?;
        if let GraphNodeKind::Array(length) = header.kind {
            let length = length as usize;
            if length > max_count.min(8_000_000) {
                return Err(PeleError::new(
                    "Public model graph exceeds work budget",
                    ErrorCode::Limit,
                )
                .into());
            }
            let mut strings = HashMap::<u32, Arc<JsString>>::new();
            let mut values = vec![Value::Atom(Atom::Undefined); length];
            let mut start = 0;
            while start < header.properties {
                let mut count = 0;
                read.properties(*id as usize, start, &mut |properties| {
                    count = properties.len();
                    for property in properties {
                        let key = string(read, property.key)?;
                        if let Some(index) = key.well_formed().and_then(|s| {
                            s.parse::<usize>()
                                .ok()
                                .filter(|i| *i < length && i.to_string() == s)
                        }) {
                            values[index] = capture_value(read, &property.value, &mut strings)?;
                        }
                    }
                    Ok(())
                })?;
                if count == 0 || count > header.properties - start {
                    return Err(invalid("Invalid captured property batch").into());
                }
                start += count;
            }
            return Ok(values);
        }
        let GraphNodeKind::Map(items) = header.kind else {
            return Err(invalid("Native live iterable must be an Array or Map").into());
        };
        if items > max_count.min(8_000_000) {
            return Err(
                PeleError::new("Public model graph exceeds work budget", ErrorCode::Limit).into(),
            );
        }
        // Retained strings belong to this snapshot. Repeated catalog IDs must
        // share ownership: copying each occurrence multiplies admitted text by
        // the number of Map entries before the renderer can reject a value.
        let mut strings = HashMap::<u32, Arc<JsString>>::new();
        let mut values = Vec::with_capacity(items);
        let mut start = 0;
        while start < items {
            let mut count = 0;
            read.map_entries(*id as usize, start, &mut |entries| {
                count = entries.len();
                if count == 0 || count > items - start {
                    return Err(invalid("Invalid captured Map batch"));
                }
                for (key, value) in entries {
                    values.push(Value::Sequence(Arc::from([
                        capture_value(read, key, &mut strings)?,
                        capture_value(read, value, &mut strings)?,
                    ])));
                }
                Ok(())
            })?;
            start += count;
        }
        Ok(values)
    }
    fn snapshot_map_values(
        &mut self,
        value: &Value,
        max_count: usize,
    ) -> Result<Vec<Value>, PublicError> {
        let Value::Atom(Atom::Ref(id)) = value else {
            return Err(type_error("value.values is not a function".into()));
        };
        let document = unsafe { public::document(self.owner) }?;
        let graph = document.snapshot_read()?;
        let read = graph.reader();
        let header = read.header(*id as usize)?;
        let GraphNodeKind::Map(items) = header.kind else {
            return Err(type_error("value.values is not a function".into()));
        };
        let mut start = 0;
        while start < header.properties {
            let mut count = 0;
            read.properties(*id as usize, start, &mut |properties| {
                count = properties.len();
                if count == 0 || count > header.properties - start {
                    return Err(invalid("Invalid captured property batch"));
                }
                for p in properties {
                    if string(read, p.key)?.is("values") {
                        return Err(invalid(
                            "Native live Map values override is not implemented",
                        ));
                    }
                }
                Ok(())
            })?;
            start += count;
        }
        if items > max_count.min(8_000_000) {
            return Err(
                PeleError::new("Public model graph exceeds work budget", ErrorCode::Limit).into(),
            );
        }
        let mut strings = HashMap::<u32, Arc<JsString>>::new();
        let mut values = Vec::with_capacity(items);
        let mut start = 0;
        while start < items {
            let mut count = 0;
            read.map_entries(*id as usize, start, &mut |entries| {
                count = entries.len();
                if count == 0 || count > items - start {
                    return Err(invalid("Invalid captured Map batch"));
                }
                for (_, value) in entries {
                    values.push(capture_value(read, value, &mut strings)?);
                }
                Ok(())
            })?;
            start += count;
        }
        Ok(values)
    }
    fn is_array(&mut self, value: &Value) -> Result<bool, PublicError> {
        match value {
            Value::Sequence(_) => Ok(true),
            Value::Atom(Atom::Ref(id)) => {
                let document = unsafe { public::document(self.owner) }?;
                let graph = document.snapshot_read()?;
                Ok(matches!(
                    graph.reader().header(*id as usize)?.kind,
                    GraphNodeKind::Array(_)
                ))
            }
            _ => Ok(false),
        }
    }
    fn own_keys(&mut self, value: &Value) -> Result<Vec<JsString>, PublicError> {
        let mut keys = Vec::<Arc<JsString>>::new();
        let mut seen = std::collections::HashSet::<Arc<JsString>>::new();
        let mut units = 0usize;
        let mut add = |key: Arc<JsString>| -> Result<(), PeleError> {
            if seen.contains(&key) {
                return Ok(());
            }
            if keys.len() == 1_000_000 || key.0.len() > 16_000_000 - units {
                return Err(PeleError::new(
                    "Public model graph exceeds work budget",
                    ErrorCode::Limit,
                ));
            }
            units += key.0.len();
            seen.insert(Arc::clone(&key));
            keys.push(key);
            Ok(())
        };
        match value {
            Value::Atom(Atom::Undefined | Atom::Null) => {
                return Err(type_error(
                    "Cannot convert undefined or null to object".into(),
                ))
            }
            Value::Object(properties) => {
                for (key, _) in properties.iter() {
                    add(Arc::clone(key))?;
                }
            }
            Value::Sequence(values) => {
                if values.len() > 1_000_000 {
                    return Err(PeleError::new(
                        "Public model graph exceeds work budget",
                        ErrorCode::Limit,
                    )
                    .into());
                }
                for index in 0..values.len() {
                    add(Arc::new(JsString::from(index.to_string().as_str())))?;
                }
            }
            Value::String(text) => {
                if text.0.len() > 1_000_000 {
                    return Err(PeleError::new(
                        "Public model graph exceeds work budget",
                        ErrorCode::Limit,
                    )
                    .into());
                }
                for index in 0..text.0.len() {
                    add(Arc::new(JsString::from(index.to_string().as_str())))?;
                }
            }
            Value::Atom(Atom::Ref(id)) => {
                let document = unsafe { public::document(self.owner) }?;
                let graph = document.snapshot_read()?;
                let read = graph.reader();
                let header = read.header(*id as usize)?;
                let mut start = 0;
                let mut strings = HashMap::new();
                while start < header.properties {
                    let mut count = 0;
                    read.properties(*id as usize, start, &mut |properties| {
                        count = properties.len();
                        if count == 0 || count > header.properties - start {
                            return Err(invalid("Invalid captured property batch"));
                        }
                        for p in properties.iter().filter(|p| p.flags & 1 != 0) {
                            add(capture_text(read, p.key, &mut strings)?)?;
                        }
                        Ok(())
                    })?;
                    start += count;
                }
            }
            _ => {}
        }
        fn index(key: &JsString) -> Option<u32> {
            if key.0.is_empty() || key.0.len() > 10 || (key.0.len() > 1 && key.0[0] == 48) {
                return None;
            }
            let mut value = 0u32;
            for unit in &key.0 {
                if !(48..=57).contains(unit) {
                    return None;
                }
                value = value.checked_mul(10)?.checked_add(u32::from(*unit - 48))?;
            }
            (value < u32::MAX).then_some(value)
        }
        // Stable sort: array-index keys first, other names retain property order.
        keys.sort_by_key(|key| index(key).map_or((1, 0), |index| (0, index)));
        Ok(keys.into_iter().map(|key| key.as_ref().clone()).collect())
    }
    fn kind(&self, value: &Value) -> LiveValueKind {
        match value {
            Value::String(_) => LiveValueKind::String,
            Value::Sequence(_) | Value::Object(_) | Value::Iterator(_) => LiveValueKind::Object,
            Value::Atom(atom) => match atom {
                Atom::Undefined => LiveValueKind::Undefined,
                Atom::Null => LiveValueKind::Null,
                Atom::Bool(_) => LiveValueKind::Boolean,
                Atom::Number(_) => LiveValueKind::Number,
                Atom::BuiltinFunction(_) => LiveValueKind::Function,
                _ => LiveValueKind::Object,
            },
        }
    }
    fn number(&self, value: &Value) -> Option<f64> {
        if let Value::Atom(Atom::Number(n)) = value {
            Some(*n)
        } else {
            None
        }
    }
    fn truthy(&self, value: &Value) -> bool {
        match value {
            Value::String(s) => !s.0.is_empty(),
            Value::Atom(Atom::Undefined | Atom::Null) => false,
            Value::Atom(Atom::Bool(b)) => *b,
            Value::Atom(Atom::Number(n)) => *n != 0. && !n.is_nan(),
            _ => true,
        }
    }
    fn string_units(&self, value: &Value) -> Result<Option<JsString>, PublicError> {
        Ok(if let Value::String(s) = value {
            Some(s.as_ref().clone())
        } else {
            None
        })
    }
    fn to_string(&mut self, value: &Value) -> Result<JsString, PublicError> {
        if let Value::String(s) = value {
            return Ok(s.as_ref().clone());
        }
        let text = match value {
            Value::Atom(Atom::Undefined) => "undefined".into(),
            Value::Atom(Atom::Null) => "null".into(),
            Value::Atom(Atom::Bool(b)) => b.to_string(),
            Value::Atom(Atom::Number(n)) => pele::compat::public::live_number_string(*n),
            _ => {
                return Err(invalid("Native live object string coercion is not implemented").into())
            }
        };
        Ok(JsString::from(text.as_str()))
    }
    fn require_string(
        &mut self,
        value: &Value,
        site: LiveStringUse,
    ) -> Result<JsString, PublicError> {
        if let Value::String(s) = value {
            return Ok(s.as_ref().clone());
        }
        match value {
            Value::Atom(Atom::Undefined | Atom::Null) => Err(type_error(format!(
                "Cannot read properties of {} (reading '{}')",
                if matches!(value, Value::Atom(Atom::Null)) {
                    "null"
                } else {
                    "undefined"
                },
                site.method
            ))),
            _ => Err(type_error(format!(
                "{}.{} is not a function",
                site.expression_receiver, site.method
            ))),
        }
    }
}
enum State {
    Ready(DetachedRenderJob),
    Running,
    Complete(DetachedRenderCompletion),
    Finished,
}
/// Owner must outlive this job; free only after run/finish calls return.
pub struct PeleDetachedRenderJobV15 {
    owner: *mut PelePublicDocument,
    root: Value,
    state: Mutex<State>,
}
fn prepare(
    owner: *mut PelePublicDocument,
    revision: u64,
    request: DetachedRenderRequest,
    out: *mut *mut PeleDetachedRenderJobV15,
) -> Result<PeleResult, PeleError> {
    let out = unsafe { out.as_mut() }.ok_or_else(|| invalid("Null detached job output"))?;
    let mut document = unsafe { public::document(owner) }?;
    let Some(job) = document.prepare_detached_render(revision, request)? else {
        return public::success(serde_json::json!({"selected":false}));
    };
    let graph = document.snapshot_read()?;
    let root = own(graph.reader(), graph.reader().root())?;
    let result = public::success(serde_json::json!({"selected":true}))?;
    *out = Box::into_raw(Box::new(PeleDetachedRenderJobV15 {
        owner,
        root,
        state: Mutex::new(State::Ready(job)),
    }));
    Ok(result)
}
/// # Safety
/// Same input bounds as v9. Owner outlives the returned job and every callback.
#[no_mangle]
pub unsafe extern "C" fn pele_public_prepare_detached_render_v15(
    owner: *mut PelePublicDocument,
    revision: u64,
    bytes: *const u8,
    len: usize,
    numbers: *const PeleRenderNumbersV9,
    kind: u32,
    bits: u64,
    out: *mut *mut PeleDetachedRenderJobV15,
) -> *mut PeleResult {
    if !out.is_null() {
        unsafe { *out = ptr::null_mut() }
    }
    public::bounded_call(|| {
        if !unsafe { public::document(owner) }?.has_detached_public_render() {
            return public::success(serde_json::json!({"selected":false}));
        }
        let (options, limit) =
            unsafe { super::render_numbers::options(bytes, len, numbers, kind, bits) }?;
        prepare(
            owner,
            revision,
            DetachedRenderRequest::Normal {
                options,
                limit,
                enforce_native_bytes: true,
            },
            out,
        )
    })
}
/// # Safety
/// Same input bounds as v13. Owner outlives the returned job and every callback.
#[no_mangle]
pub unsafe extern "C" fn pele_public_prepare_detached_raw_v15(
    owner: *mut PelePublicDocument,
    revision: u64,
    config: *const u8,
    config_len: usize,
    bytes: *const u8,
    len: usize,
    numbers: *const PeleRenderNumbersV9,
    out: *mut *mut PeleDetachedRenderJobV15,
) -> *mut PeleResult {
    if !out.is_null() {
        unsafe { *out = ptr::null_mut() }
    }
    public::bounded_call(|| {
        if !unsafe { public::document(owner) }?.has_detached_public_render() {
            return public::success(serde_json::json!({"selected":false}));
        }
        let config = unsafe { super::prepared::config(config, config_len) }?;
        let settings = unsafe { super::render_numbers::raw_options(bytes, len, numbers) }?;
        prepare(
            owner,
            revision,
            DetachedRenderRequest::Raw { config, settings },
            out,
        )
    })
}
/// # Safety
/// Job/owner/resources remain live synchronously. No concurrent free or use.
#[no_mangle]
pub unsafe extern "C" fn pele_detached_render_run_v15(
    job: *mut PeleDetachedRenderJobV15,
    resources: *const PeleHostResourcesV3,
) -> *mut PeleResult {
    public::bounded_call(|| {
        let job = unsafe { job.as_ref() }.ok_or_else(|| invalid("Null detached job"))?;
        // Validate resource setup before consuming the job; no callbacks occur here.
        unsafe {
            host_functions::with_resources(job.owner, resources, |host| {
                let inner = {
                    let mut state = job
                        .state
                        .try_lock()
                        .map_err(|_| invalid("Detached job is already in use"))?;
                    if !matches!(*state, State::Ready(_)) {
                        return Err(invalid("Public render job already dispatched"));
                    }
                    match std::mem::replace(&mut *state, State::Running) {
                        State::Ready(value) => value,
                        _ => unreachable!(),
                    }
                };
                let mut reader = Reader {
                    owner: job.owner,
                    root: job.root.clone(),
                    identities: HashMap::new(),
                    access: call::Access::default(),
                    iterator_handles: 0,
                    iterator_advances: 0,
                };
                let completion = inner.run(&mut reader, host);
                *job.state
                    .lock()
                    .map_err(|_| invalid("Detached job is unavailable"))? =
                    State::Complete(completion);
                public::success(serde_json::json!({"completed":true}))
            })
        }
    })
}
/// # Safety
/// Owner/job are live. out_revision writable. Finish consumes the completion;
/// out_revision acknowledges current committed state even if export fails.
#[no_mangle]
pub unsafe extern "C" fn pele_public_finish_detached_render_v15(
    owner: *mut PelePublicDocument,
    job: *mut PeleDetachedRenderJobV15,
    out_revision: *mut u64,
) -> *mut PeleResult {
    public::bounded_call(|| {
        let revision = unsafe { out_revision.as_mut() }
            .ok_or_else(|| invalid("Null detached revision output"))?;
        let job = unsafe { job.as_ref() }.ok_or_else(|| invalid("Null detached job"))?;
        if job.owner != owner {
            return Err(invalid("Public render job belongs to another document"));
        }
        let completion = {
            let mut state = job
                .state
                .try_lock()
                .map_err(|_| invalid("Detached job is already in use"))?;
            if !matches!(*state, State::Complete(_)) {
                return Err(invalid("Public render job has no completion"));
            }
            match std::mem::replace(&mut *state, State::Finished) {
                State::Complete(value) => value,
                _ => unreachable!(),
            }
        };
        let mut document = unsafe { public::document(owner) }?;
        let kind = document.diagram_type().clone();
        let result = (|| {
            let outcome = document.finish_detached_render(completion)?;
            let result = outcome.result.map(|value| match value {
                DetachedRenderValue::Normal(value) => value,
                DetachedRenderValue::Raw(rendered) => pele::compat::RenderResult {
                    diagram_type: kind,
                    rendered,
                },
            });
            public::binary_metadata(
                outcome.revision,
                &outcome.graph,
                &result.as_ref().map(pele::compat::render_result_transport),
            )
        })();
        *revision = document.revision();
        result
    })
}
/// # Safety
/// Free exactly once after all calls/callbacks return; owner remains live.
#[no_mangle]
pub unsafe extern "C" fn pele_detached_render_job_free_v15(job: *mut PeleDetachedRenderJobV15) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if !job.is_null() {
            drop(unsafe { Box::from_raw(job) })
        }
    }));
}
