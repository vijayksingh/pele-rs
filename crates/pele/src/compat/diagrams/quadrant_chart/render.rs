//! Pele diagrams/quadrant/render.ts, pinned 458322a5 (MIT, kepano).
use super::{PointStyle, QuadrantModel, QuadrantText};
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    svg::{self, edges::theme::resolve_style, esc_text, label_svg, num, series_color},
    text::{layout_label, Label},
    Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
// Number.parseInt without a radix accepts a sign, decimal prefix or 0x hex.
fn parse_int(text: &str) -> f64 {
    let text = text.trim_start_matches(crate::compat::source::space);
    let (sign, text) = if let Some(s) = text.strip_prefix('-') {
        (-1., s)
    } else if let Some(s) = text.strip_prefix('+') {
        (1., s)
    } else {
        (1., text)
    };
    let hex = text.starts_with("0x") || text.starts_with("0X");
    let text = if hex { &text[2..] } else { text };
    let n = text
        .bytes()
        .take_while(|c| {
            if hex {
                c.is_ascii_hexdigit()
            } else {
                c.is_ascii_digit()
            }
        })
        .count();
    if n == 0 {
        return f64::NAN;
    }
    let text = &text[..n];
    let value = if !hex {
        text.parse::<f64>().unwrap_or(f64::INFINITY)
    } else {
        let mut leading = true;
        let (mut bits, mut mant, mut round, mut sticky) = (0usize, 0u64, false, false);
        for c in text.bytes() {
            let digit = (c as char).to_digit(16).unwrap();
            for k in (0..4).rev() {
                let bit = (digit >> k) & 1;
                if leading && bit == 0 {
                    continue;
                }
                leading = false;
                bits += 1;
                if bits <= 53 {
                    mant = (mant << 1) | bit as u64
                } else if bits == 54 {
                    round = bit != 0
                } else {
                    sticky |= bit != 0
                }
            }
        }
        if bits <= 53 {
            mant as f64
        } else if bits > 1024 {
            f64::INFINITY
        } else {
            if round && (sticky || mant & 1 != 0) {
                mant += 1
            }
            (mant as f64) * 2_f64.powi((bits - 53) as i32)
        }
    };
    sign * value
}
fn setting<'a>(config: &'a Object, key: &str) -> Option<&'a Value> {
    config
        .get("quadrantChart")
        .and_then(Value::as_object)
        .and_then(|c| c.get(key))
}
fn length(config: &Object, key: &str, fallback: f64, max: f64) -> f64 {
    match setting(config, key) {
        Some(Value::Number(n)) if *n >= 0. => n.js_min(max),
        _ => fallback,
    }
}
fn label(
    text: Option<&QuadrantText>,
    size: f64,
    max: f64,
    r: &mut dyn RenderResources,
    o: &RenderSettings,
) -> Result<Label, PeleError> {
    layout_label(
        text.map(|t| t.text.as_str()),
        text.is_some_and(|t| t.label_type == "markdown"),
        r,
        o,
        size,
        max.js_max(40.),
        0,
    )
}
fn clear(taken: &[f64], from: usize, x0: f64, y0: f64, x1: f64, y1: f64) -> bool {
    for k in (from..taken.len()).step_by(4) {
        if x0 < taken[k + 2] && x1 > taken[k] && y0 < taken[k + 3] && y1 > taken[k + 1] {
            return false;
        }
    }
    true
}
fn hex(s: &str) -> String {
    if s.starts_with('#') {
        s.into()
    } else {
        format!("#{s}")
    }
}
fn class_names(s: &str) -> String {
    let mut out = String::new();
    let mut gap = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '_' | '-') {
            if gap && !out.is_empty() {
                out.push(' ')
            }
            out.push(c);
            gap = false
        } else {
            gap = true
        }
    }
    if out.is_empty() {
        out
    } else {
        format!(" {out}")
    }
}
struct Dot {
    cx: f64,
    cy: f64,
    reach: f64,
    text: Label,
}
#[derive(Clone, Copy)]
pub(super) enum Axis {
    XLeft,
    XRight,
    YBottom,
    YTop,
}
pub(super) struct PointGeometry {
    pub x: f64,
    pub y: f64,
    pub style: PointStyle,
}
/// Captures only values already read by the pinned renderer. Model handles stay
/// live across resource calls; the geometry and SVG algorithm below is shared.
pub(super) trait PhaseInput {
    type Error: From<PeleError>;
    type Text;
    type Point;
    type Points;
    fn has_points(&mut self) -> Result<bool, Self::Error>;
    fn title(&mut self) -> Result<Option<String>, Self::Error>;
    fn axis_has_text(&mut self, axis: Axis) -> Result<bool, Self::Error>;
    fn axis(&mut self, axis: Axis) -> Result<Self::Text, Self::Error>;
    fn present(&self, text: &Self::Text) -> bool;
    fn text(&mut self, text: &Self::Text) -> Result<Option<QuadrantText>, Self::Error>;
    fn quadrant(&mut self, index: usize) -> Result<Self::Text, Self::Error>;
    fn points(&mut self) -> Result<Self::Points, Self::Error>;
    fn next_point(
        &mut self,
        points: &Self::Points,
        index: usize,
    ) -> Result<Option<Self::Point>, Self::Error>;
    fn geometry(&mut self, point: &Self::Point) -> Result<Option<PointGeometry>, Self::Error>;
    fn point_text(&mut self, point: &Self::Point) -> Result<Self::Text, Self::Error>;
    fn point_identity(&mut self, point: &Self::Point) -> Result<(String, String), Self::Error>;
    fn accessibility(&mut self, description: bool) -> Result<Option<String>, Self::Error>;
}
struct Typed<'a>(&'a QuadrantModel);
impl Typed<'_> {
    fn axis_value(&self, axis: Axis) -> &Option<QuadrantText> {
        match axis {
            Axis::XLeft => &self.0.x_axis_left,
            Axis::XRight => &self.0.x_axis_right,
            Axis::YBottom => &self.0.y_axis_bottom,
            Axis::YTop => &self.0.y_axis_top,
        }
    }
}
impl PhaseInput for Typed<'_> {
    type Error = PeleError;
    type Text = Option<QuadrantText>;
    type Point = usize;
    type Points = ();
    fn has_points(&mut self) -> Result<bool, PeleError> {
        Ok(!self.0.points.is_empty())
    }
    fn title(&mut self) -> Result<Option<String>, PeleError> {
        Ok(self.0.title.clone())
    }
    fn axis_has_text(&mut self, a: Axis) -> Result<bool, PeleError> {
        Ok(self
            .axis_value(a)
            .as_ref()
            .is_some_and(|t| !t.text.is_empty()))
    }
    fn axis(&mut self, a: Axis) -> Result<Self::Text, PeleError> {
        Ok(self.axis_value(a).clone())
    }
    fn present(&self, t: &Self::Text) -> bool {
        t.is_some()
    }
    fn text(&mut self, t: &Self::Text) -> Result<Self::Text, PeleError> {
        Ok(t.clone())
    }
    fn quadrant(&mut self, k: usize) -> Result<Self::Text, PeleError> {
        Ok(self.0.quadrants[k].clone())
    }
    fn points(&mut self) -> Result<(), PeleError> {
        Ok(())
    }
    fn next_point(&mut self, _: &(), i: usize) -> Result<Option<usize>, PeleError> {
        Ok((i < self.0.points.len()).then_some(i))
    }
    fn geometry(&mut self, i: &usize) -> Result<Option<PointGeometry>, PeleError> {
        let p = &self.0.points[*i];
        if !(p.x >= 0. && p.x <= 1. && p.y >= 0. && p.y <= 1.) {
            return Ok(None);
        }
        let shared = self
            .0
            .classes
            .iter()
            .find(|e| e.0 == p.class_name)
            .map(|e| &e.1);
        Ok(Some(PointGeometry {
            x: p.x,
            y: p.y,
            style: PointStyle {
                radius: p.style.radius.or(shared.and_then(|s| s.radius)),
                color: p
                    .style
                    .color
                    .clone()
                    .or_else(|| shared.and_then(|s| s.color.clone())),
                stroke_color: p
                    .style
                    .stroke_color
                    .clone()
                    .or_else(|| shared.and_then(|s| s.stroke_color.clone())),
                stroke_width: p
                    .style
                    .stroke_width
                    .clone()
                    .or_else(|| shared.and_then(|s| s.stroke_width.clone())),
                order: Vec::new(),
            },
        }))
    }
    fn point_text(&mut self, i: &usize) -> Result<Self::Text, PeleError> {
        Ok(Some(self.0.points[*i].text.clone()))
    }
    fn point_identity(&mut self, i: &usize) -> Result<(String, String), PeleError> {
        let p = &self.0.points[*i];
        Ok((p.class_name.clone(), p.text.text.clone()))
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
    m: &QuadrantModel,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_phases(&mut Typed(m), c, o, r)
}
pub(super) fn render_phases<P: PhaseInput>(
    m: &mut P,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    let size = o.font_size.unwrap_or(16.);
    let axis_size = round(size * 0.875);
    let point_size = round(size * 0.75);
    let natural = round(length(c, "chartWidth", 500., 4000.).js_max(100.));
    let width = if let Some(available) = o.max_width.filter(|n| *n > 0.) {
        280f64.js_min(natural).js_max(natural.js_min(available))
    } else {
        natural
    };
    let width = round(width);
    let height = round(length(c, "chartHeight", 500., 4000.).js_max(100.) * width / natural);
    let outer = length(c, "quadrantPadding", o.padding.unwrap_or(8.), 200.);
    let x_pad = length(c, "xAxisLabelPadding", 5., 200.);
    let y_pad = length(c, "yAxisLabelPadding", 5., 200.);
    let has_points = m.has_points()?;
    let title_text = m.title()?;
    let title = layout_label(
        title_text.as_deref(),
        false,
        r,
        o,
        size,
        (width - 2. * outer).js_max(40.),
        1,
    )?;
    let title_pad = length(c, "titlePadding", 10., 200.);
    let title_space = if title.height > 0. {
        title.height + title_pad
    } else {
        0.
    };
    let has_x = m.axis_has_text(Axis::XLeft)? || m.axis_has_text(Axis::XRight)?;
    let has_y = m.axis_has_text(Axis::YBottom)? || m.axis_has_text(Axis::YTop)?;
    let x_below =
        has_points || matches!(setting(c,"xAxisPosition"),Some(Value::String(s))if s=="bottom");
    let y_right = matches!(setting(c,"yAxisPosition"),Some(Value::String(s))if s=="right");
    let side = width - 2. * outer;
    let x_left_handle = m.axis(Axis::XLeft)?;
    let x_right_present = {
        let value = m.axis(Axis::XRight)?;
        m.present(&value)
    };
    let x_left_text = m.text(&x_left_handle)?;
    let x_left = label(
        x_left_text.as_ref(),
        axis_size,
        if x_right_present {
            side / 2. - 16.
        } else {
            side
        },
        r,
        o,
    )?;
    let x_right_handle = m.axis(Axis::XRight)?;
    let x_right_text = m.text(&x_right_handle)?;
    let x_right = label(x_right_text.as_ref(), axis_size, side / 2. - 16., r, o)?;
    let x_height = x_left.height.js_max(x_right.height);
    let x_space = if has_x { x_height + x_pad } else { 0. };
    let tall = height - 2. * outer - title_space - x_space;
    let y_bottom_handle = m.axis(Axis::YBottom)?;
    let y_top_present = {
        let value = m.axis(Axis::YTop)?;
        m.present(&value)
    };
    let y_bottom_text = m.text(&y_bottom_handle)?;
    let y_bottom = label(
        y_bottom_text.as_ref(),
        axis_size,
        if y_top_present { tall / 2. - 16. } else { tall },
        r,
        o,
    )?;
    let y_top_handle = m.axis(Axis::YTop)?;
    let y_top_text = m.text(&y_top_handle)?;
    let y_top = label(y_top_text.as_ref(), axis_size, tall / 2. - 16., r, o)?;
    let y_height = y_bottom.height.js_max(y_top.height);
    let y_space = if has_y { y_height + y_pad } else { 0. };
    let left = outer + if y_right { 0. } else { y_space };
    let top = outer + title_space + if x_below { 0. } else { x_space };
    let w = (side - y_space).js_max(0.);
    let h = tall.js_max(0.);
    let tile_w = ((w - 4.) / 2.).js_max(0.);
    let tile_h = ((h - 4.) / 2.).js_max(0.);
    let mut taken = vec![
        left + tile_w,
        top,
        left + w - tile_w,
        top + h,
        left,
        top + tile_h,
        left + w,
        top + h - tile_h,
    ];
    let text_top = length(c, "quadrantTextTopPadding", 5., 200.);
    let mut quadrants = String::new();
    for k in 0..4 {
        let x = if k == 0 || k == 3 {
            left + w - tile_w
        } else {
            left
        };
        let y = if k < 2 { top } else { top + h - tile_h };
        let quadrant = m.quadrant(k)?;
        let text_value = m.text(&quadrant)?;
        let text = label(text_value.as_ref(), size, tile_w - 16., r, o)?;
        let cx = x + tile_w / 2.;
        let cy = if has_points {
            y + text_top + text.height / 2.
        } else {
            y + tile_h / 2.
        };
        taken.extend([
            cx - text.width / 2.,
            cy - text.height / 2.,
            cx + text.width / 2.,
            cy + text.height / 2.,
        ]);
        quadrants.push_str(&format!("<g class=\"pele-quadrant\" data-id=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"var(--pele-radius,4px)\"/>{}</g>",k+1,num(x),num(y),num(tile_w),num(tile_h),label_svg(&text,cx,cy," class=\"pele-label\" fill=\"var(--_fg)\" stroke=\"none\"")));
    }
    let mut axes = String::new();
    if has_x {
        let y = if x_below {
            top + h + x_pad + x_height / 2.
        } else {
            top - x_pad - x_height / 2.
        };
        axes.push_str(&label_svg(
            &x_left,
            if x_right.height > 0. {
                left + w / 4.
            } else {
                left + x_left.width / 2.
            },
            y,
            "",
        ));
        axes.push_str(&label_svg(&x_right, left + w * 3. / 4., y, ""));
    }
    if has_y {
        let x = if y_right {
            left + w + y_pad + y_height / 2.
        } else {
            left - y_pad - y_height / 2.
        };
        let turned = |text: &Label, y: f64| {
            if text.height > 0. {
                format!(
                    "<g transform=\"translate({},{}) rotate(-90)\">{}</g>",
                    num(x),
                    num(y),
                    label_svg(text, 0., 0., "")
                )
            } else {
                String::new()
            }
        };
        axes.push_str(&turned(
            &y_bottom,
            if y_top.height > 0. {
                top + h * 3. / 4.
            } else {
                top + h - y_bottom.width / 2.
            },
        ));
        axes.push_str(&turned(&y_top, top + h / 4.));
    }
    let radius = length(c, "pointRadius", 5., 200.);
    let text_pad = length(c, "pointTextPadding", 5., 200.);
    let fill = series_color(0);
    let mut dots = Vec::new();
    let mut circles = Vec::new();
    let points = m.points()?;
    let mut point_index = 0;
    while let Some(point) = m.next_point(&points, point_index)? {
        point_index += 1;
        let Some(geometry) = m.geometry(&point)? else {
            continue;
        };
        let (x, y) = (geometry.x, geometry.y);
        let p = &geometry.style;
        let radius = p.radius.unwrap_or(radius);
        let radius = if radius.is_nan() {
            radius
        } else {
            radius.js_min(2000.)
        };
        let color = p.color.as_ref();
        let stroke_color = p.stroke_color.as_ref();
        let stroke_width = p.stroke_width.as_ref();
        let mut styles = Vec::new();
        if let Some(color) = color.filter(|s| !s.is_empty()) {
            styles.push(format!("fill:{}", hex(color)))
        }
        if let Some(sw) = stroke_width.filter(|s| !s.is_empty()) {
            if let Some(sc) = stroke_color.filter(|s| !s.is_empty()) {
                styles.push(format!("stroke:{}", hex(sc)))
            }
            styles.push(format!("stroke-width:{sw}"))
        }
        let shape = resolve_style(&styles).shape;
        let cx = left + x * w;
        let cy = top + (1. - y) * h;
        let reach = radius
            + stroke_width.filter(|s| !s.is_empty()).map_or(0., |s| {
                let n = parse_int(s);
                if n.is_nan() {
                    n
                } else {
                    n.js_min(400.) / 2.
                }
            });
        let point_text_handle = m.point_text(&point)?;
        let point_text = m.text(&point_text_handle)?;
        dots.push(Dot {
            cx,
            cy,
            reach,
            text: label(point_text.as_ref(), point_size, 200., r, o)?,
        });
        let (class_name, data_id) = m.point_identity(&point)?;
        circles.push(format!(
            "<circle class=\"pele-point{}\" data-id=\"{}\" cx=\"{}\" cy=\"{}\" r=\"{}\"{}{shape}/>",
            class_names(&class_name),
            esc_text(&data_id)?,
            num(cx),
            num(cy),
            num(radius),
            if stroke_width.is_some_and(|s| !s.is_empty())
                && !stroke_color.is_some_and(|s| !s.is_empty())
            {
                format!(" stroke=\"{fill}\"")
            } else {
                String::new()
            }
        ));
    }
    let careful = dots.len() <= 300;
    if careful {
        for dot in &dots {
            taken.extend([
                dot.cx - dot.reach,
                dot.cy - dot.reach,
                dot.cx + dot.reach,
                dot.cy + dot.reach,
            ]);
        }
    }
    let mut labels = String::new();
    for Dot {
        cx,
        cy,
        reach,
        text,
    } in &dots
    {
        let half_w = text.width / 2.;
        let half_h = text.size * 0.6 + (text.height - text.line_height) / 2.;
        let away = reach + text_pad;
        let spots = [
            *cx,
            cy + away + half_h,
            *cx,
            cy - away - half_h,
            cx + away + half_w,
            *cy,
            cx - away - half_w,
            *cy,
        ];
        let mut tx = *cx;
        let mut ty = spots[1];
        let mut found = false;
        for k in (0..16).step_by(2) {
            let spot = k % 8;
            let x = if spot < 4 && text.width < w {
                spots[spot].js_max(left + half_w).js_min(left + w - half_w)
            } else {
                spots[spot]
            };
            let y = spots[spot + 1];
            if x - half_w < left
                || x + half_w > left + w
                || y - half_h < top
                || y + half_h > top + h
            {
                continue;
            }
            let free = !careful
                || clear(
                    &taken,
                    if k < 8 { 0 } else { 8 },
                    x - half_w,
                    y - half_h,
                    x + half_w,
                    y + half_h,
                );
            if !found || free {
                tx = x;
                ty = y;
                found = true
            }
            if free {
                break;
            }
        }
        if careful {
            taken.extend([tx - half_w, ty - half_h, tx + half_w, ty + half_h]);
        }
        labels.push_str(&label_svg(text, tx, ty, " class=\"pele-label\""));
    }
    let content = format!(
        "{}<g class=\"pele-quadrants\" fill=\"var(--_s)\" stroke=\"var(--_b)\">{quadrants}</g>{}{}",
        label_svg(
            &title,
            width / 2.,
            title_pad / 2. + title.height / 2.,
            " class=\"pele-title\" font-weight=\"var(--_tw)\""
        ),
        if axes.is_empty() {
            String::new()
        } else {
            format!(
                "<g class=\"pele-axis-labels\" font-size=\"{}\" fill=\"var(--_m)\">{axes}</g>",
                svg::js_num(axis_size)
            )
        },
        if dots.is_empty() {
            String::new()
        } else {
            circles.reverse();
            format!("<g class=\"pele-points\" fill=\"{fill}\">{}</g><g class=\"pele-point-labels\" font-size=\"{}\">{labels}</g>",circles.join(""),svg::js_num(point_size))
        }
    );
    let acc_title = m.accessibility(false)?;
    let acc_descr = m.accessibility(true)?;
    Ok(Rendered {
        svg: svg::document(
            "quadrantChart",
            width,
            height,
            size,
            o,
            acc_title.as_deref(),
            acc_descr.as_deref(),
            &content,
        )?,
        width,
        height,
        links: Vec::new(),
    })
}
