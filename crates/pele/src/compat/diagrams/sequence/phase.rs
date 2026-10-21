//! Sequence geometry chooses reads; values and iterators retain their original receivers.
use super::model::{MessageText, SeqActor, SeqBox, SeqMessage, SeqNumbering, SequenceModel};
use crate::compat::PeleError;
pub(super) trait PhaseInput {
    type Value: Clone;
    type Iterator;
    type Error: From<PeleError>;
    fn begin_draw(&mut self) {}
    fn close(&mut self, _iterator: &mut Self::Iterator) -> Result<(), Self::Error> {
        Ok(())
    }
    fn keep_text(&mut self, _units: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    fn root(&self) -> Self::Value;
    fn get(&mut self, v: &Self::Value, key: &'static str) -> Result<Self::Value, Self::Error>;
    fn index(&mut self, v: &Self::Value, i: usize) -> Result<Self::Value, Self::Error>;
    fn count(&mut self, v: &Self::Value) -> Result<usize, Self::Error>;
    fn number(&self, v: &Self::Value) -> Option<f64>;
    fn truthy(&self, v: &Self::Value) -> bool;
    fn text(&mut self, v: &Self::Value) -> Result<Option<String>, Self::Error>;
    fn identity(&mut self, v: &Self::Value) -> Result<Option<u64>, Self::Error>;
    fn entries(&mut self, v: &Self::Value) -> Result<Self::Iterator, Self::Error>;
    fn next(
        &mut self,
        v: &mut Self::Iterator,
    ) -> Result<Option<(Self::Value, Self::Value)>, Self::Error>;
    fn lookup(&mut self, v: &Self::Value, key: &str) -> Result<Self::Value, Self::Error>;
    fn step(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn label_text(&mut self, v: &Self::Value) -> Result<Option<String>, Self::Error> {
        self.text(v)
    }
    fn label_field(
        &mut self,
        v: &Self::Value,
        key: &'static str,
    ) -> Result<Option<String>, Self::Error> {
        let value = self.get(v, key)?;
        self.label_text(&value)
    }
    fn field(&mut self, v: &Self::Value, k: &'static str) -> Result<Option<String>, Self::Error> {
        let v = self.get(v, k)?;
        self.text(&v)
    }
    fn byte(&mut self, v: &Self::Value, k: &'static str) -> Result<Option<u8>, Self::Error> {
        let v = self.get(v, k)?;
        Ok(self.number(&v).and_then(|n| {
            if n.is_finite() && n >= 0. && n <= 255. && n.fract() == 0. {
                Some(n as u8)
            } else {
                None
            }
        }))
    }
    fn flag(&mut self, v: &Self::Value, k: &'static str) -> Result<bool, Self::Error> {
        let v = self.get(v, k)?;
        Ok(self.truthy(&v))
    }
    fn at_field(
        &mut self,
        v: &Self::Value,
        i: usize,
        k: &'static str,
    ) -> Result<Option<String>, Self::Error> {
        let m = self.index(v, i)?;
        self.field(&m, k)
    }
    fn at_byte(
        &mut self,
        v: &Self::Value,
        i: usize,
        k: &'static str,
    ) -> Result<Option<u8>, Self::Error> {
        let m = self.index(v, i)?;
        self.byte(&m, k)
    }
    fn metadata(&mut self, key: &'static str) -> Result<Option<String>, Self::Error> {
        let r = self.root();
        let mut v = self.get(&r, key)?;
        if key != "title" {
            if !self.truthy(&v) {
                return Ok(None);
            }
            v = self.get(&r, key)?;
        }
        self.label_text(&v)
    }
}
#[derive(Clone)]
pub(super) enum Value<'a> {
    Root(&'a SequenceModel),
    Actors(&'a SequenceModel),
    Actor(&'a SequenceModel, &'a SeqActor),
    Messages(&'a [SeqMessage]),
    Message(&'a SeqMessage),
    Box(&'a SeqBox),
    Numbering(&'a SeqNumbering),
    Positions(&'a [(String, usize)]),
    Properties(&'a SeqActor),
    Text(&'a str),
    Number(f64),
    Bool(bool),
    Undefined,
}
pub(super) struct Iterator<'a> {
    model: &'a SequenceModel,
    index: usize,
}
pub(super) struct Typed<'a>(pub &'a SequenceModel);
impl<'a> PhaseInput for Typed<'a> {
    type Value = Value<'a>;
    type Iterator = Iterator<'a>;
    type Error = PeleError;
    fn root(&self) -> Self::Value {
        Value::Root(self.0)
    }
    fn get(&mut self, v: &Self::Value, k: &'static str) -> Result<Self::Value, PeleError> {
        use Value::*;
        Ok(match (v, k) {
            (Root(m), "actors") => Actors(m),
            (Root(m), "messages") => Messages(&m.messages),
            (Root(m), "createdActors") => Positions(&m.created_actors),
            (Root(m), "destroyedActors") => Positions(&m.destroyed_actors),
            (Root(m), "title") => m.title.as_deref().map(Text).unwrap_or(Undefined),
            (Root(m), "accTitle") => m.acc_title.as_deref().map(Text).unwrap_or(Undefined),
            (Root(m), "accDescr") => m.acc_descr.as_deref().map(Text).unwrap_or(Undefined),
            (Actors(m), "size") => Number(m.actors.len() as f64),
            (Actor(_, a), "type") => Text(&a.r#type),
            (Actor(_, a), "description") => Text(&a.description),
            (Actor(_, a), "wrap") => Bool(a.wrap),
            (Actor(m, a), "box") => a
                .box_index
                .and_then(|i| m.boxes.get(i))
                .map(Box)
                .unwrap_or(Undefined),
            (Actor(_, a), "properties") => Properties(a),
            (Message(m), "from") => m.from.as_deref().map(Text).unwrap_or(Undefined),
            (Message(m), "to") => m.to.as_deref().map(Text).unwrap_or(Undefined),
            (Message(m), "id") => Text(&m.id),
            (Message(m), "message") => match &m.message {
                MessageText::Text(s) => Text(s),
                MessageText::Numbering(n) => Numbering(n),
            },
            (Message(m), "type") => m.r#type.map(|n| Number(n as f64)).unwrap_or(Undefined),
            (Message(m), "placement") => m.placement.map(|n| Number(n as f64)).unwrap_or(Undefined),
            (Message(m), "centralConnection") => m
                .central_connection
                .map(|n| Number(n as f64))
                .unwrap_or(Undefined),
            (Message(m), "wrap") => Bool(m.wrap),
            (Numbering(n), "start") => n.start.map(Number).unwrap_or(Undefined),
            (Numbering(n), "step") => n.step.map(Number).unwrap_or(Undefined),
            (Numbering(n), "visible") => Bool(n.visible),
            (Box(b), "name") => b.name.as_deref().map(Text).unwrap_or(Undefined),
            (Box(b), "fill") => Text(&b.fill),
            (Box(b), "wrap") => Bool(b.wrap),
            _ => Undefined,
        })
    }
    fn index(&mut self, v: &Self::Value, i: usize) -> Result<Self::Value, PeleError> {
        Ok(match v {
            Value::Messages(ms) => ms.get(i).map(Value::Message).unwrap_or(Value::Undefined),
            _ => Value::Undefined,
        })
    }
    fn count(&mut self, v: &Self::Value) -> Result<usize, PeleError> {
        Ok(match v {
            Value::Messages(ms) => ms.len(),
            Value::Actors(m) => m.actors.len(),
            _ => 0,
        })
    }
    fn number(&self, v: &Self::Value) -> Option<f64> {
        if let Value::Number(n) = v {
            Some(*n)
        } else {
            None
        }
    }
    fn truthy(&self, v: &Self::Value) -> bool {
        match v {
            Value::Undefined => false,
            Value::Bool(v) => *v,
            Value::Number(v) => *v != 0. && !v.is_nan(),
            Value::Text(s) => !s.is_empty(),
            _ => true,
        }
    }
    fn text(&mut self, v: &Self::Value) -> Result<Option<String>, PeleError> {
        Ok(if let Value::Text(s) = v {
            Some((*s).into())
        } else {
            None
        })
    }
    fn identity(&mut self, v: &Self::Value) -> Result<Option<u64>, PeleError> {
        Ok(match v {
            Value::Box(b) => Some(*b as *const SeqBox as usize as u64),
            _ => None,
        })
    }
    fn entries(&mut self, v: &Self::Value) -> Result<Self::Iterator, PeleError> {
        match v {
            Value::Actors(m) => Ok(Iterator { model: m, index: 0 }),
            _ => unreachable!("typed actors Map"),
        }
    }
    fn next(
        &mut self,
        v: &mut Self::Iterator,
    ) -> Result<Option<(Self::Value, Self::Value)>, PeleError> {
        let pair = v
            .model
            .actors
            .get(v.index)
            .map(|(k, a)| (Value::Text(k), Value::Actor(v.model, a)));
        v.index += 1;
        Ok(pair)
    }
    fn lookup(&mut self, v: &Self::Value, key: &str) -> Result<Self::Value, PeleError> {
        Ok(match v {
            Value::Positions(p) => p
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, n)| Value::Number(*n as f64)),
            Value::Properties(a) => a
                .properties
                .iter()
                .find(|(k, _)| k == key)
                .and_then(|(_, v)| v.as_str())
                .map(Value::Text),
            _ => None,
        }
        .unwrap_or(Value::Undefined))
    }
}
