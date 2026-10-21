//! Rust-selected GitGraph reads, shared by typed and owned live drawing.
use super::model::{GitCommit, GitModel};
use crate::compat::{ErrorCode, PeleError};
pub(super) enum SequenceKey {
    Native(usize),
    Number(f64),
}
impl SequenceKey {
    pub fn compare(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Native(a), Self::Native(b)) => a.cmp(b),
            (Self::Number(a), Self::Number(b)) => {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            }
            _ => unreachable!("one input domain per draw"),
        }
    }
}
pub(super) trait PhaseInput {
    type Error: From<PeleError>;
    type Value: Clone;
    type TagIterator;
    fn tags_begin(&mut self, value: &Self::Value) -> Result<Self::TagIterator, Self::Error>;
    fn tags_next(
        &mut self,
        iterator: &mut Self::TagIterator,
    ) -> Result<Option<Self::Value>, Self::Error>;
    fn tags_close(&mut self, iterator: &mut Self::TagIterator) -> Result<(), Self::Error>;
    fn begin_draw(&mut self) {}
    fn root(&mut self) -> Self::Value;
    fn get(&mut self, v: &Self::Value, key: &'static str) -> Result<Self::Value, Self::Error>;
    fn count(&mut self, v: &Self::Value) -> Result<usize, Self::Error>;
    fn index(&mut self, v: &Self::Value, i: usize) -> Result<Self::Value, Self::Error>;
    fn present(&mut self, v: &Self::Value, i: usize) -> Result<bool, Self::Error>;
    fn map_values(&mut self, v: &Self::Value) -> Result<Vec<Self::Value>, Self::Error>;
    fn text(&mut self, v: &Self::Value) -> Result<String, Self::Error>;
    fn number(&self, v: &Self::Value) -> Option<f64>;
    fn string_key(&mut self, v: &Self::Value) -> Result<Option<String>, Self::Error>;
    fn is_true(&self, v: &Self::Value) -> bool;
    fn nullish(&self, v: &Self::Value) -> bool;
    fn work(&mut self, _n: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    fn keep_label(&mut self, _n: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    fn field_text(
        &mut self,
        v: &Self::Value,
        k: &'static str,
    ) -> Result<Option<String>, Self::Error> {
        let v = self.get(v, k)?;
        if self.nullish(&v) {
            Ok(None)
        } else {
            self.text(&v).map(Some)
        }
    }
    fn field_number(
        &mut self,
        v: &Self::Value,
        k: &'static str,
    ) -> Result<Option<f64>, Self::Error> {
        let v = self.get(v, k)?;
        Ok(self.number(&v))
    }
    fn sequence(&mut self, v: &Self::Value) -> Result<SequenceKey, Self::Error> {
        Ok(SequenceKey::Number(
            self.field_number(v, "seq")?.unwrap_or(f64::NAN),
        ))
    }
    fn comparison_number(&self, v: &Self::Value) -> Result<f64, Self::Error>;
    fn cached_kind(&self, v: &Self::Value) -> Result<u8, Self::Error> {
        let n = self.comparison_number(v)?;
        Ok(
            if self.number(v).is_some() && (0. ..=4.).contains(&n) && n.fract() == 0. {
                n as u8
            } else if n < 3. {
                0
            } else {
                255
            },
        )
    }
    fn kind(&mut self, v: &Self::Value, key: &'static str) -> Result<u8, Self::Error> {
        let value = self.get(v, key)?;
        self.cached_kind(&value)
    }
    fn strings(&mut self, v: &Self::Value) -> Result<Vec<Option<String>>, Self::Error> {
        let mut out = Vec::new();
        let mut i = 0;
        while i < self.count(v)? {
            self.work(1)?;
            let item = self.index(v, i)?;
            out.push(self.string_key(&item)?);
            i += 1;
        }
        Ok(out)
    }
}
#[derive(Clone, Copy)]
pub(super) enum TypedValue<'a> {
    Undefined,
    Text(&'a str),
    Number(f64),
    Bool(bool),
    Root(&'a GitModel),
    Commit(&'a GitCommit),
    Strings(&'a [String]),
    Commits(&'a [(String, GitCommit)]),
}
pub(super) struct Typed<'a>(pub &'a GitModel);
impl<'a> PhaseInput for Typed<'a> {
    type Error = PeleError;
    type Value = TypedValue<'a>;
    type TagIterator = std::slice::Iter<'a, String>;
    fn tags_begin(&mut self, value: &Self::Value) -> Result<Self::TagIterator, PeleError> {
        let TypedValue::Strings(values) = value else {
            unreachable!("typed tags")
        };
        Ok(values.iter())
    }
    fn tags_next(
        &mut self,
        iterator: &mut Self::TagIterator,
    ) -> Result<Option<Self::Value>, PeleError> {
        Ok(iterator.next().map(|value| TypedValue::Text(value)))
    }
    fn tags_close(&mut self, _: &mut Self::TagIterator) -> Result<(), PeleError> {
        Ok(())
    }

    fn root(&mut self) -> Self::Value {
        TypedValue::Root(self.0)
    }
    fn get(&mut self, v: &Self::Value, k: &'static str) -> Result<Self::Value, PeleError> {
        use TypedValue::*;
        Ok(match (*v, k) {
            (Root(v), "direction") => Text(&v.direction),
            (Root(v), "lanes") => Strings(&v.lanes),
            (Root(v), "commits") => Commits(&v.commits),
            (Root(v), "title") => v.title.as_deref().map_or(Undefined, Text),
            (Root(v), "accTitle") => v.acc_title.as_deref().map_or(Undefined, Text),
            (Root(v), "accDescr") => v.acc_descr.as_deref().map_or(Undefined, Text),
            (Commit(v), "id") => Text(&v.id),
            (Commit(v), "branch") => Text(&v.branch),
            (Commit(v), "message") => Text(&v.message),
            (Commit(v), "seq") => Number(v.seq as f64),
            (Commit(v), "type") => Number(v.commit_type as f64),
            (Commit(v), "customType") => v.custom_type.map_or(Undefined, |v| Number(v as f64)),
            (Commit(v), "customId") => v.custom_id.map_or(Undefined, Bool),
            (Commit(v), "parents") => Strings(&v.parents),
            (Commit(v), "tags") => Strings(&v.tags),
            _ => Undefined,
        })
    }
    fn count(&mut self, v: &Self::Value) -> Result<usize, PeleError> {
        Ok(match v {
            TypedValue::Strings(v) => v.len(),
            _ => 0,
        })
    }
    fn index(&mut self, v: &Self::Value, i: usize) -> Result<Self::Value, PeleError> {
        Ok(match v {
            TypedValue::Strings(v) => v
                .get(i)
                .map_or(TypedValue::Undefined, |s| TypedValue::Text(s)),
            _ => TypedValue::Undefined,
        })
    }
    fn present(&mut self, v: &Self::Value, i: usize) -> Result<bool, PeleError> {
        Ok(i < self.count(v)?)
    }
    fn map_values(&mut self, v: &Self::Value) -> Result<Vec<Self::Value>, PeleError> {
        Ok(match v {
            TypedValue::Commits(v) => v.iter().map(|(_, c)| TypedValue::Commit(c)).collect(),
            _ => vec![],
        })
    }
    fn text(&mut self, v: &Self::Value) -> Result<String, PeleError> {
        match v {
            TypedValue::Text(s) => Ok((*s).into()),
            _ => Err(PeleError::new(
                "GitGraph text must be a string",
                ErrorCode::Semantic,
            )),
        }
    }
    fn string_key(&mut self, v: &Self::Value) -> Result<Option<String>, PeleError> {
        Ok(match v {
            TypedValue::Text(s) => Some((*s).into()),
            _ => None,
        })
    }
    fn number(&self, v: &Self::Value) -> Option<f64> {
        if let TypedValue::Number(n) = v {
            Some(*n)
        } else {
            None
        }
    }
    fn sequence(&mut self, v: &Self::Value) -> Result<SequenceKey, PeleError> {
        if let TypedValue::Commit(v) = v {
            Ok(SequenceKey::Native(v.seq))
        } else {
            unreachable!()
        }
    }
    fn comparison_number(&self, v: &Self::Value) -> Result<f64, PeleError> {
        Ok(self.number(v).unwrap_or(f64::NAN))
    }
    fn is_true(&self, v: &Self::Value) -> bool {
        matches!(v, TypedValue::Bool(true))
    }
    fn nullish(&self, v: &Self::Value) -> bool {
        matches!(v, TypedValue::Undefined)
    }
}
