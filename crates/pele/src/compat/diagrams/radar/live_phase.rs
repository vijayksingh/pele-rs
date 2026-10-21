//! Owned Radar phases retain current-draw arrays while reading their live fields.
use super::render::{Geometry, PhaseInput};
use crate::compat::{
    public::{LiveGeometryError, LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
use std::collections::HashMap;
const WORK: usize = 8_000_000;
const TEXT: usize = 16_000_000;
pub(crate) struct RadarDetachedInput {
    limit: Option<PeleError>,
}
impl RadarDetachedInput {
    pub(super) fn new(error: Option<PeleError>) -> Self {
        Self {
            limit: error.filter(|e| e.code == ErrorCode::Limit),
        }
    }
}
fn error(message: &str, code: ErrorCode) -> PublicError {
    let mut error = PeleError::new(message, code);
    error.diagram_type = Some("radar".into());
    error.into()
}
fn invalid(message: &str) -> PublicError {
    error(message, ErrorCode::Semantic)
}
fn limit(message: &str) -> PublicError {
    error(message, ErrorCode::Limit)
}
struct HostPhase<'a, R: LiveRenderRead> {
    read: &'a mut R,
    axes: Option<R::Value>,
    curves: Option<R::Value>,
    count: usize,
    circle: bool,
    text_lengths: HashMap<(u8, usize), usize>,
    text_units: usize,
}
impl<R: LiveRenderRead> HostPhase<'_, R> {
    fn root_field(&mut self, key: &str) -> Result<R::Value, PublicError> {
        let root = self.read.root();
        self.read.get(&root, key)
    }
    fn length(&mut self, array: &R::Value) -> Result<usize, PublicError> {
        let value = self.read.get(array, "length")?;
        let Some(n) = self.read.number(&value) else {
            return Err(invalid("Radar collections must be Arrays"));
        };
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(invalid("Radar collections must be Arrays"));
        }
        if n > WORK as f64 {
            return Err(limit("Radar geometry exceeds native 8000000 work budget"));
        }
        Ok(n as usize)
    }
    fn number(&self, value: &R::Value) -> Result<f64, PublicError> {
        match self.read.kind(value) {
            LiveValueKind::Number => self
                .read
                .number(value)
                .ok_or_else(|| invalid("Radar numeric value missing")),
            LiveValueKind::Undefined => Ok(f64::NAN),
            LiveValueKind::Null => Ok(0.),
            LiveValueKind::Boolean => Ok(if self.read.truthy(value) { 1. } else { 0. }),
            _ => Err(invalid(
                "Radar numeric coercion for this runtime kind is not implemented",
            )),
        }
    }
    fn text(
        &mut self,
        value: &R::Value,
        receiver: &'static str,
        field: (u8, usize),
    ) -> Result<String, PublicError> {
        let units = self.read.require_string(
            value,
            LiveStringUse {
                method: "includes",
                expression_receiver: receiver,
            },
        )?;
        // Count each consumed field once per draw, using its largest retained
        // value if a callback replaces it between shape and legend phases.
        let previous = self.text_lengths.get(&field).copied().unwrap_or(0);
        let length = previous.max(units.0.len());
        self.text_units = self
            .text_units
            .checked_add(length - previous)
            .ok_or_else(|| limit("Radar imported text size overflow"))?;
        if self.text_units > TEXT {
            return Err(limit(
                "Radar imported text exceeds native 16000000 code-unit work budget",
            ));
        }
        self.text_lengths.insert(field, length);
        units
            .well_formed()
            .ok_or_else(|| invalid("Radar rendering of lone UTF-16 surrogates is not implemented"))
    }
    fn array_field(
        &mut self,
        axes: bool,
        index: usize,
        key: &str,
    ) -> Result<R::Value, PublicError> {
        let array = if axes {
            self.axes.as_ref()
        } else {
            self.curves.as_ref()
        }
        .expect("begin_draw initializes arrays")
        .clone();
        let object = self.read.index(&array, index)?;
        if axes && self.read.kind(&object) == LiveValueKind::Undefined {
            let site = if key == "label" {
                LiveGeometryError::IndexedLabel
            } else {
                LiveGeometryError::IndexedName
            };
            return Err(self.read.geometry_error(site));
        }
        self.read.get(&object, key)
    }
    fn label(&mut self, axes: bool, index: usize) -> Result<Option<String>, PublicError> {
        let value = self.array_field(axes, index, "label")?;
        if self.read.kind(&value) == LiveValueKind::Undefined {
            return Ok(None);
        }
        self.text(&value, "t", (if axes { 1 } else { 3 }, index))
            .map(Some)
    }
}
impl<R: LiveRenderRead> PhaseInput for HostPhase<'_, R> {
    type Error = PublicError;
    fn missing_point(&mut self, site: LiveGeometryError) -> PublicError {
        self.read.geometry_error(site)
    }
    fn begin_draw(&mut self) -> Result<Geometry, PublicError> {
        self.text_lengths.clear();
        self.text_units = 0;
        let axes = self.root_field("axes")?;
        let curves = self.root_field("curves")?;
        let options = self.root_field("options")?;
        let ticks = self.read.get(&options, "ticks")?;
        let ticks = self.number(&ticks)?;
        let graticule = self.read.get(&options, "graticule")?;
        let circle = self
            .read
            .string_units(&graticule)?
            .is_some_and(|s| s.is("circle"));
        let min = self.read.get(&options, "min")?;
        let min = self.number(&min)?;
        let count = self.length(&axes)?;
        // Array.map performs HasProperty and Get in index order, preserving
        // holes in its computed geometry even if callbacks later edit axes.
        let mut present = Vec::with_capacity(count);
        for index in 0..count {
            let has = self.read.has_index(&axes, index)?;
            if has {
                let _ = self.read.index(&axes, index)?;
            }
            present.push(has);
        }
        // Root options is read again at the source's max access.
        let max_options = self.root_field("options")?;
        let max_value = self.read.get(&max_options, "max")?;
        let mut max = if self.read.kind(&max_value) == LiveValueKind::Null {
            f64::NEG_INFINITY
        } else {
            self.number(&max_value)?
        };
        self.axes = Some(axes);
        self.curves = Some(curves);
        self.count = count;
        self.circle = circle;
        let curves_len = self.curves_len()?;
        if ticks > WORK as f64 {
            return Err(limit("Radar ticks exceed native 8000000 work budget"));
        }
        let grid = ticks.max(0.).ceil() as usize;
        if grid
            .saturating_mul(if circle { 1 } else { count })
            .saturating_add(count.saturating_mul(curves_len))
            > WORK
        {
            return Err(limit("Radar geometry exceeds native 8000000 work budget"));
        }
        if self.read.kind(&max_value) == LiveValueKind::Null {
            let mut scanned = 0usize;
            for i in 0..curves_len {
                let entries = self.curve_entries(i)?;
                scanned = scanned.saturating_add(entries.len());
                if scanned > WORK {
                    return Err(limit("Radar geometry exceeds native 8000000 work budget"));
                }
                for value in entries {
                    if value > max {
                        max = value;
                    }
                }
            }
        }
        Ok(Geometry {
            count,
            present,
            ticks,
            circle,
            min,
            max,
        })
    }
    fn axis_label(&mut self, i: usize) -> Result<Option<String>, PublicError> {
        self.label(true, i)
    }
    fn axis_name(&mut self, i: usize) -> Result<String, PublicError> {
        let v = self.array_field(true, i, "name")?;
        self.text(&v, "e", (0, i))
    }
    fn curves_len(&mut self) -> Result<usize, PublicError> {
        let curves = self
            .curves
            .as_ref()
            .expect("begin_draw initializes curves")
            .clone();
        let count = self.length(&curves)?;
        if count.saturating_mul(self.count) > WORK {
            return Err(limit("Radar geometry exceeds native 8000000 work budget"));
        }
        Ok(count)
    }
    fn curve_entries(&mut self, i: usize) -> Result<Vec<f64>, PublicError> {
        let entries = self.array_field(false, i, "entries")?;
        let length = self.length(&entries)?;
        let mut values = Vec::with_capacity(length);
        for i in 0..length {
            let value = self.read.index(&entries, i)?;
            values.push(self.number(&value)?);
        }
        Ok(values)
    }
    fn curve_name(&mut self, i: usize) -> Result<String, PublicError> {
        let v = self.array_field(false, i, "name")?;
        self.text(&v, "e", (2, i))
    }
    fn show_legend(&mut self) -> Result<bool, PublicError> {
        let options = self.root_field("options")?;
        let value = self.read.get(&options, "showLegend")?;
        Ok(self.read.truthy(&value))
    }
    fn curve_label(&mut self, i: usize) -> Result<Option<String>, PublicError> {
        self.label(false, i)
    }
    fn title(&mut self) -> Result<Option<String>, PublicError> {
        let v = self.root_field("title")?;
        if self.read.kind(&v) == LiveValueKind::Undefined {
            return Ok(None);
        }
        self.text(&v, "t", (4, 0)).map(Some)
    }
    fn accessibility(&mut self, description: bool) -> Result<Option<String>, PublicError> {
        let key = if description { "accDescr" } else { "accTitle" };
        let v = self.root_field(key)?;
        if !self.read.truthy(&v) {
            return Ok(None);
        }
        let v = self.root_field(key)?;
        self.text(&v, "e", (if description { 6 } else { 5 }, 0))
            .map(Some)
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    seed: &RadarDetachedInput,
    config: &Object,
    settings: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    if let Some(error) = &seed.limit {
        return Err(error.clone().into());
    }
    let mut input = HostPhase {
        read,
        axes: None,
        curves: None,
        count: 0,
        circle: false,
        text_lengths: HashMap::new(),
        text_units: 0,
    };
    super::render::render_phases(&mut input, config, settings, resources)
}

/// Borrowed public data uses the same phase schedule as detached host values.
/// In particular, sparse axes cannot be eagerly projected into a dense DTO.
pub(super) fn render_graph(
    graph: &crate::compat::public::ModelGraph,
    seed: &RadarDetachedInput,
    config: &Object,
    settings: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    use crate::compat::public::{Atom, JavaScriptErrorKind, JsString, ModelGraph, Node};
    struct Read<'a>(&'a ModelGraph);
    fn type_error(message: String) -> PublicError {
        PublicError::JavaScript {
            name: JavaScriptErrorKind::TypeError,
            message: JsString::from(message.as_str()),
        }
    }
    impl LiveRenderRead for Read<'_> {
        type Value = Atom;
        fn root(&self) -> Atom {
            self.0.root.clone()
        }
        fn get(&mut self, receiver: &Atom, key: &str) -> Result<Atom, PublicError> {
            match receiver {
                Atom::Undefined | Atom::Null => {
                    return Err(type_error(format!(
                        "Cannot read properties of {} (reading '{key}')",
                        if matches!(receiver, Atom::Null) {
                            "null"
                        } else {
                            "undefined"
                        }
                    )))
                }
                Atom::Ref(id) => {
                    let node = &self.0.nodes[*id as usize];
                    if let Node::Array { length, .. } = node {
                        if key == "length" {
                            return Ok(Atom::Number(f64::from(*length)));
                        }
                    }
                    return Ok(node
                        .properties()
                        .iter()
                        .find(|p| self.0.strings[p.key as usize].is(key))
                        .map_or(Atom::Undefined, |p| p.value.clone()));
                }
                _ => {}
            }
            Ok(Atom::Undefined)
        }
        fn index(&mut self, receiver: &Atom, index: usize) -> Result<Atom, PublicError> {
            self.get(receiver, &index.to_string())
        }
        fn has_index(&mut self, receiver: &Atom, index: usize) -> Result<bool, PublicError> {
            let Atom::Ref(id) = receiver else {
                return Err(invalid("Radar collections must be Arrays"));
            };
            let key = index.to_string();
            Ok(self.0.nodes[*id as usize]
                .properties()
                .iter()
                .any(|p| self.0.strings[p.key as usize].is(&key)))
        }
        fn kind(&self, value: &Atom) -> LiveValueKind {
            match value {
                Atom::Undefined => LiveValueKind::Undefined,
                Atom::Null => LiveValueKind::Null,
                Atom::Bool(_) => LiveValueKind::Boolean,
                Atom::Number(_) => LiveValueKind::Number,
                Atom::String(_) => LiveValueKind::String,
                Atom::BuiltinFunction(_) => LiveValueKind::Function,
                _ => LiveValueKind::Object,
            }
        }
        fn number(&self, value: &Atom) -> Option<f64> {
            if let Atom::Number(v) = value {
                Some(*v)
            } else {
                None
            }
        }
        fn truthy(&self, value: &Atom) -> bool {
            match value {
                Atom::Undefined | Atom::Null | Atom::Bool(false) => false,
                Atom::Number(n) => *n != 0. && !n.is_nan(),
                Atom::String(id) => !self.0.strings[*id as usize].0.is_empty(),
                _ => true,
            }
        }
        fn string_units(&self, value: &Atom) -> Result<Option<JsString>, PublicError> {
            Ok(if let Atom::String(id) = value {
                Some(self.0.strings[*id as usize].clone())
            } else {
                None
            })
        }
        fn to_string(&mut self, value: &Atom) -> Result<JsString, PublicError> {
            self.require_string(
                value,
                LiveStringUse {
                    method: "includes",
                    expression_receiver: "e",
                },
            )
        }
        fn require_string(
            &mut self,
            value: &Atom,
            site: LiveStringUse,
        ) -> Result<JsString, PublicError> {
            if let Some(s) = self.string_units(value)? {
                return Ok(s);
            }
            Err(type_error(format!(
                "{}.{} is not a function",
                site.expression_receiver, site.method
            )))
        }
    }
    render_detached(seed, config, settings, &mut Read(graph), resources)
}
