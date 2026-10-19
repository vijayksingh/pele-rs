//! TreeView's returned plain wrapper and shared node graph, Pele 458322a5 (MIT kepano).
//! TreeViewDb/addNode/stack are private construction state, not returned methods.
//! Existing row/icon/guide renderer stays authoritative; cyclic/expanded graphs,
//! sparse active arrays, exotic coercion/callbacks and active lone UTF16 have gaps.
use super::{NodeType, TreeNode, TreeView, TreeViewModel};
use crate::compat::{
    public::*, DiagramModule, DiagramType, ErrorCode, Object, PeleError, RenderResources,
    RenderSettings, Rendered,
};
use std::collections::HashMap;
const fn f(name: &'static str, kind: FieldKind, undefined: bool, optional: bool) -> FieldSchema {
    FieldSchema {
        name,
        kind,
        undefined,
        optional,
        nullable: false,
        flags: 7,
    }
}
static SCHEMA: ModelSchema = ModelSchema {
    id: "pele/treeView@1",
    version: 1,
    root: Some(RecordId(0)),
    records: &[
        RecordSchema {
            id: RecordId(0),
            name: "TreeView",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("type", FieldKind::String, false, false),
                f("title", FieldKind::String, true, false),
                f("accTitle", FieldKind::String, true, false),
                f("accDescr", FieldKind::String, true, false),
                f("root", FieldKind::Record(RecordId(1)), false, false),
                f("count", FieldKind::Number, false, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(1),
            name: "TreeViewNode",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("id", FieldKind::Number, false, false),
                f("level", FieldKind::Number, false, false),
                f("name", FieldKind::String, false, false),
                f("nodeType", FieldKind::String, false, false),
                f("icon", FieldKind::String, true, true),
                f("cssClass", FieldKind::String, true, true),
                f("description", FieldKind::String, true, true),
                f("children", FieldKind::Array, false, false),
            ],
            members: &[],
        },
    ],
    prototypes: &[],
    methods: &[],
};
struct Cache {
    model: TreeViewModel,
    unknown_types: Vec<usize>,
}
pub struct TreeViewPublicState {
    graph: ModelGraph,
    cache: Result<Cache, PeleError>,
}
fn invalid(s: &str) -> PeleError {
    let mut e = PeleError::new(s, ErrorCode::Semantic);
    e.diagram_type = Some("treeView".into());
    e
}
fn limit() -> PeleError {
    let mut e = invalid("TreeView public graph exceeds native work budget");
    e.code = ErrorCode::Limit;
    e
}
fn reference(a: Option<&Atom>) -> Result<u32, PeleError> {
    if let Some(Atom::Ref(i)) = a {
        Ok(*i)
    } else {
        Err(invalid(
            "TreeView active record or collection requires an object",
        ))
    }
}
fn field<'a>(g: &'a ModelGraph, id: u32, key: &str) -> Option<&'a Atom> {
    g.nodes[id as usize]
        .properties()
        .iter()
        .find_map(|p| g.strings[p.key as usize].is(key).then_some(&p.value))
}
fn truthy(g: &ModelGraph, a: Option<&Atom>) -> bool {
    match a {
        None | Some(Atom::Undefined | Atom::Null | Atom::Bool(false)) => false,
        Some(Atom::Number(n)) => *n != 0. && !n.is_nan(),
        Some(Atom::String(i)) => !g.strings[*i as usize].0.is_empty(),
        _ => true,
    }
}
struct Budget {
    scans: usize,
    text: usize,
    values: usize,
}
impl Budget {
    fn scan(&mut self, g: &ModelGraph, id: u32, n: usize) -> Result<(), PeleError> {
        self.scans = self
            .scans
            .saturating_add(g.nodes[id as usize].properties().len().saturating_mul(n));
        if self.scans > 32_000_000 {
            Err(limit())
        } else {
            Ok(())
        }
    }
    fn text(&mut self, g: &ModelGraph, a: Option<&Atom>) -> Result<String, PeleError> {
        let Some(Atom::String(i)) = a else {
            return Err(invalid(
                "TreeView active text coercion for this runtime kind is not implemented",
            ));
        };
        let s = &g.strings[*i as usize];
        self.text = self.text.saturating_add(s.0.len());
        if self.text > 16_000_000 {
            return Err(limit());
        }
        s.well_formed()
            .ok_or_else(|| invalid("TreeView rendering of lone UTF-16 text is not implemented"))
    }
    fn metadata(
        &mut self,
        g: &ModelGraph,
        id: u32,
        key: &str,
        accessible: bool,
    ) -> Result<Option<String>, PeleError> {
        let a = field(g, id, key);
        if matches!(a, None | Some(Atom::Undefined)) || (accessible && !truthy(g, a)) {
            Ok(None)
        } else {
            self.text(g, a).map(Some)
        }
    }
    fn falsy_text(
        &mut self,
        g: &ModelGraph,
        id: u32,
        key: &str,
    ) -> Result<Option<String>, PeleError> {
        let a = field(g, id, key);
        if !truthy(g, a) {
            Ok(None)
        } else {
            self.text(g, a).map(Some)
        }
    }
    fn children(&mut self, g: &ModelGraph, id: u32) -> Result<Vec<u32>, PeleError> {
        let id = reference(field(g, id, "children"))?;
        self.scan(g, id, 1)?;
        let Node::Array { length, properties } = &g.nodes[id as usize] else {
            return Err(invalid("TreeView active children must be Arrays"));
        };
        self.values = self.values.saturating_add(*length as usize);
        if self.values > 1_000_000 {
            return Err(limit());
        }
        let mut indexed = HashMap::new();
        for p in properties {
            let s = &g.strings[p.key as usize];
            if s.0.is_empty() || s.0.len() > 10 {
                continue;
            }
            if let Some(k) = s.well_formed() {
                if let Ok(i) = k.parse::<u32>() {
                    if i < *length && i.to_string() == k {
                        indexed.insert(i, reference(Some(&p.value))?);
                    }
                }
            }
        }
        if indexed.len() != *length as usize {
            return Err(invalid(
                "TreeView sparse active children are not implemented",
            ));
        }
        Ok((0..*length).map(|i| indexed[&i]).collect())
    }
}
fn imported(g: &ModelGraph) -> Result<Cache, PeleError> {
    let mut b = Budget {
        scans: 0,
        text: 0,
        values: 0,
    };
    let r = reference(Some(&g.root))?;
    b.scan(g, r, 5)?;
    let title = b.metadata(g, r, "title", false)?;
    let acc_title = b.metadata(g, r, "accTitle", true)?;
    let acc_descr = b.metadata(g, r, "accDescr", true)?;
    let root = reference(field(g, r, "root"))?;
    let mut m = TreeViewModel {
        diagram_type: DiagramType::TreeView,
        title,
        acc_title,
        acc_descr,
        root: 0,
        count: 0,
        nodes: vec![],
    };
    let mut unknown_types = vec![];
    let mut index = HashMap::new();
    let mut colors = HashMap::new();
    let mut edges: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut sizes: HashMap<u32, (usize, usize)> = HashMap::new();
    let mut units: HashMap<u32, usize> = HashMap::new();
    let mut pending = vec![(root, false)];
    while let Some((id, exit)) = pending.pop() {
        if exit {
            let (count, text) = edges[&id].iter().fold((1usize, units[&id]), |(n, t), c| {
                let (cn, ct) = sizes[c];
                (n.saturating_add(cn), t.saturating_add(ct))
            });
            if count > 1_000_000 || text > 16_000_000 {
                return Err(limit());
            }
            sizes.insert(id, (count, text));
            colors.insert(id, 2);
            continue;
        }
        match colors.get(&id) {
            Some(1) => return Err(invalid("TreeView cyclic active children are not supported")),
            Some(2) => continue,
            _ => {}
        }
        b.scan(g, id, if id == root { 1 } else { 6 })?;
        if index.len() >= 1_000_000 {
            return Err(limit());
        }
        let children = b.children(g, id)?;
        let before = b.text;
        let n = if id == root {
            TreeNode {
                id: 0,
                level: -1,
                name: "/".into(),
                node_type: NodeType::Directory,
                icon: None,
                css_class: None,
                description: None,
                children: vec![],
            }
        } else {
            let name = b.text(g, field(g, id, "name"))?;
            let node_type = if matches!(field(g,id,"nodeType"),Some(Atom::String(i))if g.strings[*i as usize].is("directory"))
            {
                NodeType::Directory
            } else {
                if !matches!(field(g,id,"nodeType"),Some(Atom::String(i))if g.strings[*i as usize].is("file"))
                {
                    unknown_types.push(m.nodes.len());
                }
                NodeType::File
            };
            let icon = b.falsy_text(g, id, "icon")?;
            let css_class = b.metadata(g, id, "cssClass", false)?;
            let description = b.falsy_text(g, id, "description")?;
            TreeNode {
                id: m.nodes.len(),
                level: 0,
                name,
                node_type,
                icon,
                css_class,
                description,
                children: vec![],
            }
        };
        units.insert(id, b.text - before);
        index.insert(id, m.nodes.len());
        m.nodes.push(n);
        colors.insert(id, 1);
        pending.push((id, true));
        for &c in children.iter().rev() {
            pending.push((c, false))
        }
        edges.insert(id, children);
    }
    for (id, children) in edges {
        m.nodes[index[&id]].children = children.into_iter().map(|c| index[&c]).collect();
    }
    m.root = index[&root];
    Ok(Cache {
        model: m,
        unknown_types,
    })
}
fn array(out: &mut GraphWriter, values: Vec<Atom>) -> u32 {
    let properties = values
        .into_iter()
        .enumerate()
        .map(|(i, a)| out.property(&i.to_string(), a))
        .collect::<Vec<_>>();
    out.node(Node::Array {
        length: properties.len() as u32,
        properties,
    })
}
fn optional(out: &mut GraphWriter, s: Option<&str>) -> Atom {
    s.map_or(Atom::Undefined, |s| Atom::String(out.string(s)))
}
impl PublicDiagramModule for TreeView {
    type PublicState = TreeViewPublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let cache = imported(&graph);
        Ok(TreeViewPublicState { graph, cache })
    }
    fn create_public(&self, m: &TreeViewModel) -> Result<TreeViewPublicState, PeleError> {
        if m.root >= m.nodes.len()
            || m.nodes
                .iter()
                .any(|n| n.children.iter().any(|&i| i >= m.nodes.len()))
        {
            return Err(invalid(
                "TreeView typed model contains an invalid node reference",
            ));
        }
        let mut out = GraphWriter::default();
        let ids = m
            .nodes
            .iter()
            .map(|_| out.node(Node::Object { properties: vec![] }))
            .collect::<Vec<_>>();
        let mut filled = vec![];
        for (i, n) in m.nodes.iter().enumerate() {
            let name = out.string(n.name.as_str());
            let kind = out.string(if n.node_type == NodeType::Directory {
                "directory"
            } else {
                "file"
            });
            let mut properties = vec![
                out.property("id", Atom::Number(n.id as f64)),
                out.property("level", Atom::Number(n.level as f64)),
                out.property("name", Atom::String(name)),
                out.property("nodeType", Atom::String(kind)),
            ];
            for (k, v) in [
                ("icon", &n.icon),
                ("cssClass", &n.css_class),
                ("description", &n.description),
            ] {
                if i != m.root || v.is_some() {
                    let value = optional(&mut out, v.as_deref());
                    properties.push(out.property(k, value));
                }
            }
            let children = array(
                &mut out,
                n.children.iter().map(|&c| Atom::Ref(ids[c])).collect(),
            );
            properties.push(out.property("children", Atom::Ref(children)));
            filled.push((ids[i], properties));
        }
        let kind = out.string("treeView");
        let mut properties = vec![out.property("type", Atom::String(kind))];
        for (k, v) in [
            ("title", &m.title),
            ("accTitle", &m.acc_title),
            ("accDescr", &m.acc_descr),
        ] {
            let a = optional(&mut out, v.as_deref());
            properties.push(out.property(k, a));
        }
        properties.extend([
            out.property("root", Atom::Ref(ids[m.root])),
            out.property("count", Atom::Number(m.count as f64)),
        ]);
        let root = out.node(Node::Object { properties });
        let mut graph = out.finish(Atom::Ref(root))?;
        for (id, p) in filled {
            *graph.nodes[id as usize].properties_mut() = p;
        }
        graph.validate_schema(&SCHEMA)?;
        let cache = imported(&graph);
        Ok(TreeViewPublicState { graph, cache })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, s: &TreeViewPublicState) -> Result<ModelGraph, PeleError> {
        Ok(s.graph.clone())
    }
    fn apply_public(&self, s: &mut TreeViewPublicState, g: ModelGraph) -> Result<(), PeleError> {
        s.cache = imported(&g);
        s.graph = g;
        Ok(())
    }
    fn render_public(
        &self,
        s: &mut TreeViewPublicState,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        let cache = s.cache.as_ref().map_err(Clone::clone)?;
        if cache.unknown_types.is_empty() || !super::icons::Config::new(c).show {
            return self.render(&cache.model, c, o, r);
        }
        // Only exact 'file' enables extension/name icon detection in upstream. Unknown
        // runtime types still draw file rows and fall back to the built-in file icon.
        let mut m = cache.model.clone();
        for &i in &cache.unknown_types {
            if m.nodes[i].icon.is_none() {
                m.nodes[i].icon = Some("mermaid-treeview:file".into());
            }
        }
        self.render(&m, c, o, r)
    }
}
