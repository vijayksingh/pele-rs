//! Reads chosen by the shared Block renderer; no borrowed document crosses callbacks.
use super::model::{BlockClassDef, BlockModel};
use crate::compat::{ErrorCode, PeleError};
pub(super) trait PhaseInput {
    type Value: Clone;
    type Error: From<PeleError>;
    fn begin_draw(&mut self) {}
    fn root(&mut self) -> Self::Value;
    fn get(&mut self, value: &Self::Value, key: &'static str) -> Result<Self::Value, Self::Error>;
    fn index(&mut self, value: &Self::Value, index: usize) -> Result<Self::Value, Self::Error>;
    fn count(&mut self, value: &Self::Value) -> Result<usize, Self::Error>;
    fn text(&mut self, value: &Self::Value) -> Result<String, Self::Error>;
    fn truthy(&self, value: &Self::Value) -> bool;
    fn nullish(&self, value: &Self::Value) -> bool;
    fn number(&self, value: &Self::Value) -> Option<f64>;
    fn map_get_text(&mut self, value: &Self::Value, key: &str) -> Result<Self::Value, Self::Error>;
    fn join(&mut self, value: &Self::Value) -> Result<String, Self::Error>;
    fn work(&mut self, _count: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    fn keep_text(&mut self, _count: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    fn field_text(
        &mut self,
        value: &Self::Value,
        key: &'static str,
    ) -> Result<Option<String>, Self::Error> {
        let value = self.get(value, key)?;
        if self.nullish(&value) {
            Ok(None)
        } else {
            self.text(&value).map(Some)
        }
    }
    fn field_number(
        &mut self,
        value: &Self::Value,
        key: &'static str,
    ) -> Result<Option<f64>, Self::Error> {
        let value = self.get(value, key)?;
        if self.nullish(&value) {
            return Ok(None);
        }
        self.number(&value).map(Some).ok_or_else(|| {
            PeleError::new(
                "Block numeric runtime kind is not implemented",
                ErrorCode::Semantic,
            )
            .into()
        })
    }
    fn strings(&mut self, value: &Self::Value) -> Result<Vec<String>, Self::Error> {
        if self.nullish(value) {
            return Ok(vec![]);
        }
        let mut result = vec![];
        let mut i = 0;
        while i < self.count(value)? {
            self.work(1)?;
            let value = self.index(value, i)?;
            result.push(self.text(&value)?);
            i += 1;
        }
        Ok(result)
    }
}
#[derive(Clone, Copy)]
pub(super) enum Value<'a> {
    Undefined,
    Text(&'a str),
    Number(f64),
    Root(&'a BlockModel),
    Node(&'a BlockModel, usize),
    Nodes(&'a BlockModel, &'a [usize]),
    Strings(&'a [String]),
    Classes(&'a [(String, BlockClassDef)]),
    Class(&'a BlockClassDef),
}
pub(super) struct Typed<'a> {
    pub model: &'a BlockModel,
    pub acc_title: Option<&'a str>,
    pub acc_descr: Option<&'a str>,
}
impl<'a> PhaseInput for Typed<'a> {
    type Value = Value<'a>;
    type Error = PeleError;
    fn root(&mut self) -> Self::Value {
        Value::Root(self.model)
    }
    fn get(&mut self, value: &Self::Value, key: &'static str) -> Result<Self::Value, PeleError> {
        use Value::*;
        Ok(match (value, key) {
            (Root(m), "root") => Node(m, m.root),
            (Root(m), "edges") => Nodes(m, &m.edges),
            (Root(m), "classes") => Classes(&m.classes),
            (Root(m), "title") => m.title.as_deref().map_or(Undefined, Text),
            (Root(_), "accTitle") => self.acc_title.map_or(Undefined, Text),
            (Root(_), "accDescr") => self.acc_descr.map_or(Undefined, Text),
            (Class(c), "styles") => Strings(&c.styles),
            (Node(m, i), k) => {
                let b = &m.nodes[*i];
                match k {
                    "children" => b.children.as_deref().map_or(Undefined, |v| Nodes(m, v)),
                    "classes" => b.classes.as_deref().map_or(Undefined, Strings),
                    "styles" => b.styles.as_deref().map_or(Undefined, Strings),
                    "directions" => b.directions.as_deref().map_or(Undefined, Strings),
                    "columns" => b.columns.map_or(Undefined, Number),
                    "widthInColumns" => b.width_in_columns.map_or(Undefined, Number),
                    "id" => b.id.as_deref().map_or(Undefined, Text),
                    "type" => b.kind.as_deref().map_or(Undefined, Text),
                    "label" => b.label.as_deref().map_or(Undefined, Text),
                    "start" => b.start.as_deref().map_or(Undefined, Text),
                    "end" => b.end.as_deref().map_or(Undefined, Text),
                    "arrowTypeStart" => b.arrow_type_start.as_deref().map_or(Undefined, Text),
                    "arrowTypeEnd" => b.arrow_type_end.as_deref().map_or(Undefined, Text),
                    "thickness" => b.thickness.as_deref().map_or(Undefined, Text),
                    "pattern" => b.pattern.as_deref().map_or(Undefined, Text),
                    _ => Undefined,
                }
            }
            _ => Undefined,
        })
    }
    fn index(&mut self, value: &Self::Value, index: usize) -> Result<Self::Value, PeleError> {
        Ok(match value {
            Value::Nodes(m, v) => v
                .get(index)
                .map_or(Value::Undefined, |id| Value::Node(m, *id)),
            Value::Strings(v) => v.get(index).map_or(Value::Undefined, |v| Value::Text(v)),
            _ => Value::Undefined,
        })
    }
    fn count(&mut self, value: &Self::Value) -> Result<usize, PeleError> {
        Ok(match value {
            Value::Nodes(_, v) => v.len(),
            Value::Strings(v) => v.len(),
            _ => 0,
        })
    }
    fn text(&mut self, value: &Self::Value) -> Result<String, PeleError> {
        if let Value::Text(s) = value {
            Ok((*s).into())
        } else {
            Err(PeleError::new(
                "Block text must be a string",
                ErrorCode::Semantic,
            ))
        }
    }
    fn nullish(&self, value: &Self::Value) -> bool {
        matches!(value, Value::Undefined)
    }
    fn truthy(&self, value: &Self::Value) -> bool {
        match value {
            Value::Undefined => false,
            Value::Text(s) => !s.is_empty(),
            Value::Number(n) => *n != 0. && !n.is_nan(),
            _ => true,
        }
    }
    fn number(&self, value: &Self::Value) -> Option<f64> {
        if let Value::Number(n) = value {
            Some(*n)
        } else {
            None
        }
    }
    fn map_get_text(&mut self, value: &Self::Value, key: &str) -> Result<Self::Value, PeleError> {
        Ok(match value {
            Value::Classes(c) => c
                .iter()
                .find(|(name, _)| name == key)
                .map_or(Value::Undefined, |(_, v)| Value::Class(v)),
            _ => Value::Undefined,
        })
    }
    fn join(&mut self, value: &Self::Value) -> Result<String, PeleError> {
        Ok(self.strings(value)?.join(" "))
    }
}
