//! Grammar nodes and child arrays stay opaque across measurement callbacks.
use super::phase::{NodeType, PhaseInput};
use crate::compat::{
    public::{LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
pub(crate) struct RailroadDetachedInput {
    limit: Option<PeleError>,
}
impl RailroadDetachedInput {
    pub(super) fn new(error: Option<PeleError>) -> Self {
        Self {
            limit: error.filter(|e| e.code == ErrorCode::Limit),
        }
    }
}
fn invalid(message: &str) -> PublicError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("railroad".into());
    e.into()
}
fn limit() -> PublicError {
    let mut e = PeleError::new(
        "Railroad live rendering exceeds work/text budget.",
        ErrorCode::Limit,
    );
    e.diagram_type = Some("railroad".into());
    e.into()
}
struct Host<'a, R> {
    read: &'a mut R,
    units: usize,
    steps: usize,
}
pub(crate) enum Children<V> {
    Retained(V),
    Owned(Vec<V>),
}
impl<R: LiveRenderRead> Host<'_, R> {
    fn tick(&mut self) -> Result<(), PublicError> {
        self.steps += 1;
        if self.steps > 8_000_000 {
            Err(limit())
        } else {
            Ok(())
        }
    }
    fn nullish(&self, value: &R::Value) -> bool {
        matches!(
            self.read.kind(value),
            LiveValueKind::Null | LiveValueKind::Undefined
        )
    }
    fn field(&mut self, key: &str) -> Result<R::Value, PublicError> {
        let root = self.read.root();
        self.read.get(&root, key)
    }
    fn length(&mut self, array: &R::Value) -> Result<usize, PublicError> {
        self.tick()?;
        let v = self.read.get(array, "length")?;
        let n = self
            .read
            .number(&v)
            .ok_or_else(|| invalid("Railroad live collections must be ordinary Arrays"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(invalid("Railroad live collections must be ordinary Arrays"));
        }
        if n > 1_000_000. {
            return Err(limit());
        }
        Ok(n as usize)
    }
    fn string(&mut self, value: &R::Value) -> Result<String, PublicError> {
        let units = self.read.require_string(
            value,
            LiveStringUse {
                method: "includes",
                expression_receiver: "e",
            },
        )?;
        self.units = self.units.saturating_add(units.0.len());
        if self.units > 16_000_000 {
            return Err(limit());
        }
        units.well_formed().ok_or_else(|| {
            invalid("Railroad rendering of lone UTF-16 surrogates is not implemented")
        })
    }
}
impl<R: LiveRenderRead> PhaseInput for Host<'_, R> {
    type Error = PublicError;
    type Node = R::Value;
    type Children = Children<R::Value>;
    type Rules = R::Value;
    type Rule = R::Value;
    fn begin_draw(&mut self) {
        self.units = 0;
        self.steps = 0;
    }
    fn identity(&mut self, node: &R::Value) -> Result<u64, PublicError> {
        self.read
            .object_identity(node)?
            .ok_or_else(|| invalid("Railroad definitions must be objects"))
    }
    fn children(&mut self, node: &R::Value) -> Result<Self::Children, PublicError> {
        self.tick()?;
        let mut element = self.read.get(node, "element")?;
        if self.nullish(&element) {
            element = self.read.get(node, "primary")?;
        }
        if self.nullish(&element) {
            element = self.read.get(node, "suffix")?;
        }
        let mut list = self.read.get(node, "alternatives")?;
        if self.nullish(&list) {
            list = self.read.get(node, "elements")?;
        }
        let mut result = if self.nullish(&list) {
            Children::Owned(if self.read.truthy(&element) {
                vec![element]
            } else {
                vec![]
            })
        } else {
            Children::Retained(list)
        };
        let base = self.read.get(node, "base")?;
        if self.read.truthy(&base) {
            let mut captured = vec![base];
            let postfixes = self.read.get(node, "postfixes")?;
            let mut index = 0;
            while index < self.length(&postfixes)? {
                self.tick()?;
                let postfix = self.read.index(&postfixes, index)?;
                index += 1;
                let except = self.read.get(&postfix, "except")?;
                if self.read.truthy(&except) {
                    captured.push(except);
                }
            }
            result = Children::Owned(captured);
        }
        Ok(result)
    }
    fn length(&mut self, children: &Self::Children) -> Result<usize, PublicError> {
        match children {
            Children::Retained(v) => self.length(v),
            Children::Owned(v) => Ok(v.len()),
        }
    }
    fn at(&mut self, children: &Self::Children, index: usize) -> Result<R::Value, PublicError> {
        self.tick()?;
        match children {
            Children::Retained(v) => self.read.index(v, index),
            Children::Owned(v) => Ok(v[index].clone()),
        }
    }
    fn kind(&mut self, node: &R::Value) -> Result<NodeType, PublicError> {
        let value = self.read.get(node, "type")?;
        let name = self
            .read
            .string_units(&value)?
            .and_then(|s| s.well_formed());
        match name.as_deref() {
            Some("terminal") => Ok(NodeType::Terminal),
            Some("nonterminal") => Ok(NodeType::Nonterminal),
            Some("special") => Ok(NodeType::Special),
            Some("sequence") => Ok(NodeType::Sequence),
            Some("choice") => Ok(NodeType::Choice),
            Some("optional") => Ok(NodeType::Optional),
            Some("repetition") => Ok(NodeType::Repetition),
            _ => Err(invalid(
                "Railroad unknown IR node type is not representable in the native renderer",
            )),
        }
    }
    fn text(&mut self, node: &R::Value, key: &'static str) -> Result<String, PublicError> {
        let value = self.read.get(node, key)?;
        self.string(&value)
    }
    fn repetition(&mut self, node: &R::Value) -> Result<(f64, f64), PublicError> {
        let min = self.read.get(node, "min")?;
        let max = self.read.get(node, "max")?;
        Ok((
            self.read
                .number(&min)
                .ok_or_else(|| invalid("Railroad repetition bounds must be numbers"))?,
            self.read
                .number(&max)
                .ok_or_else(|| invalid("Railroad repetition bounds must be numbers"))?,
        ))
    }
    fn rules(&mut self) -> Result<R::Value, PublicError> {
        self.field("rules")
    }
    fn next_rule(
        &mut self,
        rules: &R::Value,
        index: usize,
    ) -> Result<Option<R::Value>, PublicError> {
        if index >= self.length(rules)? {
            return Ok(None);
        }
        self.tick()?;
        self.read.index(rules, index).map(Some)
    }
    fn definition(&mut self, rule: &R::Value) -> Result<R::Value, PublicError> {
        self.read.get(rule, "definition")
    }
    fn rule_name(&mut self, rule: &R::Value) -> Result<String, PublicError> {
        let value = self.read.get(rule, "name")?;
        self.string(&value)
    }
    fn metadata(&mut self, key: &'static str) -> Result<Option<String>, PublicError> {
        let mut value = self.field(key)?;
        if key != "title" {
            if !self.read.truthy(&value) {
                return Ok(None);
            }
            value = self.field(key)?;
        }
        if self.nullish(&value) {
            Ok(None)
        } else {
            self.string(&value).map(Some)
        }
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    seed: &RailroadDetachedInput,
    config: &Object,
    settings: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    if let Some(error) = &seed.limit {
        return Err(error.clone().into());
    }
    super::render::render_phases(
        &mut Host {
            read,
            units: 0,
            steps: 0,
        },
        config,
        settings,
        resources,
    )
}
