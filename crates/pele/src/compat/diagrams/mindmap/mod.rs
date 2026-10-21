//! Pele diagrams/mindmap/{index,db,parser}.ts, 458322a5 (MIT, kepano).
mod live_phase;
pub(crate) use live_phase::{render_detached, MindmapDetachedInput};
mod render;
use super::common::outline::{parse_outline, OutlineDb};
use crate::compat::{
    DiagramModule, DiagramType, ErrorCode, ModuleDescriptor, Object, ParseContext, PeleError,
    RenderResources, RenderSettings, Rendered,
};
use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{Serialize, Serializer};
#[derive(Clone, Debug)]
pub struct MindmapNode {
    pub id: usize,
    pub node_id: String,
    pub level: i64,
    pub descr: String,
    pub node_type: u8,
    pub children: Vec<usize>,
    pub is_root: bool,
    pub section: Option<usize>,
    pub class: Option<String>,
    pub icon: Option<String>,
    /// Parser provenance for the observable own-key insertion order.
    pub(crate) public_property_order: Vec<String>,
}
#[derive(Clone, Debug)]
pub struct MindmapModel {
    pub title: Option<String>,
    pub nodes: Vec<MindmapNode>,
    pub base_level: i64,
    pub open: Vec<usize>,
}
struct NodeView<'a>(&'a MindmapModel, usize, bool);
struct Children<'a>(&'a MindmapModel, &'a [usize]);
impl Serialize for Children<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(self.1.len()))?;
        for &i in self.1 {
            seq.serialize_element(&NodeView(self.0, i, true))?;
        }
        seq.end()
    }
}
impl Serialize for NodeView<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let n = &self.0.nodes[self.1];
        let mut out = s.serialize_struct(
            "MindmapNode",
            7 + usize::from(n.section.is_some())
                + usize::from(n.class.is_some())
                + usize::from(n.icon.is_some()),
        )?;
        out.serialize_field("id", &n.id)?;
        out.serialize_field("nodeId", &n.node_id)?;
        out.serialize_field("level", &n.level)?;
        out.serialize_field("descr", &n.descr)?;
        out.serialize_field("type", &n.node_type)?;
        out.serialize_field(
            "children",
            &Children(self.0, if self.2 { &n.children } else { &[] }),
        )?;
        out.serialize_field("isRoot", &n.is_root)?;
        if let Some(section) = n.section {
            out.serialize_field("section", &section)?;
        }
        if let Some(class) = &n.class {
            out.serialize_field("class", class)?;
        }
        if let Some(icon) = &n.icon {
            out.serialize_field("icon", icon)?;
        }
        out.end()
    }
}
impl Serialize for MindmapModel {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        projection(self).map_err(serde::ser::Error::custom)?;
        let mut out = s.serialize_struct("MindmapModel", 4 + usize::from(self.title.is_some()))?;
        out.serialize_field("type", "mindmap")?;
        if let Some(title) = &self.title {
            out.serialize_field("title", title)?;
        }
        let ids: Vec<_> = (0..self.nodes.len()).collect();
        out.serialize_field("nodes", &Children(self, &ids))?;
        out.serialize_field("baseLevel", &self.base_level)?;
        out.serialize_field("open", &Children(self, &self.open))?;
        out.end()
    }
}
// Keep typed models/rendering independent of recursive JSON transport. The public
// projection repeats shared descendant objects, so size is computed bottom-up first.
fn projection(model: &MindmapModel) -> Result<(), PeleError> {
    const LIMIT: usize = 16 * 1024 * 1024;
    let mut depth = vec![0usize; model.nodes.len()];
    for node in &model.nodes {
        if depth[node.id] >= 48 {
            return Err(PeleError::new(
                "Mindmap model exceeds safe JSON projection depth",
                ErrorCode::Limit,
            ));
        }
        for &child in &node.children {
            depth[child] = depth[node.id] + 1;
        }
    }
    let mut sizes = vec![0usize; model.nodes.len()];
    for i in (0..model.nodes.len()).rev() {
        let node = &model.nodes[i];
        let own = serde_json::to_string(&NodeView(model, i, false))
            .expect("flat mindmap node serializes")
            .len();
        sizes[i] = node
            .children
            .iter()
            .fold(own + node.children.len().saturating_sub(1), |n, &child| {
                n.saturating_add(sizes[child]).min(LIMIT + 1)
            });
    }
    let mut header =
        serde_json::json!({"type":"mindmap","nodes":[],"baseLevel":model.base_level,"open":[]});
    if let Some(title) = &model.title {
        header["title"] = serde_json::Value::String(title.clone());
    }
    let mut total = header.to_string().len()
        + model.nodes.len().saturating_sub(1)
        + model.open.len().saturating_sub(1);
    for &size in &sizes {
        total = total.saturating_add(size);
        if total > LIMIT {
            return Err(PeleError::new(
                "Module model is too large for JSON transport",
                ErrorCode::Limit,
            ));
        }
    }
    for &i in &model.open {
        total = total.saturating_add(sizes[i]);
        if total > LIMIT {
            return Err(PeleError::new(
                "Module model is too large for JSON transport",
                ErrorCode::Limit,
            ));
        }
    }
    Ok(())
}
struct Db {
    model: MindmapModel,
    base: i64,
    open: Vec<usize>,
}
impl OutlineDb for Db {
    fn add_node(
        &mut self,
        level: usize,
        id: String,
        descr: String,
        node_type: u8,
        _shape_data: Option<String>,
    ) -> Result<(), PeleError> {
        let is_root = self.model.nodes.is_empty();
        if is_root {
            self.base = level as i64;
        }
        let level = level as i64 - self.base;
        while self
            .open
            .last()
            .is_some_and(|&i| self.model.nodes[i].level >= level)
        {
            self.open.pop();
        }
        let parent = self.open.last().copied();
        if parent.is_none() && !is_root {
            let mut error = PeleError::new(
                format!("There can be only one root. No parent could be found for (\"{descr}\")"),
                ErrorCode::Semantic,
            );
            error.diagram_type = Some("mindmap".into());
            return Err(error);
        }
        let id_number = self.model.nodes.len();
        let section = parent.and_then(|p| {
            if self.model.nodes[p].is_root {
                Some(self.model.nodes[p].children.len() % 11)
            } else {
                self.model.nodes[p].section
            }
        });
        self.model.nodes.push(MindmapNode {
            id: id_number,
            node_id: id,
            level,
            descr,
            node_type,
            children: Vec::new(),
            is_root,
            section,
            class: None,
            icon: None,
            public_property_order: [
                "id", "nodeId", "level", "descr", "type", "children", "isRoot",
            ]
            .into_iter()
            .chain(section.map(|_| "section"))
            .map(str::to_owned)
            .collect(),
        });
        if let Some(p) = parent {
            self.model.nodes[p].children.push(id_number);
        }
        self.open.push(id_number);
        Ok(())
    }
    fn decorate_node(&mut self, icon: Option<String>, class: Option<String>) {
        if let Some(n) = self.model.nodes.last_mut() {
            if let Some(icon) = icon.filter(|s| !s.is_empty()) {
                if n.icon.is_none() {
                    n.public_property_order.push("icon".into());
                }
                n.icon = Some(icon);
            }
            if let Some(class) = class.filter(|s| !s.is_empty()) {
                if n.class.is_none() {
                    n.public_property_order.push("class".into());
                }
                n.class = Some(class);
            }
        }
    }
}
#[derive(Default)]
pub struct Mindmap;
impl DiagramModule for Mindmap {
    type Model = MindmapModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Mindmap)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<Self::Model, PeleError> {
        let mut db = Db {
            model: MindmapModel::empty(c.title),
            base: 0,
            open: Vec::new(),
        };
        parse_outline(c.source, &mut db, false)?;
        db.model.base_level = db.base;
        db.model.open = db.open;
        Ok(db.model)
    }
    fn validate_model_projection(&self, model: &Self::Model) -> Result<(), PeleError> {
        projection(model)
    }
    fn render(
        &self,
        m: &Self::Model,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(m, c, o, r)
    }
}

mod public_model;
pub use public_model::MindmapPublicState;

impl MindmapModel {
    /// Empty class state shared by parser population and native construction.
    pub(super) fn empty(title: Option<&str>) -> Self {
        Self {
            title: title.map(str::to_owned),
            nodes: Vec::new(),
            base_level: 0,
            open: Vec::new(),
        }
    }
}
