//! Shared preflight for the actual propertyOrder/array JSON row walk.
use super::json::{JsonAtom, JsonValue};
use crate::compat::{ErrorCode, PeleError};
use std::collections::HashMap;
fn limit() -> PeleError {
    let mut e = PeleError::new(
        "Usecase public model exceeds native graph/work budget",
        ErrorCode::Limit,
    );
    e.diagram_type = Some("usecase".into());
    e
}
const TEXT: usize = 16_000_000;
struct RowBudget {
    units: usize,
    work: usize,
}
impl RowBudget {
    fn tick(&mut self, n: usize) -> Result<(), PeleError> {
        self.work = self.work.saturating_add(n);
        if self.work > 8_000_000 {
            Err(limit())
        } else {
            Ok(())
        }
    }
}
pub(super) fn validate_json_rows(
    value: &JsonValue,
    order: &[(String, Vec<String>)],
    units: &mut usize,
    work: &mut usize,
) -> Result<(), PeleError> {
    let mut budget = RowBudget {
        units: *units,
        work: *work,
    };
    enum Segment<'a> {
        Key(&'a str),
        Index(usize),
    }
    enum Visit<'a> {
        Node(usize, Option<Segment<'a>>),
        Leave(bool, usize),
    }
    let order: HashMap<_, _> = order.iter().map(|(k, v)| (k.as_str(), v)).collect();
    let mut path = Vec::new();
    let mut display = 0usize;
    let mut pending = vec![Visit::Node(value.root, None)];
    let mut lookups: HashMap<usize, HashMap<&str, usize>> = HashMap::new();
    let row = |this: &mut RowBudget, path: usize, atom: &JsonAtom| -> Result<(), PeleError> {
        let text = match atom {
            JsonAtom::String(s) => s.encode_utf16().count(),
            JsonAtom::Number(n) => crate::compat::svg::js_num(*n).len(),
            JsonAtom::Bool(v) => {
                if *v {
                    4
                } else {
                    5
                }
            }
            JsonAtom::Null => 4,
            JsonAtom::Array(_) | JsonAtom::Object(_) => 2,
            _ => 9,
        };
        this.units = this
            .units
            .saturating_add(path.saturating_mul(2))
            .saturating_add(text);
        if this.units > TEXT {
            Err(limit())
        } else {
            Ok(())
        }
    };
    while let Some(visit) = pending.pop() {
        budget.tick(1)?;
        let (id, segment) = match visit {
            Visit::Leave(added, old) => {
                if added {
                    path.pop();
                }
                display = old;
                continue;
            }
            Visit::Node(id, segment) => (id, segment),
        };
        let old = display;
        let added = segment.is_some();
        if let Some(segment) = segment {
            display = match &segment {
                Segment::Key(k) => display
                    .saturating_add(k.encode_utf16().count())
                    .saturating_add(usize::from(display > 0)),
                Segment::Index(i) => display.saturating_add(i.to_string().len() + 2),
            };
            path.push(segment)
        }
        pending.push(Visit::Leave(added, old));
        let current = value.atoms.get(id).unwrap_or(&JsonAtom::Undefined);
        match current {
            JsonAtom::Array(children) => {
                if children.is_empty() {
                    row(&mut budget, display, current)?
                } else if children
                    .iter()
                    .all(|id| !matches!(value.atoms[*id], JsonAtom::Object(_) | JsonAtom::Array(_)))
                {
                    for id in children {
                        if !matches!(value.atoms[*id], JsonAtom::Hole) {
                            budget.tick(1)?;
                            row(&mut budget, display, &value.atoms[*id])?
                        }
                    }
                } else {
                    budget.tick(children.len())?;
                    for (index, id) in children.iter().enumerate().rev() {
                        if !matches!(value.atoms[*id], JsonAtom::Hole) {
                            pending.push(Visit::Node(*id, Some(Segment::Index(index))))
                        }
                    }
                }
            }
            JsonAtom::Object(entries) => {
                let mut pointer = String::new();
                for segment in &path {
                    match segment {
                        Segment::Key(key) => {
                            pointer.push('/');
                            pointer.push_str(&key.replace('~', "~0").replace('/', "~1"))
                        }
                        Segment::Index(index) => {
                            pointer.push('/');
                            pointer.push_str(&index.to_string())
                        }
                    }
                }
                budget.tick(pointer.len())?;
                let keys = order.get(pointer.as_str()).map_or_else(
                    || {
                        entries
                            .iter()
                            .map(|(key, _)| key.as_str())
                            .collect::<Vec<_>>()
                    },
                    |keys| keys.iter().map(String::as_str).collect(),
                );
                if keys.is_empty() {
                    row(&mut budget, display, current)?
                }
                let lookup = lookups
                    .entry(id)
                    .or_insert_with(|| entries.iter().map(|(k, v)| (k.as_str(), *v)).collect());
                budget.tick(keys.len())?;
                for key in keys.into_iter().rev() {
                    pending.push(Visit::Node(
                        lookup.get(key).copied().unwrap_or(value.atoms.len()),
                        Some(Segment::Key(key)),
                    ))
                }
            }
            JsonAtom::Hole => {}
            atom => row(&mut budget, display, atom)?,
        }
    }
    *units = budget.units;
    *work = budget.work;
    Ok(())
}
