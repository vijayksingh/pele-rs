//! Shared value vocabulary; each renderer read selects its own live phase.
use super::model::*;
use crate::compat::PeleError;
use std::sync::Arc;
pub(super) trait PhaseInput {
    type Value: Clone;
    type Error: From<PeleError>;
    fn begin_draw(&mut self) {}
    fn root(&self) -> Self::Value;
    fn get(&mut self, v: &Self::Value, key: &'static str) -> Result<Self::Value, Self::Error>;
    fn count(&mut self, v: &Self::Value) -> Result<usize, Self::Error>;
    fn index(&mut self, v: &Self::Value, index: usize) -> Result<Self::Value, Self::Error>;
    fn present(&mut self, v: &Self::Value, index: usize) -> Result<bool, Self::Error>;
    fn identity(&mut self, v: &Self::Value) -> Result<u64, Self::Error>;
    fn text(&mut self, v: &Self::Value) -> Result<Option<String>, Self::Error>;
    fn truthy(&self, v: &Self::Value) -> bool;
    fn join(&mut self, v: &Self::Value) -> Result<String, Self::Error>;
    fn step(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn field(&mut self, v: &Self::Value, key: &'static str) -> Result<String, Self::Error> {
        let v = self.get(v, key)?;
        Ok(self.text(&v)?.unwrap_or_default())
    }
    fn root_field(&mut self, key: &'static str) -> Result<Self::Value, Self::Error> {
        self.get(&self.root(), key)
    }
    fn metadata(&mut self, key: &'static str) -> Result<Option<String>, Self::Error> {
        let mut v = self.root_field(key)?;
        if key != "title" {
            if !self.truthy(&v) {
                return Ok(None);
            }
            v = self.root_field(key)?;
        }
        self.text(&v)
    }
}
#[derive(Clone)]
pub(super) enum Value<'a> {
    Root(&'a EventModelingModel),
    Lane(&'a Arc<EmLane>),
    Box(&'a Arc<EmBox>),
    Relation(&'a EmRelation),
    Note(&'a EmNote),
    Spec(&'a EmSpecification),
    Lanes(&'a [Arc<EmLane>]),
    Boxes(&'a [Arc<EmBox>]),
    Relations(&'a [EmRelation]),
    Notes(&'a [EmNote]),
    Specs(&'a [EmSpecification]),
    Strings(&'a [String]),
    Text(&'a str),
    Bool(bool),
    Undefined,
}
pub(super) struct Typed<'a>(pub &'a EventModelingModel);
impl<'a> PhaseInput for Typed<'a> {
    type Value = Value<'a>;
    type Error = PeleError;
    fn root(&self) -> Self::Value {
        Value::Root(self.0)
    }
    fn get(&mut self, v: &Self::Value, key: &'static str) -> Result<Self::Value, PeleError> {
        use Value::*;
        Ok(match (v, key) {
            (Root(m), "lanes") => Lanes(&m.lanes),
            (Root(m), "boxes") => Boxes(&m.boxes),
            (Root(m), "relations") => Relations(&m.relations),
            (Root(m), "notes") => Notes(&m.notes),
            (Root(m), "specifications") => Specs(&m.specifications),
            (Root(m), "title") => m.title.as_deref().map(Text).unwrap_or(Undefined),
            (Root(m), "accTitle") => m.acc_title.as_deref().map(Text).unwrap_or(Undefined),
            (Root(m), "accDescr") => m.acc_descr.as_deref().map(Text).unwrap_or(Undefined),
            (Lane(v), "label") => Text(&v.label),
            (Box(v), "lane") => Lane(&v.lane),
            (Box(v), "name") => Text(&v.name),
            (Box(v), "id") => Text(&v.id),
            (Box(v), "data") => v.data.as_deref().map(Text).unwrap_or(Undefined),
            (Box(v), "reset") => Bool(v.reset),
            (Box(v), "kind") => Text(match v.kind {
                EntityKind::Ui => "ui",
                EntityKind::Command => "command",
                EntityKind::Event => "event",
                EntityKind::Readmodel => "readmodel",
                EntityKind::Processor => "processor",
            }),
            (Relation(v), "source") => Box(&v.source),
            (Relation(v), "target") => Box(&v.target),
            (Note(v), "frame") => Text(&v.frame),
            (Note(v), "text") => Text(&v.text),
            (Spec(v), "frame") => Text(&v.frame),
            (Spec(v), "given") => Strings(&v.given),
            (Spec(v), "when") => Strings(&v.when),
            (Spec(v), "then") => Strings(&v.then),
            _ => Undefined,
        })
    }
    fn count(&mut self, v: &Self::Value) -> Result<usize, PeleError> {
        use Value::*;
        Ok(match v {
            Lanes(v) => v.len(),
            Boxes(v) => v.len(),
            Relations(v) => v.len(),
            Notes(v) => v.len(),
            Specs(v) => v.len(),
            Strings(v) => v.len(),
            _ => 0,
        })
    }
    fn index(&mut self, v: &Self::Value, i: usize) -> Result<Self::Value, PeleError> {
        use Value::*;
        Ok(match v {
            Lanes(v) => v.get(i).map(Lane),
            Boxes(v) => v.get(i).map(Box),
            Relations(v) => v.get(i).map(Relation),
            Notes(v) => v.get(i).map(Note),
            Specs(v) => v.get(i).map(Spec),
            Strings(v) => v.get(i).map(|v| Text(v)),
            _ => None,
        }
        .unwrap_or(Undefined))
    }
    fn present(&mut self, v: &Self::Value, i: usize) -> Result<bool, PeleError> {
        Ok(i < self.count(v)?)
    }
    fn identity(&mut self, v: &Self::Value) -> Result<u64, PeleError> {
        Ok(match v {
            Value::Lane(v) => Arc::as_ptr(v) as usize as u64,
            Value::Box(v) => Arc::as_ptr(v) as usize as u64,
            _ => 0,
        })
    }
    fn text(&mut self, v: &Self::Value) -> Result<Option<String>, PeleError> {
        Ok(if let Value::Text(v) = v {
            Some((*v).into())
        } else {
            None
        })
    }
    fn truthy(&self, v: &Self::Value) -> bool {
        match v {
            Value::Undefined => false,
            Value::Bool(v) => *v,
            Value::Text(v) => !v.is_empty(),
            _ => true,
        }
    }
    fn join(&mut self, v: &Self::Value) -> Result<String, PeleError> {
        Ok(if let Value::Strings(v) = v {
            v.join(", ")
        } else {
            String::new()
        })
    }
}
