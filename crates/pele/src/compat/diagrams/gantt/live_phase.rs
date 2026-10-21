//! Detached Gantt draws retain task handles; each later field read stays live.
use super::{dates::Calendar, phase::PhaseInput};
use crate::compat::{
    public::{LiveCallArgument, LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
use std::collections::HashMap;
pub(crate) struct GanttDetachedInput {
    limit: Option<PeleError>,
}
impl GanttDetachedInput {
    pub(super) fn new(error: Option<&PeleError>) -> Self {
        Self {
            limit: error.filter(|e| e.code == ErrorCode::Limit).cloned(),
        }
    }
}
fn invalid(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("gantt".into());
    e
}
fn limit() -> PeleError {
    let mut e = invalid("Gantt public model exceeds native graph/work budget");
    e.code = ErrorCode::Limit;
    e
}
fn type_error(message: &str) -> PublicError {
    PublicError::JavaScript {
        name: crate::compat::public::JavaScriptErrorKind::TypeError,
        message: message.into(),
    }
}
struct Host<'a, R: LiveRenderRead> {
    read: &'a mut R,
    text: HashMap<(u64, &'static str), usize>,
    units: usize,
    kept: usize,
    steps: usize,
}
impl<R: LiveRenderRead> Host<'_, R> {
    fn id(&mut self, v: &R::Value) -> Result<u64, PublicError> {
        self.read
            .object_identity(v)?
            .ok_or_else(|| invalid("Gantt task must be an object").into())
    }
    fn count(&mut self, v: &R::Value) -> Result<usize, PublicError> {
        let n = self.read.get(v, "length")?;
        let n = self
            .read
            .number(&n)
            .ok_or_else(|| invalid("Gantt collection must be an Array"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(invalid("Gantt collection must be an Array").into());
        }
        if n > 1_000_000. {
            return Err(limit().into());
        }
        Ok(n as usize)
    }
    fn string(
        &mut self,
        owner: &R::Value,
        key: &'static str,
        v: &R::Value,
    ) -> Result<String, PublicError> {
        let s = self.read.require_string(
            v,
            LiveStringUse {
                method: "split",
                expression_receiver: "text",
            },
        )?;
        let id = self.id(owner)?;
        let old = self.text.get(&(id, key)).copied().unwrap_or(0);
        if s.0.len() > old {
            self.units = self.units.checked_add(s.0.len() - old).ok_or_else(limit)?;
            if self.units > 16_000_000 {
                return Err(limit().into());
            }
            self.text.insert((id, key), s.0.len());
        }
        s.well_formed().ok_or_else(|| {
            invalid("Gantt rendering of lone UTF-16 surrogates is not implemented").into()
        })
    }
    fn nullish(&self, v: &R::Value) -> bool {
        matches!(
            self.read.kind(v),
            LiveValueKind::Null | LiveValueKind::Undefined
        )
    }
}
impl<R: LiveRenderRead> PhaseInput for Host<'_, R> {
    type Task = R::Value;
    type Error = PublicError;
    fn root_text(&mut self, key: &'static str) -> Result<Option<String>, PublicError> {
        self.work(1)?;
        let root = self.read.root();
        let v = self.read.get(&root, key)?;
        if self.nullish(&v) {
            Ok(None)
        } else {
            self.string(&root, key, &v).map(Some)
        }
    }
    fn root_bool(&mut self, key: &'static str) -> Result<bool, PublicError> {
        self.work(1)?;
        let root = self.read.root();
        let v = self.read.get(&root, key)?;
        Ok(self.read.truthy(&v))
    }
    fn tasks(
        &mut self,
        now: f64,
        resources: &mut dyn RenderResources,
    ) -> Result<Vec<R::Value>, PublicError> {
        let root = self.read.root();
        let method = self.read.get(&root, "getTasks")?;
        let tasks = self.read.call(
            &method,
            &root,
            &[LiveCallArgument::Number(now)],
            resources,
            "t.getTasks is not a function",
        )?;
        if !self.read.is_array(&tasks)? {
            return Err(type_error("Ee.filter is not a function"));
        }
        let count = self.count(&tasks)?;
        let mut out = Vec::with_capacity(count);
        for i in 0..count {
            self.work(1)?;
            if self.read.has_index(&tasks, i)? {
                out.push(self.read.index(&tasks, i)?);
            }
        }
        Ok(out)
    }
    fn text(&mut self, t: &R::Value, key: &'static str) -> Result<String, PublicError> {
        self.work(1)?;
        let v = self.read.get(t, key)?;
        self.string(t, key, &v)
    }
    fn flag(&mut self, t: &R::Value, key: &'static str) -> Result<bool, PublicError> {
        self.work(1)?;
        let v = self.read.get(t, key)?;
        Ok(self.read.truthy(&v))
    }
    fn date(
        &mut self,
        t: &R::Value,
        key: &'static str,
        r: &mut dyn RenderResources,
    ) -> Result<f64, PublicError> {
        self.work(1)?;
        let v = self.read.get(t, key)?;
        self.read.to_number(&v, r)
    }
    fn render_end(
        &mut self,
        t: &R::Value,
        r: &mut dyn RenderResources,
    ) -> Result<f64, PublicError> {
        self.work(1)?;
        let mut v = self.read.get(t, "renderEndTime")?;
        if self.nullish(&v) {
            v = self.read.get(t, "endTime")?
        }
        self.read.to_number(&v, r)
    }
    fn order(&mut self, t: &R::Value, r: &mut dyn RenderResources) -> Result<f64, PublicError> {
        self.date(t, "order", r)
    }
    fn identity(&mut self, t: &R::Value) -> Result<u64, PublicError> {
        self.id(t)
    }
    fn clickable(&mut self, t: &R::Value) -> Result<bool, PublicError> {
        let values = self.read.get(t, "classes")?;
        if matches!(self.read.kind(&values), LiveValueKind::String) {
            return Ok(self
                .read
                .string_units(&values)?
                .is_some_and(|s| s.well_formed().is_some_and(|s| s.contains("clickable"))));
        }
        if !self.read.is_array(&values)? {
            return Err(type_error("l.classes.includes is not a function"));
        }
        let count = self.count(&values)?;
        for i in 0..count {
            self.work(1)?;
            let value = self.read.index(&values, i)?;
            if self
                .read
                .string_units(&value)?
                .is_some_and(|s| s.is("clickable"))
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn link(&mut self, t: &R::Value) -> Result<Option<(String, String)>, PublicError> {
        let root = self.read.root();
        let links = self.read.get(&root, "links")?;
        let id = self.read.get(t, "id")?;
        let url = self.read.map_get(&links, &id)?;
        if matches!(self.read.kind(&url), LiveValueKind::Undefined) {
            return Ok(None);
        }
        let url = self.string(&links, "link-value", &url)?;
        // Link helper receives a fresh task ID read after the lookup.
        let id = self.text(t, "id")?;
        Ok(Some((id, url)))
    }
    fn has_exclusions(&mut self) -> Result<bool, PublicError> {
        let root = self.read.root();
        let excluded = self.read.get(&root, "excludes")?;
        if self.count(&excluded)? > 0 {
            return Ok(true);
        }
        let included = self.read.get(&root, "includes")?;
        Ok(self.count(&included)? > 0)
    }
    fn invalid(&mut self, day: f64, cal: &mut Calendar<'_>) -> Result<bool, PublicError> {
        let root = self.read.root();
        let f = self.read.get(&root, "isInvalidDate")?;
        let value = self.read.call(
            &f,
            &root,
            &[LiveCallArgument::Date(day)],
            cal.resources,
            "t.isInvalidDate is not a function",
        )?;
        Ok(self.read.truthy(&value))
    }
    fn work(&mut self, n: usize) -> Result<(), PublicError> {
        self.steps = self.steps.checked_add(n).ok_or_else(limit)?;
        if self.steps > 8_000_000 {
            Err(limit().into())
        } else {
            Ok(())
        }
    }
    fn keep_text(&mut self, n: usize) -> Result<(), PublicError> {
        self.kept = self.kept.checked_add(n).ok_or_else(limit)?;
        if self.kept > 16_000_000 {
            Err(limit().into())
        } else {
            Ok(())
        }
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    seed: &GanttDetachedInput,
    c: &Object,
    o: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    if let Some(e) = &seed.limit {
        return Err(e.clone().into());
    }
    let now = if let Some(now) = o.now.filter(|n| n.is_finite()) {
        now
    } else {
        resources.now_millis()?
    };
    let mut input = Host {
        read,
        text: HashMap::new(),
        units: 0,
        kept: 0,
        steps: 0,
    };
    let mut cal = Calendar::public(resources);
    let result = super::render::draw(
        &mut input,
        c,
        o,
        &mut cal,
        o.font_size.unwrap_or(16.),
        o.padding.unwrap_or(8.),
        now,
    );
    if let Some(error) = cal.error {
        return Err(error.into());
    }
    result
}
