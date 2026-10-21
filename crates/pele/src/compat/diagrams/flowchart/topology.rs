//! Shared containment/collapse stages from Pele flowchart/graph.ts 458322a5 (MIT).
//! Calls are selected here; hosts supply values and never graph policy.
use super::model::FlowchartModel;
use crate::compat::Value;
use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
};

#[derive(Default)]
pub(super) struct Topology {
    pub declaration: HashMap<String, usize>,
    pub hidden: HashSet<String>,
    pub collapsed_into: HashMap<String, String>,
}
pub(super) trait Source {
    type Value: Clone;
    type Iter;
    type Error;
    fn groups(&mut self) -> Self::Value;
    fn members(&mut self, group: &Self::Value) -> Result<Self::Value, Self::Error>;
    fn begin(&mut self, value: &Self::Value) -> Result<Self::Iter, Self::Error>;
    fn next(&mut self, it: &mut Self::Iter) -> Result<Option<Self::Value>, Self::Error>;
    fn close(&mut self, it: &mut Self::Iter);
    fn id(&mut self, group: &Self::Value) -> Result<String, Self::Error>;
    fn child(&mut self, value: &Self::Value) -> Result<String, Self::Error>;
    fn lookup_exists(&mut self, id: &str) -> Result<bool, Self::Error>;
    fn lookup_collapsed(&mut self, id: &str) -> Result<bool, Self::Error>;
    fn own_collapsed(&mut self, group: &Self::Value) -> Result<bool, Self::Error>;
    fn copy_text(&mut self, text: &str) -> Result<(), Self::Error>;
    fn work(&mut self, count: usize) -> Result<(), Self::Error>;
}
fn copied<S: Source>(s: &mut S, text: &str) -> Result<String, S::Error> {
    s.copy_text(text)?;
    Ok(text.into())
}
fn each<S: Source>(
    s: &mut S,
    values: &S::Value,
    mut body: impl FnMut(&mut S, S::Value) -> Result<(), S::Error>,
) -> Result<(), S::Error> {
    let mut iterator = s.begin(values)?;
    while let Some(value) = s.next(&mut iterator)? {
        let result = s.work(1).and_then(|()| body(s, value));
        if let Err(error) = result {
            s.close(&mut iterator);
            return Err(error);
        }
    }
    Ok(())
}
pub(super) fn build<S: Source>(s: &mut S) -> Result<Topology, S::Error> {
    let groups = s.groups();
    let mut parent = HashMap::<String, String>::new();
    each(s, &groups, |s, group| {
        let members = s.members(&group)?;
        each(s, &members, |s, member| {
            let child = s.child(&member)?;
            if s.lookup_exists(&child)? {
                parent.insert(child, s.id(&group)?);
            }
            Ok(())
        })
    })?;
    let mut children = HashMap::<String, Vec<String>>::new();
    each(s, &groups, |s, group| {
        let id = s.id(&group)?;
        if let Some(parent) = parent.get(&id) {
            children.entry(copied(s, parent)?).or_default().push(id);
        }
        Ok(())
    })?;
    let mut result = Topology::default();
    each(s, &groups, |s, group| {
        let id = s.id(&group)?;
        if parent.contains_key(&id) {
            return Ok(());
        }
        let mut pending = vec![id];
        while let Some(id) = pending.pop() {
            s.work(1)?;
            if result.declaration.contains_key(&id) {
                continue;
            }
            let n = result.declaration.len();
            result.declaration.insert(copied(s, &id)?, n);
            if let Some(children) = children.get(&id) {
                s.work(children.len())?;
                for child in children.iter().rev() {
                    pending.push(copied(s, child)?);
                }
            }
        }
        Ok(())
    })?;
    let mut any_collapsed = false;
    each(s, &groups, |s, group| {
        if s.own_collapsed(&group)? {
            any_collapsed = true;
        }
        Ok(())
    })?;
    if !any_collapsed {
        return Ok(result);
    }
    let mut outermost = HashMap::<String, Option<String>>::new();
    each(s, &groups, |s, group| {
        let mut chain = Vec::new();
        let mut seen = HashSet::new();
        let mut current = Some(s.id(&group)?);
        while let Some(id) = current.as_ref() {
            s.work(1)?;
            if outermost.contains_key(id) || !seen.insert(copied(s, id)?) {
                break;
            }
            chain.push(copied(s, id)?);
            current = parent.get(id).map(|v| copied(s, v)).transpose()?;
        }
        let mut ancestor = current
            .as_ref()
            .and_then(|id| outermost.get(id).and_then(|v| v.as_ref()))
            .map(|v| copied(s, v))
            .transpose()?;
        for id in chain.iter().rev() {
            s.work(1)?;
            if ancestor.is_none() && s.lookup_collapsed(id)? {
                ancestor = Some(copied(s, id)?);
            }
            outermost.insert(
                copied(s, id)?,
                ancestor.as_ref().map(|v| copied(s, v)).transpose()?,
            );
        }
        if let Some(ancestor) = ancestor {
            // Getter/call may change this same group before these later reads.
            let id = s.id(&group)?;
            if id != ancestor {
                result.hidden.insert(copied(s, &id)?);
                result.collapsed_into.insert(id, copied(s, &ancestor)?);
            }
            let members = s.members(&group)?;
            each(s, &members, |s, child| {
                let id = s.child(&child)?;
                if id != ancestor {
                    result.hidden.insert(copied(s, &id)?);
                    result.collapsed_into.insert(id, copied(s, &ancestor)?);
                }
                Ok(())
            })?;
        }
        Ok(())
    })?;
    Ok(result)
}
#[derive(Clone)]
enum TypedValue<'a> {
    Groups,
    Group(usize),
    Members(usize),
    Child(&'a str),
}
struct Typed<'a> {
    model: &'a FlowchartModel,
    lookup: HashMap<&'a str, usize>,
}
impl<'a> Source for Typed<'a> {
    type Value = TypedValue<'a>;
    type Iter = (TypedValue<'a>, usize);
    type Error = Infallible;
    fn groups(&mut self) -> Self::Value {
        TypedValue::Groups
    }
    fn members(&mut self, g: &Self::Value) -> Result<Self::Value, Infallible> {
        let TypedValue::Group(i) = g else {
            unreachable!()
        };
        Ok(TypedValue::Members(*i))
    }
    fn begin(&mut self, v: &Self::Value) -> Result<Self::Iter, Infallible> {
        Ok((v.clone(), 0))
    }
    fn next(&mut self, it: &mut Self::Iter) -> Result<Option<Self::Value>, Infallible> {
        let i = it.1;
        it.1 += 1;
        Ok(match &it.0 {
            TypedValue::Groups => (i < self.model.subgraphs.len()).then_some(TypedValue::Group(i)),
            TypedValue::Members(g) => self.model.subgraphs[*g]
                .nodes
                .get(i)
                .map(|s| TypedValue::Child(s)),
            _ => unreachable!(),
        })
    }
    fn close(&mut self, _: &mut Self::Iter) {}
    fn id(&mut self, g: &Self::Value) -> Result<String, Infallible> {
        let TypedValue::Group(i) = g else {
            unreachable!()
        };
        Ok(self.model.subgraphs[*i].id.clone())
    }
    fn child(&mut self, v: &Self::Value) -> Result<String, Infallible> {
        let TypedValue::Child(s) = v else {
            unreachable!()
        };
        Ok((*s).into())
    }
    fn lookup_exists(&mut self, id: &str) -> Result<bool, Infallible> {
        Ok(self.lookup.contains_key(id))
    }
    fn lookup_collapsed(&mut self, id: &str) -> Result<bool, Infallible> {
        Ok(self
            .lookup
            .get(id)
            .is_some_and(|i| collapsed(self.model.subgraphs[*i].metadata.as_ref())))
    }
    fn own_collapsed(&mut self, g: &Self::Value) -> Result<bool, Infallible> {
        let TypedValue::Group(i) = g else {
            unreachable!()
        };
        Ok(collapsed(self.model.subgraphs[*i].metadata.as_ref()))
    }
    fn copy_text(&mut self, _: &str) -> Result<(), Infallible> {
        Ok(())
    }
    fn work(&mut self, _: usize) -> Result<(), Infallible> {
        Ok(())
    }
}
pub(super) fn collapsed(metadata: Option<&crate::compat::Object>) -> bool {
    metadata.and_then(|m| m.get("view")) == Some(&Value::String("collapsed".into()))
}
pub(super) fn typed(model: &FlowchartModel) -> Topology {
    let lookup = model
        .subgraphs
        .iter()
        .enumerate()
        .map(|(i, g)| (g.id.as_str(), i))
        .collect();
    match build(&mut Typed { model, lookup }) {
        Ok(v) => v,
        Err(e) => match e {},
    }
}
