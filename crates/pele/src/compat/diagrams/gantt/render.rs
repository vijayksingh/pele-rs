//! Pele gantt/render.ts direct translation, 458322a5 (MIT, kepano).
use super::{
    axis::{auto_ticks, interval_ticks, time_format},
    dates::Calendar,
    model::GanttModel,
    phase::{PhaseInput, Typed},
};
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    source::url::linked,
    svg::{
        document, edges::theme::resolve_style, esc, esc_text, label_svg, label_svg_with_resources,
        num,
    },
    text::{layout_label, measure, Label},
    Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
use regex::Regex;
use std::{collections::HashMap, sync::OnceLock};
fn setting(c: &Object, key: &str, min: f64, max: f64) -> Option<f64> {
    match c
        .get("gantt")
        .and_then(Value::as_object)
        .and_then(|c| c.get(key))
    {
        Some(Value::Number(n)) if n.is_finite() => Some(n.js_max(min).js_min(max)),
        _ => None,
    }
}
fn text<'a>(c: &'a Object, key: &str) -> Option<&'a str> {
    match c
        .get("gantt")
        .and_then(Value::as_object)
        .and_then(|c| c.get(key))
    {
        Some(Value::String(s)) => Some(s),
        _ => None,
    }
}
fn flag(c: &Object, key: &str) -> bool {
    c.get("gantt")
        .and_then(Value::as_object)
        .and_then(|c| c.get(key))
        == Some(&Value::Bool(true))
}
fn one_line(
    raw: &str,
    size: f64,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Label, PeleError> {
    static R: OnceLock<Regex> = OnceLock::new();
    let raw = R
        .get_or_init(|| compiled_regex(r"(?i)<br\s*/?>").unwrap())
        .replace_all(raw, " ");
    let mut label = layout_label(Some(&raw), false, r, o, size, 4000., 0)?;
    if label.lines.len() > 1 {
        label.lines.truncate(1);
        label.widths.truncate(1);
        label.width = label.widths[0];
        label.height = label.line_height
    }
    Ok(label)
}
struct Bar<T> {
    task: T,
    label: Label,
    row: usize,
    x0: f64,
    x1: f64,
    label_x: f64,
    inside: bool,
}
pub fn render(
    model: &GanttModel,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    let size = o.font_size.unwrap_or(16.);
    let pad = o.padding.unwrap_or(8.);
    let now = if let Some(now) = o.now.filter(|n| n.is_finite()) {
        now
    } else {
        r.now_millis()?
    };
    let mut cal = Calendar::new(r);
    let mut m = model.clone();
    let mut missing = false;
    let mut input = Typed {
        model: &mut m,
        schedule: true,
        filter_callable: true,
        missing_filter: &mut missing,
    };
    let result = draw(&mut input, c, o, &mut cal, size, pad, now)?;
    if let Some(e) = cal.error {
        return Err(e);
    }
    Ok(result)
}
/// Draw an already scheduled public model. Scheduling and its mutations belong
/// to the public store; draw's calendar-cache work remains observable afterward.
/// A plain declared GanttModel has getTasks, but not the DB-only date filter.
/// Check at the actual filter call after preceding callbacks, never eagerly.
pub(super) fn render_scheduled_checked(
    model: &mut GanttModel,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    filter_callable: bool,
) -> Result<Rendered, crate::compat::public::PublicError> {
    let mut cal = Calendar::public(r);
    let now = model.now;
    let mut missing_filter = false;
    let mut input = Typed {
        model,
        schedule: false,
        filter_callable,
        missing_filter: &mut missing_filter,
    };
    let result = draw(
        &mut input,
        c,
        o,
        &mut cal,
        o.font_size.unwrap_or(16.),
        o.padding.unwrap_or(8.),
        now,
    );
    if let Some(e) = cal.error {
        return Err(e.into());
    }
    if missing_filter {
        return Err(crate::compat::public::PublicError::JavaScript {
            name: crate::compat::public::JavaScriptErrorKind::TypeError,
            message: "t.isInvalidDate is not a function".into(),
        });
    }
    result.map_err(Into::into)
}

pub(super) fn draw<I: PhaseInput>(
    m: &mut I,
    c: &Object,
    o: &RenderSettings,
    cal: &mut Calendar<'_>,
    size: f64,
    pad: f64,
    now: f64,
) -> Result<Rendered, I::Error> {
    let label_size = setting(c, "fontSize", 4., 72.).unwrap_or(round(size * 0.8125));
    let section_size = setting(c, "sectionFontSize", 4., 72.).unwrap_or(label_size);
    let tick_size = round(size * 0.75);
    let bar_h = setting(c, "barHeight", 4., 400.).unwrap_or(round(label_size * 1.85));
    let bar_gap = setting(c, "barGap", 0., 400.).unwrap_or(round(bar_h / 4.));
    let row_h = bar_h + bar_gap;
    let section_styles =
        (setting(c, "numberSectionStyles", 1., 1000.).unwrap_or(4.) + 0.5).floor() as usize;
    let compact = m.root_text("displayMode")?.as_deref() == Some("compact");
    let top_axis = m.root_bool("topAxis")? || flag(c, "topAxis");
    let tasks = m.tasks(now, cal.resources)?;
    let mut row_tasks = Vec::new();
    for task in &tasks {
        m.work(1)?;
        if !m.flag(task, "vert")? {
            row_tasks.push(task.clone());
        }
    }
    let mut verts = Vec::new();
    for task in &tasks {
        m.work(1)?;
        if m.flag(task, "vert")? && m.scheduled(task, cal.resources)? {
            verts.push(task.clone());
        }
    }
    let mut categories: Vec<String> = Vec::new();
    for task in &row_tasks {
        m.work(1)?;
        let section = m.text(task, "section")?;
        m.work(categories.len())?;
        if !categories.contains(&section) {
            m.keep_text(section.encode_utf16().count())?;
            categories.push(section);
        }
    }
    let mut row_of = HashMap::new();
    let mut row_section: Vec<String> = Vec::new();
    if compact {
        // The upstream section groups capture handles before the first measure.
        let mut groups: Vec<(String, Vec<I::Task>)> = Vec::new();
        for task in &row_tasks {
            m.work(1)?;
            let section = m.text(task, "section")?;
            m.work(groups.len())?;
            if let Some((_, group)) = groups.iter_mut().find(|(key, _)| key == &section) {
                group.push(task.clone())
            } else {
                m.keep_text(section.encode_utf16().count())?;
                groups.push((section, vec![task.clone()]));
            }
        }
        for (section, group) in groups {
            let mut sorted = Vec::new();
            for task in &group {
                m.work(1)?;
                if m.scheduled(task, cal.resources)? {
                    sorted.push(task.clone());
                }
            }
            sort_tasks(&mut sorted, |a, b| -> Result<_, I::Error> {
                let delta = m.date(a, "startTime", cal.resources)?
                    - m.date(b, "startTime", cal.resources)?;
                let delta = if delta == 0. || delta.is_nan() {
                    m.order(a, cal.resources)? - m.order(b, cal.resources)?
                } else {
                    delta
                };
                Ok(delta.partial_cmp(&0.).unwrap_or(std::cmp::Ordering::Equal))
            })?;
            let offset = row_section.len();
            let mut free = Vec::new();
            for task in sorted {
                m.work(1)?;
                let start = m.date(&task, "startTime", cal.resources)?;
                m.work(free.len())?;
                let row = free
                    .iter()
                    .position(|&end| start >= end)
                    .unwrap_or(free.len());
                let end = m.date(&task, "endTime", cal.resources)?;
                if row == free.len() {
                    free.push(end)
                } else {
                    free[row] = end
                }
                row_of.insert(m.identity(&task)?, offset + row);
            }
            let mut rows = free.len();
            for task in group {
                m.work(1)?;
                let id = m.identity(&task)?;
                if !row_of.contains_key(&id) {
                    row_of.insert(id, offset + rows);
                    rows += 1;
                }
            }
            for _ in 0..rows.max(1) {
                m.keep_text(section.encode_utf16().count())?;
                row_section.push(section.clone());
            }
        }
    } else {
        for task in &row_tasks {
            m.work(1)?;
            row_of.insert(m.identity(task)?, row_section.len());
            let section = m.text(task, "section")?;
            m.keep_text(section.encode_utf16().count())?;
            row_section.push(section);
        }
    }
    let row_count = row_section.len();
    let (mut start, mut stop) = (f64::INFINITY, f64::NEG_INFINITY);
    for task in &tasks {
        m.work(1)?;
        if m.scheduled(task, cal.resources)? {
            start = start.js_min(m.date(task, "startTime", cal.resources)?);
            stop = stop.js_max(m.date(task, "endTime", cal.resources)?);
        }
    }
    let title_text = m.root_text("title")?;
    m.keep_text(title_text.as_deref().unwrap_or("").encode_utf16().count())?;
    let title = layout_label(
        title_text.as_deref(),
        false,
        cal.resources,
        o,
        size,
        4000.,
        1,
    )?;
    let title_h = if title.height > 0. {
        title.height + 16.
    } else {
        0.
    };
    if !(start <= stop) {
        let width = (title.width.js_max(1.) + 2. * pad).ceil();
        let height = (title.height.js_max(1.) + 2. * pad).ceil();
        let content = label_svg(
            &title,
            width / 2.,
            pad + title.height / 2.,
            " class=\"pele-title\" font-weight=\"var(--_tw)\"",
        );
        return Ok(Rendered {
            svg: document(
                "gantt",
                width,
                height,
                size,
                o,
                m.root_text("accTitle")?.as_deref(),
                m.root_text("accDescr")?.as_deref(),
                &content,
            )?,
            width,
            height,
            links: vec![],
        });
    }
    let mut section_labels = HashMap::new();
    let mut section_width = 0f64;
    for section in &categories {
        m.work(1)?;
        m.keep_text(section.encode_utf16().count())?;
        let label = layout_label(
            Some(section),
            false,
            cal.resources,
            o,
            section_size,
            180.,
            0,
        )?;
        section_width = section_width.js_max(label.width);
        section_labels.insert(section.clone(), label);
    }
    let axis = m.root_text("axisFormat")?;
    let axis_format = if let Some(s) = axis.filter(|s| !s.is_empty()) {
        s
    } else if m.root_text("dateFormat")?.as_deref() == Some("D") {
        "%d".into()
    } else {
        text(c, "axisFormat").unwrap_or("%Y-%m-%d").into()
    };
    let a = time_format(cal, &axis_format, start);
    cal.check()?;
    let start_width = measure(cal.resources, o, &a, tick_size, 0)?;
    let b = time_format(cal, &axis_format, stop);
    cal.check()?;
    let sample_width = start_width.js_max(measure(cal.resources, o, &b, tick_size, 0)?);
    let left = pad
        + setting(c, "leftPadding", 0., 2000.).unwrap_or(
            (if section_width > 0. {
                section_width + 24.
            } else {
                0.
            })
            .js_max(sample_width / 2.),
        );
    let right_pad =
        setting(c, "rightPadding", 0., 2000.).unwrap_or(16f64.js_max(sample_width / 2.));
    let natural = setting(c, "useWidth", 100., 20000.).unwrap_or(800.);
    let target = if let Some(a) = o.max_width.filter(|n| *n > 0.) {
        natural.js_min(280.).js_max(natural.js_min(a))
    } else {
        natural
    };
    let plot_width = 240f64.js_max(target - left - right_pad - pad);
    let right = left + plot_width;
    let scale = if stop > start {
        plot_width / (stop - start)
    } else {
        0.
    };
    let x = |t: f64| {
        if scale == 0. {
            left + plot_width / 2.
        } else {
            left + (t - start) * scale
        }
    };
    let tick_h = round(tick_size * 1.75);
    let top_padding = setting(c, "topPadding", 0., 2000.);
    let grid_start = setting(c, "gridLineStartPadding", 0., 2000.);
    let head_h = pad + title_h + if top_axis { tick_h } else { 0. };
    let plot_top = head_h.js_max(top_padding.unwrap_or(0.));
    let grid_lead = match (top_padding, grid_start) {
        (Some(top), Some(grid)) => (top - grid).js_min(plot_top - head_h).js_max(0.),
        _ => 0.,
    };
    let plot_h = row_count.max(1) as f64 * row_h;
    let plot_bottom = plot_top + plot_h;
    let mut bars = vec![];
    let mut extent = right + right_pad;
    for t in &row_tasks {
        m.work(1)?;
        let raw = m.text(t, "task")?;
        m.keep_text(raw.encode_utf16().count())?;
        let label = one_line(&raw, label_size, o, cal.resources)?;
        let row = row_of[&m.identity(t)?];
        if !m.scheduled(t, cal.resources)? {
            let label_x = left + 8. + label.width / 2.;
            extent = extent.js_max(label_x + label.width / 2.);
            bars.push(Bar {
                task: t.clone(),
                label,
                row,
                x0: f64::NAN,
                x1: f64::NAN,
                label_x,
                inside: false,
            });
            continue;
        }
        let mut x0 = x(m.date(t, "startTime", cal.resources)?);
        let mut x1 = x(m.render_end(t, cal.resources)?)
            .js_max(x0 + if m.flag(t, "milestone")? { 0. } else { 2. });
        if m.flag(t, "milestone")? {
            let mid = x0 + (x(m.date(t, "endTime", cal.resources)?) - x0) / 2.;
            x0 = mid - bar_h / 2.;
            x1 = mid + bar_h / 2.
        }
        let inside = !m.flag(t, "milestone")? && label.width + 16. <= x1 - x0;
        let mut label_x = (x0 + x1) / 2.;
        if !inside {
            let fits_right = x1 + 8. + label.width <= right + right_pad;
            let fits_left = x0 - 8. - label.width >= left;
            label_x = if fits_right || !fits_left {
                x1 + 8. + label.width / 2.
            } else {
                x0 - 8. - label.width / 2.
            };
            extent = extent.js_max(label_x + label.width / 2.)
        }
        bars.push(Bar {
            task: t.clone(),
            label,
            row,
            x0,
            x1,
            label_x,
            inside,
        })
    }
    let interval = m
        .root_text("tickInterval")?
        .filter(|s| !s.is_empty())
        .or_else(|| text(c, "tickInterval").map(str::to_owned))
        .unwrap_or_default();
    let weekday = m
        .root_text("weekday")?
        .or_else(|| text(c, "weekday").map(str::to_owned))
        .unwrap_or_else(|| "sunday".into());
    let count = (plot_width / (sample_width + 24.)).floor().clamp(2., 10.);
    let ticks = interval_ticks(
        cal,
        &interval,
        &weekday,
        start,
        stop,
        (plot_width / 3.).ceil() as usize,
    )
    .unwrap_or_else(|| auto_ticks(cal, start, stop, count));
    cal.check()?;
    let (mut grid, mut axis_bottom, mut axis_top) = (String::new(), String::new(), String::new());
    let mut last_right = f64::NEG_INFINITY;
    let mut last_label = String::new();
    let bottom_y = plot_bottom + tick_h / 2. + 2.;
    let top_y = plot_top - grid_lead - tick_h / 2. - 2.;
    for t in ticks {
        m.work(1)?;
        let tx = x(t);
        grid += &format!(
            "M{},{}V{}",
            num(tx),
            num(plot_top - grid_lead),
            num(plot_bottom)
        );
        let label = time_format(cal, &axis_format, t);
        cal.check()?;
        let width = measure(cal.resources, o, &label, tick_size, 0)?;
        if tx - width / 2. < last_right + 8. || label == last_label {
            continue;
        }
        last_right = tx + width / 2.;
        last_label = label.clone();
        extent = extent.js_max(last_right);
        let body = format!("text-anchor=\"middle\">{}</text>", esc(&label));
        axis_bottom += &format!(
            "<text x=\"{}\" y=\"{}\" {body}",
            num(tx),
            num(bottom_y + tick_size * 0.35)
        );
        if top_axis {
            axis_top += &format!(
                "<text x=\"{}\" y=\"{}\" {body}",
                num(tx),
                num(top_y + tick_size * 0.35)
            )
        }
    }
    let (mut vert_lines, mut vert_labels) = (String::new(), String::new());
    let mut vert_ends = vec![];
    let vert_top = plot_bottom + tick_h + 4.;
    let vert_line = round(label_size * 1.5);
    sort_tasks(&mut verts, |a, b| -> Result<_, I::Error> {
        let delta =
            m.date(a, "startTime", cal.resources)? - m.date(b, "startTime", cal.resources)?;
        Ok(delta.partial_cmp(&0.).unwrap_or(std::cmp::Ordering::Equal))
    })?;
    for t in &verts {
        m.work(1)?;
        let vx = x(m.date(t, "startTime", cal.resources)?);
        let raw = m.text(t, "task")?;
        m.keep_text(raw.encode_utf16().count())?;
        let label = one_line(&raw, label_size, o, cal.resources)?;
        let cx = (pad + label.width / 2.).js_max(vx);
        m.work(vert_ends.len())?;
        let line = vert_ends
            .iter()
            .position(|&end| cx - label.width / 2. >= end + 8.)
            .unwrap_or(vert_ends.len());
        if line == vert_ends.len() {
            vert_ends.push(cx + label.width / 2.)
        } else {
            vert_ends[line] = cx + label.width / 2.
        }
        extent = extent.js_max(vert_ends[line]);
        vert_lines += &format!(
            "<path class=\"pele-vert\" data-id=\"{}\" d=\"M{},{}V{}\"/>",
            esc_text(&m.text(t, "id")?)?,
            num(vx),
            num(plot_top - grid_lead),
            num(plot_bottom)
        );
        vert_labels += &label_svg(
            &label,
            cx,
            vert_top + line as f64 * vert_line + vert_line / 2.,
            &format!(
                " class=\"pele-vert-label\" data-id=\"{}\"",
                esc_text(&m.text(t, "id")?)?
            ),
        )
    }
    let width = (extent.js_max(pad + title.width) + pad).ceil();
    let height = (plot_bottom
        + tick_h
        + if !vert_ends.is_empty() {
            4. + vert_ends.len() as f64 * vert_line
        } else {
            0.
        }
        + pad)
        .ceil();
    let (mut bands, mut section_out) = (String::new(), String::new());
    let mut row = 0;
    while row < row_count {
        m.work(1)?;
        let section = &row_section[row];
        let mut end = row + 1;
        while end < row_count && row_section[end] == *section {
            end += 1
        }
        let index = categories.iter().position(|s| s == section).unwrap() % section_styles;
        let y = plot_top + row as f64 * row_h;
        let h = (end - row) as f64 * row_h;
        if categories.len() > 1 || !section.is_empty() {
            bands+=&format!("<rect class=\"pele-section pele-section-{index}\" x=\"0\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\"/>",num(y),num(width),num(h),if index%2==0{"var(--_s)"}else{"none"})
        }
        let label = &section_labels[section];
        section_out += &label_svg_with_resources(
            label,
            pad + 8. + label.width / 2.,
            y + h / 2.,
            &format!(
                " class=\"pele-section-label\" data-id=\"{}\"",
                esc_text(section)?
            ),
            cal.resources,
        )?;
        row = end
    }
    let mut excluded = String::new();
    if m.has_exclusions()? && cal.add(start, 6., "y") > stop {
        let mut from = None;
        let mut to = None;
        let mut day = start;
        loop {
            m.work(1)?;
            let invalid = day <= stop && m.invalid(day, cal)?;
            if invalid {
                if from.is_none() {
                    from = Some(day)
                }
                to = Some(day)
            } else if let (Some(a), Some(b)) = (from.take(), to.take()) {
                let a = cal.today(a);
                cal.check()?;
                let end_civil = cal.fields(b).map(|f| f.civil);
                cal.check()?;
                let Some(mut end_civil) = end_civil else {
                    continue;
                };
                end_civil.hour = 23;
                end_civil.minute = 59;
                end_civil.second = 59;
                end_civil.millisecond = 999;
                let b = cal.local(end_civil);
                cal.check()?;
                let ex0 = left.js_max(x(a));
                let ex1 = right.js_min(x(b));
                if ex1 > ex0 {
                    excluded += &format!(
                        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/>",
                        num(ex0),
                        num(plot_top),
                        num(ex1 - ex0),
                        num(plot_h)
                    )
                }
            }
            if day > stop {
                break;
            }
            day = cal.add_days(day, 1.);
            cal.check()?;
            if !day.is_finite() {
                break;
            }
        }
    }
    let mut links = vec![];
    let mut tasks_out = String::new();
    for bar in bars {
        let t = &bar.task;
        m.work(1)?;
        let cy = plot_top + bar.row as f64 * row_h + row_h / 2.;
        let state = if m.flag(t, "active")? {
            "active"
        } else if m.flag(t, "done")? {
            "done"
        } else {
            ""
        };
        let mut classes = "pele-task".to_string();
        if !state.is_empty() {
            classes += &format!(" pele-{state}")
        }
        if m.flag(t, "crit")? {
            classes += " pele-crit"
        }
        if m.flag(t, "milestone")? {
            classes += " pele-milestone"
        }
        if m.clickable(t)? {
            classes += " pele-clickable"
        }
        let mut shape = String::new();
        let mut text_fill = "";
        if bar.x0.is_nan() {
            classes += " pele-unscheduled";
            text_fill = " fill=\"var(--_m)\""
        } else {
            let color = if m.flag(t, "crit")? {
                "var(--pele-critical,#e45756)"
            } else {
                "var(--_c)"
            };
            let paint = if state == "done" {
                "fill=\"var(--_a)\"".into()
            } else if state == "active" {
                format!("fill=\"{color}\" fill-opacity=\"0.25\"")
            } else {
                format!("fill=\"{color}\"")
            };
            let stroke = format!(
                "stroke=\"{}\"",
                if state == "done" && !m.flag(t, "crit")? {
                    "var(--_b)"
                } else {
                    color
                }
            );
            if m.flag(t, "milestone")? {
                let cx = (bar.x0 + bar.x1) / 2.;
                let r = bar_h / 2.;
                shape=format!("<path d=\"M{},{}L{},{}L{},{}L{},{}Z\" {paint} {stroke} stroke-linejoin=\"round\"/>",num(cx),num(cy-r),num(cx+r),num(cy),num(cx),num(cy+r),num(cx-r),num(cy))
            } else {
                shape=format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"var(--pele-radius,4px)\" {paint} {stroke}/>",num(bar.x0),num(cy-bar_h/2.),num(bar.x1-bar.x0),num(bar_h))
            }
            if bar.inside {
                if state.is_empty() {
                    text_fill = " fill=\"var(--_bg)\""
                }
            } else if state == "done" {
                text_fill = " fill=\"var(--_m)\""
            }
        }
        let mut body = shape
            + &label_svg(
                &bar.label,
                bar.label_x,
                cy,
                &format!(" class=\"pele-label\"{text_fill}"),
            );
        if let Some((id, url)) = m.link(t)? {
            m.keep_text(
                id.encode_utf16()
                    .count()
                    .saturating_add(url.encode_utf16().count()),
            )?;
            body = linked(&body, &url, &id, &mut links, o, "")
        }
        tasks_out += &format!(
            "<g class=\"{classes}\" data-id=\"{}\">{body}</g>",
            esc_text(&m.text(t, "id")?)?
        )
    }
    let mut today = String::new();
    if m.root_text("todayMarker")?.as_deref() != Some("off")
        && now >= start
        && now <= stop
        && stop > start
    {
        let tx = x(now);
        let top = plot_top - grid_lead;
        let marker = m.root_text("todayMarker")?.unwrap_or_default();
        let style = if marker.is_empty() {
            String::new()
        } else {
            resolve_style(
                &m.root_text("todayMarker")?
                    .unwrap_or_default()
                    .split(',')
                    .map(str::to_string)
                    .collect::<Vec<_>>(),
            )
            .line
        };
        today=format!("<g class=\"pele-today\"><path d=\"M{},{}V{}\" fill=\"none\" stroke=\"var(--_c)\" stroke-width=\"2\"{style}/>{}</g>",num(tx),num(top),num(plot_bottom),if style.is_empty(){format!("<path d=\"M{},{}H{}L{},{}Z\" fill=\"var(--_c)\"/>",num(tx-4.),num(top-5.),num(tx+4.),num(tx),num(top+1.))}else{String::new()})
    }
    let axis_line = format!(
        "M{},{}H{}{}",
        num(left),
        num(plot_bottom),
        num(right),
        if top_axis {
            format!(
                "M{},{}H{}",
                num(left),
                num(plot_top - grid_lead),
                num(right)
            )
        } else {
            String::new()
        }
    );
    let content=format!("{}{}{}<g class=\"pele-grid\" fill=\"none\"><path d=\"{grid}\" stroke=\"var(--_b)\" stroke-dasharray=\"1 3\"/><path d=\"{axis_line}\" stroke=\"var(--_b)\"/></g>{}{today}<g class=\"pele-tasks\" font-size=\"{}\">{tasks_out}</g><g class=\"pele-section-labels\" font-size=\"{}\">{section_out}</g><g class=\"pele-axis\" font-size=\"{}\" fill=\"var(--_m)\">{axis_bottom}{axis_top}</g>{}",label_svg(&title,width/2.,pad+title.height/2.," class=\"pele-title\" font-weight=\"var(--_tw)\""),if bands.is_empty(){String::new()}else{format!("<g class=\"pele-sections\">{bands}</g>")},if excluded.is_empty(){String::new()}else{format!("<g class=\"pele-excluded\" fill=\"var(--_a)\">{excluded}</g>")},if vert_lines.is_empty(){String::new()}else{format!("<g class=\"pele-verts\" fill=\"none\" stroke=\"var(--_l)\" stroke-width=\"1.5\" stroke-dasharray=\"4 3\">{vert_lines}</g>")},num(label_size),num(section_size),num(tick_size),if vert_labels.is_empty(){String::new()}else{format!("<g class=\"pele-vert-labels\" font-size=\"{}\">{vert_labels}</g>",num(label_size))});
    Ok(Rendered {
        svg: document(
            "gantt",
            width,
            height,
            size,
            o,
            m.root_text("accTitle")?.as_deref(),
            m.root_text("accDescr")?.as_deref(),
            &content,
        )?,
        width,
        height,
        links,
    })
}

fn compiled_regex(pattern: &str) -> Result<Regex, regex::Error> {
    Regex::new(&pattern.replace(r"\s", crate::compat::text::SPACE))
}

fn sort_tasks<T, E>(
    items: &mut [T],
    mut compare: impl FnMut(&T, &T) -> Result<std::cmp::Ordering, E>,
) -> Result<(), E> {
    let mut error = None;
    items.sort_by(|a, b| {
        if error.is_some() {
            return std::cmp::Ordering::Equal;
        }
        match compare(a, b) {
            Ok(v) => v,
            Err(e) => {
                error = Some(e);
                std::cmp::Ordering::Equal
            }
        }
    });
    match error {
        Some(e) => Err(e),
        None => Ok(()),
    }
}
