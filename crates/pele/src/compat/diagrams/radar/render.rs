//! Pele diagrams/radar/render.ts and common/fit-width.ts, 458322a5 (MIT, kepano).
use super::RadarModel;
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    public::LiveGeometryError,
    svg::{self, esc_text, js_num, label_svg, num, series_color},
    text::layout_label,
    Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
use std::f64::consts::PI;
#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}
fn option(config: &Object, key: &str, fallback: f64, min: f64, max: f64) -> f64 {
    match config
        .get("radar")
        .and_then(Value::as_object)
        .and_then(|v| v.get(key))
    {
        Some(Value::Number(value)) if *value >= min => value.js_min(max),
        _ => fallback,
    }
}
fn relative_radius(value: f64, min: f64, max: f64, radius: f64) -> f64 {
    radius * (value.js_max(min).js_min(max) - min) / (max - min)
}
fn polygon(points: &[Point]) -> String {
    points
        .iter()
        .map(|p| format!("{},{}", num(p.x), num(p.y)))
        .collect::<Vec<_>>()
        .join(" ")
}
fn closed_round_curve(points: &[Point], tension: f64) -> String {
    let n = points.len();
    let mut d = format!("M{},{}", num(points[0].x), num(points[0].y));
    for i in 0..n {
        let p0 = points[(i + n - 1) % n];
        let p1 = points[i];
        let p2 = points[(i + 1) % n];
        let p3 = points[(i + 2) % n];
        d.push_str(&format!(
            " C{},{} {},{} {},{}",
            num(p1.x + (p2.x - p0.x) * tension),
            num(p1.y + (p2.y - p0.y) * tension),
            num(p2.x - (p3.x - p1.x) * tension),
            num(p2.y - (p3.y - p1.y) * tension),
            num(p2.x),
            num(p2.y)
        ))
    }
    d + " Z"
}
pub(super) struct Geometry {
    pub count: usize,
    pub present: Vec<bool>,
    pub ticks: f64,
    pub circle: bool,
    pub min: f64,
    pub max: f64,
}
pub(super) trait PhaseInput {
    type Error: From<PeleError>;
    fn missing_point(&mut self, site: LiveGeometryError) -> Self::Error;
    fn begin_draw(&mut self) -> Result<Geometry, Self::Error>;
    fn axis_label(&mut self, index: usize) -> Result<Option<String>, Self::Error>;
    fn axis_name(&mut self, index: usize) -> Result<String, Self::Error>;
    fn curves_len(&mut self) -> Result<usize, Self::Error>;
    fn curve_entries(&mut self, index: usize) -> Result<Vec<f64>, Self::Error>;
    fn curve_name(&mut self, index: usize) -> Result<String, Self::Error>;
    fn show_legend(&mut self) -> Result<bool, Self::Error>;
    fn curve_label(&mut self, index: usize) -> Result<Option<String>, Self::Error>;
    fn title(&mut self) -> Result<Option<String>, Self::Error>;
    fn accessibility(&mut self, description: bool) -> Result<Option<String>, Self::Error>;
}
struct Typed<'a>(&'a RadarModel);
impl PhaseInput for Typed<'_> {
    type Error = PeleError;
    fn missing_point(&mut self, _site: LiveGeometryError) -> PeleError {
        let mut error = PeleError::new(
            "Cannot read properties of undefined (reading 'x')",
            crate::compat::ErrorCode::Semantic,
        );
        error.name = "TypeError".into();
        error
    }
    fn begin_draw(&mut self) -> Result<Geometry, PeleError> {
        let max = self.0.options.max.unwrap_or_else(|| {
            let mut max = f64::NEG_INFINITY;
            for curve in &self.0.curves {
                for &value in &curve.entries {
                    if value > max {
                        max = value;
                    }
                }
            }
            max
        });
        Ok(Geometry {
            count: self.0.axes.len(),
            present: vec![true; self.0.axes.len()],
            ticks: self.0.options.ticks,
            circle: self.0.options.graticule == "circle",
            min: self.0.options.min,
            max,
        })
    }
    fn axis_label(&mut self, i: usize) -> Result<Option<String>, PeleError> {
        Ok(Some(self.0.axes[i].label.clone()))
    }
    fn axis_name(&mut self, i: usize) -> Result<String, PeleError> {
        Ok(self.0.axes[i].name.clone())
    }
    fn curves_len(&mut self) -> Result<usize, PeleError> {
        Ok(self.0.curves.len())
    }
    fn curve_entries(&mut self, i: usize) -> Result<Vec<f64>, PeleError> {
        Ok(self.0.curves[i].entries.clone())
    }
    fn curve_name(&mut self, i: usize) -> Result<String, PeleError> {
        Ok(self.0.curves[i].name.clone())
    }
    fn show_legend(&mut self) -> Result<bool, PeleError> {
        Ok(self.0.options.show_legend)
    }
    fn curve_label(&mut self, i: usize) -> Result<Option<String>, PeleError> {
        Ok(Some(self.0.curves[i].label.clone()))
    }
    fn title(&mut self) -> Result<Option<String>, PeleError> {
        Ok(self.0.title.clone())
    }
    fn accessibility(&mut self, d: bool) -> Result<Option<String>, PeleError> {
        Ok(if d {
            self.0.acc_descr.clone()
        } else {
            self.0.acc_title.clone()
        })
    }
}
pub(super) fn render(
    model: &RadarModel,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_phases(&mut Typed(model), config, options, resources)
}
pub(super) fn render_phases<P: PhaseInput>(
    input: &mut P,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    let natural = option(config, "width", 320., 1., 4000.);
    let height = option(config, "height", 320., 1., 4000.);
    let mut result = draw(
        input,
        config,
        options,
        resources,
        natural,
        height * natural / natural,
        false,
    )?;
    let Some(available) = options.max_width.filter(|v| *v > 0.) else {
        return Ok(result);
    };
    if result.width <= available {
        return Ok(result);
    }
    let floor = 160f64.js_min(natural);
    let mut plot = natural;
    result = draw(
        input,
        config,
        options,
        resources,
        plot,
        height * plot / natural,
        true,
    )?;
    for _ in 0..3 {
        if !(result.width > available && plot > floor) {
            break;
        }
        plot = (plot - (result.width - available)).js_max(floor);
        result = draw(
            input,
            config,
            options,
            resources,
            plot,
            height * plot / natural,
            true,
        )?
    }
    Ok(result)
}
fn draw<P: PhaseInput>(
    input: &mut P,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
    width: f64,
    height: f64,
    tight: bool,
) -> Result<Rendered, P::Error> {
    let size = options.font_size.unwrap_or(16.);
    let small = round(size * 0.875);
    let pad = options.padding.unwrap_or(8.);
    let axis_scale = option(config, "axisScaleFactor", 1., 0., 10.);
    let label_factor = option(config, "axisLabelFactor", 1.05, 0., 10.);
    let tension = option(config, "curveTension", 0.17, 0., 1.);
    let radius = width.js_min(height) / 2.;
    let Geometry {
        count,
        present,
        ticks,
        circle,
        min,
        max,
    } = input.begin_draw()?;
    let unit: Vec<_> = (0..count)
        .map(|i| {
            let angle = 2. * PI * i as f64 / count as f64 - PI / 2.;
            Point {
                x: angle.cos(),
                y: angle.sin(),
            }
        })
        .collect();
    let (mut x0, mut x1, mut y0, mut y1) = (
        -width / 2. - option(config, "marginLeft", 16., 0., 2000.),
        width / 2. + option(config, "marginRight", 16., 0., 2000.),
        -height / 2. - option(config, "marginTop", 16., 0., 2000.),
        height / 2. + option(config, "marginBottom", 16., 0., 2000.),
    );
    let mut grid = String::new();
    let mut i = 0;
    while (i as f64) < ticks {
        let r = radius.js_min(radius * (i as f64 + 1.) / ticks);
        if circle {
            grid.push_str(&format!("<circle r=\"{}\"/>", num(r)))
        } else {
            let points = unit
                .iter()
                .enumerate()
                .map(|(index, u)| {
                    if !present[index] {
                        return Err(input.missing_point(LiveGeometryError::PolygonPoint));
                    }
                    Ok(Point {
                        x: u.x * r,
                        y: u.y * r,
                    })
                })
                .collect::<Result<Vec<_>, P::Error>>()?;
            grid.push_str(&format!("<polygon points=\"{}\"/>", polygon(&points)))
        }
        i += 1
    }
    let (mut spokes, mut labels) = (String::new(), String::new());
    for (i, u) in unit.iter().enumerate() {
        if !present[i] {
            continue;
        }
        let reach = radius * axis_scale;
        spokes.push_str(&format!("M0,0L{},{}", num(u.x * reach), num(u.y * reach)));
        let label_text = input.axis_label(i)?;
        let label = layout_label(
            label_text.as_deref(),
            false,
            resources,
            options,
            small,
            4000.,
            0,
        )?;
        let at = radius * label_factor + 6.;
        let cx = u.x * at
            + if u.x > 0.01 {
                label.width / 2.
            } else if u.x < -0.01 {
                -label.width / 2.
            } else {
                0.
            };
        let cy = u.y * at
            + if u.y > 0.01 {
                label.height / 2.
            } else if u.y < -0.01 {
                -label.height / 2.
            } else {
                0.
            };
        labels.push_str(&format!(
            "<g class=\"pele-axis-label\" data-id=\"{}\">{}</g>",
            esc_text(&input.axis_name(i)?)?,
            label_svg(&label, cx, cy, "")
        ));
        x0 = x0.js_min(cx - label.width / 2.);
        x1 = x1.js_max(cx + label.width / 2.);
        y0 = y0.js_min(cy - label.height / 2.);
        y1 = y1.js_max(cy + label.height / 2.)
    }
    let mut shapes = String::new();
    let curves_len = input.curves_len()?;
    for index in 0..curves_len {
        let entries = input.curve_entries(index)?;
        if entries.len() != count {
            continue;
        }
        let points: Vec<_> = entries
            .iter()
            .enumerate()
            .map(|(i, value)| {
                let r = relative_radius(*value, min, max, radius);
                let safe = if r.is_finite() { r } else { 0. };
                if !present[i] {
                    return Err(input.missing_point(LiveGeometryError::IndexedPoint));
                }
                let u = unit[i];
                Ok(Point {
                    x: u.x * safe,
                    y: u.y * safe,
                })
            })
            .collect::<Result<Vec<_>, P::Error>>()?;
        let color = series_color(index);
        let attrs = format!(
            " class=\"pele-curve\" data-id=\"{}\" fill=\"{color}\" stroke=\"{color}\"",
            esc_text(&input.curve_name(index)?)?
        );
        shapes.push_str(&if circle {
            if points.is_empty() {
                return Err(input.missing_point(LiveGeometryError::FirstCurvePoint));
            }
            format!(
                "<path{attrs} d=\"{}\"/>",
                closed_round_curve(&points, tension)
            )
        } else {
            format!("<polygon{attrs} points=\"{}\"/>", polygon(&points))
        })
    }
    let mut legend = Vec::new();
    if input.show_legend()? {
        let length = input.curves_len()?;
        for index in 0..length {
            let text = input.curve_label(index)?;
            legend.push(layout_label(
                text.as_deref(),
                false,
                resources,
                options,
                small,
                4000.,
                0,
            )?)
        }
    }
    let legend_width = legend.iter().map(|l| l.width).fold(0., NumberMath::js_max);
    let legend_height = legend.len() as f64 * 26.;
    let legend_x = if tight {
        -(12. + 8. + legend_width) / 2.
    } else {
        x1 + 24.
    };
    let legend_top = if tight { y1 + 12. } else { -legend_height / 2. };
    if !legend.is_empty() && tight {
        x0 = x0.js_min(legend_x);
        x1 = x1.js_max(-legend_x);
        y1 = legend_top + legend_height
    } else if !legend.is_empty() {
        x1 = legend_x + 12. + 8. + legend_width;
        y0 = y0.js_min(-legend_height / 2.);
        y1 = y1.js_max(legend_height / 2.)
    }
    let mut legend_out = String::new();
    for (index, label) in legend.iter().enumerate() {
        let y = legend_top + index as f64 * 26. + 13.;
        let color = series_color(index);
        legend_out.push_str(&format!("<g class=\"pele-legend-item\" data-id=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"12\" height=\"12\" rx=\"2\" fill=\"{color}\" fill-opacity=\"0.4\" stroke=\"{color}\"/>{}</g>",esc_text(&input.curve_name(index)?)?,num(legend_x),num(y-6.),label_svg(label,legend_x+12.+8.+label.width/2.,y,"")))
    }
    let title_room = options
        .max_width
        .filter(|v| *v > 0.)
        .map_or(4000., |v| (v - 2. * pad).js_max(80.));
    let title_text = input.title()?;
    let title = layout_label(
        title_text.as_deref(),
        false,
        resources,
        options,
        size,
        title_room,
        1,
    )?;
    let title_height = if title.height > 0. {
        title.height + 8.
    } else {
        0.
    };
    let body_width = (x1 - x0).js_max(title.width);
    let total_width = (body_width + 2. * pad).ceil();
    let total_height = (y1 - y0 + title_height + 2. * pad).ceil();
    let origin = format!(
        "translate({},{})",
        num(pad + (body_width - (x1 - x0)) / 2. - x0),
        num(pad + title_height - y0)
    );
    let mut content = label_svg(
        &title,
        total_width / 2.,
        pad + title.height / 2.,
        " class=\"pele-title\" font-weight=\"var(--_tw)\"",
    );
    content.push_str(&format!("<g transform=\"{origin}\">"));
    if !grid.is_empty() {
        content.push_str(&format!(
            "<g class=\"pele-graticule\" fill=\"none\" stroke=\"var(--_a)\">{grid}</g>"
        ))
    }
    if !spokes.is_empty() {
        content.push_str(&format!(
            "<path class=\"pele-axes\" d=\"{spokes}\" fill=\"none\" stroke=\"var(--_b)\"/>"
        ))
    }
    if !shapes.is_empty() {
        content.push_str(&format!("<g class=\"pele-curves\" fill-opacity=\"0.2\" stroke-width=\"1.5\" stroke-linejoin=\"round\">{shapes}</g>"))
    }
    if !labels.is_empty() {
        content.push_str(&format!(
            "<g class=\"pele-axis-labels\" font-size=\"{}\">{labels}</g>",
            js_num(small)
        ))
    }
    if !legend_out.is_empty() {
        content.push_str(&format!(
            "<g class=\"pele-legend\" font-size=\"{}\">{legend_out}</g>",
            js_num(small)
        ))
    }
    content.push_str("</g>");
    let acc_title = input.accessibility(false)?;
    let acc_descr = input.accessibility(true)?;
    Ok(Rendered {
        svg: svg::document(
            "radar",
            total_width,
            total_height,
            size,
            options,
            acc_title.as_deref(),
            acc_descr.as_deref(),
            &content,
        )?,
        width: total_width,
        height: total_height,
        links: Vec::new(),
    })
}
