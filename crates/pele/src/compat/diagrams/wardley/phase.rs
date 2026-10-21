//! Field access for the single Wardley drawing algorithm.
use super::model::*;
use crate::compat::PeleError;
pub(super) trait PhaseInput {
    type Error: From<PeleError>;
    type Value: Clone;
    fn begin_draw(&mut self) {}
    fn root(&mut self) -> Self::Value;
    fn get(&mut self, value: &Self::Value, key: &'static str) -> Result<Self::Value, Self::Error>;
    fn count(&mut self, value: &Self::Value) -> Result<usize, Self::Error>;
    fn index(&mut self, value: &Self::Value, index: usize) -> Result<Self::Value, Self::Error>;
    fn has_index(&mut self, value: &Self::Value, index: usize) -> Result<bool, Self::Error> {
        Ok(index < self.count(value)?)
    }
    fn nullish(&self, value: &Self::Value) -> bool;
    fn truthy(&self, value: &Self::Value) -> bool;
    fn number(&self, value: &Self::Value) -> Option<f64>;
    fn text(&mut self, value: &Self::Value) -> Result<String, Self::Error>;
    fn identity(&mut self, value: &Self::Value) -> Result<u64, Self::Error>;
    fn work(&mut self, _count: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    fn keep_label(&mut self, _units: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    fn field_text(
        &mut self,
        v: &Self::Value,
        k: &'static str,
    ) -> Result<Option<String>, Self::Error> {
        let t = self.get(v, k)?;
        if self.nullish(&t) {
            Ok(None)
        } else {
            self.text(&t).map(Some)
        }
    }
    fn field_number(
        &mut self,
        v: &Self::Value,
        k: &'static str,
    ) -> Result<Option<f64>, Self::Error> {
        let t = self.get(v, k)?;
        Ok(self.number(&t))
    }
    fn field_truth(&mut self, v: &Self::Value, k: &'static str) -> Result<bool, Self::Error> {
        let t = self.get(v, k)?;
        Ok(self.truthy(&t))
    }
}
#[derive(Clone, Copy)]
pub(super) enum TypedValue<'a> {
    Undefined,
    Text(&'a str),
    Number(f64),
    Bool(bool),
    Root(&'a WardleyModel),
    Node(&'a WardleyNode),
    Link(&'a WardleyLink),
    Trend(&'a WardleyTrend),
    Pipeline(&'a WardleyPipeline),
    Annotation(&'a WardleyAnnotation),
    Note(&'a WardleyNote),
    Force(&'a WardleyForce),
    Point(&'a WardleyPoint),
    Axes(&'a WardleyAxes),
    Size(&'a WardleySize),
    Nodes(&'a [WardleyNode]),
    Links(&'a [WardleyLink]),
    Trends(&'a [WardleyTrend]),
    Pipelines(&'a [WardleyPipeline]),
    Annotations(&'a [WardleyAnnotation]),
    Notes(&'a [WardleyNote]),
    Forces(&'a [WardleyForce]),
    Points(&'a [WardleyPoint]),
    Strings(&'a [String]),
    Numbers(&'a [f64]),
}
pub(super) struct Typed<'a>(pub &'a WardleyModel);
impl<'a> PhaseInput for Typed<'a> {
    type Error = PeleError;
    type Value = TypedValue<'a>;
    fn root(&mut self) -> Self::Value {
        TypedValue::Root(self.0)
    }
    fn get(&mut self, v: &Self::Value, k: &'static str) -> Result<Self::Value, PeleError> {
        use TypedValue::*;
        Ok(match (*v, k) {
            (Root(v), "title") => ot(v.title.as_deref()),
            (Root(v), "accTitle") => ot(v.acc_title.as_deref()),
            (Root(v), "accDescr") => ot(v.acc_descr.as_deref()),
            (Root(v), "axes") => Axes(&v.axes),
            (Root(v), "size") => v.size.as_ref().map_or(Undefined, Size),
            (Root(v), "annotationsBox") => v.annotations_box.as_ref().map_or(Undefined, Point),
            (Root(v), "nodes") => Nodes(&v.nodes),
            (Root(v), "links") => Links(&v.links),
            (Root(v), "trends") => Trends(&v.trends),
            (Root(v), "pipelines") => Pipelines(&v.pipelines),
            (Root(v), "annotations") => Annotations(&v.annotations),
            (Root(v), "notes") => Notes(&v.notes),
            (Root(v), "accelerators") => Forces(&v.accelerators),
            (Root(v), "deaccelerators") => Forces(&v.deaccelerators),
            (Node(v), "id") => Text(&v.id),
            (Node(v), "label") => Text(&v.label),
            (Node(v), "x") => Number(v.x),
            (Node(v), "y") => Number(v.y),
            (Node(v), "className") => ot(v.class_name.as_deref()),
            (Node(v), "sourceStrategy") => ot(v.source_strategy.as_deref()),
            (Node(v), "labelOffsetX") => v.label_offset_x.map_or(Undefined, Number),
            (Node(v), "labelOffsetY") => v.label_offset_y.map_or(Undefined, Number),
            (Node(v), "inertia") => v.inertia.map_or(Undefined, Bool),
            (Node(v), "inPipeline") => v.in_pipeline.map_or(Undefined, Bool),
            (Node(v), "isPipelineParent") => v.is_pipeline_parent.map_or(Undefined, Bool),
            (Link(v), "source") => Text(&v.source),
            (Link(v), "target") => Text(&v.target),
            (Link(v), "dashed") => Bool(v.dashed),
            (Link(v), "label") => ot(v.label.as_deref()),
            (Link(v), "flow") => ot(v.flow.as_deref()),
            (Trend(v), "nodeId") => Text(&v.node_id),
            (Trend(v), "targetX") => Number(v.target_x),
            (Trend(v), "targetY") => Number(v.target_y),
            (Pipeline(v), "nodeId") => Text(&v.node_id),
            (Pipeline(v), "componentIds") => Strings(&v.component_ids),
            (Annotation(v), "number") => Number(v.number),
            (Annotation(v), "text") => ot(v.text.as_deref()),
            (Annotation(v), "coordinates") => Points(&v.coordinates),
            (Note(v), "text") => Text(&v.text),
            (Note(v), "x") => Number(v.x),
            (Note(v), "y") => Number(v.y),
            (Force(v), "name") => Text(&v.name),
            (Force(v), "x") => Number(v.x),
            (Force(v), "y") => Number(v.y),
            (Point(v), "x") => Number(v.x),
            (Point(v), "y") => Number(v.y),
            (Size(v), "width") => Number(v.width),
            (Size(v), "height") => Number(v.height),
            (Axes(v), "xLabel") => ot(v.x_label.as_deref()),
            (Axes(v), "yLabel") => ot(v.y_label.as_deref()),
            (Axes(v), "stages") => v.stages.as_ref().map_or(Undefined, |s| Strings(s)),
            (Axes(v), "stageBoundaries") => v
                .stage_boundaries
                .as_ref()
                .map_or(Undefined, |s| Numbers(s)),
            _ => Undefined,
        })
    }
    fn count(&mut self, v: &Self::Value) -> Result<usize, PeleError> {
        use TypedValue::*;
        Ok(match v {
            Nodes(v) => v.len(),
            Links(v) => v.len(),
            Trends(v) => v.len(),
            Pipelines(v) => v.len(),
            Annotations(v) => v.len(),
            Notes(v) => v.len(),
            Forces(v) => v.len(),
            Points(v) => v.len(),
            Strings(v) => v.len(),
            Numbers(v) => v.len(),
            _ => 0,
        })
    }
    fn index(&mut self, v: &Self::Value, i: usize) -> Result<Self::Value, PeleError> {
        use TypedValue::*;
        Ok(match v {
            Nodes(v) => v.get(i).map_or(Undefined, Node),
            Links(v) => v.get(i).map_or(Undefined, Link),
            Trends(v) => v.get(i).map_or(Undefined, Trend),
            Pipelines(v) => v.get(i).map_or(Undefined, Pipeline),
            Annotations(v) => v.get(i).map_or(Undefined, Annotation),
            Notes(v) => v.get(i).map_or(Undefined, Note),
            Forces(v) => v.get(i).map_or(Undefined, Force),
            Points(v) => v.get(i).map_or(Undefined, Point),
            Strings(v) => v.get(i).map_or(Undefined, |s| Text(s)),
            Numbers(v) => v.get(i).map_or(Undefined, |n| Number(*n)),
            _ => Undefined,
        })
    }
    fn nullish(&self, v: &Self::Value) -> bool {
        matches!(v, TypedValue::Undefined)
    }
    fn truthy(&self, v: &Self::Value) -> bool {
        use TypedValue::*;
        match v {
            Undefined => false,
            Bool(b) => *b,
            Text(s) => !s.is_empty(),
            Number(n) => *n != 0. && !n.is_nan(),
            _ => true,
        }
    }
    fn number(&self, v: &Self::Value) -> Option<f64> {
        if let TypedValue::Number(n) = v {
            Some(*n)
        } else {
            None
        }
    }
    fn text(&mut self, v: &Self::Value) -> Result<String, PeleError> {
        if let TypedValue::Text(s) = v {
            Ok((*s).into())
        } else {
            Ok(String::new())
        }
    }
    fn identity(&mut self, v: &Self::Value) -> Result<u64, PeleError> {
        if let TypedValue::Node(n) = v {
            Ok(*n as *const WardleyNode as usize as u64)
        } else {
            Ok(0)
        }
    }
}
fn ot(v: Option<&str>) -> TypedValue<'_> {
    v.map_or(TypedValue::Undefined, TypedValue::Text)
}
