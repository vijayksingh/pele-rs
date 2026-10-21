//! Mutable Radar's plain objects/arrays. Import concrete values, retain the exact identity graph.
//! This native codec does not provide JavaScript prototype dispatch or TypeError identity.
//! Sparse active arrays, active lone surrogates and unsupported numeric coercions report gaps.
//! Independent geometry/text work budgets do not reapply the parser's 32-tick clamp.
use super::{Radar, RadarAxis, RadarCurve, RadarModel, RadarOptions};
use crate::compat::{
    public::{Atom, GraphWriter, ModelGraph, Node, PublicDiagramModule},
    DiagramModule, DiagramType, ErrorCode, Object, PeleError, RenderResources, RenderSettings,
    Rendered,
};
use std::collections::HashMap;
const WORK_BUDGET: usize = 8_000_000;
const TEXT_UNITS_BUDGET: usize = 16_000_000;
pub struct RadarPublicState {
    graph: ModelGraph,
    model: Result<RadarModel, PeleError>,
}
fn invalid(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("radar".into());
    e
}
fn limit(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Limit);
    e.diagram_type = Some("radar".into());
    e
}
fn reference(atom: Option<&Atom>) -> Result<u32, PeleError> {
    match atom {
        Some(Atom::Ref(id)) => Ok(*id),
        _ => Err(invalid("Radar field requires an object reference")),
    }
}
fn field<'a>(g: &'a ModelGraph, id: u32, key: &str) -> Option<&'a Atom> {
    g.nodes
        .get(id as usize)?
        .properties()
        .iter()
        .find_map(|p| g.strings[p.key as usize].is(key).then_some(&p.value))
}
fn array(g: &ModelGraph, id: u32) -> Result<Vec<&Atom>, PeleError> {
    let Some(Node::Array { length, properties }) = g.nodes.get(id as usize) else {
        return Err(invalid("Radar collections must be Arrays"));
    };
    let mut indexed = HashMap::new();
    for p in properties {
        let units = &g.strings[p.key as usize].0;
        if units.is_empty() || units.len() > 10 || (units.len() > 1 && units[0] == 48) {
            continue;
        }
        let mut index = 0u64;
        let mut canonical = true;
        for &c in units {
            if !(48..=57).contains(&c) {
                canonical = false;
                break;
            }
            index = index * 10 + u64::from(c - 48)
        }
        if canonical && index < u64::from(*length) {
            indexed.entry(index as u32).or_insert(&p.value);
        }
    }
    if indexed.len() != *length as usize {
        return Err(invalid("Radar sparse active arrays are not implemented"));
    }
    Ok((0..*length).map(|i| indexed[&i]).collect())
}
struct TextBudget(usize);
impl TextBudget {
    fn text(
        &mut self,
        g: &ModelGraph,
        value: Option<&Atom>,
        undefined_empty: bool,
    ) -> Result<String, PeleError> {
        match value {
            None | Some(Atom::Undefined) if undefined_empty => return Ok(String::new()),
            _ => {}
        }
        let value = value.ok_or_else(|| invalid("Radar text field is absent"))?;
        let text = g.text(value)?;
        self.0 = self
            .0
            .checked_add(text.0.len())
            .ok_or_else(|| limit("Radar imported text size overflow"))?;
        if self.0 > TEXT_UNITS_BUDGET {
            return Err(limit(
                "Radar imported text exceeds native 16000000 code-unit work budget",
            ));
        }
        text.well_formed()
            .ok_or_else(|| invalid("Radar rendering of lone UTF-16 surrogates is not implemented"))
    }
    fn metadata(
        &mut self,
        g: &ModelGraph,
        key: &str,
        accessible: bool,
    ) -> Result<Option<String>, PeleError> {
        let value = g.field(key)?;
        if matches!(value, None | Some(Atom::Undefined)) || (accessible && !truthy(g, value)) {
            return Ok(None);
        }
        self.text(g, value, false).map(Some)
    }
}
fn truthy(g: &ModelGraph, value: Option<&Atom>) -> bool {
    match value {
        None | Some(Atom::Undefined | Atom::Null | Atom::Bool(false)) => false,
        Some(Atom::Number(n)) => *n != 0. && !n.is_nan(),
        Some(Atom::String(id)) => !g.strings[*id as usize].0.is_empty(),
        _ => true,
    }
}
fn number(value: Option<&Atom>) -> Result<f64, PeleError> {
    match value {
        Some(Atom::Number(n)) => Ok(*n),
        None | Some(Atom::Undefined) => Ok(f64::NAN),
        Some(Atom::Null | Atom::Bool(false)) => Ok(0.),
        Some(Atom::Bool(true)) => Ok(1.),
        _ => Err(invalid(
            "Radar numeric coercion for this runtime kind is not implemented",
        )),
    }
}
fn imported(g: &ModelGraph) -> Result<RadarModel, PeleError> {
    let options = reference(g.field("options")?)?;
    let show_legend = truthy(g, field(g, options, "showLegend"));
    let graticule = if matches!(field(g,options,"graticule"),Some(Atom::String(id))if g.strings[*id as usize].is("circle"))
    {
        "circle"
    } else {
        "polygon"
    };
    let ticks = number(field(g, options, "ticks"))?;
    if ticks > WORK_BUDGET as f64 {
        return Err(limit("Radar ticks exceed native 8000000 work budget"));
    }
    let max = match field(g, options, "max") {
        Some(Atom::Null) => None,
        value => Some(number(value)?),
    };
    let min = number(field(g, options, "min"))?;
    let axis_values = array(g, reference(g.field("axes")?)?)?;
    let curve_values = array(g, reference(g.field("curves")?)?)?;
    let grid = ticks.max(0.).ceil() as usize;
    let work = grid
        .saturating_mul(if graticule == "circle" {
            1
        } else {
            axis_values.len()
        })
        .saturating_add(axis_values.len().saturating_mul(curve_values.len()));
    if work > WORK_BUDGET {
        return Err(limit("Radar geometry exceeds native 8000000 work budget"));
    }
    let mut text = TextBudget(0);
    let mut axes = Vec::with_capacity(axis_values.len());
    for value in axis_values {
        let id = reference(Some(value))?;
        axes.push(RadarAxis {
            name: text.text(g, field(g, id, "name"), false)?,
            label: text.text(g, field(g, id, "label"), true)?,
        });
    }
    let mut curves = Vec::with_capacity(curve_values.len());
    for value in curve_values {
        let id = reference(Some(value))?;
        let entries = array(g, reference(field(g, id, "entries"))?)?
            .into_iter()
            .map(|v| number(Some(v)))
            .collect::<Result<Vec<_>, _>>()?;
        if axes.is_empty() && entries.is_empty() && graticule == "circle" {
            return Err(invalid("Radar circle curves require at least one axis"));
        }
        let name = if entries.len() == axes.len() || show_legend {
            text.text(g, field(g, id, "name"), false)?
        } else {
            String::new()
        };
        let label = if show_legend {
            text.text(g, field(g, id, "label"), true)?
        } else {
            String::new()
        };
        curves.push(RadarCurve {
            name,
            label,
            entries,
        });
    }
    Ok(RadarModel {
        diagram_type: DiagramType::Radar,
        title: text.metadata(g, "title", false)?,
        acc_title: text.metadata(g, "accTitle", true)?,
        acc_descr: text.metadata(g, "accDescr", true)?,
        axes,
        curves,
        options: RadarOptions {
            show_legend,
            ticks,
            max,
            min,
            graticule: graticule.into(),
        },
    })
}
fn array_node(out: &mut GraphWriter, values: impl IntoIterator<Item = Atom>) -> u32 {
    let properties: Vec<_> = values
        .into_iter()
        .enumerate()
        .map(|(i, v)| out.property(&i.to_string(), v))
        .collect();
    out.node(Node::Array {
        length: properties.len() as u32,
        properties,
    })
}
impl PublicDiagramModule for Radar {
    type PublicState = RadarPublicState;
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let model = imported(&graph);
        Ok(RadarPublicState { graph, model })
    }
    fn create_public(&self, model: &RadarModel) -> Result<RadarPublicState, PeleError> {
        let mut out = GraphWriter::default();
        let mut axis_ids = vec![];
        let mut curve_ids = vec![];
        for axis in &model.axes {
            let name = out.string(axis.name.as_str());
            let label = out.string(axis.label.as_str());
            let properties = vec![
                out.property("name", Atom::String(name)),
                out.property("label", Atom::String(label)),
            ];
            axis_ids.push(Atom::Ref(out.node(Node::Object { properties })));
        }
        let axes = array_node(&mut out, axis_ids);
        for curve in &model.curves {
            let name = out.string(curve.name.as_str());
            let label = out.string(curve.label.as_str());
            let entries = array_node(&mut out, curve.entries.iter().copied().map(Atom::Number));
            let properties = vec![
                out.property("name", Atom::String(name)),
                out.property("label", Atom::String(label)),
                out.property("entries", Atom::Ref(entries)),
            ];
            curve_ids.push(Atom::Ref(out.node(Node::Object { properties })));
        }
        let curves = array_node(&mut out, curve_ids);
        let style = out.string(model.options.graticule.as_str());
        let properties = vec![
            out.property("showLegend", Atom::Bool(model.options.show_legend)),
            out.property("ticks", Atom::Number(model.options.ticks)),
            out.property("max", model.options.max.map_or(Atom::Null, Atom::Number)),
            out.property("min", Atom::Number(model.options.min)),
            out.property("graticule", Atom::String(style)),
        ];
        let options = out.node(Node::Object { properties });
        let kind = out.string("radar");
        let mut properties = vec![out.property("type", Atom::String(kind))];
        for (key, value) in [
            ("title", &model.title),
            ("accTitle", &model.acc_title),
            ("accDescr", &model.acc_descr),
        ] {
            let value = value
                .as_deref()
                .map_or(Atom::Undefined, |v| Atom::String(out.string(v)));
            properties.push(out.property(key, value));
        }
        properties.extend([
            out.property("axes", Atom::Ref(axes)),
            out.property("curves", Atom::Ref(curves)),
            out.property("options", Atom::Ref(options)),
        ]);
        let root = out.node(Node::Object { properties });
        let graph = out.finish(Atom::Ref(root))?;
        let model = imported(&graph);
        Ok(RadarPublicState { graph, model })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, state: &RadarPublicState) -> Result<ModelGraph, PeleError> {
        Ok(state.graph.clone())
    }
    fn apply_public(
        &self,
        state: &mut RadarPublicState,
        graph: ModelGraph,
    ) -> Result<(), PeleError> {
        state.model = imported(&graph);
        state.graph = graph;
        Ok(())
    }
    fn detached_public_render_capability(
        &self,
    ) -> Option<crate::compat::public::DetachedRenderCapability> {
        Some(crate::compat::public::DetachedRenderCapability::radar())
    }
    fn detached_public_render(
        &self,
        state: &Self::PublicState,
    ) -> Option<crate::compat::public::DetachedPublicRender> {
        Some(crate::compat::public::DetachedPublicRender::radar(
            super::RadarDetachedInput::new(state.model.as_ref().err().cloned()),
        ))
    }
    fn render_public(
        &self,
        state: &mut RadarPublicState,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render(
            state.model.as_ref().map_err(Clone::clone)?,
            config,
            options,
            host,
        )
    }
}
