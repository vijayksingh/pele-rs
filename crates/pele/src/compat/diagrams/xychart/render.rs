//! Direct Pele xychart/render.ts axis layout and clipped series rendering, 458322a5 (MIT, kepano).
use super::{
    model::XyChartModel,
    phase::{PhaseInput, Typed},
    ticks::*,
};
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    svg::{self, esc, esc_text, js_num, label_svg, num, series_color},
    text::{layout_label, Label},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered, TextRequest, Value,
};
const TICK: f64 = 4.;
const SWATCH: f64 = 12.;
const ROW: f64 = 26.;
const LEGEND_GAP: f64 = 24.;
const LIMIT: f64 = 1e300;
#[derive(Clone)]
struct Mark {
    at: f64,
    label: Label,
}
fn on(config: Option<&Object>, key: &str, fallback: bool) -> bool {
    match config.and_then(|o| o.get(key)) {
        Some(Value::Bool(b)) => *b,
        _ => fallback,
    }
}
fn dimension(value: Option<&Value>, fallback: f64) -> f64 {
    match value {
        Some(Value::Number(n)) if *n > 0. => round(n.js_max(100.).js_min(4000.)),
        _ => fallback,
    }
}
fn finite(value: Option<f64>) -> Option<f64> {
    value
        .filter(|v| v.is_finite())
        .map(|v| v.js_max(-LIMIT).js_min(LIMIT))
}
fn text(
    raw: Option<&str>,
    size: f64,
    max: f64,
    resources: &mut dyn RenderResources,
    options: &RenderSettings,
) -> Result<Label, PeleError> {
    layout_label(raw, false, resources, options, size, max, 0)
}
fn measure(
    raw: &str,
    size: f64,
    r: &mut dyn RenderResources,
    o: &RenderSettings,
) -> Result<f64, PeleError> {
    r.text_width(TextRequest {
        text: raw,
        size,
        style: 0,
        font_family: o.font_family.as_deref(),
        font_family_mono: o.font_family_mono.as_deref(),
    })
}
fn widest(marks: &[Mark]) -> f64 {
    marks.iter().fold(0_f64, |w, m| w.js_max(m.label.width))
}
fn aligned(label: &Label, x: f64, y: f64, anchor: &str) -> String {
    if label.lines.len() == 1
        && label.lines[0].len() == 1
        && label.lines[0][0].icon.is_none()
        && label.lines[0][0].style == 0
    {
        format!(
            "<text x=\"{}\" y=\"{}\" text-anchor=\"{anchor}\">{}</text>",
            num(x),
            num(y + label.size * 0.35),
            esc(&label.lines[0][0].text)
        )
    } else {
        label_svg(
            label,
            x + if anchor == "start" {
                label.width / 2.
            } else if anchor == "end" {
                -label.width / 2.
            } else {
                0.
            },
            y,
            "",
        )
    }
}
fn value_marks(
    length: f64,
    spacing: f64,
    lo: &mut f64,
    hi: &mut f64,
    exact: bool,
    small: f64,
    r: &mut dyn RenderResources,
    o: &RenderSettings,
) -> Result<Vec<Mark>, PeleError> {
    let step = tick_step(
        (*hi - *lo).abs(),
        (length / spacing).floor().js_min(10.).js_max(2.),
    );
    if !exact {
        *lo = tick_value(step, (tick_index(step, *lo) + 1e-9).floor());
        *hi = tick_value(step, (tick_index(step, *hi) - 1e-9).ceil());
    }
    ticks_between(lo.js_min(*hi), lo.js_max(*hi), step)
        .into_iter()
        .map(|v| {
            Ok(Mark {
                at: (v - *lo) / (*hi - *lo),
                label: text(Some(&js_num(v)), small, 4000., r, o)?,
            })
        })
        .collect()
}
#[allow(clippy::too_many_arguments)]
fn category_marks<P: PhaseInput>(
    input: &mut P,
    x: &P::Value,
    categories: &[Label],
    length: f64,
    slot: f64,
    inset: f64,
    x0: Option<f64>,
    x1: Option<f64>,
    ranged: bool,
    horizontal: bool,
    category_width: f64,
    category_height: f64,
    small: f64,
    r: &mut dyn RenderResources,
    o: &RenderSettings,
) -> Result<Vec<Mark>, P::Error> {
    let mut marks = vec![];
    if input.field_is(x, "type", "band")? {
        let need = if categories.is_empty() {
            8.
        } else if horizontal {
            category_height
        } else {
            category_width + 12.
        };
        let every = (need / (length * slot)).ceil().js_max(1.);
        let mut i = 0.;
        loop {
            let names = input.get(x, "categories")?;
            // Preserve the upstream for-loop condition: NaN is unordered.
            if !(i < input.count(&names)? as f64) {
                break;
            }
            input.work(1)?;
            let n = i as usize;
            marks.push(Mark {
                at: (i + 0.5) * slot,
                label: categories.get(n).cloned().unwrap_or_default(),
            });
            i += every;
        }
    } else if ranged {
        let x0 = x0.unwrap();
        let x1 = x1.unwrap();
        let step = tick_step(
            (x1 - x0).abs(),
            (length / if horizontal { 32. } else { 80. })
                .floor()
                .js_min(10.)
                .js_max(2.),
        );
        for v in ticks_between(x0.js_min(x1), x0.js_max(x1), step) {
            marks.push(Mark {
                at: inset + ((v - x0) / (x1 - x0)) * (1. - 2. * inset),
                label: text(Some(&js_num(v)), small, 4000., r, o)?,
            })
        }
    } else if let Some(x0) = x0 {
        marks.push(Mark {
            at: 0.5,
            label: text(Some(&js_num(x0)), small, 4000., r, o)?,
        })
    }
    Ok(marks)
}
pub(super) fn render(
    model: &XyChartModel,
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
    input.begin_draw();
    let root = input.root();
    let initial_plots = input.get(&root, "plots")?;
    if input.count(&initial_plots)? == 0 {
        let mut e = PeleError::new(
            "No Plot to render, please provide a plot with some data",
            ErrorCode::Semantic,
        );
        e.diagram_type = Some("xychart".into());
        return Err(e.into());
    }
    let chart = config.get("xyChart").and_then(Value::as_object);
    let size = options.font_size.unwrap_or(16.);
    let small = round(size * 0.875);
    let tiny = round(size * 0.75);
    let pad = options.padding.unwrap_or(8.);
    let natural = dimension(chart.and_then(|c| c.get("width")), 700.);
    let width = if let Some(available) = options.max_width.filter(|v| *v > 0.) {
        natural.js_min(240.).js_max(natural.js_min(available))
    } else {
        natural
    };
    let height = round(
        dimension(chart.and_then(|c| c.get("height")), 500.) * 0.6_f64.js_max(width / natural),
    );
    let orientation = input.get(&root, "orientation")?;
    let horizontal = if input.nullish(&orientation) {
        matches!(chart.and_then(|c|c.get("chartOrientation")),Some(Value::String(s))if s=="horizontal")
    } else {
        input.is_text(&orientation, "horizontal")?
    };
    let show_values = on(chart, "showDataLabel", false);
    let x = input.get(&root, "xAxis")?;
    let plots = input.get(&root, "plots")?;
    let line_height = text(Some("0"), small, 4000., resources, options)?.height;
    let tiny_height = text(Some("0"), tiny, 4000., resources, options)?.height;
    let title_raw = if on(chart, "showTitle", true) {
        input.field_text(&root, "title")?
    } else {
        None
    };
    if let Some(t) = &title_raw {
        input.keep_text(t.encode_utf16().count())?;
    }
    let title = layout_label(
        title_raw.as_deref(),
        false,
        resources,
        options,
        size,
        width - 2. * pad,
        1,
    )?;
    let title_space = if title.height > 0. {
        title.height + 12.
    } else {
        0.
    };
    let mut legend = vec![];
    let mut legend_width = 0_f64;
    if on(chart, "showLegend", true) {
        let length = input.count(&plots)?;
        for index in 0..length {
            input.work(1)?;
            if !input.has(&plots, index)? {
                continue;
            }
            let plot = input.index(&plots, index)?;
            let check = input.get(&plot, "title")?;
            if input.is_text(&check, "")? {
                continue;
            }
            let raw = input.field_text(&plot, "title")?;
            if let Some(t) = &raw {
                input.keep_text(t.encode_utf16().count())?;
            }
            let label = text(raw.as_deref(), small, width * 0.3, resources, options)?;
            legend_width = legend_width.js_max(label.width + SWATCH + 8.);
            legend.push((index, label));
        }
    }
    let legend_space = if !legend.is_empty() {
        legend_width + LEGEND_GAP
    } else {
        0.
    };
    let mut bars = 0usize;
    let length = input.count(&plots)?;
    for i in 0..length {
        input.work(1)?;
        if input.has(&plots, i)? {
            let p = input.index(&plots, i)?;
            if input.field_is(&p, "type", "bar")? {
                bars += 1;
            }
        }
    }
    let y = input.get(&root, "yAxis")?;
    let min = input.get(&y, "min")?;
    let mut lo = input.number(&min).unwrap_or(f64::NAN);
    let y = input.get(&root, "yAxis")?;
    let max = input.get(&y, "max")?;
    let mut hi = input.number(&max).unwrap_or(f64::NAN);
    let set = input.get(&root, "hasSetYAxis")?;
    let exact = input.truthy(&set) && lo.is_finite() && hi.is_finite() && lo != hi;
    if exact {
        lo = finite(Some(lo)).unwrap();
        hi = finite(Some(hi)).unwrap();
    } else {
        lo = f64::INFINITY;
        hi = f64::NEG_INFINITY;
        let mut pi = 0;
        while pi < input.count(&plots)? {
            input.work(1)?;
            let p = input.index(&plots, pi)?;
            pi += 1;
            let data = input.get(&p, "data")?;
            let mut i = 0;
            while i < input.count(&data)? {
                input.work(1)?;
                let point = input.index(&data, i)?;
                i += 1;
                let raw = input.index(&point, 1)?;
                let Some(v) = finite(input.number(&raw)) else {
                    continue;
                };
                if v < lo {
                    lo = v
                }
                if v > hi {
                    hi = v
                }
            }
        }
        if lo > hi {
            lo = 0.;
            hi = 0.
        }
        if bars > 0 {
            lo = lo.js_min(0.);
            hi = hi.js_max(0.)
        }
        if lo == hi {
            if lo == 0. {
                hi = 1.
            } else {
                lo -= lo.abs() / 2.;
                hi += hi.abs() / 2.;
            }
        }
    }
    let mut count = 1;
    if input.field_is(&x, "type", "band")? {
        let cats = input.get(&x, "categories")?;
        count = count.max(input.count(&cats)?);
    } else {
        let mut i = 0;
        while i < input.count(&plots)? {
            input.work(1)?;
            let p = input.index(&plots, i)?;
            i += 1;
            let data = input.get(&p, "data")?;
            count = count.max(input.count(&data)?);
        }
    }
    let slot = 1. / count as f64;
    let inset = if input.field_is(&x, "type", "band")? || bars > 0 {
        slot / 2.
    } else {
        0.
    };
    let x0 = if input.field_is(&x, "type", "linear")? {
        let v = input.get(&x, "min")?;
        finite(input.number(&v))
    } else {
        None
    };
    let x1 = if input.field_is(&x, "type", "linear")? {
        let v = input.get(&x, "max")?;
        finite(input.number(&v))
    } else {
        None
    };
    let ranged = x0.is_some() && x1.is_some() && x0 != x1;
    let place = |input: &mut P, i: usize, n: usize| -> Result<f64, P::Error> {
        Ok(if input.field_is(&x, "type", "band")? {
            (i as f64 + 0.5) * slot
        } else {
            inset
                + if n > 1 {
                    i as f64 / (n - 1) as f64
                } else if ranged {
                    0.
                } else {
                    0.5
                } * (1. - 2. * inset)
        })
    };
    let category_conf = chart
        .and_then(|c| c.get("xAxis"))
        .and_then(Value::as_object);
    let value_conf = chart
        .and_then(|c| c.get("yAxis"))
        .and_then(Value::as_object);
    let left_conf = if horizontal {
        category_conf
    } else {
        value_conf
    };
    let bottom_conf = if horizontal {
        value_conf
    } else {
        category_conf
    };
    let left_raw = if on(left_conf, "showTitle", true) {
        let axis = if horizontal {
            x.clone()
        } else {
            input.get(&root, "yAxis")?
        };
        input.field_text(&axis, "title")?
    } else {
        None
    };
    if let Some(t) = &left_raw {
        input.keep_text(t.encode_utf16().count())?;
    }
    let left_title = text(left_raw.as_deref(), small, height * 0.8, resources, options)?;
    let bottom_raw = if on(bottom_conf, "showTitle", true) {
        let axis = if horizontal {
            input.get(&root, "yAxis")?
        } else {
            x.clone()
        };
        input.field_text(&axis, "title")?
    } else {
        None
    };
    if let Some(t) = &bottom_raw {
        input.keep_text(t.encode_utf16().count())?;
    }
    let bottom_title = text(
        bottom_raw.as_deref(),
        small,
        width * 0.8,
        resources,
        options,
    )?;
    let left_tick = if on(left_conf, "showTick", true) {
        TICK
    } else {
        0.
    };
    let bottom_tick = if on(bottom_conf, "showTick", true) {
        TICK
    } else {
        0.
    };
    let left_labels = on(left_conf, "showLabel", true);
    let bottom_labels = on(bottom_conf, "showLabel", true);
    let wrap_at = if horizontal {
        width * 0.3
    } else {
        48_f64.js_max((width - 2. * pad - 64. - legend_space) * slot - 8.)
    };
    let mut categories = vec![];
    if input.field_is(&x, "type", "band")? && on(category_conf, "showLabel", true) {
        let cats = input.get(&x, "categories")?;
        let length = input.count(&cats)?;
        for i in 0..length {
            input.work(1)?;
            if !input.has(&cats, i)? {
                categories.push(Label::default());
                continue;
            }
            let v = input.index(&cats, i)?;
            let t = input.optional_text(&v)?;
            if let Some(t) = &t {
                input.keep_text(t.encode_utf16().count())?;
            }
            categories.push(text(t.as_deref(), small, wrap_at, resources, options)?);
        }
    }
    let category_width = categories.iter().fold(0_f64, |w, l| w.js_max(l.width));
    let category_height = categories.iter().fold(0_f64, |h, l| h.js_max(l.height));
    let mut point_label_width = 0_f64;
    let mut value_width = 0_f64;
    let mut pi = 0;
    while pi < input.count(&plots)? {
        input.work(1)?;
        let plot = input.index(&plots, pi)?;
        pi += 1;
        let point_labels = input.get(&plot, "pointLabels")?;
        if !input.nullish(&point_labels) {
            let mut i = 0;
            while i < input.count(&point_labels)? {
                input.work(1)?;
                let v = input.index(&point_labels, i)?;
                i += 1;
                let label = input.text(&v)?;
                point_label_width =
                    point_label_width.js_max(measure(&label, tiny, resources, options)?);
            }
        }
        if show_values && horizontal && input.field_is(&plot, "type", "bar")? {
            let data = input.get(&plot, "data")?;
            let mut i = 0;
            while i < input.count(&data)? {
                input.work(1)?;
                let point = input.index(&data, i)?;
                i += 1;
                let v = input.index(&point, 1)?;
                let s = finite(input.number(&v)).map(js_num).unwrap_or_default();
                value_width = value_width.js_max(measure(&s, tiny, resources, options)?);
            }
        }
    }
    let bottom_label_height = if !bottom_labels {
        0.
    } else if horizontal || !input.field_is(&x, "type", "band")? {
        line_height
    } else {
        category_height
    };
    let bottom_space = bottom_tick
        + if bottom_label_height > 0. {
            bottom_label_height + 4.
        } else {
            0.
        }
        + if bottom_title.height > 0. {
            bottom_title.height + 4.
        } else {
            0.
        };
    let headroom = if horizontal {
        0.
    } else if point_label_width > 0. || show_values {
        tiny_height + 4.
    } else if left_labels {
        line_height / 2.
    } else {
        0.
    };
    let top = pad + title_space + headroom;
    let plot_height = (height - pad - bottom_space - top).js_max(1.);
    let bottom = top + plot_height;
    let left_marks = if horizontal {
        category_marks(
            input,
            &x,
            &categories,
            plot_height,
            slot,
            inset,
            x0,
            x1,
            ranged,
            horizontal,
            category_width,
            category_height,
            small,
            resources,
            options,
        )?
    } else {
        value_marks(
            plot_height,
            56.,
            &mut lo,
            &mut hi,
            exact,
            small,
            resources,
            options,
        )?
    };
    let left_label_width = if left_labels { widest(&left_marks) } else { 0. };
    let left =
        pad + if left_title.height > 0. {
            left_title.height + 6.
        } else {
            0.
        } + if left_label_width > 0. {
            left_label_width + 6.
        } else {
            0.
        } + left_tick;
    let beside = (if point_label_width > 0. {
        point_label_width + 12.
    } else {
        0.
    })
    .js_max(if value_width > 0. {
        value_width + 8.
    } else {
        0.
    });
    let reach = if horizontal {
        (width * 0.25).js_min(beside)
    } else {
        0.
    };
    let mut plot_width = (width - pad - left - legend_space - reach).js_max(1.);
    let bottom_marks = if horizontal {
        value_marks(
            plot_width, 80., &mut lo, &mut hi, exact, small, resources, options,
        )?
    } else {
        category_marks(
            input,
            &x,
            &categories,
            plot_width,
            slot,
            inset,
            x0,
            x1,
            ranged,
            horizontal,
            category_width,
            category_height,
            small,
            resources,
            options,
        )?
    };
    if bottom_labels && legend_space + reach == 0. {
        let over = bottom_marks.iter().fold(0_f64, |over, m| {
            over.js_max(m.at * plot_width + m.label.width / 2. - plot_width)
        });
        plot_width = (plot_width - over).js_max(1.);
    }
    let right = left + plot_width;
    let px = |c: f64, v: f64| {
        if horizontal {
            left + v * plot_width
        } else {
            left + c * plot_width
        }
    };
    let py = |c: f64, v: f64| {
        if horizontal {
            top + c * plot_height
        } else {
            bottom - v * plot_height
        }
    };
    let scale = |value: f64| (value - lo) / (hi - lo);
    let clamp = |v: f64| v.js_max(0.).js_min(1.);
    let rule = |v: f64| {
        if horizontal {
            format!("M{},{}V{}", num(px(0., v)), num(top), num(bottom))
        } else {
            format!("M{},{}H{}", num(left), num(py(0., v)), num(right))
        }
    };
    let mut grid = String::new();
    for mark in if horizontal {
        &bottom_marks
    } else {
        &left_marks
    } {
        grid.push_str(&rule(mark.at))
    }
    let mut axis = String::new();
    let mut labels = String::new();
    if on(left_conf, "showAxisLine", true) {
        axis.push_str(&format!("M{},{}V{}", num(left), num(top), num(bottom)))
    }
    if on(bottom_conf, "showAxisLine", true) {
        axis.push_str(&format!("M{},{}H{}", num(left), num(bottom), num(right)))
    }
    for m in &left_marks {
        let y = if horizontal {
            top + m.at * plot_height
        } else {
            bottom - m.at * plot_height
        };
        if left_tick != 0. {
            axis.push_str(&format!("M{},{}h{}", num(left), num(y), js_num(-left_tick)))
        }
        if left_labels {
            labels.push_str(&aligned(&m.label, left - left_tick - 6., y, "end"))
        }
    }
    for m in &bottom_marks {
        let at = left + m.at * plot_width;
        if bottom_tick != 0. {
            axis.push_str(&format!(
                "M{},{}v{}",
                num(at),
                num(bottom),
                js_num(bottom_tick)
            ))
        }
        if bottom_labels {
            labels.push_str(&aligned(
                &m.label,
                at,
                bottom + bottom_tick + 4. + m.label.height / 2.,
                "middle",
            ))
        }
    }
    let span = if horizontal { plot_height } else { plot_width } * slot;
    let group = span * 0.7;
    let thickness = group / bars.max(1) as f64;
    let gap = if bars > 1 {
        2_f64.js_min(thickness * 0.2)
    } else {
        0.
    };
    let base = clamp(scale(0.));
    if bars > 0 && base > 0. && base < 1. {
        axis.push_str(&rule(base))
    }
    let mut series = String::new();
    let mut lines = String::new();
    let mut outside = String::new();
    let mut point_labels = String::new();
    let mut bar = 0;
    let length = input.count(&plots)?;
    for index in 0..length {
        input.work(1)?;
        if !input.has(&plots, index)? {
            continue;
        }
        let plot = input.index(&plots, index)?;
        let color = series_color(index);
        let t = input.get(&plot, "title")?;
        let id = if input.truthy(&t) {
            let t = input.field_text(&plot, "title")?.unwrap_or_default();
            format!(" data-id=\"{}\"", esc_text(&t)?)
        } else {
            String::new()
        };
        let band = input.field_is(&x, "type", "band")?;
        let data = input.get(&plot, "data")?;
        let data_len = input.count(&data)?;
        let n = if band { data_len.min(count) } else { data_len };
        if input.field_is(&plot, "type", "bar")? {
            let offset =
                (bar as f64 - (bars as f64 - 1.) / 2.) * thickness - (thickness - gap) / 2.;
            bar += 1;
            let mut rects = String::new();
            for i in 0..n {
                input.work(1)?;
                let data = input.get(&plot, "data")?;
                let point = input.index(&data, i)?;
                let raw = input.index(&point, 1)?;
                let Some(value) = finite(input.number(&raw)) else {
                    continue;
                };
                let c = place(input, i, n)?;
                let v = clamp(scale(value));
                let from = base.js_min(v);
                let to = base.js_max(v);
                let bx = if horizontal {
                    px(c, from)
                } else {
                    px(c, 0.) + offset
                };
                let by = if horizontal {
                    py(c, 0.) + offset
                } else {
                    py(c, to)
                };
                let bw = if horizontal {
                    (to - from) * plot_width
                } else {
                    thickness - gap
                };
                let bh = if horizontal {
                    thickness - gap
                } else {
                    (to - from) * plot_height
                };
                rects.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/>",
                    num(bx),
                    num(by),
                    num(bw),
                    num(bh)
                ));
                if !show_values {
                    continue;
                }
                let label = js_num(value);
                let w = measure(&label, tiny, resources, options)?;
                let out = if v >= base { 1. } else { -1. };
                let mut tx = bx + bw / 2.;
                let mut ty = by + bh / 2.;
                let mut anchor = "middle";
                if horizontal {
                    if bh < tiny {
                        continue;
                    }
                    tx = if out > 0. { bx + bw } else { bx } + out * 6.;
                    anchor = if out > 0. { "start" } else { "end" };
                } else {
                    if w + 2. > span / bars as f64 {
                        continue;
                    }
                    ty = if out > 0. { by } else { by + bh } - out * (tiny_height / 2. + 2.);
                }
                outside.push_str(&format!(
                    "<text x=\"{}\" y=\"{}\" text-anchor=\"{anchor}\">{label}</text>",
                    num(tx),
                    num(ty + tiny * 0.35)
                ));
            }
            series.push_str(&format!(
                "<g class=\"pele-series pele-bars\"{id} fill=\"{color}\">{rects}</g>"
            ));
            continue;
        }
        let mut d = String::new();
        let mut dots = String::new();
        let mut pen = false;
        let mut pc = 0.;
        let mut pv = f64::NAN;
        for i in 0..n {
            input.work(1)?;
            let data = input.get(&plot, "data")?;
            let point = input.index(&data, i)?;
            let raw = input.index(&point, 1)?;
            let Some(value) = finite(input.number(&raw)) else {
                pv = f64::NAN;
                continue;
            };
            let c = place(input, i, n)?;
            let v = scale(value);
            let visible = (0. ..=1.).contains(&v);
            if pv.is_nan() {
                if visible {
                    d.push_str(&format!("M{},{}", num(px(c, v)), num(py(c, v))))
                }
                pen = visible;
            } else {
                let dv = v - pv;
                let mut t0 = 0_f64;
                let mut t1 = 1_f64;
                if dv != 0. {
                    let a = -pv / dv;
                    let b = (1. - pv) / dv;
                    t0 = t0.js_max(a.js_min(b));
                    t1 = t1.js_min(a.js_max(b));
                } else if !visible {
                    t0 = 2.
                }
                if t0 <= t1 {
                    let c0 = pc + (c - pc) * t0;
                    let v0 = pv + dv * t0;
                    let c1 = pc + (c - pc) * t1;
                    let v1 = pv + dv * t1;
                    if !pen || t0 > 0. {
                        d.push_str(&format!("M{},{}", num(px(c0, v0)), num(py(c0, v0))))
                    }
                    d.push_str(&format!("L{},{}", num(px(c1, v1)), num(py(c1, v1))))
                }
                pen = visible;
            }
            pc = c;
            pv = v;
            if !visible {
                continue;
            }
            let cx = px(c, v);
            let cy = py(c, v);
            if span >= 12. {
                dots.push_str(&format!(
                    "<circle cx=\"{}\" cy=\"{}\" r=\"3\"/>",
                    num(cx),
                    num(cy)
                ))
            }
            let labels = input.get(&plot, "pointLabels")?;
            let raw = if input.nullish(&labels) {
                None
            } else {
                let v = input.index(&labels, i)?;
                input.optional_text(&v)?
            };
            if let Some(t) = &raw {
                input.keep_text(t.encode_utf16().count())?;
            }
            let label = text(raw.as_deref(), tiny, 4000., resources, options)?;
            if label.height > 0. {
                let half = label.width / 2.;
                point_labels.push_str(&if horizontal {
                    aligned(&label, cx + 10., cy, "start")
                } else {
                    aligned(
                        &label,
                        cx.js_min(right - half).js_max(left + half),
                        cy - 6. - label.height / 2.,
                        "middle",
                    )
                });
            }
        }
        lines.push_str(&format!("<g class=\"pele-series pele-line\"{id} fill=\"{color}\">{}{dots}</g>",if d.is_empty(){String::new()}else{format!("<path d=\"{d}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>")}));
    }
    let mut titles = String::new();
    if left_title.height > 0. {
        titles.push_str(&format!(
            "<g transform=\"translate({},{}) rotate(-90)\">{}</g>",
            num(pad + left_title.height / 2.),
            num(top + plot_height / 2.),
            label_svg(&left_title, 0., 0., "")
        ))
    }
    titles.push_str(&label_svg(
        &bottom_title,
        left + plot_width / 2.,
        height - pad - bottom_title.height / 2.,
        "",
    ));
    let legend_height: f64 = legend
        .iter()
        .map(|(_, label)| ROW.js_max(label.height + 6.))
        .sum();
    let mut ly = top + ((plot_height - legend_height) / 2.).js_max(0.);
    let lx = right + reach + LEGEND_GAP;
    let mut legend_out = String::new();
    for (index, label) in &legend {
        let row = ROW.js_max(label.height + 6.);
        if ly + row > height {
            break;
        }
        let y = ly + row / 2.;
        let color = series_color(*index);
        let plot = input.index(&plots, *index)?;
        let legend_title = input.field_text(&plot, "title")?.unwrap_or_default();
        let plot = input.index(&plots, *index)?;
        let shape = if input.field_is(&plot, "type", "bar")? {
            format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"2\" fill=\"{color}\"/>",
                num(lx),
                num(y - SWATCH / 2.),
                js_num(SWATCH),
                js_num(SWATCH)
            )
        } else {
            format!("<path d=\"M{},{}h{}\" stroke=\"{color}\" stroke-width=\"2\" stroke-linecap=\"round\"/>",num(lx),num(y),js_num(SWATCH))
        };
        legend_out.push_str(&format!(
            "<g class=\"pele-legend-item\" data-id=\"{}\">{shape}{}</g>",
            esc_text(&legend_title)?,
            label_svg(label, lx + SWATCH + 8. + label.width / 2., y, "")
        ));
        ly += row;
    }
    let content = format!(
        "{}{}<g class=\"pele-plot\">{series}{lines}</g>{}{}{}{}{}{}",
        label_svg(
            &title,
            width / 2.,
            pad + title.height / 2.,
            " class=\"pele-title\" font-weight=\"var(--_tw)\""
        ),
        if grid.is_empty() {
            String::new()
        } else {
            format!("<path class=\"pele-grid\" d=\"{grid}\" stroke=\"var(--_a)\"/>")
        },
        if axis.is_empty() {
            String::new()
        } else {
            format!("<path class=\"pele-axis\" d=\"{axis}\" fill=\"none\" stroke=\"var(--_b)\"/>")
        },
        group_text("pele-axis-labels", small, Some("var(--_m)"), &labels),
        group_text("pele-axis-titles", small, None, &titles),
        group_text("pele-data-labels", tiny, None, &outside),
        group_text("pele-point-labels", tiny, None, &point_labels),
        group_text("pele-legend", small, None, &legend_out)
    );
    let acc_title = input.field_text(&root, "accTitle")?;
    let acc_descr = input.field_text(&root, "accDescr")?;
    let svg = svg::document(
        "xychart",
        width,
        height,
        size,
        options,
        acc_title.as_deref(),
        acc_descr.as_deref(),
        &content,
    )?;
    Ok(Rendered {
        svg,
        width,
        height,
        links: vec![],
    })
}
fn group_text(class: &str, size: f64, fill: Option<&str>, body: &str) -> String {
    if body.is_empty() {
        String::new()
    } else {
        format!(
            "<g class=\"{class}\" font-size=\"{}\"{}>{body}</g>",
            js_num(size),
            fill.map(|f| format!(" fill=\"{f}\"")).unwrap_or_default()
        )
    }
}
