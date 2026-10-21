//! Phase reads shared by typed XY charts and detached public owners.
use super::model::{LinearAxis, Orientation, Plot, PlotType, XAxis, XyChartModel};
use crate::compat::{ErrorCode, PeleError};
pub(super) trait PhaseInput {
    type Value: Clone;
    type Error: From<PeleError>;
    fn begin_draw(&mut self) {}
    fn root(&mut self) -> Self::Value;
    fn get(&mut self, v: &Self::Value, key: &'static str) -> Result<Self::Value, Self::Error>;
    fn index(&mut self, v: &Self::Value, i: usize) -> Result<Self::Value, Self::Error>;
    fn has(&mut self, v: &Self::Value, i: usize) -> Result<bool, Self::Error>;
    fn count(&mut self, v: &Self::Value) -> Result<usize, Self::Error>;
    fn text(&mut self, v: &Self::Value) -> Result<String, Self::Error>;
    fn number(&self, v: &Self::Value) -> Option<f64>;
    fn nullish(&self, v: &Self::Value) -> bool;
    fn truthy(&self, v: &Self::Value) -> bool;
    fn is_text(&mut self, v: &Self::Value, s: &str) -> Result<bool, Self::Error>;
    fn work(&mut self, _n: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    fn keep_text(&mut self, _n: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    fn optional_text(&mut self, v: &Self::Value) -> Result<Option<String>, Self::Error> {
        if !self.truthy(v) {
            Ok(None)
        } else {
            self.text(v).map(Some)
        }
    }
    fn field_text(
        &mut self,
        v: &Self::Value,
        k: &'static str,
    ) -> Result<Option<String>, Self::Error> {
        let t = self.get(v, k)?;
        self.optional_text(&t)
    }
    fn field_is(&mut self, v: &Self::Value, k: &'static str, s: &str) -> Result<bool, Self::Error> {
        let t = self.get(v, k)?;
        self.is_text(&t, s)
    }
}
#[derive(Clone, Copy)]
pub(super) enum Value<'a> {
    Undefined,
    Text(&'a str),
    Number(f64),
    Bool(bool),
    Root(&'a XyChartModel),
    X(&'a XAxis),
    Y(&'a LinearAxis),
    Plots(&'a [Plot]),
    Plot(&'a Plot),
    Strings(&'a [String]),
    Data(&'a [(String, Option<f64>)]),
    Point(&'a (String, Option<f64>)),
}
pub(super) struct Typed<'a>(pub &'a XyChartModel);
impl<'a> PhaseInput for Typed<'a> {
    type Value = Value<'a>;
    type Error = PeleError;
    fn root(&mut self) -> Self::Value {
        Value::Root(self.0)
    }
    fn get(&mut self, v: &Self::Value, k: &'static str) -> Result<Self::Value, PeleError> {
        use Value::*;
        Ok(match (v, k) {
            (Root(m), "plots") => Plots(&m.plots),
            (Root(m), "xAxis") => X(&m.x_axis),
            (Root(m), "yAxis") => Y(&m.y_axis),
            (Root(m), "title") => m.title.as_deref().map_or(Undefined, Text),
            (Root(m), "accTitle") => m.acc_title.as_deref().map_or(Undefined, Text),
            (Root(m), "accDescr") => m.acc_descr.as_deref().map_or(Undefined, Text),
            (Root(m), "orientation") => m.orientation.map_or(Undefined, |o| {
                Text(if o == Orientation::Horizontal {
                    "horizontal"
                } else {
                    "vertical"
                })
            }),
            (Root(m), "hasSetYAxis") => Bool(m.has_set_y_axis),
            (X(x), "title") => Text(x.title()),
            (X(XAxis::Band { .. }), "type") => Text("band"),
            (X(XAxis::Linear { .. }), "type") => Text("linear"),
            (X(XAxis::Band { categories, .. }), "categories") => Strings(categories),
            (X(XAxis::Linear { min, .. }), "min") => Number(*min),
            (X(XAxis::Linear { max, .. }), "max") => Number(*max),
            (Y(y), "title") => Text(&y.title),
            (Y(y), "min") => Number(y.min),
            (Y(y), "max") => Number(y.max),
            (Plot(p), "title") => Text(&p.title),
            (Plot(p), "type") => Text(if p.plot_type == PlotType::Bar {
                "bar"
            } else {
                "line"
            }),
            (Plot(p), "data") => Data(&p.data),
            (Plot(p), "pointLabels") => p.point_labels.as_deref().map_or(Undefined, Strings),
            _ => Undefined,
        })
    }
    fn index(&mut self, v: &Self::Value, i: usize) -> Result<Self::Value, PeleError> {
        use Value::*;
        Ok(match v {
            Plots(v) => v.get(i).map_or(Undefined, Plot),
            Strings(v) => v.get(i).map_or(Undefined, |s| Text(s)),
            Data(v) => v.get(i).map_or(Undefined, Point),
            Point(v) => match i {
                0 => Text(&v.0),
                1 => v.1.map_or(Undefined, Number),
                _ => Undefined,
            },
            _ => Undefined,
        })
    }
    fn count(&mut self, v: &Self::Value) -> Result<usize, PeleError> {
        use Value::*;
        Ok(match v {
            Plots(v) => v.len(),
            Strings(v) => v.len(),
            Data(v) => v.len(),
            Point(_) => 2,
            _ => 0,
        })
    }
    fn has(&mut self, v: &Self::Value, i: usize) -> Result<bool, PeleError> {
        Ok(i < self.count(v)?)
    }
    fn text(&mut self, v: &Self::Value) -> Result<String, PeleError> {
        if let Value::Text(s) = v {
            Ok((*s).into())
        } else {
            Err(PeleError::new(
                "XYChart text must be a string",
                ErrorCode::Semantic,
            ))
        }
    }
    fn number(&self, v: &Self::Value) -> Option<f64> {
        if let Value::Number(n) = v {
            Some(*n)
        } else {
            None
        }
    }
    fn nullish(&self, v: &Self::Value) -> bool {
        matches!(v, Value::Undefined)
    }
    fn truthy(&self, v: &Self::Value) -> bool {
        match v {
            Value::Undefined => false,
            Value::Text(s) => !s.is_empty(),
            Value::Number(n) => *n != 0. && !n.is_nan(),
            Value::Bool(b) => *b,
            _ => true,
        }
    }
    fn is_text(&mut self, v: &Self::Value, s: &str) -> Result<bool, PeleError> {
        Ok(matches!(v,Value::Text(t) if *t==s))
    }
}
