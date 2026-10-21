//! Live Journey field reads retain task/array identities across resource callbacks.
use super::render::PhaseInput;
use crate::compat::{
    public::{LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
pub(crate) struct JourneyDetachedInput {
    limit: Option<PeleError>,
}
impl JourneyDetachedInput {
    pub(super) fn new(error: Option<&PeleError>) -> Self {
        Self {
            limit: error.filter(|e| e.code == ErrorCode::Limit).cloned(),
        }
    }
}
struct HostPhase<'a, R>(&'a mut R);
fn invalid(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("journey".into());
    e
}
// Bound each retained logical collection; reading it again in another phase
// starts another budget rather than charging repeated scalar observations.
fn charge(units: &mut usize, count: usize) -> Result<(), PeleError> {
    let next = units
        .checked_add(count)
        .filter(|n| *n <= 16_000_000)
        .ok_or_else(|| {
            let mut e = invalid("Journey public operation exceeds native work budget");
            e.code = ErrorCode::Limit;
            e
        })?;
    *units = next;
    Ok(())
}
impl<R: LiveRenderRead> HostPhase<'_, R> {
    fn field(&mut self, key: &str) -> Result<R::Value, PublicError> {
        let root = self.0.root();
        self.0.get(&root, key)
    }
    fn string(&mut self, value: &R::Value) -> Result<String, PublicError> {
        self.string_bounded(value, &mut 0)
    }
    fn string_bounded(
        &mut self,
        value: &R::Value,
        units: &mut usize,
    ) -> Result<String, PublicError> {
        let text = self.0.require_string(
            value,
            LiveStringUse {
                method: "replace",
                expression_receiver: "e",
            },
        )?;
        charge(units, text.0.len())?;
        text.well_formed().ok_or_else(|| {
            invalid("Journey rendering of lone UTF-16 surrogates is not implemented").into()
        })
    }
    fn length(&mut self, array: &R::Value) -> Result<usize, PublicError> {
        let v = self.0.get(array, "length")?;
        let n = self
            .0
            .number(&v)
            .ok_or_else(|| invalid("Journey collections must be Arrays"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(invalid("Journey collections must be Arrays").into());
        }
        if n > 1_000_000. {
            let mut e = invalid("Journey public operation exceeds native work budget");
            e.code = ErrorCode::Limit;
            return Err(e.into());
        }
        Ok(n as usize)
    }
}
impl<R: LiveRenderRead> PhaseInput for HostPhase<'_, R> {
    type Error = PublicError;
    type Task = R::Value;
    type Tasks = R::Value;
    fn retain_text(&mut self, units: &mut usize, text: &str) -> Result<(), PublicError> {
        charge(units, text.encode_utf16().count()).map_err(Into::into)
    }
    fn metadata(&mut self, key: &str) -> Result<Option<String>, PublicError> {
        let value = self.field(key)?;
        if self.0.kind(&value) == LiveValueKind::Undefined
            || (key != "title" && !self.0.truthy(&value))
        {
            Ok(None)
        } else {
            self.string(&value).map(Some)
        }
    }
    fn tasks(&mut self) -> Result<R::Value, PublicError> {
        self.field("tasks")
    }
    fn count(&mut self, tasks: &R::Value) -> Result<usize, PublicError> {
        self.length(tasks)
    }
    fn has(&mut self, tasks: &R::Value, index: usize) -> Result<bool, PublicError> {
        self.0.has_index(tasks, index)
    }
    fn task(&mut self, tasks: &R::Value, index: usize) -> Result<R::Value, PublicError> {
        self.0.index(tasks, index)
    }
    fn text(&mut self, task: &R::Value, key: &str) -> Result<String, PublicError> {
        let v = self.0.get(task, key)?;
        self.string(&v)
    }
    fn score(&mut self, task: &R::Value) -> Result<f64, PublicError> {
        let v = self.0.get(task, "score")?;
        self.0
            .number(&v)
            .ok_or_else(|| invalid("Journey score coercion is not implemented").into())
    }
    fn people(&mut self, task: &R::Value) -> Result<Vec<String>, PublicError> {
        let array = self.0.get(task, "people")?;
        let count = self.length(&array)?;
        let mut result = Vec::new();
        let mut units = 0;
        for i in 0..count {
            if self.0.has_index(&array, i)? {
                let v = self.0.index(&array, i)?;
                result.push(self.string_bounded(&v, &mut units)?);
            }
        }
        Ok(result)
    }
    fn actors(&mut self, tasks: &R::Value) -> Result<Vec<String>, PublicError> {
        let mut names = Vec::new();
        let mut units = 0;
        let mut i = 0;
        while i < self.length(tasks)? {
            let task = self.0.index(tasks, i)?;
            let people = self.0.get(&task, "people")?;
            let mut j = 0;
            while j < self.length(&people)? {
                let v = self.0.index(&people, j)?;
                names.push(self.string_bounded(&v, &mut units)?);
                if names.len() > 8_000_000 {
                    let mut e = invalid("Journey public operation exceeds native work budget");
                    e.code = ErrorCode::Limit;
                    return Err(e.into());
                }
                j += 1
            }
            i += 1
        }
        names.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
        names.dedup();
        names.retain(|s| !s.is_empty());
        Ok(names)
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    input: &JourneyDetachedInput,
    _: &Object,
    settings: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    if let Some(e) = &input.limit {
        return Err(e.clone().into());
    }
    super::render::render_phases(&mut HostPhase(read), settings, resources)
}
