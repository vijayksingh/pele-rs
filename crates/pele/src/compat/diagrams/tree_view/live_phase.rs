//! TreeView host handles remain owned by one detached render job. Rust chooses
//! each read phase; geometry, row traversal and icon policy remain shared.
use super::{icons::IconInput, render::PhaseInput};
use crate::compat::{
    public::{JsString, LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
use std::collections::HashMap;
pub(crate) struct TreeViewDetachedInput {
    limit: Option<PeleError>,
}
impl TreeViewDetachedInput {
    pub(super) fn new(error: Option<PeleError>) -> Self {
        Self {
            limit: error.filter(|e| e.code == ErrorCode::Limit),
        }
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    seed: &TreeViewDetachedInput,
    c: &Object,
    o: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    if let Some(e) = &seed.limit {
        return Err(e.clone().into());
    }
    super::render::render_phases(
        &mut HostPhase {
            read,
            text: HashMap::new(),
            units: 0,
        },
        c,
        o,
        resources,
    )
}
struct HostPhase<'a, R> {
    read: &'a mut R,
    text: HashMap<(u64, &'static str), usize>,
    units: usize,
}
fn invalid(message: &str) -> PublicError {
    PeleError::new(message, ErrorCode::Semantic).into()
}
fn limit() -> PublicError {
    PeleError::new(
        "TreeView public graph exceeds native work budget",
        ErrorCode::Limit,
    )
    .into()
}
impl<R: LiveRenderRead> HostPhase<'_, R> {
    fn root_field(&mut self, field: &str) -> Result<R::Value, PublicError> {
        let root = self.read.root();
        self.read.get(&root, field)
    }
    fn strict(
        &mut self,
        owner: &R::Value,
        field: &'static str,
        value: &R::Value,
        method: &'static str,
        receiver: &'static str,
    ) -> Result<String, PublicError> {
        let value = self.read.require_string(
            value,
            LiveStringUse {
                method,
                expression_receiver: receiver,
            },
        )?;
        let identity = self
            .read
            .object_identity(owner)?
            .ok_or_else(|| invalid("TreeView node must be an object"))?;
        let old = self.text.get(&(identity, field)).copied().unwrap_or(0);
        let length = value.0.len();
        if length > old {
            self.units = self.units.checked_add(length - old).ok_or_else(limit)?;
            if self.units > 16_000_000 {
                return Err(limit());
            }
            self.text.insert((identity, field), length);
        }
        value
            .well_formed()
            .ok_or_else(|| invalid("UTF-16 TreeView text rendering is not implemented"))
    }
    fn same_text(
        &mut self,
        owner: &R::Value,
        field: &str,
        expected: &str,
    ) -> Result<bool, PublicError> {
        let v = self.read.get(owner, field)?;
        Ok(self
            .read
            .string_units(&v)?
            .is_some_and(|s| s == JsString::from(expected)))
    }
}
impl<R: LiveRenderRead> IconInput for HostPhase<'_, R> {
    type Node = R::Value;
    type Error = PublicError;
    fn icon(&mut self, node: &R::Value) -> Result<Option<String>, PublicError> {
        let v = self.read.get(node, "icon")?;
        if !self.read.truthy(&v) {
            return Ok(None);
        }
        self.strict(node, "icon", &v, "includes", "t").map(Some)
    }
    fn name(&mut self, node: &R::Value) -> Result<String, PublicError> {
        let v = self.read.get(node, "name")?;
        self.strict(node, "name", &v, "replace", "e")
    }
    fn file(&mut self, node: &R::Value) -> Result<bool, PublicError> {
        self.same_text(node, "nodeType", "file")
    }
    fn directory(&mut self, node: &R::Value) -> Result<bool, PublicError> {
        self.same_text(node, "nodeType", "directory")
    }
}
impl<R: LiveRenderRead> PhaseInput for HostPhase<'_, R> {
    type Children = R::Value;
    fn root_children(&mut self) -> Result<R::Value, PublicError> {
        let root = self.root_field("root")?;
        self.read.get(&root, "children")
    }
    fn children(&mut self, node: &R::Value) -> Result<R::Value, PublicError> {
        self.read.get(node, "children")
    }
    fn length(&mut self, children: &R::Value) -> Result<usize, PublicError> {
        let length = self.read.get(children, "length")?;
        let n = self
            .read
            .number(&length)
            .ok_or_else(|| invalid("Invalid TreeView children length"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(invalid("Invalid TreeView children length"));
        }
        if n > 1_000_000. {
            return Err(limit());
        }
        Ok(n as usize)
    }
    fn has_index(&mut self, children: &R::Value, i: usize) -> Result<bool, PublicError> {
        self.read.has_index(children, i)
    }
    fn index(&mut self, children: &R::Value, i: usize) -> Result<R::Value, PublicError> {
        self.read.index(children, i)
    }
    fn identity(&mut self, node: &R::Value) -> Result<u64, PublicError> {
        self.read
            .object_identity(node)?
            .ok_or_else(|| invalid("TreeView node must be an object"))
    }
    fn title(&mut self) -> Result<Option<String>, PublicError> {
        let root = self.read.root();
        let v = self.read.get(&root, "title")?;
        if self.read.kind(&v) == LiveValueKind::Undefined {
            return Ok(None);
        }
        self.strict(&root, "title", &v, "includes", "t").map(Some)
    }
    fn description(
        &mut self,
        node: &R::Value,
        reread: bool,
    ) -> Result<Option<String>, PublicError> {
        let mut v = self.read.get(node, "description")?;
        if !self.read.truthy(&v) {
            return Ok(None);
        }
        if reread {
            v = self.read.get(node, "description")?;
        }
        self.strict(node, "description", &v, "replace", "e")
            .map(Some)
    }
    fn css_class(&mut self, node: &R::Value) -> Result<Option<String>, PublicError> {
        let v = self.read.get(node, "cssClass")?;
        if self.read.kind(&v) == LiveValueKind::Undefined {
            return Ok(None);
        }
        self.strict(node, "cssClass", &v, "split", "r.cssClass")
            .map(Some)
    }
    fn accessibility(&mut self, description: bool) -> Result<Option<String>, PublicError> {
        let root = self.read.root();
        let field = if description { "accDescr" } else { "accTitle" };
        let v = self.read.get(&root, field)?;
        if !self.read.truthy(&v) {
            return Ok(None);
        }
        let v = self.read.get(&root, field)?;
        self.strict(&root, field, &v, "includes", "e").map(Some)
    }
}
