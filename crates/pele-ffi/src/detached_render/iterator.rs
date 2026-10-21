//! Ordinary native iteration retains coordinates, never a document guard.
//! A final graph does not encode JS Map tombstones: deletion/reinsertion history
//! is intentionally not reconstructed from a later snapshot.
use super::{
    capture_value, invalid, public, Arc, Atom, ErrorCode, GraphNodeKind, HashMap, Mutex, PeleError,
    PublicError, Reader, Value,
};
#[derive(Clone, Copy)]
enum Kind {
    Map,
    Set,
    Array,
}
enum Source {
    Node(u32, Kind),
    Sequence(Arc<[Value]>),
    String(Arc<super::JsString>),
}
pub(super) struct Cursor {
    source: Source,
    index: usize,
    done: bool,
}
fn limit() -> PublicError {
    PeleError::new("Live iteration exceeds work budget", ErrorCode::Limit).into()
}
impl Reader {
    pub(super) fn begin_iterator(&mut self, value: &Value) -> Result<Value, PublicError> {
        if self.iterator_handles >= 1_000_000 {
            return Err(limit());
        }
        let source = match value {
            Value::Atom(Atom::Ref(id)) => {
                let document = unsafe { public::document(self.owner) }?;
                let graph = document.snapshot_read()?;
                let read = graph.reader();
                let kind =
                    match read.header(*id as usize)?.kind {
                        GraphNodeKind::Map(_) => Kind::Map,
                        GraphNodeKind::Set(_) => Kind::Set,
                        GraphNodeKind::Array(_) => Kind::Array,
                        _ => return Err(invalid(
                            "Native live iteration requires an ordinary Map, Set, Array or String",
                        )
                        .into()),
                    };
                Source::Node(*id, kind)
            }
            Value::Sequence(v) => Source::Sequence(Arc::clone(v)),
            Value::String(v) => Source::String(Arc::clone(v)),
            _ => {
                return Err(invalid(
                    "Native live iteration requires an ordinary Map, Set, Array or String",
                )
                .into())
            }
        };
        self.iterator_handles += 1;
        Ok(Value::Iterator(Arc::new(Mutex::new(Cursor {
            source,
            index: 0,
            done: false,
        }))))
    }
    pub(super) fn next_iterator(&mut self, value: &Value) -> Result<Option<Value>, PublicError> {
        let Value::Iterator(cursor) = value else {
            return Err(invalid("Foreign native live iterator").into());
        };
        let mut cursor = cursor
            .try_lock()
            .map_err(|_| invalid("Native live iterator is already in use"))?;
        if cursor.done {
            return Ok(None);
        }
        if self.iterator_advances >= 8_000_000 {
            cursor.done = true;
            return Err(limit());
        }
        self.iterator_advances += 1;
        let index = cursor.index;
        let result = (|| -> Result<(Option<Value>, usize), PublicError> {
            match &cursor.source {
                Source::Sequence(values) => Ok((values.get(index).cloned(), 1)),
                Source::String(text) => {
                    let Some(&first) = text.0.get(index) else {
                        return Ok((None, 0));
                    };
                    let width = if (0xd800..=0xdbff).contains(&first)
                        && text
                            .0
                            .get(index + 1)
                            .is_some_and(|v| (0xdc00..=0xdfff).contains(v))
                    {
                        2
                    } else {
                        1
                    };
                    Ok((
                        Some(Value::String(Arc::new(super::JsString(
                            text.0[index..index + width].to_vec(),
                        )))),
                        width,
                    ))
                }
                Source::Node(id, kind) => {
                    // Copy only the selected yielded value under this short guard.
                    let document = unsafe { public::document(self.owner) }?;
                    let graph = document.snapshot_read()?;
                    let read = graph.reader();
                    let header = read.header(*id as usize)?;
                    match (kind, header.kind) {
                        (Kind::Map, GraphNodeKind::Map(length)) => {
                            if index >= length {
                                return Ok((None, 0));
                            }
                            let mut pair = None;
                            let mut strings = HashMap::new();
                            read.map_entries(*id as usize, index, &mut |entries| {
                                if entries.len() > length - index {
                                    return Err(invalid("Invalid captured Map batch"));
                                }
                                let (key, value) = entries
                                    .first()
                                    .ok_or_else(|| invalid("Invalid captured Map batch"))?;
                                pair = Some(Value::Sequence(Arc::from([
                                    capture_value(read, key, &mut strings)?,
                                    capture_value(read, value, &mut strings)?,
                                ])));
                                Ok(())
                            })?;
                            Ok((pair, 1))
                        }
                        (Kind::Set, GraphNodeKind::Set(length)) => {
                            if index >= length {
                                return Ok((None, 0));
                            }
                            let mut value = None;
                            let mut strings = HashMap::new();
                            read.set_values(*id as usize, index, &mut |values| {
                                if values.len() > length - index {
                                    return Err(invalid("Invalid captured Set batch"));
                                }
                                let item = values
                                    .first()
                                    .ok_or_else(|| invalid("Invalid captured Set batch"))?;
                                value = Some(capture_value(read, item, &mut strings)?);
                                Ok(())
                            })?;
                            Ok((value, 1))
                        }
                        (Kind::Array, GraphNodeKind::Array(length)) => {
                            if index >= length as usize {
                                return Ok((None, 0));
                            }
                            let key = index.to_string();
                            let mut start = 0;
                            let mut value = Value::Atom(Atom::Undefined);
                            while start < header.properties {
                                let mut count = 0;
                                let mut found = false;
                                read.properties(*id as usize, start, &mut |properties| {
                                    count = properties.len();
                                    for p in properties {
                                        if super::string(read, p.key)?.is(&key) {
                                            value = super::own(read, p.value.clone())?;
                                            found = true;
                                            break;
                                        }
                                    }
                                    Ok(())
                                })?;
                                if found {
                                    break;
                                }
                                if count == 0 || count > header.properties - start {
                                    return Err(invalid("Invalid captured property batch").into());
                                }
                                start += count;
                            }
                            Ok((Some(value), 1))
                        }
                        _ => Err(invalid("Native live iterator coordinate changed kind").into()),
                    }
                }
            }
        })();
        match result {
            Ok((Some(value), step)) => {
                cursor.index = cursor.index.checked_add(step).ok_or_else(limit)?;
                Ok(Some(value))
            }
            Ok((None, _)) => {
                cursor.done = true;
                Ok(None)
            }
            Err(error) => {
                cursor.done = true;
                Err(error)
            }
        }
    }
    pub(super) fn close_iterator(&mut self, value: &Value) -> Result<(), PublicError> {
        let Value::Iterator(cursor) = value else {
            return Err(invalid("Foreign native live iterator").into());
        };
        cursor
            .try_lock()
            .map_err(|_| invalid("Native live iterator is already in use"))?
            .done = true;
        // Native ordinary collection iterators have no user-defined return method.
        Ok(())
    }
}
