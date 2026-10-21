//! Captured collection handles, live fields at the existing architecture phases.
use super::{model::*, render::PhaseInput};
use crate::compat::{
    public::{LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
use std::collections::HashMap;
pub(crate) struct ArchitectureDetachedInput {
    limit: Option<PeleError>,
}
impl ArchitectureDetachedInput {
    pub(super) fn new(error: Option<&PeleError>) -> Self {
        Self {
            limit: error.filter(|e| e.code == ErrorCode::Limit).cloned(),
        }
    }
}
fn error(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("architecture".into());
    e
}
fn limit() -> PeleError {
    let mut e = error("Architecture public model exceeds native graph/work budget");
    e.code = ErrorCode::Limit;
    e
}
struct Host<'a, R: LiveRenderRead> {
    read: &'a mut R,
    nodes: Vec<R::Value>,
    groups: Vec<R::Value>,
    units: usize,
    work: usize,
    text: HashMap<(u8, usize, &'static str), usize>,
}
impl<R: LiveRenderRead> Host<'_, R> {
    fn tick(&mut self, n: usize) -> Result<(), PublicError> {
        self.work = self.work.saturating_add(n);
        if self.work > 8_000_000 {
            Err(limit().into())
        } else {
            Ok(())
        }
    }
    fn string(
        &mut self,
        v: &R::Value,
        slot: (u8, usize, &'static str),
    ) -> Result<String, PublicError> {
        let s = self.read.require_string(
            v,
            LiveStringUse {
                method: "includes",
                expression_receiver: "t",
            },
        )?;
        let old = self.text.entry(slot).or_default();
        if s.0.len() > *old {
            self.units = self.units.saturating_add(s.0.len() - *old);
            *old = s.0.len();
        }
        if self.units > 16_000_000 {
            return Err(limit().into());
        }
        s.well_formed()
            .ok_or_else(|| error("Architecture rendering requires well-formed UTF16 text").into())
    }
    fn field(
        &mut self,
        v: &R::Value,
        key: &'static str,
        kind: u8,
        i: usize,
    ) -> Result<String, PublicError> {
        let x = self.read.get(v, key)?;
        self.string(&x, (kind, i, key))
    }
    fn optional(
        &mut self,
        v: &R::Value,
        key: &'static str,
        kind: u8,
        i: usize,
    ) -> Result<Option<String>, PublicError> {
        let x = self.read.get(v, key)?;
        if matches!(
            self.read.kind(&x),
            LiveValueKind::Undefined | LiveValueKind::Null
        ) {
            Ok(None)
        } else {
            self.string(&x, (kind, i, key)).map(Some)
        }
    }
    fn truthy_text(
        &mut self,
        v: &R::Value,
        key: &'static str,
        kind: u8,
        i: usize,
    ) -> Result<Option<String>, PublicError> {
        let x = self.read.get(v, key)?;
        if !self.read.truthy(&x) {
            Ok(None)
        } else {
            self.string(&x, (kind, i, key)).map(Some)
        }
    }
    fn boolean(&mut self, v: &R::Value, key: &str) -> Result<bool, PublicError> {
        let x = self.read.get(v, key)?;
        Ok(self.read.truthy(&x))
    }
    fn length(&mut self, v: &R::Value) -> Result<usize, PublicError> {
        let x = self.read.get(v, "length")?;
        let n = self
            .read
            .number(&x)
            .ok_or_else(|| error("Architecture collection must be an Array"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > 1_000_000. {
            return Err(limit().into());
        }
        Ok(n as usize)
    }
    fn root_field(&mut self, key: &str) -> Result<R::Value, PublicError> {
        let root = self.read.root();
        self.read.get(&root, key)
    }
    fn edge_at(&mut self, i: usize, full: bool, label: bool) -> Result<ArchEdge, PublicError> {
        self.tick(1)?;
        let edges = self.root_field("edges")?;
        let v = self.read.index(&edges, i)?;
        let lhs_id = self.field(&v, "lhsId", 2, i)?;
        let rhs_id = self.field(&v, "rhsId", 2, i)?;
        let lhs_dir = self
            .optional(&v, "lhsDir", 2, i)?
            .and_then(|s| s.chars().next())
            .unwrap_or('B');
        let rhs_dir = self
            .optional(&v, "rhsDir", 2, i)?
            .and_then(|s| s.chars().next())
            .unwrap_or('B');
        Ok(ArchEdge {
            lhs_id,
            rhs_id,
            lhs_dir,
            rhs_dir,
            lhs_into: if full {
                self.boolean(&v, "lhsInto")?
            } else {
                false
            },
            rhs_into: if full {
                self.boolean(&v, "rhsInto")?
            } else {
                false
            },
            lhs_group: if full {
                self.boolean(&v, "lhsGroup")?
            } else {
                false
            },
            rhs_group: if full {
                self.boolean(&v, "rhsGroup")?
            } else {
                false
            },
            title: if label {
                self.optional(&v, "title", 2, i)?
            } else {
                None
            },
        })
    }
}
impl<R: LiveRenderRead> PhaseInput for Host<'_, R> {
    type Error = PublicError;
    fn begin(&mut self) -> Result<super::render::Geometry, PublicError> {
        self.units = 0;
        self.work = 0;
        self.text.clear();
        let v = self.root_field("nodes")?;
        self.nodes = self.read.snapshot_map_values(&v, 1_000_000)?;
        let v = self.root_field("groups")?;
        self.groups = self.read.snapshot_map_values(&v, 1_000_000)?;
        let mut m = ArchitectureModel::empty(None);
        for i in 0..self.nodes.len() {
            self.tick(1)?;
            let v = self.nodes[i].clone();
            let id = self.field(&v, "id", 0, i)?;
            let parent = self.optional(&v, "in", 0, i)?;
            m.nodes.push((
                id.clone(),
                ArchNode {
                    id,
                    node_type: String::new(),
                    parent,
                    title: None,
                    icon: None,
                    icon_text: None,
                },
            ));
        }
        for i in 0..self.groups.len() {
            self.tick(1)?;
            let v = self.groups[i].clone();
            let id = self.field(&v, "id", 1, i)?;
            let parent = self.optional(&v, "in", 1, i)?;
            m.groups.push((
                id.clone(),
                ArchGroup {
                    id,
                    parent,
                    title: None,
                    icon: None,
                },
            ));
        }
        let edges = self.root_field("edges")?;
        let count = self.length(&edges)?;
        for i in 0..count {
            m.edges.push(self.edge_at(i, false, false)?);
        }
        let hints = self.root_field("layoutHints")?;
        let count = self.length(&hints)?;
        for i in 0..count {
            self.tick(1)?;
            let v = self.read.index(&hints, i)?;
            let direction = self.field(&v, "direction", 3, i)?;
            let members = self.read.get(&v, "members")?;
            let count = self.length(&members)?;
            let mut values = Vec::with_capacity(count);
            for j in 0..count {
                self.tick(1)?;
                let x = self.read.index(&members, j)?;
                values.push(self.string(&x, (4, self.work, "member"))?);
            }
            m.layout_hints.push(ArchLayoutHint {
                direction,
                members: values,
            });
        }
        let count = m.nodes.len().saturating_add(m.groups.len());
        if count.saturating_mul(
            count
                .saturating_add(m.edges.len())
                .saturating_add(
                    m.layout_hints
                        .iter()
                        .map(|h| h.members.len())
                        .sum::<usize>(),
                )
                .saturating_add(1),
        ) > 32_000_000
        {
            return Err(limit().into());
        }
        super::public_model::ancestry(&m.groups)?;
        Ok(super::render::geometry(&m))
    }
    fn node_label(&mut self, i: usize) -> Result<Option<String>, PublicError> {
        let v = self.nodes[i].clone();
        let kind = self.field(&v, "type", 0, i)?;
        if kind == "service" {
            self.optional(&v, "title", 0, i)
        } else {
            Ok(None)
        }
    }
    fn edge_count(&mut self) -> Result<usize, PublicError> {
        let v = self.root_field("edges")?;
        self.length(&v)
    }
    fn edge(&mut self, i: usize, label: bool) -> Result<ArchEdge, PublicError> {
        self.edge_at(i, true, label)
    }
    fn node_type(&mut self, i: usize) -> Result<String, PublicError> {
        let v = self.nodes[i].clone();
        self.field(&v, "type", 0, i)
    }
    fn group_title(&mut self, i: usize) -> Result<Option<String>, PublicError> {
        let v = self.groups[i].clone();
        self.optional(&v, "title", 1, i)
    }
    fn group_icon(&mut self, i: usize) -> Result<Option<String>, PublicError> {
        let v = self.groups[i].clone();
        self.truthy_text(&v, "icon", 1, i)
    }
    fn group_id(&mut self, i: usize) -> Result<String, PublicError> {
        let v = self.groups[i].clone();
        self.field(&v, "id", 1, i)
    }
    fn node_paint(&mut self, i: usize) -> Result<ArchNode, PublicError> {
        let v = self.nodes[i].clone();
        let id = self.field(&v, "id", 0, i)?;
        let node_type = self.field(&v, "type", 0, i)?;
        let icon = if node_type == "junction" {
            None
        } else {
            self.truthy_text(&v, "icon", 0, i)?
        };
        let icon_text = if node_type == "junction" || icon.is_some() {
            None
        } else {
            self.truthy_text(&v, "iconText", 0, i)?
        };
        Ok(ArchNode {
            id,
            node_type,
            icon,
            icon_text,
            title: None,
            parent: None,
        })
    }
    fn title(&mut self) -> Result<Option<String>, PublicError> {
        let root = self.read.root();
        self.optional(&root, "title", 5, 0)
    }
    fn accessibility(&mut self) -> Result<(Option<String>, Option<String>), PublicError> {
        let root = self.read.root();
        let a = self.truthy_text(&root, "accTitle", 5, 0)?;
        let b = self.truthy_text(&root, "accDescr", 5, 0)?;
        Ok((a, b))
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    input: &ArchitectureDetachedInput,
    read: &mut R,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    if let Some(error) = &input.limit {
        return Err(error.clone().into());
    }
    super::render::render_phase(
        &mut Host {
            read,
            nodes: vec![],
            groups: vec![],
            units: 0,
            work: 0,
            text: HashMap::new(),
        },
        config,
        options,
        resources,
    )
}
