//! Detached browser reads at the pinned Sankey render phases.
//! Values belong to one host job, never to a mutable native document borrow.
use super::{layout, public_model::Projection, render::PhaseInput};
use crate::compat::{
    public::{JavaScriptErrorKind, JsString, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};

#[doc(hidden)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum LiveValueKind {
    Undefined,
    Null,
    Boolean,
    Number,
    String,
    Symbol,
    BigInt,
    Object,
    Function,
}
#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct LiveStringUse {
    pub method: &'static str,
    pub expression_receiver: &'static str,
}
/// Scalar operations only. Rust owns field selection, ordering and layout.
#[doc(hidden)]
pub trait LiveRenderRead {
    type Value: Clone;
    fn root(&self) -> Self::Value;
    fn get(&mut self, receiver: &Self::Value, key: &str) -> Result<Self::Value, PublicError>;
    fn index(&mut self, receiver: &Self::Value, index: usize) -> Result<Self::Value, PublicError>;
    /// Capture yielded values once, keeping pair/object identity for later reads.
    /// The bound is checked before retaining another value; no callback borrow
    /// may survive this operation. Existing custom readers opt in explicitly.
    fn snapshot_iterable(
        &mut self,
        _value: &Self::Value,
        _max_count: usize,
    ) -> Result<Vec<Self::Value>, PublicError> {
        Err(PeleError::new(
            "Live iterable capture is not implemented",
            ErrorCode::Semantic,
        )
        .into())
    }
    /// Array identity without property access or coercion.
    fn is_array(&mut self, value: &Self::Value) -> Result<bool, PublicError> {
        if !matches!(
            self.kind(value),
            LiveValueKind::Object | LiveValueKind::Function
        ) {
            return Ok(false);
        }
        Err(PeleError::new(
            "Live Array classification is not implemented",
            ErrorCode::Semantic,
        )
        .into())
    }
    /// Own enumerable string keys, in ECMAScript key order. No value reads.
    fn own_keys(
        &mut self,
        _receiver: &Self::Value,
    ) -> Result<Vec<crate::compat::public::JsString>, PublicError> {
        Err(PeleError::new(
            "Live own-key capture is not implemented",
            ErrorCode::Semantic,
        )
        .into())
    }
    /// Call receiver.values() once, then capture its yielded values in order.
    fn snapshot_map_values(
        &mut self,
        _receiver: &Self::Value,
        _max_count: usize,
    ) -> Result<Vec<Self::Value>, PublicError> {
        Err(PeleError::new(
            "Live Map value capture is not implemented",
            ErrorCode::Semantic,
        )
        .into())
    }
    /// JavaScript HasProperty for an indexed key, including inherited entries.
    fn has_index(&mut self, _receiver: &Self::Value, _index: usize) -> Result<bool, PublicError> {
        Err(PeleError::new(
            "Live indexed membership is not implemented",
            ErrorCode::Semantic,
        )
        .into())
    }
    /// Object spread into a fresh ordinary object. Child identities stay shared.
    fn shallow_copy(&mut self, _value: &Self::Value) -> Result<Self::Value, PublicError> {
        Err(PeleError::new("Live shallow copy is not implemented", ErrorCode::Semantic).into())
    }
    /// Ordinary dynamic receiver.get(key), including its method lookup and this.
    fn map_get(
        &mut self,
        _receiver: &Self::Value,
        _key: &Self::Value,
    ) -> Result<Self::Value, PublicError> {
        Err(PeleError::new("Live map lookup is not implemented", ErrorCode::Semantic).into())
    }
    /// Dynamic map.get with a native string key, without diagram interpretation.
    fn map_get_text(
        &mut self,
        _receiver: &Self::Value,
        _key: &str,
    ) -> Result<Self::Value, PublicError> {
        Err(PeleError::new(
            "Live string-key map lookup is not implemented",
            ErrorCode::Semantic,
        )
        .into())
    }
    /// Ordinary Array.join, bounded before retaining the resulting UTF16 value.
    fn array_join(
        &mut self,
        _receiver: &Self::Value,
        _separator: &str,
    ) -> Result<JsString, PublicError> {
        Err(PeleError::new("Live array join is not implemented", ErrorCode::Semantic).into())
    }
    /// Stable opaque identity within this job; no coercion or object traversal.
    /// Native graph references retain the existing graph-coordinate domain.
    fn object_identity(&mut self, value: &Self::Value) -> Result<Option<u64>, PublicError> {
        if !matches!(
            self.kind(value),
            LiveValueKind::Object | LiveValueKind::Function
        ) {
            return Ok(None);
        }
        Err(PeleError::new(
            "Live object identity is not implemented",
            ErrorCode::Semantic,
        )
        .into())
    }
    fn kind(&self, value: &Self::Value) -> LiveValueKind;
    fn number(&self, value: &Self::Value) -> Option<f64>;
    fn truthy(&self, value: &Self::Value) -> bool;
    fn string_units(&self, value: &Self::Value) -> Result<Option<JsString>, PublicError>;
    fn to_string(&mut self, value: &Self::Value) -> Result<JsString, PublicError>;
    fn require_string(
        &mut self,
        value: &Self::Value,
        site: LiveStringUse,
    ) -> Result<JsString, PublicError>;
}

pub(crate) struct SankeyDetachedInput {
    // Pins the projection through nested apply/helper replacements. Semantic
    // import failures are diagnosed by phase reads, not eagerly replayed here.
    projection: Result<Projection, PeleError>,
}
impl SankeyDetachedInput {
    pub(super) fn new(projection: Result<Projection, PeleError>) -> Self {
        Self { projection }
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    seed: &SankeyDetachedInput,
    config: &Object,
    settings: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    // Keep native allocation budgets at their existing latent dispatch stage.
    // A deleted/wrong-type host label must instead fail at the actual next read.
    if let Err(error) = &seed.projection {
        if error.code == ErrorCode::Limit {
            return Err(error.clone().into());
        }
    }
    super::render::render_phases(&mut HostPhase(read), config, settings, resources)
}
struct HostPhase<'a, R>(&'a mut R);
fn type_error(message: &str) -> PublicError {
    PublicError::JavaScript {
        name: JavaScriptErrorKind::TypeError,
        message: JsString::from(message),
    }
}
fn text(units: JsString) -> Result<String, PublicError> {
    units.well_formed().ok_or_else(|| {
        let mut error = PeleError::new(
            "Sankey rendering of lone UTF-16 surrogates is not implemented",
            ErrorCode::Semantic,
        );
        error.diagram_type = Some("sankey".into());
        error.into()
    })
}
impl<R: LiveRenderRead> HostPhase<'_, R> {
    fn root_field(&mut self, key: &str) -> Result<R::Value, PublicError> {
        let root = self.0.root();
        self.0.get(&root, key)
    }
    fn strict_text(
        &mut self,
        value: &R::Value,
        receiver: &'static str,
    ) -> Result<String, PublicError> {
        text(self.0.require_string(
            value,
            LiveStringUse {
                method: "includes",
                expression_receiver: receiver,
            },
        )?)
    }
    fn length(&mut self, array: &R::Value) -> Result<usize, PublicError> {
        let value = self.0.get(array, "length")?;
        let number = self
            .0
            .number(&value)
            .ok_or_else(|| type_error("Invalid array length"))?;
        if !number.is_finite() || number < 0. || number.fract() != 0. || number > u32::MAX as f64 {
            return Err(type_error("Invalid array length"));
        }
        // A dense membership already consumes this many references under the
        // existing graph work budget. Guard before native vector allocation.
        if number > 8_000_000. {
            return Err(
                PeleError::new("Public model graph exceeds work budget", ErrorCode::Limit).into(),
            );
        }
        Ok(number as usize)
    }
    fn endpoint(&self, value: &R::Value) -> Result<usize, PublicError> {
        if let Some(n) = self.0.number(value) {
            return Ok(
                if n.is_finite() && n >= 0. && n.fract() == 0. && n < usize::MAX as f64 {
                    n as usize
                } else {
                    usize::MAX
                },
            );
        }
        if let Some(s) = self.0.string_units(value)? {
            if let Some(s) = s.well_formed() {
                if let Ok(n) = s.parse::<usize>() {
                    if n.to_string() == s {
                        return Ok(n);
                    }
                }
            }
        }
        Ok(usize::MAX)
    }
}
impl<R: LiveRenderRead> PhaseInput for HostPhase<'_, R> {
    type Error = PublicError;
    type Label = R::Value;
    type Text = String;
    fn layout(&mut self, options: layout::Options) -> Result<layout::Layout, PublicError> {
        let nodes = self.root_field("nodes")?;
        let node_count = self.length(&nodes)?;
        let links = self.root_field("links")?;
        let count = self.length(&links)?;
        if node_count
            .checked_add(count)
            .is_none_or(|total| total > 8_000_000)
        {
            return Err(
                PeleError::new("Public model graph exceeds work budget", ErrorCode::Limit).into(),
            );
        }
        let mut geometry = Vec::with_capacity(count);
        for i in 0..count {
            let link = self.0.index(&links, i)?;
            let source = self.0.get(&link, "source")?;
            let source = self.0.get(&source, "index")?;
            let target = self.0.get(&link, "target")?;
            let target = self.0.get(&target, "index")?;
            // Number.isFinite does not coerce; the JS conditional reads value
            // again for >0 and once more for its successful result.
            let finite = self.0.get(&link, "value")?;
            let value = if self.0.number(&finite).is_some_and(f64::is_finite) {
                let positive = self.0.get(&link, "value")?;
                if self.0.number(&positive).is_some_and(|n| n > 0.) {
                    let value = self.0.get(&link, "value")?;
                    self.0.number(&value).unwrap_or(f64::NAN)
                } else {
                    0.
                }
            } else {
                0.
            };
            geometry.push(layout::Link {
                source: self.endpoint(&source)?,
                target: self.endpoint(&target)?,
                value,
                thickness: 0.,
                source_y: 0.,
                target_y: 0.,
            });
        }
        if geometry
            .iter()
            .any(|link| link.source >= node_count || link.target >= node_count)
        {
            return Err(type_error(
                "Cannot read properties of undefined (reading 'push')",
            ));
        }
        match layout::layout_values(node_count, geometry, options) {
            Ok(layout) => Ok(layout),
            Err(layout::Failure::Cycle(index)) => {
                let value = self.label(index)?;
                let id = text(self.0.to_string(&value)?)?;
                let mut error = PeleError::new(
                    format!(
                        "Sankey diagrams cannot have circular links, and \"{id}\" is part of one."
                    ),
                    ErrorCode::Semantic,
                );
                error.diagram_type = Some("sankey".into());
                Err(error.into())
            }
        }
    }
    fn label(&mut self, index: usize) -> Result<R::Value, PublicError> {
        let nodes = self.root_field("nodes")?;
        let node = self.0.index(&nodes, index)?;
        self.0.get(&node, "id")
    }
    fn text(&mut self, label: &R::Value) -> Result<String, PublicError> {
        self.strict_text(label, "e")
    }
    fn color_key(&mut self, index: usize) -> Result<String, PublicError> {
        let label = self.label(index)?;
        text(self.0.to_string(&label)?)
    }
    fn title(&mut self) -> Result<Option<String>, PublicError> {
        let value = self.root_field("title")?;
        if self.0.kind(&value) == LiveValueKind::Undefined {
            return Ok(None);
        }
        self.strict_text(&value, "t").map(Some)
    }
    fn accessibility(&mut self, description: bool) -> Result<Option<String>, PublicError> {
        let key = if description { "accDescr" } else { "accTitle" };
        let value = self.root_field(key)?;
        if !self.0.truthy(&value) {
            return Ok(None);
        }
        let value = self.root_field(key)?;
        self.strict_text(&value, "e").map(Some)
    }
}
