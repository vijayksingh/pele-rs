//! Owned live C4 handles; no document borrow crosses resource callbacks.
use super::{
    model::ElementRef,
    phase::{Collection, PhaseInput},
};
use crate::compat::{
    public::{JavaScriptErrorKind, LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
use std::collections::HashMap;
pub(crate) struct C4DetachedInput {
    error: Option<PeleError>,
    helper: bool,
}
impl C4DetachedInput {
    pub(super) fn new(error: Option<&PeleError>, helper: bool) -> Self {
        Self{error:error.filter(|e|e.code==ErrorCode::Limit||e.message=="C4 missing or replaced getC4Shape render callbacks are not implemented").cloned(),helper}
    }
}
fn invalid(message: &str) -> PublicError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("c4".into());
    e.into()
}
fn limit() -> PublicError {
    let mut e = PeleError::new(
        "C4 live rendering exceeds native work/text budget",
        ErrorCode::Limit,
    );
    e.diagram_type = Some("c4".into());
    e.into()
}
struct Host<'a, R> {
    read: &'a mut R,
    helper: bool,
    helper_kind: LiveValueKind,
    helper_identity: Option<u64>,
    ids: HashMap<u64, usize>,
    text: HashMap<(u64, &'static str), usize>,
    units: usize,
    retained: usize,
    steps: usize,
}
impl<R: LiveRenderRead> Host<'_, R> {
    fn root_field(&mut self, key: &str) -> Result<R::Value, PublicError> {
        let root = self.read.root();
        self.read.get(&root, key)
    }
    fn id(&mut self, value: &R::Value) -> Result<ElementRef, PublicError> {
        let id = self
            .read
            .object_identity(value)?
            .ok_or_else(|| invalid("C4 element must be an object"))?;
        let next = self.ids.len();
        if next > 1_000_000 {
            return Err(limit());
        }
        Ok(ElementRef::Shape(*self.ids.entry(id).or_insert(next)))
    }
    fn string(
        &mut self,
        owner: &R::Value,
        key: &'static str,
        value: &R::Value,
    ) -> Result<Option<String>, PublicError> {
        if matches!(
            self.read.kind(value),
            LiveValueKind::Null | LiveValueKind::Undefined
        ) {
            return Ok(None);
        }
        let units = self.read.require_string(
            value,
            LiveStringUse {
                method: "includes",
                expression_receiver: "t",
            },
        )?;
        let id = self
            .read
            .object_identity(owner)?
            .ok_or_else(|| invalid("C4 field receiver must be an object"))?;
        let old = self.text.get(&(id, key)).copied().unwrap_or(0);
        if units.0.len() > old {
            self.units = self
                .units
                .checked_add(units.0.len() - old)
                .ok_or_else(limit)?;
            if self.units > 16_000_000 {
                return Err(limit());
            }
            self.text.insert((id, key), units.0.len());
        }
        units
            .well_formed()
            .map(Some)
            .ok_or_else(|| invalid("C4 rendering of lone UTF-16 surrogates is not implemented"))
    }
}
impl<R: LiveRenderRead> PhaseInput for Host<'_, R> {
    type Error = PublicError;
    type Record = R::Value;
    type List = R::Value;
    fn begin_draw(&mut self) {
        self.ids.clear();
        self.text.clear();
        self.units = 0;
        self.retained = 0;
        self.steps = 0;
    }
    fn root(&self) -> R::Value {
        self.read.root()
    }
    fn list(&mut self, name: Collection) -> Result<R::Value, PublicError> {
        self.root_field(match name {
            Collection::Boundaries => "boundaries",
            Collection::Shapes => "shapes",
            Collection::Relations => "rels",
        })
    }
    fn len(&mut self, list: &R::Value) -> Result<usize, PublicError> {
        self.step()?;
        let value = self.read.get(list, "length")?;
        let n = self
            .read
            .number(&value)
            .ok_or_else(|| invalid("C4 collections must be ordinary Arrays"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(invalid("C4 collections must be ordinary Arrays"));
        }
        if n > 1_000_000. {
            return Err(limit());
        }
        Ok(n as usize)
    }
    fn at(&mut self, list: &R::Value, index: usize) -> Result<R::Value, PublicError> {
        self.step()?;
        self.read.index(list, index)
    }
    fn present(&mut self, list: &R::Value, index: usize) -> Result<bool, PublicError> {
        self.read.has_index(list, index)
    }
    fn identity(&mut self, record: &R::Value) -> Result<ElementRef, PublicError> {
        self.id(record)
    }
    fn parent(&mut self, record: &R::Value) -> Result<Option<ElementRef>, PublicError> {
        let value = self.read.get(record, "parent")?;
        if !self.read.truthy(&value) {
            Ok(None)
        } else {
            self.id(&value).map(Some)
        }
    }
    fn text(
        &mut self,
        record: &R::Value,
        key: &'static str,
    ) -> Result<Option<String>, PublicError> {
        let value = self.read.get(record, key)?;
        self.string(record, key, &value)
    }
    fn flag(&mut self, record: &R::Value, key: &'static str) -> Result<bool, PublicError> {
        let value = self.read.get(record, key)?;
        Ok(self.read.truthy(&value))
    }
    fn number(&mut self, record: &R::Value, key: &'static str) -> Result<Option<f64>, PublicError> {
        let value = self.read.get(record, key)?;
        Ok(Some(match self.read.kind(&value) {
            LiveValueKind::Undefined => f64::NAN,
            LiveValueKind::Null => 0.,
            LiveValueKind::Boolean => {
                if self.read.truthy(&value) {
                    1.
                } else {
                    0.
                }
            }
            LiveValueKind::Number => self.read.number(&value).unwrap(),
            LiveValueKind::String => self
                .read
                .string_units(&value)?
                .and_then(|v| v.well_formed())
                .map_or(f64::NAN, |v| super::public_model::js_number(&v)),
            _ => return Err(invalid("C4 object-to-number coercion is not implemented")),
        }))
    }
    fn lookup(&mut self, alias: &str) -> Result<Option<ElementRef>, PublicError> {
        self.missing_helper()?;
        let map = self.root_field("shapeIndex")?;
        let mut value = self.read.map_get_text(&map, alias)?;
        if matches!(
            self.read.kind(&value),
            LiveValueKind::Null | LiveValueKind::Undefined
        ) {
            let map = self.root_field("boundaryIndex")?;
            value = self.read.map_get_text(&map, alias)?;
        }
        if !self.read.truthy(&value) {
            Ok(None)
        } else {
            self.id(&value).map(Some)
        }
    }
    fn missing_helper(&mut self) -> Result<(), PublicError> {
        if !self.helper {
            return Err(PublicError::JavaScript {
                name: JavaScriptErrorKind::TypeError,
                message: "t.getC4Shape is not a function".into(),
            });
        }
        let helper = self.root_field("getC4Shape")?;
        let kind = self.read.kind(&helper);
        let identity = self.read.object_identity(&helper)?;
        if kind == self.helper_kind && identity == self.helper_identity {
            return Ok(());
        }
        if kind != LiveValueKind::Function {
            return Err(PublicError::JavaScript {
                name: JavaScriptErrorKind::TypeError,
                message: "t.getC4Shape is not a function".into(),
            });
        }
        Err(invalid(
            "C4 missing or replaced getC4Shape render callbacks are not implemented",
        ))
    }

    fn retain_label(&mut self, text: Option<&str>) -> Result<(), PublicError> {
        self.retained = self
            .retained
            .checked_add(text.map_or(0, |s| s.encode_utf16().count()))
            .ok_or_else(limit)?;
        if self.retained > 16_000_000 {
            Err(limit())
        } else {
            Ok(())
        }
    }
    fn step(&mut self) -> Result<(), PublicError> {
        self.steps = self.steps.checked_add(1).ok_or_else(limit)?;
        if self.steps > 8_000_000 {
            Err(limit())
        } else {
            Ok(())
        }
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    seed: &C4DetachedInput,
    _config: &Object,
    settings: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    if let Some(error) = &seed.error {
        return Err(error.clone().into());
    }
    // Capture the authenticated pre-callback method once, not at each redraw.
    // C15's inherited graph-method read is Undefined; web reads the real function.
    let root = read.root();
    let helper = read.get(&root, "getC4Shape")?;
    let helper_kind = read.kind(&helper);
    let helper_identity = read.object_identity(&helper)?;
    super::render::render_phases(
        &mut Host {
            read,
            helper: seed.helper,
            helper_kind,
            helper_identity,
            ids: HashMap::new(),
            text: HashMap::new(),
            units: 0,
            retained: 0,
            steps: 0,
        },
        settings,
        resources,
    )
}
