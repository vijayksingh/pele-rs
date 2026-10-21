//! Own job values, observing collections only when source rendering reaches them.
use super::{model::DomainName, render::PhaseInput};
use crate::compat::{
    public::{LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};

pub(crate) struct CynefinDetachedInput {
    limit: Option<PeleError>,
}
impl CynefinDetachedInput {
    pub(super) fn new(error: Option<PeleError>) -> Self {
        Self {
            limit: error.filter(|e| e.code == ErrorCode::Limit),
        }
    }
}
fn failure(message: &str, code: ErrorCode) -> PublicError {
    let mut error = PeleError::new(message, code);
    error.diagram_type = Some("cynefin".into());
    error.into()
}
fn limit() -> PublicError {
    failure(
        "Cynefin live rendering exceeds native graph/work budget",
        ErrorCode::Limit,
    )
}
struct HostPhase<'a, R> {
    read: &'a mut R,
    units: usize,
    work: usize,
}
impl<R: LiveRenderRead> HostPhase<'_, R> {
    fn tick(&mut self) -> Result<(), PublicError> {
        self.work += 1;
        if self.work > 8_000_000 {
            Err(limit())
        } else {
            Ok(())
        }
    }
    fn root_field(&mut self, key: &str) -> Result<R::Value, PublicError> {
        let root = self.read.root();
        self.read.get(&root, key)
    }
    fn length(&mut self, value: &R::Value) -> Result<usize, PublicError> {
        self.tick()?;
        let length = self.read.get(value, "length")?;
        let n = self.read.number(&length).ok_or_else(|| {
            failure(
                "Cynefin live collections must be ordinary Arrays",
                ErrorCode::Semantic,
            )
        })?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(failure(
                "Cynefin live collections must be ordinary Arrays",
                ErrorCode::Semantic,
            ));
        }
        if n > 1_000_000. {
            return Err(limit());
        }
        Ok(n as usize)
    }
    fn text(&mut self, value: &R::Value) -> Result<String, PublicError> {
        let value = self.read.require_string(
            value,
            LiveStringUse {
                method: "includes",
                expression_receiver: "t",
            },
        )?;
        self.units = self.units.saturating_add(value.0.len());
        if self.units > 16_000_000 {
            return Err(limit());
        }
        value.well_formed().ok_or_else(|| {
            failure(
                "Cynefin rendering of lone UTF-16 surrogates is not implemented",
                ErrorCode::Semantic,
            )
        })
    }
    fn optional_label(&mut self, value: &R::Value) -> Result<Option<String>, PublicError> {
        if matches!(
            self.read.kind(value),
            LiveValueKind::Undefined | LiveValueKind::Null
        ) {
            Ok(None)
        } else {
            self.text(value).map(Some)
        }
    }
    fn domain(&self, value: &R::Value) -> Result<Option<DomainName>, PublicError> {
        Ok(self
            .read
            .string_units(value)?
            .and_then(|s| s.well_formed())
            .and_then(|s| match s.as_str() {
                "complex" => Some(DomainName::Complex),
                "complicated" => Some(DomainName::Complicated),
                "clear" => Some(DomainName::Clear),
                "chaotic" => Some(DomainName::Chaotic),
                "confusion" => Some(DomainName::Confusion),
                _ => None,
            }))
    }
    fn id_part(&mut self, value: &R::Value) -> Result<String, PublicError> {
        let units = self.read.to_string(value)?;
        self.units = self.units.saturating_add(units.0.len());
        if self.units > 16_000_000 {
            return Err(limit());
        }
        units.well_formed().ok_or_else(|| {
            failure(
                "Cynefin rendering of lone UTF-16 surrogates is not implemented",
                ErrorCode::Semantic,
            )
        })
    }
}
impl<R: LiveRenderRead> PhaseInput for HostPhase<'_, R> {
    type Error = PublicError;
    type Items = Option<R::Value>;
    type Transitions = R::Value;
    type Transition = R::Value;
    fn items(&mut self, name: DomainName) -> Result<Self::Items, PublicError> {
        let domains = self.root_field("domains")?;
        let domain = self.read.map_get_text(&domains, name.as_str())?;
        if matches!(
            self.read.kind(&domain),
            LiveValueKind::Undefined | LiveValueKind::Null
        ) {
            return Ok(None);
        }
        let items = self.read.get(&domain, "items")?;
        Ok(
            if matches!(
                self.read.kind(&items),
                LiveValueKind::Undefined | LiveValueKind::Null
            ) {
                None
            } else {
                Some(items)
            },
        )
    }
    fn item_count(&mut self, items: &Self::Items) -> Result<usize, PublicError> {
        match items {
            None => Ok(0),
            Some(items) => self.length(items),
        }
    }
    fn item_label(&mut self, items: &Self::Items, index: usize) -> Result<String, PublicError> {
        self.tick()?;
        let item = self
            .read
            .index(items.as_ref().expect("shown items have receiver"), index)?;
        let label = self.read.get(&item, "label")?;
        // layoutLabel treats nullish labels as empty text.
        self.optional_label(&label).map(|s| s.unwrap_or_default())
    }
    fn metadata(&mut self, key: &'static str) -> Result<Option<String>, PublicError> {
        let mut value = self.root_field(key)?;
        if key != "title" {
            if !self.read.truthy(&value) {
                return Ok(None);
            }
            value = self.root_field(key)?;
        }
        self.optional_label(&value)
    }
    fn transitions(&mut self) -> Result<Self::Transitions, PublicError> {
        self.root_field("transitions")
    }
    fn next_transition(
        &mut self,
        rows: &Self::Transitions,
        index: usize,
    ) -> Result<Option<Self::Transition>, PublicError> {
        if index >= self.length(rows)? {
            return Ok(None);
        }
        self.tick()?;
        self.read.index(rows, index).map(Some)
    }
    fn endpoints(
        &mut self,
        row: &Self::Transition,
    ) -> Result<(Option<DomainName>, Option<DomainName>), PublicError> {
        let from = self.read.get(row, "from")?;
        let start = self.domain(&from)?;
        let to = self.read.get(row, "to")?;
        let end = self.domain(&to)?;
        Ok((start, end))
    }
    fn transition_label(&mut self, row: &Self::Transition) -> Result<Option<String>, PublicError> {
        let value = self.read.get(row, "label")?;
        if !self.read.truthy(&value) {
            return Ok(None);
        }
        let value = self.read.get(row, "label")?;
        self.optional_label(&value)
    }
    fn transition_id(&mut self, row: &Self::Transition) -> Result<(String, String), PublicError> {
        let value = self.read.get(row, "from")?;
        let from = self.id_part(&value)?;
        let value = self.read.get(row, "to")?;
        let to = self.id_part(&value)?;
        Ok((from, to))
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    seed: &CynefinDetachedInput,
    config: &Object,
    settings: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    if let Some(error) = &seed.limit {
        return Err(error.clone().into());
    }
    super::render::render_phases(
        &mut HostPhase {
            read,
            units: 0,
            work: 0,
        },
        config,
        settings,
        resources,
    )
}
