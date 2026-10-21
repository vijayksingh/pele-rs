//! Pele timeline/render.ts and common/{fit,fit-width}.ts, 458322a5 (MIT, kepano).
use super::TimelineModel;
use crate::compat::{
    number_math::{round, NumberMath},
    source::space,
    svg::{document, esc_text, label_svg, num, series_color},
    text::{layout_label, measure, Label, SPACE},
    Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
use regex::Regex;
use std::sync::OnceLock;
const PAD_X: f64 = 12.;
const PAD_Y: f64 = 8.;
const GAP: f64 = 20.;
const AXIS: f64 = 20.;
const SPINE: f64 = 14.;
const EVENT_GAP: f64 = 8.;
const SECTION_GAP: f64 = 8.;
const TAIL: f64 = 12.;
const ARROW: f64 = 8.;
const ON_COLOR: &str = " fill=\"var(--_bg)\"";
const LINE: &str = " fill=\"none\" stroke=\"var(--_b)\"";
struct Group {
    section: Option<usize>,
    periods: Vec<usize>,
}
fn plain(
    raw: Option<&str>,
    r: &mut dyn RenderResources,
    o: &RenderSettings,
    size: f64,
    max: f64,
    base: u8,
) -> Result<Label, PeleError> {
    let Some(raw) = raw else {
        return layout_label(None, false, r, o, size, max, base);
    };
    static BR: OnceLock<Regex> = OnceLock::new();
    let br = BR.get_or_init(|| Regex::new(&format!("(?i)^br{SPACE}*/?>")).unwrap());
    let mut text = String::new();
    for (i, c) in raw.char_indices() {
        if c == '<' && !br.is_match(&raw[i + 1..]) {
            text.push_str("&lt;")
        } else {
            text.push(c)
        }
    }
    let text = text.replace(":fa-", "&colon;fa-");
    layout_label(Some(&text), false, r, o, size, max, base)
}
fn fit(
    raw: Option<&str>,
    r: &mut dyn RenderResources,
    o: &RenderSettings,
    size: f64,
    max: f64,
    base: u8,
) -> Result<Label, PeleError> {
    let label = plain(raw, r, o, size, max, base)?;
    let Some(raw) = raw else {
        return Ok(label);
    };
    if label.width <= max {
        return Ok(label);
    }
    static UNIT: OnceLock<Regex> = OnceLock::new();
    let unit = UNIT.get_or_init(|| {
        Regex::new(&format!(
            r"(?i:<br{SPACE}*/?>)|ﬂ°°?(?-u:\w)+¶ß|&#?(?-u:\w)+;|[\s\S]"
        ))
        .unwrap()
    });
    let mut out = String::new();
    let mut width = 0.;
    for m in unit.find_iter(raw) {
        let part = m.as_str();
        let len = part.encode_utf16().count();
        if part.trim_matches(space).is_empty() || (len > 2 && part.starts_with('<')) {
            width = 0.
        } else {
            let w = measure(r, o, if len > 2 { "x" } else { part }, size, base)?;
            if width > 0. && width + w > max {
                out.push('\n');
                width = 0.
            }
            width += w
        }
        out.push_str(part)
    }
    plain(Some(&out), r, o, size, max, base)
}
fn rect(x: f64, y: f64, w: f64, h: f64, attrs: &str) -> String {
    format!(
        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"var(--pele-radius,4px)\"{attrs}/>",
        num(x),
        num(y),
        num(w),
        num(h)
    )
}
fn marker(x: f64, y: f64, dx: f64, dy: f64) -> String {
    let nx = -dy;
    let ny = dx;
    let bx = x - dx * ARROW;
    let by = y - dy * ARROW;
    let half = ARROW * 0.42;
    format!("<path class=\"pele-marker\" d=\"M{},{}L{},{}L{},{}Z\" fill=\"var(--_l)\" stroke=\"none\"/>",num(x),num(y),num(bx+nx*half),num(by+ny*half),num(bx-nx*half),num(by-ny*half))
}
/// Array-map phases preserve their captured receiver and length; individual
/// values are read only when the shared renderer reaches that element.
pub(super) trait PhaseInput {
    type Error: From<PeleError>;
    type Periods;
    type Events;
    fn downward(&mut self) -> Result<bool, Self::Error>;
    fn start(&mut self) -> Result<(Self::Periods, usize, Vec<Option<usize>>), Self::Error>;
    fn period_count(&mut self, p: &Self::Periods) -> Result<usize, Self::Error>;
    fn period_text(&mut self, p: &Self::Periods, i: usize) -> Result<Option<String>, Self::Error>;
    fn events(&mut self, p: &Self::Periods, i: usize)
        -> Result<(Self::Events, usize), Self::Error>;
    fn event(&mut self, e: &Self::Events, i: usize) -> Result<Option<String>, Self::Error>;
    fn section(&mut self, i: usize) -> Result<Option<String>, Self::Error>;
    fn title(&mut self) -> Result<Option<String>, Self::Error>;
    fn accessibility(&mut self, description: bool) -> Result<Option<String>, Self::Error>;
}
struct Typed<'a>(&'a TimelineModel);
impl PhaseInput for Typed<'_> {
    type Error = PeleError;
    type Periods = ();
    type Events = usize;
    fn downward(&mut self) -> Result<bool, PeleError> {
        Ok(self.0.direction == "TD")
    }
    fn start(&mut self) -> Result<((), usize, Vec<Option<usize>>), PeleError> {
        Ok((
            (),
            self.0.sections.len(),
            self.0
                .periods
                .iter()
                .map(|p| usize::try_from(p.section_index).ok())
                .collect(),
        ))
    }
    fn period_count(&mut self, _: &()) -> Result<usize, PeleError> {
        Ok(self.0.periods.len())
    }
    fn period_text(&mut self, _: &(), i: usize) -> Result<Option<String>, PeleError> {
        Ok(Some(self.0.periods[i].text.clone()))
    }
    fn events(&mut self, _: &(), i: usize) -> Result<(usize, usize), PeleError> {
        Ok((i, self.0.periods[i].events.len()))
    }
    fn event(&mut self, e: &usize, i: usize) -> Result<Option<String>, PeleError> {
        Ok(Some(self.0.periods[*e].events[i].clone()))
    }
    fn section(&mut self, i: usize) -> Result<Option<String>, PeleError> {
        Ok(self.0.sections.get(i).cloned())
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
fn section_label<P: PhaseInput>(
    m: &mut P,
    g: &Group,
    w: f64,
    r: &mut dyn RenderResources,
    o: &RenderSettings,
    size: f64,
) -> Result<Label, P::Error> {
    let text = match g.section {
        Some(i) => m.section(i)?,
        None => None,
    };
    Ok(fit(text.as_deref(), r, o, size, w - 2. * PAD_X, 1)?)
}
fn open<P: PhaseInput>(m: &mut P, g: &Group) -> Result<String, P::Error> {
    Ok(if let Some(i) = g.section {
        let value = m.section(i)?.ok_or_else(|| {
            PeleError::new(
                "Timeline section text is undefined",
                crate::compat::ErrorCode::Semantic,
            )
        })?;
        format!(
            "<g class=\"pele-cluster pele-section\" data-id=\"{}\">",
            esc_text(value.trim_matches(space))?
        )
    } else {
        String::new()
    })
}
fn open_period<P: PhaseInput>(m: &mut P, p: &P::Periods, i: usize) -> Result<String, P::Error> {
    let value = m.period_text(p, i)?.ok_or_else(|| {
        PeleError::new(
            "Timeline period text is undefined",
            crate::compat::ErrorCode::Semantic,
        )
    })?;
    Ok(format!(
        "<g class=\"pele-node pele-period\" data-id=\"{}\">",
        esc_text(value.trim_matches(space))?
    ))
}
pub(super) fn render(
    m: &TimelineModel,
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
    if m.downward()? {
        return draw(m, c, o, r, true);
    }
    let across = draw(m, c, o, r, false)?;
    let Some(available) = o.max_width.filter(|w| *w > 0.) else {
        return Ok(across);
    };
    if o.auto_direction == Some(false)
        || across.width <= available
        || available >= o.direction_breakpoint.filter(|v| *v > 0.).unwrap_or(640.)
    {
        return Ok(across);
    }
    let down = draw(m, c, o, r, true)?;
    Ok(if down.width < across.width {
        down
    } else {
        across
    })
}
fn draw<P: PhaseInput>(
    m: &mut P,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    vertical: bool,
) -> Result<Rendered, P::Error> {
    let size = o.font_size.unwrap_or(16.);
    let small = round(size * 0.875);
    let pad = o.padding.unwrap_or(8.);
    let multicolor = c
        .get("timeline")
        .and_then(Value::as_object)
        .and_then(|c| c.get("disableMulticolor"))
        != Some(&Value::Bool(true));
    let (periods, section_count, memberships) = m.start()?;
    let has_sections = section_count != 0;
    let mut groups: Vec<_> = (0..section_count)
        .map(|s| Group {
            section: Some(s),
            periods: Vec::new(),
        })
        .collect();
    let mut leading = Group {
        section: None,
        periods: Vec::new(),
    };
    for (i, section) in memberships.into_iter().enumerate() {
        if let Some(g) = section.and_then(|s| groups.get_mut(s)) {
            g.periods.push(i)
        } else {
            leading.periods.push(i)
        }
    }
    if !leading.periods.is_empty() {
        groups.insert(0, leading)
    }
    let room = if vertical {
        o.max_width
            .filter(|v| *v > 0.)
            .map_or(f64::INFINITY, |v| v - 2. * pad)
    } else {
        f64::INFINITY
    };
    let taken = AXIS + SPINE * 2. + 4. * PAD_X;
    let period_text = 152f64.js_min(80f64.js_max(round((room - taken) * 0.38)));
    let event_text = 264f64.js_min(110f64.js_max(room - taken - period_text));
    let mut text_w = period_text;
    let mut period_labels = Vec::new();
    let mut event_labels = Vec::new();
    for i in 0..m.period_count(&periods)? {
        let text = m.period_text(&periods, i)?;
        period_labels.push(plain(text.as_deref(), r, o, size, text_w, 0)?);
    }
    for i in 0..m.period_count(&periods)? {
        let (events, count) = m.events(&periods, i)?;
        let mut labels = Vec::with_capacity(count);
        for j in 0..count {
            let text = m.event(&events, j)?;
            labels.push(plain(
                text.as_deref(),
                r,
                o,
                small,
                if vertical { event_text } else { text_w },
                0,
            )?);
        }
        event_labels.push(labels);
    }
    let mut widest = period_labels.iter().map(|l| l.width).fold(0., f64::js_max);
    if !vertical {
        widest = event_labels
            .iter()
            .flatten()
            .map(|l| l.width)
            .fold(widest, f64::js_max)
    }
    let overflow = widest > text_w
        || (vertical && event_labels.iter().flatten().any(|l| l.width > event_text));
    if overflow {
        text_w = widest
            .ceil()
            .js_max(period_text)
            .js_min(if room == f64::INFINITY {
                216.
            } else {
                period_text
            });
        period_labels.clear();
        event_labels.clear();
        for i in 0..m.period_count(&periods)? {
            let text = m.period_text(&periods, i)?;
            period_labels.push(fit(text.as_deref(), r, o, size, text_w, 0)?);
        }
        for i in 0..m.period_count(&periods)? {
            let (events, count) = m.events(&periods, i)?;
            let mut labels = Vec::with_capacity(count);
            for j in 0..count {
                let text = m.event(&events, j)?;
                labels.push(fit(
                    text.as_deref(),
                    r,
                    o,
                    small,
                    if vertical { event_text } else { text_w },
                    0,
                )?);
            }
            event_labels.push(labels);
        }
    }
    if groups
        .iter()
        .flat_map(|g| &g.periods)
        .any(|&i| i >= period_labels.len() || i >= event_labels.len())
    {
        return Err(PeleError::new(
            "Timeline callback removed a captured period",
            crate::compat::ErrorCode::Semantic,
        )
        .into());
    }
    let line_height = round(size * 1.5);
    let col_w = text_w + 2. * PAD_X;
    let event_w = if vertical { event_text } else { text_w } + 2. * PAD_X;
    let chip_height = |l: &Label| l.height.js_max(line_height) + 2. * PAD_Y;
    let event_height = |l: &Label| l.height.js_max(round(small * 1.5)) + 2. * PAD_Y;
    let color_of = |g: &Group, i: usize| {
        if let Some(section) = g.section {
            series_color(section)
        } else if has_sections {
            "var(--_m)".into()
        } else {
            series_color(if multicolor { i } else { 0 })
        }
    };
    let has_events = event_labels.iter().any(|l| !l.is_empty());
    let slots: usize = groups.iter().map(|g| g.periods.len().max(1)).sum();
    let pitch = col_w + GAP;
    let drawn = m.period_count(&periods)? != 0;
    let body_w = if vertical {
        col_w
            + if has_events {
                AXIS + SPINE * 2. + event_w
            } else if drawn {
                AXIS + TAIL
            } else {
                0.
            }
    } else if slots > 0 {
        slots as f64 * pitch - GAP + if drawn { TAIL + ARROW } else { 0. }
    } else {
        0.
    };
    let title_text = m.title()?;
    let title = fit(title_text.as_deref(), r, o, size, body_w.js_max(320.), 1)?;
    let title_h = if title.height > 0. {
        title.height + 16.
    } else {
        0.
    };
    let width = body_w.js_max(title.width);
    let left = pad + (width - body_w) / 2.;
    let top = pad + title_h;
    let header = |g: &Group, l: &Label, x: f64, y: f64, w: f64, h: f64| {
        rect(
            x,
            y,
            w,
            h,
            &format!(" fill=\"{}\"", series_color(g.section.unwrap())),
        ) + &label_svg(
            l,
            x + w / 2.,
            y + h / 2.,
            &format!(" font-weight=\"var(--_hw)\"{ON_COLOR}"),
        )
    };
    let period = |i: usize, color: &str, x: f64, y: f64, h: f64| {
        rect(x, y, col_w, h, &format!(" fill=\"{color}\""))
            + &label_svg(&period_labels[i], x + col_w / 2., y + h / 2., ON_COLOR)
    };
    let event = |l: &Label, color: &str, x: f64, y: f64, h: f64| {
        format!(
            "<g class=\"pele-node pele-event\">{}{}</g>",
            rect(
                x,
                y,
                event_w,
                h,
                &format!(" fill=\"{color}\" fill-opacity=\"0.16\"")
            ),
            label_svg(
                l,
                x + event_w / 2.,
                y + h / 2.,
                &format!(" font-size=\"{}\"", crate::compat::svg::js_num(small))
            )
        )
    };
    let dot = |x: f64, y: f64, color: &str| {
        format!("<circle class=\"pele-dot\" cx=\"{}\" cy=\"{}\" r=\"4\" fill=\"{color}\" stroke=\"var(--_bg)\" stroke-width=\"2\"/>",num(x),num(y))
    };
    let mut body = String::new();
    let height;
    if vertical {
        let axis_x = left + col_w + AXIS;
        let spine_x = axis_x + SPINE;
        let events_x = spine_x + SPINE;
        let mut y = top;
        for g in &groups {
            body.push_str(&open(m, g)?);
            if g.section.is_some() {
                let label = section_label(m, g, body_w, r, o, size)?;
                let h = chip_height(&label);
                body.push_str(&header(g, &label, left, y, body_w, h));
                y += h + SECTION_GAP + 4.
            }
            for &i in &g.periods {
                let color = color_of(g, i);
                let h = chip_height(&period_labels[i]);
                let cy = y + h / 2.;
                let mut d = format!("M{},{}H{}", num(left + col_w), num(cy), num(axis_x));
                let mut cards = String::new();
                let mut ey = y;
                let mut last = cy;
                for label in &event_labels[i] {
                    let eh = event_height(label);
                    last = ey + eh / 2.;
                    d.push_str(&format!(
                        "M{},{}H{}",
                        num(spine_x),
                        num(last),
                        num(events_x)
                    ));
                    cards.push_str(&event(label, &color, events_x, ey, eh));
                    ey += eh + EVENT_GAP
                }
                if !cards.is_empty() {
                    d.push_str(&format!(
                        "M{},{}H{}V{}",
                        num(axis_x),
                        num(cy),
                        num(spine_x),
                        num(last)
                    ))
                }
                body.push_str(&format!(
                    "{}{}<path class=\"pele-edge\" d=\"{d}\"{LINE}/>{}{cards}</g>",
                    open_period(m, &periods, i)?,
                    period(i, &color, left, y, h),
                    dot(axis_x, cy, &color)
                ));
                y = (y + h).js_max(ey - EVENT_GAP) + GAP
            }
            if g.periods.is_empty() {
                y += GAP - SECTION_GAP - 4.
            }
            if g.section.is_some() {
                body.push_str("</g>")
            }
        }
        if !groups.is_empty() {
            y -= GAP
        }
        if drawn {
            let tip = y + TAIL + ARROW;
            body=format!("<path class=\"pele-axis\" d=\"M{},{}V{}\" fill=\"none\" stroke=\"var(--_l)\"/>{}{}",num(axis_x),num(top),num(tip-ARROW+1.),marker(axis_x,tip,0.,1.),body);
            y = tip
        }
        height = y + pad;
    } else {
        let mut header_h = 0f64;
        let mut section_labels = Vec::new();
        for g in &groups {
            if g.section.is_none() {
                section_labels.push(None)
            } else {
                let label = section_label(
                    m,
                    g,
                    g.periods.len().max(1) as f64 * pitch - GAP,
                    r,
                    o,
                    size,
                )?;
                header_h = header_h.js_max(chip_height(&label));
                section_labels.push(Some(label))
            }
        }
        let chip_h = period_labels.iter().map(chip_height).fold(0., f64::js_max);
        let periods_y = top
            + if has_sections {
                header_h + SECTION_GAP
            } else {
                0.
            };
        let axis_y = periods_y + chip_h + if chip_h > 0. { AXIS } else { 0. };
        let events_y = axis_y + AXIS;
        let mut bottom = if drawn { axis_y + TAIL } else { top + header_h };
        let mut slot = 0;
        for (g_index, g) in groups.iter().enumerate() {
            let span = g.periods.len().max(1);
            let gx = left + slot as f64 * pitch;
            body.push_str(&open(m, g)?);
            if let Some(label) = &section_labels[g_index] {
                body.push_str(&header(
                    g,
                    label,
                    gx,
                    top,
                    span as f64 * pitch - GAP,
                    header_h,
                ))
            }
            for (k, &i) in g.periods.iter().enumerate() {
                let color = color_of(g, i);
                let x = gx + k as f64 * pitch;
                let cx = x + col_w / 2.;
                let mut d = format!(
                    "M{},{}V{}",
                    num(cx),
                    num(periods_y + chip_h),
                    num(if !event_labels[i].is_empty() {
                        events_y
                    } else {
                        axis_y
                    })
                );
                let mut cards = String::new();
                let mut ey = events_y;
                for label in &event_labels[i] {
                    let eh = event_height(label);
                    if ey > events_y {
                        d.push_str(&format!("M{},{}V{}", num(cx), num(ey - EVENT_GAP), num(ey)))
                    }
                    cards.push_str(&event(label, &color, x, ey, eh));
                    ey += eh + EVENT_GAP
                }
                bottom = bottom.js_max(ey - EVENT_GAP);
                body.push_str(&format!(
                    "{}{}<path class=\"pele-edge\" d=\"{d}\"{LINE}/>{}{cards}</g>",
                    open_period(m, &periods, i)?,
                    period(i, &color, x, periods_y, chip_h),
                    dot(cx, axis_y, &color)
                ))
            }
            if g.section.is_some() {
                body.push_str("</g>")
            }
            slot += span
        }
        if drawn {
            let tip = left + body_w;
            body=format!("<path class=\"pele-axis\" d=\"M{},{}H{}\" fill=\"none\" stroke=\"var(--_l)\"/>{}{}",num(left),num(axis_y),num(tip-ARROW+1.),marker(tip,axis_y,1.,0.),body)
        }
        height = bottom + pad
    }
    let total_w = (width + 2. * pad).ceil();
    let total_h = height.js_max(pad * 2.).ceil();
    let content = label_svg(
        &title,
        total_w / 2.,
        pad + title.height / 2.,
        " class=\"pele-title\" font-weight=\"var(--_tw)\"",
    ) + &body;
    let acc_title = m.accessibility(false)?;
    let acc_descr = m.accessibility(true)?;
    Ok(Rendered {
        svg: document(
            "timeline",
            total_w,
            total_h,
            size,
            o,
            acc_title.as_deref(),
            acc_descr.as_deref(),
            &content,
        )?,
        width: total_w,
        height: total_h,
        links: Vec::new(),
    })
}
