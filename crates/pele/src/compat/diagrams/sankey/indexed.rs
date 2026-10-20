//! Private source ownership: labels live once, links retain node indexes.
//! The public SankeyModel and its serialized endpoint records stay unchanged.
use super::model::{SankeyModel, SankeyNode};

pub(super) struct Link {
    pub source: usize,
    pub target: usize,
    pub value: f64,
}

pub(super) struct IndexedSankey {
    pub nodes: Vec<SankeyNode>,
    pub links: Vec<Link>,
    pub title: Option<String>,
}

/// Borrowed input shared by both representations. Geometry and resource-call
/// order remain in the single renderer/layout implementation.
pub(super) trait Input {
    fn nodes(&self) -> &[SankeyNode];
    fn link_count(&self) -> usize;
    fn link(&self, index: usize) -> (usize, usize, f64);
    fn title(&self) -> Option<&str>;
    fn acc_title(&self) -> Option<&str>;
    fn acc_descr(&self) -> Option<&str>;
}

impl Input for SankeyModel {
    fn nodes(&self) -> &[SankeyNode] {
        &self.nodes
    }
    fn link_count(&self) -> usize {
        self.links.len()
    }
    fn link(&self, index: usize) -> (usize, usize, f64) {
        let link = &self.links[index];
        (link.source.index, link.target.index, link.value)
    }
    fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }
    fn acc_title(&self) -> Option<&str> {
        self.acc_title.as_deref()
    }
    fn acc_descr(&self) -> Option<&str> {
        self.acc_descr.as_deref()
    }
}

impl Input for IndexedSankey {
    fn nodes(&self) -> &[SankeyNode] {
        &self.nodes
    }
    fn link_count(&self) -> usize {
        self.links.len()
    }
    fn link(&self, index: usize) -> (usize, usize, f64) {
        let link = &self.links[index];
        (link.source, link.target, link.value)
    }
    fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }
    fn acc_title(&self) -> Option<&str> {
        None
    }
    fn acc_descr(&self) -> Option<&str> {
        None
    }
}
