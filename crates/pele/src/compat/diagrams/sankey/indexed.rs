//! Private source ownership: one consumed text catalog, indexed node/link rows.
//! Public SankeyModel and its serialized endpoint records stay unchanged.
use super::model::SankeyModel;
use std::{borrow::Cow, collections::HashMap};

pub(super) struct Link {
    pub source: usize,
    pub target: usize,
    pub value: f64,
}
pub(super) struct CatalogEntry {
    pub text: String,
    pub units: usize,
}
pub(super) struct IndexedSankey {
    pub catalog: Vec<CatalogEntry>,
    pub labels: Vec<usize>,
    pub indices: Vec<usize>,
    pub keys: [usize; 13],
    pub links: Vec<Link>,
    pub title: Option<usize>,
}
struct Symbol {
    id: usize,
    units: usize,
    node: Option<usize>,
}
/// Own each first-content text once; node ordinals and catalog IDs differ.
/// No semantic limits run here: complete grammar must precede admission.
pub(super) struct SourceSymbols {
    symbols: HashMap<String, Symbol>,
    labels: Vec<usize>,
    keys: [usize; 13],
    links: Vec<Link>,
    title: Option<String>,
}
impl SourceSymbols {
    pub fn new(title: Option<&str>) -> Self {
        Self {
            symbols: HashMap::new(),
            labels: Vec::new(),
            keys: [0; 13],
            links: Vec::new(),
            title: title.map(str::to_owned),
        }
    }
    fn intern(&mut self, text: Cow<'_, str>) -> usize {
        if let Some(symbol) = self.symbols.get(text.as_ref()) {
            return symbol.id;
        }
        let id = self.symbols.len();
        let units = text.encode_utf16().count();
        self.symbols.insert(
            text.into_owned(),
            Symbol {
                id,
                units,
                node: None,
            },
        );
        id
    }
    fn key(&mut self, key: &'static str) {
        self.keys[key_slot(key)] = self.intern(Cow::Borrowed(key));
    }
    pub fn node(&mut self, text: Cow<'_, str>) -> usize {
        let next = self.labels.len();
        let id = if let Some(symbol) = self.symbols.get_mut(text.as_ref()) {
            if let Some(node) = symbol.node {
                return node;
            }
            symbol.node = Some(next);
            symbol.id
        } else {
            let id = self.symbols.len();
            let units = text.encode_utf16().count();
            self.symbols.insert(
                text.into_owned(),
                Symbol {
                    id,
                    units,
                    node: Some(next),
                },
            );
            id
        };
        self.labels.push(id);
        self.key("id");
        self.key("index");
        next
    }
    pub fn link(&mut self, source: usize, target: usize, value: f64) {
        self.links.push(Link {
            source,
            target,
            value,
        });
    }
    pub fn finish(mut self) -> IndexedSankey {
        // Exact first-content order of the public GraphWriter construction.
        let mut indices = Vec::with_capacity(self.labels.len().max(self.links.len()));
        for i in 0..self.labels.len() {
            indices.push(self.intern(Cow::Owned(i.to_string())));
        }
        if !self.links.is_empty() {
            for key in ["source", "target", "value"] {
                self.key(key);
            }
        }
        for i in indices.len()..self.links.len() {
            indices.push(self.intern(Cow::Owned(i.to_string())));
        }
        for key in ["sankey", "type", "nodes", "links"] {
            self.key(key);
        }
        let title = self.title.take().map(|text| self.intern(Cow::Owned(text)));
        for key in ["title", "accTitle", "accDescr", "lookup"] {
            self.key(key);
        }
        // Hash order never defines IDs. Move owned map keys into their slots;
        // there is no retained reverse index or second owned text allocation.
        let mut catalog: Vec<Option<CatalogEntry>> =
            (0..self.symbols.len()).map(|_| None).collect();
        for (text, symbol) in self.symbols {
            catalog[symbol.id] = Some(CatalogEntry {
                text,
                units: symbol.units,
            });
        }
        IndexedSankey {
            catalog: catalog
                .into_iter()
                .map(|entry| entry.expect("source symbol slot"))
                .collect(),
            labels: self.labels,
            indices,
            keys: self.keys,
            links: self.links,
            title,
        }
    }
}
#[derive(Clone, Copy)]
pub(super) struct NodeView<'a> {
    pub id: &'a str,
    pub index: usize,
}
/// Borrowed input shared by both representations. Geometry/resource ordering
/// remains in the single renderer/layout implementation.
pub(super) trait Input {
    fn node_count(&self) -> usize;
    fn node(&self, index: usize) -> NodeView<'_>;
    fn link_count(&self) -> usize;
    fn link(&self, index: usize) -> (usize, usize, f64);
    fn title(&self) -> Option<&str>;
    fn acc_title(&self) -> Option<&str>;
    fn acc_descr(&self) -> Option<&str>;
}
impl Input for SankeyModel {
    fn node_count(&self) -> usize {
        self.nodes.len()
    }
    fn node(&self, index: usize) -> NodeView<'_> {
        let node = &self.nodes[index];
        NodeView {
            id: &node.id,
            index: node.index,
        }
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
    fn node_count(&self) -> usize {
        self.labels.len()
    }
    fn node(&self, index: usize) -> NodeView<'_> {
        NodeView {
            id: &self.catalog[self.labels[index]].text,
            index,
        }
    }
    fn link_count(&self) -> usize {
        self.links.len()
    }
    fn link(&self, index: usize) -> (usize, usize, f64) {
        let link = &self.links[index];
        (link.source, link.target, link.value)
    }
    fn title(&self) -> Option<&str> {
        self.title.map(|id| self.catalog[id].text.as_str())
    }
    fn acc_title(&self) -> Option<&str> {
        None
    }
    fn acc_descr(&self) -> Option<&str> {
        None
    }
}
pub(super) fn key_slot(key: &str) -> usize {
    match key {
        "id" => 0,
        "index" => 1,
        "source" => 2,
        "target" => 3,
        "value" => 4,
        "sankey" => 5,
        "type" => 6,
        "nodes" => 7,
        "links" => 8,
        "title" => 9,
        "accTitle" => 10,
        "accDescr" => 11,
        "lookup" => 12,
        _ => unreachable!("static Sankey field"),
    }
}
