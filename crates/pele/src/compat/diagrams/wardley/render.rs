//! Actual Wardley map projection, pipelines, links and responsive sizing, Pele 458322a5 (MIT).
use super::{
    model::*,
    phase::{PhaseInput, Typed},
};
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    svg::{
        self,
        edges::{marker, marker_trim},
        esc, esc_text, label_svg, label_svg_with_resources, num,
    },
    text::{layout_label, Label},
    Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
use std::collections::{HashMap, HashSet};
const LINE: &str = "var(--_l)";
const ACCENT: &str = "var(--_c)";
const RADIUS: &str = "var(--pele-radius,4px)";
fn setting(c: &Object, key: &str, fallback: f64, min: f64, max: f64) -> f64 {
    match c
        .get("wardley-beta")
        .and_then(Value::as_object)
        .and_then(|o| o.get(key))
    {
        Some(Value::Number(n)) if *n >= min && *n <= max => *n,
        _ => fallback,
    }
}
fn clamp(n: f64, min: f64, max: f64) -> f64 {
    if n < min {
        min
    } else if n > max {
        max
    } else {
        n
    }
}
fn dimension(c: &Object, given: Option<f64>, key: &str, fallback: f64) -> f64 {
    clamp(
        given
            .filter(|n| n.is_finite())
            .unwrap_or_else(|| setting(c, key, fallback, 1., 1e6)),
        200.,
        10000.,
    ) - 96.
}
fn start_label(label: &Label, x: f64, y: f64, attrs: &str) -> String {
    let mut shared = 3;
    let mut any = false;
    for s in label.lines.iter().flatten() {
        if s.icon.is_none() && !s.text.is_empty() {
            shared &= s.style;
            any = true
        }
    }
    if !any {
        shared = 0
    }
    let mut attrs = attrs.to_owned();
    if shared & 1 != 0 && !attrs.contains(" font-weight=\"") {
        attrs += " font-weight=\"var(--_w)\""
    }
    if shared & 2 != 0 && !attrs.contains(" font-style=\"") {
        attrs += " font-style=\"italic\""
    }
    let mut body = String::new();
    let mut baseline = y;
    for line in &label.lines {
        let mut first = true;
        for s in line {
            if s.icon.is_some() {
                continue;
            }
            let style = s.style & !shared;
            body += &format!(
                "<tspan{}{}{}>{}</tspan>",
                if first {
                    format!(" x=\"{}\" y=\"{}\"", num(x), num(baseline))
                } else {
                    String::new()
                },
                if style & 1 != 0 {
                    " font-weight=\"var(--_w)\""
                } else {
                    ""
                },
                if style & 2 != 0 {
                    " font-style=\"italic\""
                } else {
                    ""
                },
                esc(&s.text)
            );
            first = false;
        }
        baseline += label.line_height;
    }
    if body.is_empty() {
        String::new()
    } else {
        format!("<text{attrs} xml:space=\"preserve\">{body}</text>")
    }
}
struct Drawing<'a> {
    r: &'a mut dyn RenderResources,
    o: &'a RenderSettings,
    small: f64,
    plot_w: f64,
    plot_h: f64,
    tight: bool,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}
impl Drawing<'_> {
    fn grow(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) {
        if x0 < self.min_x {
            self.min_x = x0
        }
        if y0 < self.min_y {
            self.min_y = y0
        }
        if x1 > self.max_x {
            self.max_x = x1
        }
        if y1 > self.max_y {
            self.max_y = y1
        }
    }
    fn centered(
        &mut self,
        label: &Label,
        x: f64,
        y: f64,
        attrs: &str,
    ) -> Result<String, PeleError> {
        self.grow(
            x - label.width / 2.,
            y - label.height / 2.,
            x + label.width / 2.,
            y + label.height / 2.,
        );
        label_svg_with_resources(label, x, y, attrs, self.r)
    }
    fn starting(&mut self, label: &Label, x: f64, y: f64, attrs: &str) -> String {
        self.grow(
            x,
            y - label.size,
            x + label.width,
            y - label.size + label.height + 4.,
        );
        start_label(label, x, y, attrs)
    }
    fn text(
        &mut self,
        raw: &str,
        size: f64,
        width: Option<f64>,
        style: u8,
    ) -> Result<Label, PeleError> {
        let width = width.unwrap_or_else(|| {
            if self.tight {
                64f64.js_max(120f64.js_min(self.plot_w / 2.))
            } else {
                4000.
            }
        });
        layout_label(Some(raw), false, self.r, self.o, size, width, style)
    }
    fn x(&self, p: f64) -> f64 {
        p / 100. * self.plot_w
    }
    fn y(&self, p: f64) -> f64 {
        self.plot_h - p / 100. * self.plot_h
    }
}
struct Placed<N> {
    node: N,
    identity: u64,
    x: f64,
    y: f64,
}
fn reach<P: PhaseInput>(
    input: &mut P,
    at: &Placed<P::Value>,
    ux: f64,
    uy: f64,
    d: &mut Drawing<'_>,
    labels: &mut HashMap<u64, Label>,
) -> Result<f64, P::Error> {
    if input.field_text(&at.node, "className")?.as_deref() == Some("anchor") {
        if !labels.contains_key(&at.identity) {
            let text = input.field_text(&at.node, "label")?.unwrap_or_default();
            input.keep_label(text.encode_utf16().count())?;
            let label = d.text(
                &text,
                d.small,
                Some(if d.tight {
                    64f64.js_max(d.plot_w / 2.)
                } else {
                    4000.
                }),
                1,
            )?;
            labels.insert(at.identity, label);
        }
        let label = &labels[&at.identity];
        Ok(1. / (ux / (label.width / 2. + 5.)).hypot(uy / (d.small * 0.8)))
    } else if input.field_truth(&at.node, "isPipelineParent")? {
        Ok(10. / std::f64::consts::SQRT_2)
    } else {
        Ok(if input.field_truth(&at.node, "sourceStrategy")? {
            12.
        } else {
            6.
        })
    }
}
fn occupy(taken: &mut [u8], i: usize, ux: f64, uy: f64) {
    if ux.abs() < 0.25 || uy.abs() < 0.25 {
        return;
    }
    taken[i] |= if ux > 0. {
        if uy < 0. {
            1
        } else {
            2
        }
    } else if uy < 0. {
        4
    } else {
        8
    };
}
pub(super) fn render(
    model: &WardleyModel,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_phases(&mut Typed(model), c, o, r)
}
pub(super) fn render_phases<P: PhaseInput>(
    input: &mut P,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    let root = input.root();
    let given = input.get(&root, "size")?;
    let width = if input.nullish(&given) {
        None
    } else {
        input.field_number(&given, "width")?
    };
    let natural = dimension(c, width, "width", 900.);
    let mut result = draw(input, c, o, r, natural, natural, false)?;
    let Some(available) = o.max_width.filter(|n| *n > 0.) else {
        return Ok(result);
    };
    if result.width <= available {
        return Ok(result);
    }
    let floor = 160f64.js_min(natural);
    let mut plot = natural;
    result = draw(input, c, o, r, plot, natural, true)?;
    for _ in 0..3 {
        if !(result.width > available && plot > floor) {
            break;
        }
        plot = floor.js_max(plot - (result.width - available));
        result = draw(input, c, o, r, plot, natural, true)?;
    }
    Ok(result)
}
fn draw<P: PhaseInput>(
    input: &mut P,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    plot_w: f64,
    natural_w: f64,
    tight: bool,
) -> Result<Rendered, P::Error> {
    input.begin_draw();
    let root = input.root();
    let size = o.font_size.unwrap_or(16.);
    let small = round(size * 0.8125);
    let tiny = round(size * 0.6875);
    let pad = o.padding.unwrap_or(8.);
    let given = input.get(&root, "size")?;
    let height = if input.nullish(&given) {
        None
    } else {
        input.field_number(&given, "height")?
    };
    let plot_h = dimension(c, height, "height", 600.) * 0.75f64.js_max(plot_w / natural_w);
    let show_grid = matches!(
        c.get("wardley-beta")
            .and_then(Value::as_object)
            .and_then(|o| o.get("showGrid")),
        Some(Value::Bool(true))
    );
    let mut d = Drawing {
        r,
        o,
        small,
        plot_w,
        plot_h,
        tight,
        min_x: 0.,
        min_y: 0.,
        max_x: plot_w,
        max_y: plot_h,
    };
    let defaults = ["Genesis", "Custom Built", "Product", "Commodity"];
    let axes = input.get(&root, "axes")?;
    let stage_value = input.get(&axes, "stages")?;
    let stages = if input.truthy(&stage_value) && input.count(&stage_value)? > 0 {
        Some(input.get(&axes, "stages")?)
    } else {
        None
    };
    let bounds = input.get(&axes, "stageBoundaries")?;
    let stage_count = if let Some(v) = &stages {
        input.count(v)?
    } else {
        defaults.len()
    };
    let custom = !input.nullish(&bounds) && input.count(&bounds)? == stage_count;
    let mut frame = String::new();
    let mut stage_labels = String::new();
    let mut stage_bottom = plot_h;
    let mut previous = 0.;
    for i in 0..stage_count {
        if let Some(v) = &stages {
            if !input.has_index(v, i)? {
                continue;
            }
        }
        input.work(1)?;
        let stage = if let Some(v) = &stages {
            let v = input.index(v, i)?;
            input.text(&v)?
        } else {
            defaults[i].into()
        };
        let current_count = if let Some(v) = &stages {
            input.count(v)?
        } else {
            defaults.len()
        };
        let start = if custom {
            previous
        } else {
            i as f64 / current_count as f64
        };
        let end = if custom {
            {
                let v = input.index(&bounds, i)?;
                clamp(input.number(&v).unwrap_or(f64::NAN), 0., 1.)
            }
        } else {
            (i + 1) as f64 / current_count as f64
        };
        previous = end;
        if i > 0 {
            frame+=&format!("<path class=\"pele-wardley-stage\" d=\"M{},0V{}\" stroke=\"var(--_b)\" stroke-dasharray=\"4 4\"/>",num(start*plot_w),num(plot_h));
        }
        input.keep_label(stage.encode_utf16().count())?;
        let label = d.text(
            &stage,
            small,
            Some(48f64.js_max((end - start).abs() * plot_w - 12.)),
            0,
        )?;
        stage_labels += &d.centered(
            &label,
            (start + end) / 2. * plot_w,
            plot_h + 8. + label.height / 2.,
            &format!(
                " class=\"pele-wardley-stage-label\" font-size=\"{}\" fill=\"var(--_m)\"",
                svg::js_num(small)
            ),
        )?;
        stage_bottom = stage_bottom.js_max(plot_h + 8. + label.height);
    }
    if show_grid {
        for i in 1..4 {
            frame+=&format!("<path class=\"pele-wardley-grid\" d=\"M{},0V{}M0,{}H{}\" stroke=\"var(--_b)\" stroke-dasharray=\"2 6\"/>",num(plot_w*i as f64/4.),num(plot_h),num(plot_h*i as f64/4.),num(plot_w));
        }
    }
    frame += &format!(
        "<path class=\"pele-wardley-axis\" d=\"M0,0V{}H{}\" stroke=\"{LINE}\"/>",
        num(plot_h),
        num(plot_w)
    );
    let axes = input.get(&root, "axes")?;
    let x_text = input
        .field_text(&axes, "xLabel")?
        .unwrap_or_else(|| "Evolution".into());
    input.keep_label(x_text.encode_utf16().count())?;
    let x_title = d.text(&x_text, small, None, 0)?;
    let axes = input.get(&root, "axes")?;
    let y_text = input
        .field_text(&axes, "yLabel")?
        .unwrap_or_else(|| "Visibility".into());
    input.keep_label(y_text.encode_utf16().count())?;
    let y_title = d.text(&y_text, small, None, 0)?;
    let attrs = format!(
        " class=\"pele-wardley-axis-label\" font-size=\"{}\" font-weight=\"var(--_hw)\"",
        svg::js_num(small)
    );
    let axis_labels = d.centered(
        &x_title,
        plot_w / 2.,
        stage_bottom + 6. + x_title.height / 2.,
        &attrs,
    )? + &label_svg(
        &y_title,
        -16.,
        plot_h / 2.,
        &format!("{attrs} transform=\"rotate(-90 -16 {})\"", num(plot_h / 2.)),
    );
    d.grow(-16. - y_title.height / 2., 0., 0., 0.);
    let legend_top = stage_bottom + 6. + x_title.height + 10.;
    let mut positions: Vec<Placed<P::Value>> = Vec::new();
    let mut indices: HashMap<String, usize> = HashMap::new();
    let node_list = input.get(&root, "nodes")?;
    let mut ni = 0;
    while ni < input.count(&node_list)? {
        input.work(1)?;
        let node = input.index(&node_list, ni)?;
        ni += 1;
        let id = input.field_text(&node, "id")?.unwrap_or_default();
        let x = input.field_number(&node, "x")?.unwrap_or(f64::NAN);
        let y = input.field_number(&node, "y")?.unwrap_or(f64::NAN);
        let at = Placed {
            identity: input.identity(&node)?,
            node,
            x: d.x(x),
            y: d.y(y),
        };
        if let Some(&i) = indices.get(&id) {
            positions[i] = at
        } else {
            indices.insert(id, positions.len());
            positions.push(at)
        }
    }
    let mut pipelines = String::new();
    let mut members: HashMap<String, HashSet<String>> = HashMap::new();
    let pipeline_list = input.get(&root, "pipelines")?;
    let mut pi = 0;
    while pi < input.count(&pipeline_list)? {
        input.work(1)?;
        let pipeline = input.index(&pipeline_list, pi)?;
        pi += 1;
        let id = input.field_text(&pipeline, "nodeId")?.unwrap_or_default();
        let ids = input.get(&pipeline, "componentIds")?;
        let mut member_set = HashSet::new();
        let mut ci = 0;
        while ci < input.count(&ids)? {
            input.work(1)?;
            let v = input.index(&ids, ci)?;
            ci += 1;
            member_set.insert(input.text(&v)?);
        }
        members.insert(id, member_set);
        let ids = input.get(&pipeline, "componentIds")?;
        let mut parts = Vec::new();
        let mut ci = 0;
        while ci < input.count(&ids)? {
            input.work(1)?;
            let v = input.index(&ids, ci)?;
            ci += 1;
            let id = input.text(&v)?;
            if let Some(i) = indices.get(&id) {
                parts.push(*i)
            }
        }
        if parts.is_empty() {
            continue;
        }
        parts.sort_by(|a, b| {
            positions[*a]
                .x
                .partial_cmp(&positions[*b].x)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let left = positions[parts[0]].x;
        let right = positions[*parts.last().unwrap()].x;
        let y = positions[parts[0]].y;
        let top = y - 12.;
        let id = input.field_text(&pipeline, "nodeId")?.unwrap_or_default();
        if let Some(&i) = indices.get(&id) {
            positions[i].x = (left + right) / 2.;
            positions[i].y = top - 10. / 6.;
        }
        d.grow(left - 15., top, right + 15., top + 24.);
        pipelines+=&format!("<g class=\"pele-wardley-pipeline\" data-id=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"24\" rx=\"{RADIUS}\" fill=\"var(--_s)\" stroke=\"var(--_b)\"/>{}</g>",esc_text(&id)?,num(left-15.),num(top),num(right-left+30.),if parts.len()>1{format!("<path d=\"M{},{}H{}\" stroke=\"{LINE}\" stroke-dasharray=\"4 4\"/>",num(left),num(y),num(right))}else{String::new()});
    }
    let mut anchor_labels = HashMap::new();
    let mut taken = vec![0; positions.len()];
    let mut links = String::new();
    let link_list = input.get(&root, "links")?;
    let mut li = 0;
    while li < input.count(&link_list)? {
        input.work(1)?;
        let link = input.index(&link_list, li)?;
        li += 1;
        let source = input.field_text(&link, "source")?.unwrap_or_default();
        let target = input.field_text(&link, "target")?.unwrap_or_default();
        let (Some(&fi), Some(&ti)) = (indices.get(&source), indices.get(&target)) else {
            continue;
        };
        if fi == ti
            || members
                .get(target.as_str())
                .is_some_and(|m| m.contains(source.as_str()))
        {
            continue;
        }
        let from = &positions[fi];
        let to = &positions[ti];
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let distance = dx.hypot(dy);
        if distance < 0.01 {
            continue;
        }
        let ux = dx / distance;
        let uy = dy / distance;
        let start = reach(input, from, ux, uy, &mut d, &mut anchor_labels)?;
        let end = reach(input, to, ux, uy, &mut d, &mut anchor_labels)?;
        if start + end >= distance {
            continue;
        }
        occupy(&mut taken, fi, ux, uy);
        occupy(&mut taken, ti, -ux, -uy);
        let mut x1 = from.x + ux * start;
        let mut y1 = from.y + uy * start;
        let mut x2 = to.x - ux * end;
        let mut y2 = to.y - uy * end;
        let mut heads = String::new();
        if matches!(
            input.field_text(&link, "flow")?.as_deref(),
            Some("forward" | "bidirectional")
        ) {
            heads += &marker("arrow_point", x2, y2, ux, uy, LINE);
            x2 -= ux * marker_trim("arrow_point");
            y2 -= uy * marker_trim("arrow_point");
        }
        if matches!(
            input.field_text(&link, "flow")?.as_deref(),
            Some("backward" | "bidirectional")
        ) {
            heads += &marker("arrow_point", x1, y1, -ux, -uy, LINE);
            x1 += ux * marker_trim("arrow_point");
            y1 += uy * marker_trim("arrow_point");
        }
        let mut caption = String::new();
        if let Some(text) = input.field_text(&link, "label")?.filter(|s| !s.is_empty()) {
            input.keep_label(text.encode_utf16().count())?;
            let label = d.text(&text, tiny + 1., Some(200.), 0)?;
            let side = if ux >= 0. { 1. } else { -1. };
            let px = uy * side;
            let py = -ux * side;
            let away = 5. + px.abs() * label.width / 2. + py.abs() * label.height / 2.;
            caption = d.centered(
                &label,
                (from.x + to.x) / 2. + px * away,
                (from.y + to.y) / 2. + py * away,
                &format!(
                    " class=\"pele-edge-label\" font-size=\"{}\" fill=\"var(--_m)\"",
                    svg::js_num(tiny + 1.)
                ),
            )?;
        }
        links+=&format!("<g class=\"pele-edge{}\" data-id=\"{}-{}\"><path d=\"M{},{}L{},{}\" stroke=\"{LINE}\"{}/>{heads}{caption}</g>",if input.field_truth(&link,"dashed")?{" pele-wardley-dashed"}else{""},esc_text(&input.field_text(&link,"source")?.unwrap_or_default())?,esc_text(&input.field_text(&link,"target")?.unwrap_or_default())?,num(x1),num(y1),num(x2),num(y2),if input.field_truth(&link,"dashed")?{" stroke-dasharray=\"6 6\""}else{""});
    }
    let mut trends = String::new();
    let trend_list = input.get(&root, "trends")?;
    let mut ti = 0;
    while ti < input.count(&trend_list)? {
        input.work(1)?;
        let trend = input.index(&trend_list, ti)?;
        ti += 1;
        let trend_id = input.field_text(&trend, "nodeId")?.unwrap_or_default();
        let Some(&i) = indices.get(&trend_id) else {
            continue;
        };
        let origin = &positions[i];
        let tx = d.x(input.field_number(&trend, "targetX")?.unwrap_or(f64::NAN));
        let ty = d.y(input.field_number(&trend, "targetY")?.unwrap_or(f64::NAN));
        let dx = tx - origin.x;
        let dy = ty - origin.y;
        let distance = dx.hypot(dy);
        let mut arrow = String::new();
        let start = if distance > 0. {
            reach(
                input,
                origin,
                dx / distance,
                dy / distance,
                &mut d,
                &mut anchor_labels,
            )?
        } else {
            0.
        };
        if distance > start + 10. {
            let ux = dx / distance;
            let uy = dy / distance;
            let ex = tx - ux * 8.;
            let ey = ty - uy * 8.;
            arrow = format!(
                "<path d=\"M{},{}L{},{}\" stroke=\"{ACCENT}\" stroke-dasharray=\"4 4\"/>{}",
                num(origin.x + ux * start),
                num(origin.y + uy * start),
                num(ex - ux * marker_trim("arrow_point")),
                num(ey - uy * marker_trim("arrow_point")),
                marker("arrow_point", ex, ey, ux, uy, ACCENT)
            );
        }
        d.grow(tx - 6., ty - 6., tx + 6., ty + 6.);
        trends+=&format!("<g class=\"pele-wardley-trend\" data-id=\"{}\">{arrow}<circle cx=\"{}\" cy=\"{}\" r=\"6\" fill=\"var(--_bg)\" stroke=\"{ACCENT}\"/></g>",esc_text(&input.field_text(&trend,"nodeId")?.unwrap_or_default())?,num(tx),num(ty));
    }
    let mut nodes = String::new();
    let tri = 1.2 * 6.;
    let dot = 0.7 * 6.;
    for (i, at) in positions.iter().enumerate() {
        let node = &at.node;
        let (x, y) = (at.x, at.y);
        let anchor = input.field_text(node, "className")?.as_deref() == Some("anchor");
        let strategy_value = input
            .field_text(node, "sourceStrategy")?
            .filter(|s| !s.is_empty());
        let strategy = strategy_value.as_deref();
        let parent = input.field_truth(node, "isPipelineParent")?;
        let mut shape = String::new();
        if let Some(strategy) = strategy {
            let fill = match strategy {
                "outsource" => "var(--_m)",
                "buy" => "var(--_a)",
                "build" => "var(--_s)",
                _ => "var(--_bg)",
            };
            shape+=&format!("<circle class=\"pele-wardley-{strategy}\" cx=\"{}\" cy=\"{}\" r=\"12\" fill=\"{fill}\" stroke=\"{LINE}\"/>",num(x),num(y));
        }
        if strategy == Some("market") {
            let bx = tri * (std::f64::consts::PI / 6.).cos();
            let by = y + tri * (std::f64::consts::PI / 6.).sin();
            shape += &format!(
                "<path d=\"M{},{}L{},{}H{}Z\" fill=\"none\" stroke=\"{LINE}\"/>",
                num(x),
                num(y - tri),
                num(x - bx),
                num(by),
                num(x + bx)
            );
            for (cx, cy) in [(x, y - tri), (x - bx, by), (x + bx, by)] {
                shape += &format!(
                    "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"var(--_bg)\" stroke=\"{LINE}\"/>",
                    num(cx),
                    num(cy),
                    num(dot)
                );
            }
        } else if parent {
            shape+=&format!("<rect x=\"{}\" y=\"{}\" width=\"10\" height=\"10\" fill=\"var(--_bg)\" stroke=\"{LINE}\"/>",num(x-5.),num(y-5.));
        } else if !anchor {
            shape += &format!(
                "<circle cx=\"{}\" cy=\"{}\" r=\"6\" fill=\"var(--_bg)\" stroke=\"{LINE}\"/>",
                num(x),
                num(y)
            );
        }
        if input.field_truth(node, "inertia")? {
            let offset =
                if parent { 5. } else { 6. } + 10. + if strategy.is_some() { 10. } else { 0. };
            shape+=&format!("<rect class=\"pele-wardley-inertia\" x=\"{}\" y=\"{}\" width=\"4\" height=\"12\" fill=\"{LINE}\"/>",num(x+offset),num(y-6.));
        }
        let ox = input
            .field_number(node, "labelOffsetX")?
            .map(|x| clamp(x, -5000., 5000.));
        let oy = input
            .field_number(node, "labelOffsetY")?
            .map(|y| clamp(y, -5000., 5000.));
        let caption = if anchor {
            let label = if let Some(l) = anchor_labels.get(&at.identity) {
                l.clone()
            } else {
                let text = input.field_text(node, "label")?.unwrap_or_default();
                input.keep_label(text.encode_utf16().count())?;
                d.text(
                    &text,
                    small,
                    Some(if tight {
                        64f64.js_max(plot_w / 2.)
                    } else {
                        4000.
                    }),
                    1,
                )?
            };
            d.centered(
                &label,
                x + ox.unwrap_or(0.),
                y + oy.unwrap_or(-3.),
                &format!(
                    " class=\"pele-label\" font-size=\"{}\" font-weight=\"var(--_hw)\"",
                    svg::js_num(small)
                ),
            )?
        } else {
            let text = input.field_text(node, "label")?.unwrap_or_default();
            input.keep_label(text.encode_utf16().count())?;
            let label = d.text(&text, small, None, 0)?;
            let attrs = format!(" class=\"pele-label\" font-size=\"{}\"", svg::js_num(small));
            let lift = 8. + if strategy.is_some() { 10. } else { 0. };
            if ox.is_some() || oy.is_some() {
                d.starting(
                    &label,
                    x + ox.unwrap_or(lift),
                    y + oy.unwrap_or(-lift),
                    &attrs,
                )
            } else if input.field_truth(node, "inPipeline")? {
                d.centered(&label, x, y + 12. + 4. + label.height / 2., &attrs)?
            } else {
                let busy = taken[i];
                let corner = if busy & 1 == 0 {
                    1
                } else if busy & 2 == 0 {
                    2
                } else if busy & 4 == 0 {
                    4
                } else if busy & 8 == 0 {
                    8
                } else {
                    1
                };
                let right = corner == 1 || corner == 2;
                let up = corner == 1 || corner == 4;
                d.starting(
                    &label,
                    if right {
                        x + lift
                    } else {
                        x - lift - label.width
                    },
                    if up {
                        y - lift
                    } else {
                        y + lift + small * 0.75
                    },
                    &attrs,
                )
            }
        };
        d.grow(x - 12., y - 12., x + 12. + 20., y + 12.);
        nodes += &format!(
            "<g class=\"pele-node pele-wardley-{}\" data-id=\"{}\">{shape}{caption}</g>",
            if anchor {
                "anchor"
            } else if input.field_truth(node, "isPipelineParent")? {
                "pipeline-parent"
            } else if input.field_truth(node, "inPipeline")? {
                "pipeline-component"
            } else {
                "component"
            },
            esc_text(&input.field_text(node, "id")?.unwrap_or_default())?
        );
    }
    let mut annotations = String::new();
    let annotation_list = input.get(&root, "annotations")?;
    let mut ai = 0;
    while ai < input.count(&annotation_list)? {
        input.work(1)?;
        let a = input.index(&annotation_list, ai)?;
        ai += 1;
        let label = d.text(
            &svg::js_num(input.field_number(&a, "number")?.unwrap_or(f64::NAN)),
            tiny,
            Some(4000.),
            1,
        )?;
        let mut marks = String::new();
        let coords = input.get(&a, "coordinates")?;
        let mut ci = 0;
        while ci < input.count(&coords)? {
            input.work(1)?;
            let p = input.index(&coords, ci)?;
            ci += 1;
            let x = d.x(input.field_number(&p, "x")?.unwrap_or(f64::NAN));
            let y = d.y(input.field_number(&p, "y")?.unwrap_or(f64::NAN));
            let radius = 10f64.js_max(label.width / 2. + 4.);
            d.grow(x - radius, y - radius, x + radius, y + radius);
            marks += &format!(
                "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"var(--_bg)\" stroke=\"{LINE}\"/>{}",
                num(x),
                num(y),
                num(radius),
                label_svg(
                    &label,
                    x,
                    y,
                    &format!(
                        " font-size=\"{}\" font-weight=\"var(--_hw)\"",
                        svg::js_num(tiny)
                    )
                )
            );
        }
        annotations += &format!(
            "<g class=\"pele-wardley-annotation\" data-id=\"{}\">{marks}</g>",
            esc_text(&svg::js_num(
                input.field_number(&a, "number")?.unwrap_or(f64::NAN)
            ))?
        );
    }
    let list = input.get(&root, "annotations")?;
    let mut listed = Vec::new();
    let n = input.count(&list)?;
    for i in 0..n {
        input.work(1)?;
        if !input.has_index(&list, i)? {
            continue;
        }
        let a = input.index(&list, i)?;
        if input.field_truth(&a, "text")? {
            let number = input.field_number(&a, "number")?.unwrap_or(f64::NAN);
            listed.push((a, number));
        }
    }
    listed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    if !listed.is_empty() {
        let mut lines = vec![];
        let (mut width, mut height) = (0f64, 0f64);
        for (a, _) in listed {
            let caption = format!(
                "{}. {}",
                svg::js_num(input.field_number(&a, "number")?.unwrap_or(f64::NAN)),
                input.field_text(&a, "text")?.unwrap_or_default()
            );
            input.keep_label(caption.encode_utf16().count())?;
            let line = d.text(
                &caption,
                small,
                Some(if tight {
                    80f64.js_max(plot_w - 20.)
                } else {
                    320.
                }),
                0,
            )?;
            width = width.js_max(line.width);
            height += line.height;
            lines.push(line)
        }
        width += 20.;
        height += 14.;
        let b = input.get(&root, "annotationsBox")?;
        let (bx, by) = if input.truthy(&b) {
            (
                clamp(
                    d.x(input.field_number(&b, "x")?.unwrap_or(f64::NAN)),
                    0.,
                    0f64.js_max(plot_w - width),
                ),
                clamp(
                    d.y(input.field_number(&b, "y")?.unwrap_or(f64::NAN)),
                    0.,
                    0f64.js_max(plot_h - height),
                ),
            )
        } else {
            (0., legend_top)
        };
        d.grow(bx, by, bx + width, by + height);
        let mut rows = String::new();
        let mut y = by + 7.;
        for line in &lines {
            rows += &start_label(
                line,
                bx + 10.,
                y + line.line_height / 2. + small * 0.35,
                &format!(" font-size=\"{}\"", svg::js_num(small)),
            );
            y += line.height;
        }
        annotations+=&format!("<g class=\"pele-wardley-annotations\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{RADIUS}\" fill=\"var(--_bg)\" stroke=\"var(--_b)\"/>{rows}</g>",num(bx),num(by),num(width),num(height));
    }
    let mut notes = String::new();
    let note_list = input.get(&root, "notes")?;
    let mut ni = 0;
    while ni < input.count(&note_list)? {
        input.work(1)?;
        let n = input.index(&note_list, ni)?;
        ni += 1;
        let note_text = input.field_text(&n, "text")?.unwrap_or_default();
        input.keep_label(note_text.encode_utf16().count())?;
        let label = d.text(&note_text, small, Some(360f64.js_min(plot_w * 0.6)), 1)?;
        notes += &d.starting(
            &label,
            d.x(input.field_number(&n, "x")?.unwrap_or(f64::NAN)),
            d.y(input.field_number(&n, "y")?.unwrap_or(f64::NAN)),
            &format!(
                " class=\"pele-wardley-note\" font-size=\"{}\" font-weight=\"var(--_hw)\"",
                svg::js_num(small)
            ),
        );
    }
    let mut forces = String::new();
    for (forward, key) in [(true, "accelerators"), (false, "deaccelerators")] {
        let list = input.get(&root, key)?;
        let mut fi = 0;
        while fi < input.count(&list)? {
            input.work(1)?;
            let a = input.index(&list, fi)?;
            fi += 1;
            let name = input.field_text(&a, "name")?.unwrap_or_default();
            let x = d.x(input.field_number(&a, "x")?.unwrap_or(f64::NAN));
            let y = d.y(input.field_number(&a, "y")?.unwrap_or(f64::NAN));
            let tip = if forward { x + 60. } else { x };
            let tail = if forward { x } else { x + 60. };
            let neck = if forward { x + 40. } else { x + 20. };
            let path = format!(
                "M{},{}H{}V{}L{},{}L{},{}V{}H{}Z",
                num(tail),
                num(y - 15.),
                num(neck),
                num(y - 23.),
                num(tip),
                num(y),
                num(neck),
                num(y + 23.),
                num(y + 15.),
                num(tail)
            );
            d.grow(x, y - 23., x + 60., y + 23.);
            input.keep_label(name.encode_utf16().count())?;
            let label = d.text(&name, tiny + 1., Some(160.), 1)?;
            forces+=&format!("<g class=\"pele-wardley-{}\" data-id=\"{}\"><path d=\"{path}\" fill=\"var(--_s)\" stroke=\"{LINE}\"/>{}</g>",if forward{"accelerator"}else{"deaccelerator"},esc_text(&name)?,d.centered(&label,x+30.,y+35.,&format!(" font-size=\"{}\" font-weight=\"var(--_hw)\"",svg::js_num(tiny+1.)))?);
        }
    }
    let title_text = input.field_text(&root, "title")?;
    if let Some(t) = &title_text {
        input.keep_label(t.encode_utf16().count())?;
    }
    let title = layout_label(
        title_text.as_deref(),
        false,
        d.r,
        o,
        size,
        if tight { 80f64.js_max(plot_w) } else { 4000. },
        1,
    )?;
    let title_height = if title.height > 0. {
        title.height + 12.
    } else {
        0.
    };
    let left = d.min_x.js_min(plot_w / 2. - title.width / 2.);
    let right = d.max_x.js_max(plot_w / 2. + title.width / 2.);
    let tx = pad - left;
    let ty = pad + title_height - d.min_y;
    let total_width = (right - left + 2. * pad).ceil();
    let total_height = (d.max_y - d.min_y + title_height + 2. * pad).ceil();
    let content=label_svg(&title,tx+plot_w/2.,pad+title.height/2.," class=\"pele-title\" font-weight=\"var(--_tw)\"")+&format!("<g transform=\"translate({},{})\"><g class=\"pele-wardley-frame\" fill=\"none\">{frame}</g>{stage_labels}{axis_labels}{pipelines}{}{nodes}{annotations}{notes}{forces}</g>",num(tx),num(ty),if links.is_empty(){String::new()}else{format!("<g class=\"pele-edges\" fill=\"none\">{links}</g>")}+&if trends.is_empty(){String::new()}else{format!("<g class=\"pele-wardley-trends\" fill=\"none\">{trends}</g>")});
    let acc_title = input.field_text(&root, "accTitle")?;
    let acc_descr = input.field_text(&root, "accDescr")?;
    Ok(Rendered {
        svg: svg::document(
            "wardley",
            total_width,
            total_height,
            size,
            o,
            acc_title.as_deref(),
            acc_descr.as_deref(),
            &content,
        )?,
        width: total_width,
        height: total_height,
        links: vec![],
    })
}
