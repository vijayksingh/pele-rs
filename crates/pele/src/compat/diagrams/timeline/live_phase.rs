//! Timeline phase values outlive callbacks without borrowing document state.
use super::render::PhaseInput;
use crate::compat::{
    public::{LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
use std::collections::HashMap;
pub(crate) struct TimelineDetachedInput {
    limit: Option<PeleError>,
}
impl TimelineDetachedInput {
    pub(super) fn new(error: Option<PeleError>) -> Self {
        Self {
            limit: error.filter(|e| e.code == ErrorCode::Limit),
        }
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    seed: &TimelineDetachedInput,
    c: &Object,
    o: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    if let Some(error) = &seed.limit {
        return Err(error.clone().into());
    }
    super::render::render_phases(
        &mut HostPhase {
            read,
            text: HashMap::new(),
            units: 0,
            events: HashMap::new(),
            work: 0,
        },
        c,
        o,
        resources,
    )
}
fn invalid(message: &str) -> PublicError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("timeline".into());
    e.into()
}
fn limit() -> PublicError {
    let mut e = PeleError::new(
        "Timeline public model expansion exceeds the work budget",
        ErrorCode::Limit,
    );
    e.diagram_type = Some("timeline".into());
    e.into()
}
struct HostPhase<'a, R> {
    read: &'a mut R,
    text: HashMap<(u8, usize, usize), usize>,
    units: usize,
    events: HashMap<usize, usize>,
    work: usize,
}
impl<R: LiveRenderRead> HostPhase<'_, R> {
    fn field(&mut self, key: &str) -> Result<R::Value, PublicError> {
        let root = self.read.root();
        self.read.get(&root, key)
    }
    fn length(&mut self, value: &R::Value) -> Result<usize, PublicError> {
        let raw = self.read.get(value, "length")?;
        let n = self
            .read
            .number(&raw)
            .ok_or_else(|| invalid("Timeline collections must be Arrays"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(invalid("Timeline collections must be Arrays"));
        }
        if n > 1_000_000. {
            return Err(limit());
        }
        Ok(n as usize)
    }
    fn string(
        &mut self,
        value: &R::Value,
        slot: (u8, usize, usize),
    ) -> Result<Option<String>, PublicError> {
        if self.read.kind(value) == LiveValueKind::Undefined {
            return Ok(None);
        }
        let value = self.read.require_string(
            value,
            LiveStringUse {
                method: "includes",
                expression_receiver: "e",
            },
        )?;
        let old = self.text.entry(slot).or_default();
        if value.0.len() > *old {
            self.units = self.units.saturating_add(value.0.len() - *old);
            *old = value.0.len();
        }
        if self.units > 16_000_000 {
            return Err(limit());
        }
        value.well_formed().map(Some).ok_or_else(|| {
            invalid("Timeline rendering of lone UTF-16 surrogates is not implemented")
        })
    }
}
impl<R: LiveRenderRead> PhaseInput for HostPhase<'_, R> {
    type Error = PublicError;
    type Periods = R::Value;
    type Events = (usize, R::Value);
    fn downward(&mut self) -> Result<bool, PublicError> {
        let value = self.field("direction")?;
        Ok(self.read.string_units(&value)?.is_some_and(|s| s.is("TD")))
    }
    fn start(&mut self) -> Result<(R::Value, usize, Vec<Option<usize>>), PublicError> {
        self.text.clear();
        self.units = 0;
        self.events.clear();
        self.work = 0;
        let periods = self.field("periods")?;
        let sections = self.field("sections")?;
        let sections = self.length(&sections)?;
        let count = self.length(&periods)?;
        let mut memberships = Vec::with_capacity(count);
        for i in 0..count {
            let period = self.read.index(&periods, i)?;
            let index = self.read.get(&period, "sectionIndex")?;
            let n = self.read.number(&index);
            memberships.push(
                n.filter(|n| n.is_finite() && *n >= 0. && n.fract() == 0. && *n < 1_000_000.)
                    .map(|n| n as usize),
            );
        }
        Ok((periods, sections, memberships))
    }
    fn period_count(&mut self, periods: &R::Value) -> Result<usize, PublicError> {
        self.length(periods)
    }
    fn period_text(&mut self, periods: &R::Value, i: usize) -> Result<Option<String>, PublicError> {
        let period = self.read.index(periods, i)?;
        let value = self.read.get(&period, "text")?;
        self.string(&value, (0, i, 0))
    }
    fn events(
        &mut self,
        periods: &R::Value,
        i: usize,
    ) -> Result<(Self::Events, usize), PublicError> {
        let period = self.read.index(periods, i)?;
        let value = self.read.get(&period, "events")?;
        let n = self.length(&value)?;
        let old = self.events.entry(i).or_default();
        if n > *old {
            self.work = self.work.saturating_add(n - *old);
            *old = n;
        }
        if self.work > 8_000_000 {
            return Err(limit());
        }
        Ok(((i, value), n))
    }
    fn event(
        &mut self,
        (period, events): &Self::Events,
        i: usize,
    ) -> Result<Option<String>, PublicError> {
        let value = self.read.index(events, i)?;
        self.string(&value, (1, *period, i))
    }
    fn section(&mut self, i: usize) -> Result<Option<String>, PublicError> {
        let sections = self.field("sections")?;
        let value = self.read.index(&sections, i)?;
        self.string(&value, (2, i, 0))
    }
    fn title(&mut self) -> Result<Option<String>, PublicError> {
        let value = self.field("title")?;
        self.string(&value, (3, 0, 0))
    }
    fn accessibility(&mut self, description: bool) -> Result<Option<String>, PublicError> {
        let key = if description { "accDescr" } else { "accTitle" };
        let value = self.field(key)?;
        if !self.read.truthy(&value) {
            return Ok(None);
        }
        let value = self.field(key)?;
        self.string(&value, (4, usize::from(description), 0))
    }
}
