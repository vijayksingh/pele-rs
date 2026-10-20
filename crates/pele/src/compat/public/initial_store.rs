//! Storage-only construction sink shared by built-in public model codecs.
//! Writers own their output; borrowed input strings never escape a call.
use super::{Atom, GraphWriter, ModelGraph, Node};
use crate::compat::PeleError;

pub trait InitialWriter {
    type Output;
    fn string(&mut self, value: &str) -> Result<u32, PeleError>;
    fn node(&mut self, node: Node) -> Result<u32, PeleError>;
    /// A fresh ordinary record with statically ordered fields. The default
    /// retains the exact property/interner/budget path of generic publication.
    fn record<const N: usize>(
        &mut self,
        keys: &'static [&'static str; N],
        values: [Atom; N],
    ) -> Result<u32, PeleError>
    where
        Self: Sized,
    {
        let properties = keys
            .iter()
            .zip(values)
            .map(|(key, value)| self.property(key, value))
            .collect::<Result<_, _>>()?;
        self.node(Node::Object { properties })
    }
    /// A fresh array with consecutive own indices and ordinary descriptors.
    /// Values are already materialized so every value string precedes index
    /// strings. The default preserves property overrides and native budgets.
    fn dense_array(&mut self, values: &[Atom]) -> Result<u32, PeleError> {
        let mut properties = Vec::with_capacity(values.len());
        for (index, value) in values.iter().enumerate() {
            properties.push(self.property(&index.to_string(), value.clone())?);
        }
        self.node(Node::Array {
            length: properties.len() as u32,
            properties,
        })
    }
    fn finish(self, root: Atom) -> Result<Self::Output, PeleError>;
    fn abort(self) -> Result<(), PeleError>
    where
        Self: Sized,
    {
        Ok(())
    }
    fn property(
        &mut self,
        key: &str,
        value: Atom,
    ) -> Result<crate::compat::public::Property, PeleError> {
        Ok(crate::compat::public::Property {
            key: self.string(key)?,
            value,
            flags: 7,
        })
    }
}
impl InitialWriter for GraphWriter {
    type Output = ModelGraph;
    fn property(
        &mut self,
        key: &str,
        value: Atom,
    ) -> Result<crate::compat::public::Property, PeleError> {
        Ok(GraphWriter::property(self, key, value))
    }
    fn string(&mut self, value: &str) -> Result<u32, PeleError> {
        Ok(GraphWriter::string_str(self, value))
    }
    fn node(&mut self, node: Node) -> Result<u32, PeleError> {
        Ok(GraphWriter::node(self, node))
    }
    fn finish(self, root: Atom) -> Result<ModelGraph, PeleError> {
        GraphWriter::finish(self, root)
    }
}
