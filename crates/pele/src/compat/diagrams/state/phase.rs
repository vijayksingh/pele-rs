//! Shared State render values; the geometry selects each live read phase.
use super::{
    graph::{StateEdge, StateGraph, StateNode},
    model::{StateLink, StateModel},
};
use crate::compat::PeleError;
pub(super) trait PhaseInput {
    type Value: Clone;
    type Error: From<PeleError>;
    fn begin_draw(&mut self) {}
    fn root(&self) -> Self::Value;
    fn direction(&mut self) -> Result<String, Self::Error>;
    fn get(&mut self, v: &Self::Value, key: &'static str) -> Result<Self::Value, Self::Error>;
    fn count(&mut self, v: &Self::Value) -> Result<usize, Self::Error>;
    fn index(&mut self, v: &Self::Value, i: usize) -> Result<Self::Value, Self::Error>;
    fn text(&mut self, v: &Self::Value) -> Result<Option<String>, Self::Error>;
    fn truthy(&self, v: &Self::Value) -> bool;
    fn link(&mut self, id: &str) -> Result<Option<(String, String)>, Self::Error>;
    fn step(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn field(&mut self, v: &Self::Value, key: &'static str) -> Result<Option<String>, Self::Error> {
        let v = self.get(v, key)?;
        self.text(&v)
    }
    fn string(&mut self, v: &Self::Value, key: &'static str) -> Result<String, Self::Error> {
        Ok(self.field(v, key)?.unwrap_or_default())
    }
    fn strings(&mut self, v: &Self::Value) -> Result<Vec<String>, Self::Error> {
        let mut out = vec![];
        let mut i = 0;
        while i < self.count(v)? {
            self.step()?;
            let v = self.index(v, i)?;
            i += 1;
            out.push(self.text(&v)?.unwrap_or_default());
        }
        Ok(out)
    }
    fn metadata(&mut self, key: &'static str) -> Result<Option<String>, Self::Error> {
        let root = self.root();
        let mut v = self.get(&root, key)?;
        if key != "title" {
            if !self.truthy(&v) {
                return Ok(None);
            }
            v = self.get(&root, key)?;
        }
        self.text(&v)
    }
}
#[derive(Clone)]
pub(super) enum Value<'a> {
    Root(&'a StateModel),
    Graph(&'a StateGraph),
    Nodes(&'a [StateNode]),
    Edges(&'a [StateEdge]),
    Node(&'a StateNode),
    Edge(&'a StateEdge),
    Strings(&'a [String]),
    Text(&'a str),
    Bool(bool),
    Undefined,
}
pub(super) struct Typed<'a>(pub &'a StateModel);
impl<'a> PhaseInput for Typed<'a> {
    type Value = Value<'a>;
    type Error = PeleError;
    fn root(&self) -> Self::Value {
        Value::Root(self.0)
    }
    fn direction(&mut self) -> Result<String, PeleError> {
        Ok(self.0.direction.clone())
    }
    fn get(&mut self, v: &Self::Value, key: &'static str) -> Result<Self::Value, PeleError> {
        use Value::*;
        Ok(match (v, key) {
            (Root(m), "graph") => Graph(&m.graph),
            (Root(m), "title") => m.title.as_deref().map(Text).unwrap_or(Undefined),
            (Root(m), "accTitle") => m.acc_title.as_deref().map(Text).unwrap_or(Undefined),
            (Root(m), "accDescr") => m.acc_descr.as_deref().map(Text).unwrap_or(Undefined),
            (Graph(g), "nodes") => Nodes(&g.nodes),
            (Graph(g), "edges") => Edges(&g.edges),
            (Node(n), "id") => Text(&n.id),
            (Node(n), "shape") => n.shape.as_deref().map(Text).unwrap_or(Undefined),
            (Node(n), "label") => n.label.as_deref().map(Text).unwrap_or(Undefined),
            (Node(n), "isGroup") => Bool(n.is_group),
            (Node(n), "parentId") => n.parent_id.as_deref().map(Text).unwrap_or(Undefined),
            (Node(n), "noteFor") => n.note_for.as_deref().map(Text).unwrap_or(Undefined),
            (Node(n), "dir") => n.dir.as_deref().map(Text).unwrap_or(Undefined),
            (Node(n), "description") => Strings(&n.description),
            (Node(n), "cssStyles") => Strings(&n.css_styles),
            (Node(n), "cssCompiledStyles") => Strings(&n.css_compiled_styles),
            (Node(n), "cssClasses") => Text(&n.css_classes),
            (Edge(e), "start") => Text(&e.start),
            (Edge(e), "end") => Text(&e.end),
            (Edge(e), "pattern") => e.pattern.as_deref().map(Text).unwrap_or(Undefined),
            (Edge(e), "id") => Text(&e.id),
            (Edge(e), "label") => Text(&e.label),
            _ => Undefined,
        })
    }
    fn count(&mut self, v: &Self::Value) -> Result<usize, PeleError> {
        Ok(match v {
            Value::Nodes(v) => v.len(),
            Value::Edges(v) => v.len(),
            Value::Strings(v) => v.len(),
            _ => 0,
        })
    }
    fn index(&mut self, v: &Self::Value, i: usize) -> Result<Self::Value, PeleError> {
        use Value::*;
        Ok(match v {
            Nodes(v) => v.get(i).map(Node),
            Edges(v) => v.get(i).map(Edge),
            Strings(v) => v.get(i).map(|v| Text(v)),
            _ => None,
        }
        .unwrap_or(Undefined))
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
    fn link(&mut self, id: &str) -> Result<Option<(String, String)>, PeleError> {
        Ok(self
            .0
            .links
            .iter()
            .find(|(key, _)| key == id)
            .map(|(_, StateLink { url, tooltip })| (url.clone(), tooltip.clone())))
    }
}
