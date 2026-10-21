//! Owner-local retained handles. The global allocator stores identity only.
use super::*;
#[derive(Default)]
pub(crate) struct Values {
    values: HashMap<u32, Value>,
    nodes: usize,
    units: usize,
}
fn limit() -> PublicError {
    PeleError::new("Retained live value budget exceeded", ErrorCode::Limit).into()
}
impl Values {
    pub(super) fn retain(&mut self, value: &Value) -> Result<u32, PublicError> {
        if self.values.len() >= 1_000_000 {
            return Err(limit());
        }
        // Owned sequence/object captures contain no recursive references except
        // arena coordinates. Charge each occurrence before retaining its Arc.
        let mut pending = vec![value];
        let (mut nodes, mut units) = (self.nodes, self.units);
        while let Some(value) = pending.pop() {
            nodes = nodes.checked_add(1).ok_or_else(limit)?;
            if nodes > 1_000_000 {
                return Err(limit());
            }
            match value {
                Value::String(text) => units = units.checked_add(text.0.len()).ok_or_else(limit)?,
                Value::Sequence(items) => {
                    if items.len() > 1_000_000 - nodes || pending.len() > 1_000_000 - items.len() {
                        return Err(limit());
                    }
                    pending.extend(items.iter());
                }
                Value::Object(fields) => {
                    if fields.len() > 1_000_000 - nodes || pending.len() > 1_000_000 - fields.len()
                    {
                        return Err(limit());
                    }
                    for (key, value) in fields.iter() {
                        units = units.checked_add(key.0.len()).ok_or_else(limit)?;
                        if units > 16_000_000 {
                            return Err(limit());
                        }
                        pending.push(value);
                    }
                }
                Value::Iterator(_) => {
                    return Err(invalid("Native retained iterators are unsupported").into())
                }
                Value::Atom(_) => (),
            }
            if units > 16_000_000 {
                return Err(limit());
            }
        }
        let token = host_functions::allocate_reference()?;
        self.values.insert(token, value.clone());
        self.nodes = nodes;
        self.units = units;
        Ok(token)
    }
    pub(super) fn get(&self, token: u32) -> Result<Value, PublicError> {
        self.values
            .get(&token)
            .cloned()
            .ok_or_else(|| invalid("Foreign or released retained live value").into())
    }
}
