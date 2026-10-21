//! Mutable plain Treemap model codec, Pele 458322a5 (MIT, kepano).
//! Graph identity retains children arrays, shared compiled/class style arrays,
//! ordinary own undefined fields and application properties. Hierarchy import
//! and export use worklists; neither serialization nor recursive Clone is used.
use super::{Treemap, TreemapModel, TreemapNode};
use crate::compat::{
    public::{
        Atom, DetachedPublicRender, DetachedRenderCapability, GraphWriter, ModelGraph, Node,
        PublicDiagramModule,
    },
    DiagramModule, DiagramType, ErrorCode, Object, PeleError, RenderResources, RenderSettings,
    Rendered,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
pub struct TreemapPublicState {
    graph: ModelGraph,
    model: Result<TreemapModel, PeleError>,
}
fn invalid(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("treemap".into());
    e
}
fn reference(a: &Atom) -> Result<u32, PeleError> {
    if let Atom::Ref(id) = a {
        Ok(*id)
    } else {
        Err(invalid("Treemap hierarchy records must be objects"))
    }
}
fn field<'a>(g: &'a ModelGraph, id: u32, key: &str) -> Option<&'a Atom> {
    g.nodes
        .get(id as usize)?
        .properties()
        .iter()
        .find_map(|p| g.strings[p.key as usize].is(key).then_some(&p.value))
}
fn text(g: &ModelGraph, a: &Atom) -> Result<String, PeleError> {
    g.text(a)?
        .well_formed()
        .ok_or_else(|| invalid("Treemap rendering of lone UTF-16 surrogates is not implemented"))
}
fn optional(g: &ModelGraph, a: Option<&Atom>) -> Result<Option<String>, PeleError> {
    match a {
        None | Some(Atom::Undefined | Atom::Null) => Ok(None),
        Some(a) => text(g, a).map(Some),
    }
}
fn array<'a>(g: &'a ModelGraph, id: u32) -> Result<Vec<&'a Atom>, PeleError> {
    let Some(Node::Array { length, properties }) = g.nodes.get(id as usize) else {
        return Err(invalid("Treemap collections must be Arrays"));
    };
    let mut indexes = HashMap::new();
    for p in properties {
        let key = &g.strings[p.key as usize].0;
        if key.is_empty() || key.len() > 10 || (key.len() > 1 && key[0] == 48) {
            continue;
        }
        let mut at = 0u64;
        let mut valid = true;
        for &u in key {
            if !(48..=57).contains(&u) {
                valid = false;
                break;
            }
            at = at * 10 + u64::from(u - 48)
        }
        if valid && at < u64::from(*length) {
            indexes.entry(at as u32).or_insert(&p.value);
        }
    }
    if indexes.len() != *length as usize {
        return Err(invalid(
            "Treemap collection contains a missing array record",
        ));
    }
    Ok((0..*length).map(|i| indexes[&i]).collect())
}
fn styles(
    g: &ModelGraph,
    id: u32,
    cache: &mut HashMap<u32, Arc<Vec<String>>>,
) -> Result<Arc<Vec<String>>, PeleError> {
    if let Some(s) = cache.get(&id) {
        return Ok(s.clone());
    }
    let values = array(g, id)?
        .into_iter()
        .map(|a| text(g, a))
        .collect::<Result<Vec<_>, _>>()?;
    let values = Arc::new(values);
    cache.insert(id, values.clone());
    Ok(values)
}
struct Frame {
    id: u32,
    children: Vec<u32>,
    next: usize,
    node: TreemapNode,
}
fn frame(
    g: &ModelGraph,
    id: u32,
    cache: &mut HashMap<u32, Arc<Vec<String>>>,
) -> Result<Frame, PeleError> {
    let name = text(
        g,
        field(g, id, "name").ok_or_else(|| invalid("Treemap node name must be a string"))?,
    )?;
    let raw_children = field(g, id, "children");
    let (children, section) = match raw_children {
        None | Some(Atom::Undefined) => (vec![], false),
        Some(Atom::Null) => (vec![], true),
        Some(a) => (
            array(g, reference(a)?)?
                .into_iter()
                .map(reference)
                .collect::<Result<Vec<_>, _>>()?,
            true,
        ),
    };
    let css_compiled_styles = match field(g, id, "cssCompiledStyles") {
        None | Some(Atom::Undefined | Atom::Null) => None,
        Some(a) => Some(styles(g, reference(a)?, cache)?),
    };
    let value = match field(g, id, "value") {
        None | Some(Atom::Undefined) => None,
        Some(Atom::Number(v)) => Some(*v),
        _ => Some(f64::NAN),
    };
    let node = TreemapNode {
        name,
        children: if section { Some(vec![]) } else { None },
        value,
        class_selector: optional(g, field(g, id, "classSelector"))?,
        css_compiled_styles,
    };
    Ok(Frame {
        id,
        children,
        next: 0,
        node,
    })
}
fn imported(g: &ModelGraph) -> Result<TreemapModel, PeleError> {
    let roots_atom = g
        .field("roots")?
        .ok_or_else(|| invalid("Treemap roots must be an Array"))?;
    let root_ids = array(g, reference(roots_atom)?)?
        .into_iter()
        .map(reference)
        .collect::<Result<Vec<_>, _>>()?;
    let mut cache = HashMap::new();
    let mut roots = vec![];
    let mut active = HashSet::new();
    let mut work = 0usize;
    for id in root_ids {
        work += 1;
        if work > 1_000_000 {
            let mut e = invalid("Treemap hierarchy expansion exceeds native work budget");
            e.code = ErrorCode::Limit;
            return Err(e);
        }
        let mut pending = vec![frame(g, id, &mut cache)?];
        active.insert(id);
        while let Some(parent) = pending.last_mut() {
            if parent.next < parent.children.len() {
                let id = parent.children[parent.next];
                parent.next += 1;
                if !active.insert(id) {
                    return Err(invalid("Treemap hierarchy contains a cycle"));
                }
                work += 1;
                if work > 1_000_000 {
                    let mut e = invalid("Treemap hierarchy expansion exceeds native work budget");
                    e.code = ErrorCode::Limit;
                    return Err(e);
                }
                pending.push(frame(g, id, &mut cache)?);
            } else {
                let child = pending.pop().unwrap();
                active.remove(&child.id);
                if let Some(parent) = pending.last_mut() {
                    parent.node.children.as_mut().unwrap().push(child.node)
                } else {
                    roots.push(child.node)
                }
            }
        }
    }
    // The renderer consumes node.cssCompiledStyles, never model.classes.
    // Preserve the complete Map in the arena rather than traverse/copy arrays
    // that replacing the Map has detached from the rendered hierarchy.
    Ok(TreemapModel {
        diagram_type: DiagramType::Treemap,
        title: optional(g, g.field("title")?)?,
        acc_title: optional(g, g.field("accTitle")?)?,
        acc_descr: optional(g, g.field("accDescr")?)?,
        roots,
        classes: vec![],
    })
}
fn array_node(out: &mut GraphWriter, values: impl Iterator<Item = Atom>) -> u32 {
    let properties: Vec<_> = values
        .enumerate()
        .map(|(i, a)| out.property(&i.to_string(), a))
        .collect();
    out.node(Node::Array {
        length: properties.len() as u32,
        properties,
    })
}
fn string_array(out: &mut GraphWriter, values: &[String]) -> u32 {
    let values: Vec<_> = values
        .iter()
        .map(|v| Atom::String(out.string(v.as_str())))
        .collect();
    array_node(out, values.into_iter())
}
impl PublicDiagramModule for Treemap {
    type PublicState = TreemapPublicState;
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let model = imported(&graph);
        Ok(TreemapPublicState { graph, model })
    }
    fn create_public(&self, m: &TreemapModel) -> Result<Self::PublicState, PeleError> {
        let mut out = GraphWriter::default();
        let mut flat = vec![];
        let mut links: Vec<Vec<usize>> = vec![];
        let roots: Vec<_> = (0..m.roots.len()).collect();
        for n in &m.roots {
            flat.push(n);
            links.push(vec![])
        }
        let mut at = 0;
        while at < flat.len() {
            if let Some(children) = &flat[at].children {
                for child in children {
                    let i = flat.len();
                    links[at].push(i);
                    flat.push(child);
                    links.push(vec![])
                }
            }
            at += 1
        }
        let mut class_arrays = HashMap::new();
        let mut class_entries = vec![];
        for (name, styles) in &m.classes {
            let id = string_array(&mut out, styles);
            class_arrays.insert(name.as_str(), (id, styles));
            let key = out.string(name.as_str());
            class_entries.push((Atom::String(key), Atom::Ref(id)))
        }
        let mut style_arrays = HashMap::new();
        let mut compiled = vec![];
        for n in &flat {
            let id = if let Some(styles) = &n.css_compiled_styles {
                let pointer = Arc::as_ptr(styles) as usize;
                if let Some(&id) = style_arrays.get(&pointer) {
                    Some(id)
                } else {
                    let authored = n
                        .class_selector
                        .as_deref()
                        .and_then(|name| class_arrays.get(name))
                        .filter(|(_, value)| value.as_slice() == styles.as_slice())
                        .map(|(id, _)| *id);
                    let id = authored.unwrap_or_else(|| string_array(&mut out, styles));
                    style_arrays.insert(pointer, id);
                    Some(id)
                }
            } else {
                None
            };
            compiled.push(id)
        }
        let classes = out.node(Node::Map {
            entries: class_entries,
            properties: vec![],
        });
        // Reserve references arithmetically, then emit arrays before the records they
        // point at. GraphWriter permits forward references and validates at finish.
        let array_count = 1 + flat.iter().filter(|n| n.children.is_some()).count();
        let base = classes + 1 + array_count as u32;
        let root_array = array_node(
            &mut out,
            roots.into_iter().map(|i| Atom::Ref(base + i as u32)),
        );
        let mut child_arrays = vec![];
        for (i, n) in flat.iter().enumerate() {
            child_arrays.push(if n.children.is_some() {
                Some(array_node(
                    &mut out,
                    links[i].iter().map(|&k| Atom::Ref(base + k as u32)),
                ))
            } else {
                None
            })
        }
        for (i, n) in flat.iter().enumerate() {
            let name = out.string(n.name.as_str());
            let mut properties = vec![
                out.property("name", Atom::String(name)),
                out.property(
                    "children",
                    child_arrays[i].map_or(Atom::Undefined, Atom::Ref),
                ),
            ];
            let selector = n
                .class_selector
                .as_ref()
                .map_or(Atom::Undefined, |s| Atom::String(out.string(s.as_str())));
            properties.push(out.property("classSelector", selector));
            if let Some(styles) = compiled[i] {
                properties.push(out.property("cssCompiledStyles", Atom::Ref(styles)))
            }
            if let Some(v) = n.value {
                properties.push(out.property("value", Atom::Number(v)))
            }
            out.node(Node::Object { properties });
        }
        let kind = out.string("treemap");
        let mut properties = vec![out.property("type", Atom::String(kind))];
        for (key, value) in [
            ("title", &m.title),
            ("accTitle", &m.acc_title),
            ("accDescr", &m.acc_descr),
        ] {
            let value = value
                .as_ref()
                .map_or(Atom::Undefined, |v| Atom::String(out.string(v.as_str())));
            properties.push(out.property(key, value));
        }
        properties.push(out.property("roots", Atom::Ref(root_array)));
        properties.push(out.property("classes", Atom::Ref(classes)));
        let root = out.node(Node::Object { properties });
        let graph = out.finish(Atom::Ref(root))?;
        let model = imported(&graph);
        Ok(TreemapPublicState { graph, model })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, s: &Self::PublicState) -> Result<ModelGraph, PeleError> {
        Ok(s.graph.clone())
    }
    fn apply_public(&self, s: &mut Self::PublicState, g: ModelGraph) -> Result<(), PeleError> {
        g.validate()?;
        let model = imported(&g);
        s.graph = g;
        s.model = model;
        Ok(())
    }
    fn detached_public_render_capability(&self) -> Option<DetachedRenderCapability> {
        Some(DetachedRenderCapability::treemap())
    }
    fn detached_public_render(&self, state: &Self::PublicState) -> Option<DetachedPublicRender> {
        Some(DetachedPublicRender::treemap(
            super::TreemapDetachedInput::new(state.model.as_ref().err().cloned()),
        ))
    }
    fn render_public(
        &self,
        s: &mut Self::PublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render(s.model.as_ref().map_err(Clone::clone)?, c, o, h)
    }
}
