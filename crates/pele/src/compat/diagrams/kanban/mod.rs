//! Pele kanban/{index,db}.ts, 458322a5 (MIT, kepano).
mod render;
use super::common::outline::{parse_outline, OutlineDb};
use crate::compat::{
    source::yaml, DiagramModule, DiagramType, ErrorCode, ModuleDescriptor, Object, ParseContext,
    PeleError, RenderResources, RenderSettings, Rendered,
};
use serde::Serialize;
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KanbanNode {
    pub id: String,
    pub level: usize,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css_classes: Option<String>,
    /// Observable JS own-key insertion order, including section items.
    #[serde(skip)]
    pub(crate) public_property_order: Vec<String>,
}
#[derive(Clone, Debug, Serialize)]
pub struct KanbanSection {
    #[serde(flatten)]
    pub node: KanbanNode,
    pub items: Vec<KanbanNode>,
}
#[derive(Clone, Debug)]
pub struct KanbanModel {
    pub diagram_type: DiagramType,
    pub title: Option<String>,
    pub sections: Vec<KanbanSection>,
    last: Option<(usize, Option<usize>)>,
    section_level: usize,
    misplaced: Option<(usize, usize)>,
}
impl Serialize for KanbanModel {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", &self.diagram_type)?;
        if let Some(title) = &self.title {
            map.serialize_entry("title", title)?
        }
        map.serialize_entry("sections", &self.sections)?;
        if let Some((section, item)) = self.last {
            if let Some(item) = item {
                map.serialize_entry("last", &self.sections[section].items[item])?
            } else {
                map.serialize_entry("last", &self.sections[section])?
            }
        }
        map.serialize_entry("sectionLevel", &self.section_level)?;
        if let Some((section, item)) = self.misplaced {
            map.serialize_entry("misplaced", &self.sections[section].items[item])?
        }
        map.end()
    }
}

fn error(message: String, code: ErrorCode) -> PeleError {
    let mut e = PeleError::new(message, code);
    e.diagram_type = Some("kanban".into());
    e
}
impl OutlineDb for KanbanModel {
    fn add_node(
        &mut self,
        level: usize,
        id: String,
        descr: String,
        _node_type: u8,
        shape_data: Option<String>,
    ) -> Result<(), PeleError> {
        let mut node = KanbanNode {
            id,
            level,
            label: descr,
            icon: None,
            assigned: None,
            ticket: None,
            priority: None,
            css_classes: None,
            public_property_order: vec!["id".into(), "level".into(), "label".into()],
        };
        if let Some(data) = shape_data {
            let raw = if data.contains('\n') {
                format!("{data}\n")
            } else {
                format!("{{\n{data}\n}}")
            };
            let doc = yaml::parse(&raw).map_err(|e| {
                error(
                    format!("Metadata of \"{}\" is not valid YAML. {e}", node.id),
                    ErrorCode::Syntax,
                )
            })?;
            if let Some(doc) = doc.as_object() {
                let shape = doc
                    .get("shape")
                    .filter(|v| v.truthy())
                    .map(|v| v.js_string())
                    .unwrap_or_default();
                if shape != shape.to_lowercase() || shape.contains('_') {
                    return Err(error(
                        format!("No such shape: {shape}. Shape names should be lowercase."),
                        ErrorCode::Semantic,
                    ));
                }
                if let Some(value) = doc.get("label").filter(|v| v.truthy()) {
                    node.label = value.js_string()
                }
                for (key, slot) in [
                    ("icon", &mut node.icon),
                    ("assigned", &mut node.assigned),
                    ("ticket", &mut node.ticket),
                    ("priority", &mut node.priority),
                ] {
                    if let Some(value) = doc.get(key).filter(|v| v.truthy()) {
                        *slot = Some(value.js_string());
                        node.public_property_order.push(key.into())
                    }
                }
            }
        }
        if let Some((section, item)) = self.misplaced {
            let label = &self.sections[section].items[item].label;
            return Err(error(
                format!("Items without section detected, found section (\"{label}\")"),
                ErrorCode::Semantic,
            ));
        }
        if self.sections.is_empty() {
            self.section_level = level
        }
        if level == self.section_level {
            node.public_property_order.push("items".into());
            self.last = Some((self.sections.len(), None));
            self.sections.push(KanbanSection {
                node,
                items: vec![],
            })
        } else {
            let section_index = self.sections.len() - 1;
            let section = self.sections.last_mut().unwrap();
            if level < self.section_level {
                self.misplaced = Some((section_index, section.items.len()))
            }
            self.last = Some((section_index, Some(section.items.len())));
            section.items.push(node)
        }
        Ok(())
    }
    fn decorate_node(&mut self, icon: Option<String>, class: Option<String>) {
        if let Some((section, item)) = self.last {
            let section = &mut self.sections[section];
            let node = if let Some(item) = item {
                &mut section.items[item]
            } else {
                &mut section.node
            };
            if let Some(icon) = icon.filter(|s| !s.is_empty()) {
                if node.icon.is_none() {
                    node.public_property_order.push("icon".into());
                }
                node.icon = Some(icon)
            }
            if let Some(class) = class.filter(|s| !s.is_empty()) {
                if node.css_classes.is_none() {
                    node.public_property_order.push("cssClasses".into());
                }
                node.css_classes = Some(class)
            }
        }
    }
}
#[derive(Default)]
pub struct Kanban;
impl DiagramModule for Kanban {
    type Model = KanbanModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Kanban)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<Self::Model, PeleError> {
        let mut m = KanbanModel::empty(c.title);
        parse_outline(c.source, &mut m, true)?;
        Ok(m)
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
pub use public_model::KanbanPublicState;

impl KanbanModel {
    /// Empty class state shared by parser population and native construction.
    pub(super) fn empty(title: Option<&str>) -> Self {
        Self {
            diagram_type: DiagramType::Kanban,
            title: title.map(str::to_owned),
            sections: vec![],
            last: None,
            section_level: 0,
            misplaced: None,
        }
    }
}
