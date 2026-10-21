//! Direct Pele sequence/render.ts layout, lifecycle and drawing, 458322a5 (MIT, kepano).
use super::super::common::literal_label::break_label;
use super::{
    actors::{actor_kind, actor_size, draw_actor, is_figure, ActorSize},
    kinds::*,
    model::SequenceModel,
    phase::{PhaseInput, Typed},
};
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    svg::{
        self,
        edges::{
            marker,
            theme::{class_names, resolve_style},
            ARROW,
        },
        esc, esc_text, js_num, label_svg_with_resources, num,
    },
    text::{layout_label, Label},
    Object, PeleError, RenderResources, RenderSettings, Rendered, TextRequest, Value,
};
use std::collections::{HashMap, HashSet};
const SELF_W: f64 = 28.;
const SELF_H: f64 = 22.;
const ACT_W: f64 = 10.;
const ACT_STEP: f64 = 4.;
const ACT_MIN: f64 = 14.;
const NOTE_GAP: f64 = 10.;
const TAB_H: f64 = 20.;
const FRAME_TEXT: f64 = 240.;
const BADGE_H: f64 = 16.;
const CENTRAL_R: f64 = 4.;
const DASH: &str = " stroke-dasharray=\"4 4\"";
const RADIUS: &str = "var(--pele-radius,4px)";
#[derive(Clone, Copy, PartialEq)]
enum Head {
    None,
    Arrow,
    Cross,
    Open,
    HalfSolid,
    HalfStick,
}
#[derive(Clone, Copy)]
struct LineStyle {
    dotted: bool,
    head: Head,
    at_start: bool,
    at_end: bool,
    bottom: bool,
}
fn style(t: u8) -> Option<LineStyle> {
    let (dotted, head, ends, bottom) = match t {
        SOLID => (false, Head::Arrow, 1, false),
        DOTTED => (true, Head::Arrow, 1, false),
        SOLID_CROSS => (false, Head::Cross, 1, false),
        DOTTED_CROSS => (true, Head::Cross, 1, false),
        SOLID_OPEN => (false, Head::None, 1, false),
        DOTTED_OPEN => (true, Head::None, 1, false),
        SOLID_POINT => (false, Head::Open, 1, false),
        DOTTED_POINT => (true, Head::Open, 1, false),
        BIDIRECTIONAL_SOLID => (false, Head::Arrow, 2, false),
        BIDIRECTIONAL_DOTTED => (true, Head::Arrow, 2, false),
        41..=48 | 51..=58 => {
            let dotted = t >= 51;
            let n = if dotted { t - 10 } else { t };
            (
                dotted,
                if matches!(n, 43 | 44 | 47 | 48) {
                    Head::HalfStick
                } else {
                    Head::HalfSolid
                },
                if n >= 45 { 0 } else { 1 },
                n % 2 == 0,
            )
        }
        _ => return None,
    };
    Some(LineStyle {
        dotted,
        head,
        at_start: ends != 1,
        at_end: ends != 0,
        bottom,
    })
}
fn line(t: Option<u8>) -> bool {
    t.is_some_and(|t| style(t).is_some())
}
fn frame_kind(t: Option<u8>) -> Option<&'static str> {
    Some(match t? {
        LOOP_START => "loop",
        ALT_START => "alt",
        OPT_START => "opt",
        PAR_START | PAR_OVER_START => "par",
        CRITICAL_START => "critical",
        BREAK_START => "break",
        RECT_START => "rect",
        _ => return None,
    })
}
fn frame_end(t: Option<u8>) -> bool {
    matches!(
        t,
        Some(LOOP_END | ALT_END | OPT_END | PAR_END | CRITICAL_END | BREAK_END | RECT_END)
    )
}
fn divider(t: Option<u8>) -> bool {
    matches!(t, Some(ALT_ELSE | PAR_AND | CRITICAL_OPTION))
}
fn option(c: Option<&Object>, key: &str, fallback: f64) -> f64 {
    match c.and_then(|c| c.get(key)) {
        Some(Value::Number(n)) if *n >= 0. && *n <= 2000. => *n,
        _ => fallback,
    }
}
fn truth(c: Option<&Object>, key: &str) -> bool {
    c.and_then(|c| c.get(key)) == Some(&Value::Bool(true))
}
fn half(x: f64, y: f64, dir: f64, up: f64, stick: bool) -> String {
    let bx = num(x - dir * ARROW);
    if stick {
        format!(
            "<path class=\"pele-marker\" d=\"M{},{}L{bx},{}\"/>",
            num(x),
            num(y),
            num(y + up * 5.)
        )
    } else {
        format!("<path class=\"pele-marker\" d=\"M{},{}L{bx},{}V{}Z\" fill=\"var(--_l)\" stroke=\"none\"/>",num(x),num(y),num(y+up*4.5),num(y))
    }
}
fn head(kind: Head, x: f64, y: f64, dir: f64, up: f64) -> String {
    match kind {
        Head::Arrow => marker("arrow_point", x, y, dir, 0., "var(--_l)"),
        Head::Cross => marker("arrow_cross", x, y, dir, 0., "var(--_l)"),
        Head::Open => format!(
            "<path class=\"pele-marker\" d=\"M{},{}L{},{}L{},{}\" stroke-linejoin=\"round\"/>",
            num(x - dir * ARROW),
            num(y - 4.5),
            num(x),
            num(y),
            num(x - dir * ARROW),
            num(y + 4.5)
        ),
        Head::HalfSolid => half(x, y, dir, up, false),
        Head::HalfStick => half(x, y, dir, up, true),
        _ => String::new(),
    }
}
fn trim(h: Head) -> f64 {
    match h {
        Head::Arrow => ARROW - 1.,
        Head::Cross => 4.,
        _ => 0.,
    }
}
struct Numbering {
    index: Vec<f64>,
    shown: Vec<bool>,
}
fn number_messages<P: PhaseInput>(
    input: &mut P,
    messages: &P::Value,
    count: usize,
    always: bool,
) -> Result<Numbering, P::Error> {
    let (mut current, mut step, mut visible) = (1., 1., false);
    let mut index = vec![f64::NAN; count];
    let mut shown = vec![false; count];
    for i in 0..count {
        input.step()?;
        let m = input.index(messages, i)?;
        let kind = input.byte(&m, "type")?;
        if kind == Some(AUTONUMBER) {
            let spec = input.get(&m, "message")?;
            let n = input.get(&spec, "start")?;
            if let Some(n) = input.number(&n).filter(|n| *n != 0. && !n.is_nan()) {
                current = n;
            }
            let n = input.get(&spec, "step")?;
            if let Some(n) = input.number(&n).filter(|n| *n != 0. && !n.is_nan()) {
                step = n;
            }
            visible = input.flag(&spec, "visible")?;
        } else if line(kind) {
            index[i] = current;
            shown[i] = visible || always;
            current = round((current + step) * 100.) / 100.;
        }
    }
    Ok(Numbering { index, shown })
}
struct Column<T> {
    id: String,
    actor: T,
    kind: String,
    label: Label,
    size: ActorSize,
    x: f64,
    create_at: isize,
    destroy_at: isize,
    top: f64,
    life_start: f64,
    life_end: f64,
    activations: Vec<f64>,
}
#[derive(Clone, Copy, PartialEq)]
enum V {
    Line,
    SelfLoop,
    Note,
}
struct View {
    kind: V,
    a: usize,
    b: usize,
    label: Label,
    x: f64,
    w: f64,
}
struct Frame {
    kind: &'static str,
    x0: f64,
    x1: f64,
    top: f64,
    title: Label,
    tab_w: f64,
    fill: String,
    dividers: Vec<usize>,
    divider_y: Vec<f64>,
    divider_labels: Vec<Label>,
}
struct BoxRun<T> {
    value: T,
    identity: u64,
    first: usize,
    last: usize,
    title: Label,
    grow: f64,
}
struct TextContext<'a> {
    resources: &'a mut dyn RenderResources,
    options: &'a RenderSettings,
    tight: bool,
    wrap_w: f64,
}
impl TextContext<'_> {
    fn width(&mut self, s: &str, size: f64, style: u8) -> Result<f64, PeleError> {
        self.resources.text_width(TextRequest {
            text: s,
            size,
            style,
            font_family: self.options.font_family.as_deref(),
            font_family_mono: self.options.font_family_mono.as_deref(),
        })
    }
    fn text(
        &mut self,
        s: Option<&str>,
        size: f64,
        wrap: bool,
        width: Option<f64>,
        base: u8,
    ) -> Result<Label, PeleError> {
        let width = width.unwrap_or(self.wrap_w);
        let label = layout_label(
            s,
            false,
            self.resources,
            self.options,
            size,
            if self.tight || wrap {
                width
            } else {
                f64::INFINITY
            },
            base,
        )?;
        if self.tight {
            break_label(label, self.resources, self.options, 120_f64.js_max(width))
        } else {
            Ok(label)
        }
    }
    fn svg(&mut self, l: &Label, x: f64, y: f64, a: &str) -> Result<String, PeleError> {
        label_svg_with_resources(l, x, y, a, self.resources)
    }
}
fn need(needs: &mut Vec<(usize, f64)>, len: usize, lo: isize, hi: isize, width: f64) {
    if lo < 0 || hi >= len as isize {
        return;
    }
    let key = lo as usize * len + hi as usize;
    if let Some((_, v)) = needs.iter_mut().find(|(k, _)| *k == key) {
        if *v < width {
            *v = width
        }
    } else if width > 0. {
        needs.push((key, width))
    }
}
fn badge_width(
    i: usize,
    n: &Numbering,
    tiny: f64,
    ctx: &mut TextContext<'_>,
) -> Result<f64, PeleError> {
    if n.shown[i] {
        Ok(BADGE_H.js_max(ctx.width(&js_num(n.index[i]), tiny, 0)? + 8.))
    } else {
        Ok(0.)
    }
}
fn place<T>(cols: &mut [Column<T>], gaps: &[f64]) {
    for i in 1..cols.len() {
        cols[i].x = cols[i - 1].x + gaps[i - 1]
    }
}
fn extend(min_x: &mut f64, max_x: &mut f64, x0: f64, x1: f64) {
    if x0 < *min_x {
        *min_x = x0
    }
    if x1 > *max_x {
        *max_x = x1
    }
}
fn backed(
    label: &Label,
    cx: f64,
    cy: f64,
    attrs: &str,
    xs: &[f64],
    ctx: &mut TextContext<'_>,
) -> Result<String, PeleError> {
    if label.width == 0. {
        return Ok(String::new());
    }
    let l = cx - label.width / 2.;
    let at = xs.partition_point(|x| *x <= l);
    let crosses = at < xs.len() && xs[at] < l + label.width;
    let back = if crosses {
        format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"3\" fill=\"var(--_bg)\" stroke=\"none\"/>",num(l-3.),num(cy-label.height/2.+2.),num(label.width+6.),num(label.height-4.))
    } else {
        String::new()
    };
    Ok(back + &ctx.svg(label, cx, cy, attrs)?)
}
fn left_edge<T>(c: &Column<T>, extra: usize) -> f64 {
    if c.activations.len() + extra > 0 {
        c.x - ACT_W / 2.
    } else {
        c.x
    }
}
fn right_edge<T>(c: &Column<T>, extra: usize) -> f64 {
    let depth = c.activations.len() + extra;
    if depth > 0 {
        c.x + ACT_W / 2. + (depth - 1) as f64 * ACT_STEP
    } else {
        c.x
    }
}
fn end_activation<T>(
    col: &mut Column<T>,
    end: f64,
    min_x: &mut f64,
    max_x: &mut f64,
    out: &mut Vec<String>,
) -> Result<f64, PeleError> {
    let start = col.activations.pop().unwrap();
    let bottom = end.js_max(start + ACT_MIN);
    let x = col.x - ACT_W / 2. + col.activations.len() as f64 * ACT_STEP;
    extend(min_x, max_x, x, x + ACT_W);
    out.push(format!("<rect class=\"pele-activation\" data-id=\"{}\" x=\"{}\" y=\"{}\" width=\"{ACT_W}\" height=\"{}\" rx=\"2\"/>",esc_text(&col.id)?,num(x),num(start),num(bottom-start)));
    Ok(bottom)
}
fn destroy<T>(col: &mut Column<T>, line_y: f64, out: &mut String) -> Result<f64, PeleError> {
    let cx = col.x;
    let cy = line_y + 16.;
    col.life_end = cy;
    out.push_str(&format!(
        "<path class=\"pele-destroy\" data-id=\"{}\" d=\"M{},{}L{},{}M{},{}L{},{}\"/>",
        esc_text(&col.id)?,
        num(cx - 6.),
        num(cy - 6.),
        num(cx + 6.),
        num(cy + 6.),
        num(cx + 6.),
        num(cy - 6.),
        num(cx - 6.),
        num(cy + 6.)
    ));
    Ok(cy + 6.)
}
fn actor_svg<P: PhaseInput>(
    input: &mut P,
    col: &Column<P::Value>,
    top: f64,
    footer: bool,
    ctx: &mut TextContext<'_>,
) -> Result<String, P::Error> {
    let properties = input.get(&col.actor, "properties")?;
    let value = input.lookup(&properties, "class")?;
    let cls = input
        .text(&value)?
        .map(|s| class_names(&s))
        .unwrap_or_default();
    Ok(format!(
        "<g class=\"pele-node pele-actor pele-actor-{}{}{cls}\" data-id=\"{}\">{}</g>",
        col.kind,
        if footer { " pele-actor-footer" } else { "" },
        esc_text(&col.id)?,
        draw_actor(&col.kind, col.x, top, col.size, &col.label, ctx.resources)?
    ))
}

fn badge(i: usize, cx: f64, cy: f64, w: f64, tiny: f64, n: &Numbering) -> String {
    format!("<g class=\"pele-sequence-number\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{BADGE_H}\" rx=\"8\" fill=\"var(--_l)\" stroke=\"none\"/><text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"{}\" fill=\"var(--_bg)\" stroke=\"none\">{}</text></g>",num(cx-w/2.),num(cy-BADGE_H/2.),num(w),num(cx),num(cy+tiny*0.35),js_num(tiny),esc(&js_num(n.index[i])))
}
fn circle(cx: f64, cy: f64) -> String {
    format!("<circle class=\"pele-central\" cx=\"{}\" cy=\"{}\" r=\"{CENTRAL_R}\" fill=\"var(--_l)\" stroke=\"none\"/>",num(cx),num(cy))
}
pub(super) fn render(
    db: &SequenceModel,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_phases(&mut Typed(db), config, options, resources)
}
pub(super) fn render_phases<P: PhaseInput>(
    input: &mut P,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    let loose = draw(input, config, options, resources, false)?;
    let Some(available) = options.max_width.filter(|n| *n > 0.) else {
        return Ok(loose);
    };
    if loose.width <= available {
        return Ok(loose);
    }
    let tight = draw(input, config, options, resources, true)?;
    Ok(if tight.width < loose.width {
        tight
    } else {
        loose
    })
}
fn draw<P: PhaseInput>(
    input: &mut P,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
    tight: bool,
) -> Result<Rendered, P::Error> {
    input.begin_draw();
    let root = input.root();
    let conf = config.get("sequence").and_then(Value::as_object);
    let size = options.font_size.unwrap_or(16.);
    let small = round(size * 0.875);
    let tiny = round(size * 0.75);
    let pad = options.padding.unwrap_or(8.);
    let mirror = conf.and_then(|c| c.get("mirrorActors")) != Some(&Value::Bool(false));
    let right_angles = truth(conf, "rightAngles");
    let actor_margin =
        option(conf, "actorMargin", 40.).js_min(if tight { 8. } else { f64::INFINITY });
    let min_w = option(conf, "width", 64.).js_min(if tight { 48. } else { f64::INFINITY });
    let min_h = option(conf, "height", round(size * 1.5) + 20.);
    let wrap_w = option(conf, "width", 160.).js_min(if tight {
        32_f64.js_max(
            (options.max_width.unwrap_or(320.) - 2. * pad) / {
                let a = input.get(&root, "actors")?;
                let n = input.get(&a, "size")?;
                input.number(&n).unwrap_or(0.).max(1.)
            } - actor_margin
                - 24.,
        )
    } else {
        f64::INFINITY
    });
    let msg_gap = (14. + option(conf, "messageMargin", 35.) - 35.).js_max(4.);
    let note_pad = option(conf, "noteMargin", 10.).js_min(if tight { 6. } else { f64::INFINITY });
    let box_pad = option(conf, "boxMargin", 10.).js_min(if tight { 6. } else { f64::INFINITY });
    let align = match conf.and_then(|c| c.get("messageAlign")) {
        Some(Value::String(s)) if s == "left" => -1,
        Some(Value::String(s)) if s == "right" => 1,
        _ => 0,
    };
    let msg_pad = if tight { 6. } else { 12. };
    let note_over = if tight { 8. } else { 16. };
    let frame_pad = if tight { 12. } else { 20. };
    let frame_nest = if tight { 6. } else { 10. };
    let messages = input.get(&root, "messages")?;
    let count = input.count(&messages)?;
    let numbering = number_messages(input, &messages, count, truth(conf, "showSequenceNumbers"))?;
    let mut ctx = TextContext {
        resources,
        options,
        tight,
        wrap_w,
    };
    let used = if truth(conf, "hideUnusedParticipants") {
        let mut used = HashSet::new();
        let mut i = 0;
        while i < input.count(&messages)? {
            input.step()?;
            let m = input.index(&messages, i)?;
            i += 1;
            for k in ["from", "to"] {
                if let Some(v) = input.field(&m, k)? {
                    used.insert(v);
                }
            }
        }
        Some(used)
    } else {
        None
    };
    let mut next_line = vec![-1; count + 1];
    for i in (0..count).rev() {
        next_line[i] = if line(input.at_byte(&messages, i, "type")?) {
            i as isize
        } else {
            next_line[i + 1]
        };
    }
    let mut cols = vec![];
    let mut col_of = HashMap::new();
    let actors = input.get(&root, "actors")?;
    let mut iterator = input.entries(&actors)?;
    while let Some((key, actor)) = input.next(&mut iterator)? {
        let body = (|| -> Result<(), P::Error> {
            input.step()?;
            let id = input.text(&key)?.unwrap_or_default();
            if used.as_ref().is_some_and(|u| !u.contains(&id)) {
                return Ok(());
            }
            let kind = actor_kind(&input.field(&actor, "type")?.unwrap_or_default()).to_owned();
            let description = input.label_field(&actor, "description")?;
            let wrap = input.flag(&actor, "wrap")?;
            input.keep_text(description.as_deref().unwrap_or("").encode_utf16().count())?;
            let label = ctx.text(description.as_deref(), size, wrap, None, 0)?;
            let created = input.get(&root, "createdActors")?;
            let created = input.lookup(&created, &id)?;
            let destroyed = input.get(&root, "destroyedActors")?;
            let destroyed = input.lookup(&destroyed, &id)?;
            let position = |n: Option<f64>| {
                n.filter(|n| n.is_finite() && *n >= 0. && n.fract() == 0.)
                    .map(|n| n.min(count as f64) as usize)
            };
            let mut create_at = position(input.number(&created)).map_or(-1, |n| next_line[n]);
            if create_at >= 0
                && input
                    .at_field(&messages, create_at as usize, "to")?
                    .as_ref()
                    != Some(&id)
            {
                create_at = -1;
            }
            let mut destroy_at = position(input.number(&destroyed)).map_or(-1, |n| next_line[n]);
            if destroy_at >= 0
                && input
                    .at_field(&messages, destroy_at as usize, "to")?
                    .as_ref()
                    != Some(&id)
                && input
                    .at_field(&messages, destroy_at as usize, "from")?
                    .as_ref()
                    != Some(&id)
            {
                destroy_at = -1;
            }
            col_of.insert(id.clone(), cols.len());
            let actor_size = actor_size(&kind, &label, min_w, min_h, if tight { 8. } else { 16. });
            cols.push(Column {
                id: id.clone(),
                actor: actor.clone(),
                kind,
                label,
                size: actor_size,
                x: 0.,
                create_at,
                destroy_at,
                top: 0.,
                life_start: 0.,
                life_end: -1.,
                activations: vec![],
            });
            Ok(())
        })();
        if let Err(error) = body {
            let _ = input.close(&mut iterator);
            return Err(error);
        }
    }
    let mut needs = vec![];
    let mut views: Vec<Option<View>> = (0..count).map(|_| None).collect();
    for i in 0..count {
        input.step()?;
        let m = input.index(&messages, i)?;
        let (Some(&a), Some(&b), Some(raw)) = (
            input
                .field(&m, "from")?
                .as_ref()
                .and_then(|s| col_of.get(s)),
            input.field(&m, "to")?.as_ref().and_then(|s| col_of.get(s)),
            input.field(&m, "message")?,
        ) else {
            continue;
        };
        if input.byte(&m, "type")? == Some(NOTE) {
            input.keep_text(raw.encode_utf16().count())?;
            let label = ctx.text(Some(&raw), small, input.flag(&m, "wrap")?, None, 0)?;
            let w = label.width + 2. * note_pad;
            match input.byte(&m, "placement")? {
                Some(0) => need(
                    &mut needs,
                    cols.len(),
                    a as isize - 1,
                    a as isize,
                    w + 2. * NOTE_GAP + ACT_W,
                ),
                Some(1) => need(
                    &mut needs,
                    cols.len(),
                    a as isize,
                    a as isize + 1,
                    w + 2. * NOTE_GAP + ACT_W,
                ),
                _ => {
                    if a == b {
                        need(
                            &mut needs,
                            cols.len(),
                            a as isize - 1,
                            a as isize,
                            w / 2. + NOTE_GAP,
                        );
                        need(
                            &mut needs,
                            cols.len(),
                            a as isize,
                            a as isize + 1,
                            w / 2. + NOTE_GAP,
                        )
                    } else {
                        need(
                            &mut needs,
                            cols.len(),
                            a.min(b) as isize,
                            a.max(b) as isize,
                            w - 2. * note_over,
                        )
                    }
                }
            }
            views[i] = Some(View {
                kind: V::Note,
                a,
                b,
                label,
                x: 0.,
                w,
            });
        } else if line(input.byte(&m, "type")?) {
            input.keep_text(raw.encode_utf16().count())?;
            let label = ctx.text(Some(&raw), small, input.flag(&m, "wrap")?, None, 0)?;
            let badge = badge_width(i, &numbering, tiny, &mut ctx)?;
            let kind = if a == b {
                need(
                    &mut needs,
                    cols.len(),
                    a as isize,
                    a as isize + 1,
                    SELF_W + 8. + label.width + 16. + ACT_W + badge,
                );
                V::SelfLoop
            } else {
                let created = if cols[b].create_at == i as isize {
                    cols[b].size.w / 2.
                } else {
                    0.
                };
                let central = if input.byte(&m, "centralConnection")?.is_some_and(|n| n != 0) {
                    4. * CENTRAL_R
                } else {
                    0.
                };
                need(
                    &mut needs,
                    cols.len(),
                    a.min(b) as isize,
                    a.max(b) as isize,
                    if tight {
                        label.width.js_min(16_f64.js_max(wrap_w - 16.))
                    } else {
                        label.width
                    } + 2. * msg_pad
                        + ACT_W
                        + badge
                        + created
                        + central,
                );
                V::Line
            };
            views[i] = Some(View {
                kind,
                a,
                b,
                label,
                x: 0.,
                w: 0.,
            });
        }
    }
    let mut runs: Vec<BoxRun<P::Value>> = vec![];
    for (i, col) in cols.iter().enumerate() {
        let value = input.get(&col.actor, "box")?;
        if let Some(identity) = input.identity(&value)? {
            if let Some(run) = runs
                .last_mut()
                .filter(|r| r.identity == identity && r.last + 1 == i)
            {
                run.last = i;
            } else {
                let name = input.label_field(&value, "name")?;
                let wrap = input.flag(&value, "wrap")?;
                input.keep_text(name.as_deref().unwrap_or("").encode_utf16().count())?;
                let title = ctx.text(name.as_deref(), small, wrap, None, 0)?;
                runs.push(BoxRun {
                    value,
                    identity,
                    first: i,
                    last: i,
                    title,
                    grow: 0.,
                });
            }
        }
    }
    let mut gaps = vec![0.; cols.len().saturating_sub(1)];
    for i in 0..gaps.len() {
        let l = &cols[i];
        let r = &cols[i + 1];
        let lb = input.get(&l.actor, "box")?;
        let rb = input.get(&r.actor, "box")?;
        let boxed = if input.identity(&lb)? != input.identity(&rb)? {
            let lb = input.get(&l.actor, "box")?;
            let rb = input.get(&r.actor, "box")?;
            (if input.truthy(&lb) { box_pad } else { 0. })
                + (if input.truthy(&rb) { box_pad } else { 0. })
        } else {
            0.
        };
        gaps[i] = (l.size.w / 2. + r.size.w / 2. + actor_margin + boxed).js_max(
            needs
                .iter()
                .find(|(k, _)| *k == i * cols.len() + i + 1)
                .map_or(0., |(_, w)| *w),
        );
    }
    let mut spans: Vec<_> = needs
        .iter()
        .filter_map(|(key, width)| {
            let lo = key / cols.len();
            let hi = key % cols.len();
            if hi > lo + 1 {
                Some((lo, hi, *width))
            } else {
                None
            }
        })
        .collect();
    spans.sort_by_key(|(lo, hi, _)| (hi - lo, *lo));
    for (lo, hi, width) in spans {
        let total: f64 = gaps[lo..hi].iter().sum();
        if total >= width {
            continue;
        }
        let share = (width - total) / (hi - lo) as f64;
        for gap in &mut gaps[lo..hi] {
            *gap += share
        }
    }
    place(&mut cols, &gaps);
    for run in &mut runs {
        let inner = cols[run.last].x + cols[run.last].size.w / 2.
            - (cols[run.first].x - cols[run.first].size.w / 2.);
        run.grow = ((run.title.width - inner) / 2.).js_max(0.);
        if run.grow == 0. {
            continue;
        }
        if run.first > 0 {
            gaps[run.first - 1] += run.grow
        }
        if run.last + 1 < cols.len() {
            gaps[run.last] += run.grow
        }
    }
    place(&mut cols, &gaps);
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    for c in &cols {
        extend(
            &mut min_x,
            &mut max_x,
            c.x - c.size.w / 2.,
            c.x + c.size.w / 2.,
        )
    }
    for run in &runs {
        extend(
            &mut min_x,
            &mut max_x,
            cols[run.first].x - cols[run.first].size.w / 2. - box_pad - run.grow,
            cols[run.last].x + cols[run.last].size.w / 2. + box_pad + run.grow,
        )
    }
    let mut frames: Vec<Frame> = vec![];
    let mut frame_of = vec![None; count];
    let mut open: Vec<usize> = vec![];
    for i in 0..count {
        input.step()?;
        let m = input.index(&messages, i)?;
        if let Some(kind) = frame_kind(input.byte(&m, "type")?) {
            let index = frames.len();
            let fill = if kind == "rect" {
                input.field(&m, "message")?.unwrap_or_default()
            } else {
                String::new()
            };
            input.keep_text(fill.encode_utf16().count())?;
            frames.push(Frame {
                kind,
                x0: f64::INFINITY,
                x1: f64::NEG_INFINITY,
                top: 0.,
                title: Label::default(),
                tab_w: 0.,
                fill,
                dividers: vec![i],
                divider_y: vec![],
                divider_labels: vec![],
            });
            frame_of[i] = Some(index);
            open.push(index);
        } else if divider(input.byte(&m, "type")?) {
            if let Some(index) = open.last() {
                frames[*index].dividers.push(i)
            }
        } else if frame_end(input.byte(&m, "type")?) {
            let Some(index) = open.pop() else { continue };
            frame_of[i] = Some(index);
            let f = &mut frames[index];
            if f.x0 > f.x1 {
                let (x0, x1) = if cols.is_empty() {
                    (0., 120.)
                } else {
                    (cols[0].x - frame_pad, cols.last().unwrap().x + frame_pad)
                };
                f.x0 = x0;
                f.x1 = x1;
            }
            if f.kind != "rect" {
                f.tab_w = ctx.width(f.kind, tiny, 1)? + 14.;
                for k in 0..f.dividers.len() {
                    let raw = input
                        .at_field(&messages, f.dividers[k], "message")?
                        .unwrap_or_default();
                    input.keep_text(raw.encode_utf16().count().saturating_add(2))?;
                    let tab = if k == 0 { f.tab_w + 8. } else { 0. };
                    let room = f.x1 - f.x0 - 16. - tab;
                    let label = if raw.is_empty() {
                        Label::default()
                    } else {
                        ctx.text(
                            Some(&format!("[{raw}]")),
                            small,
                            true,
                            Some(room.js_max(if tight {
                                FRAME_TEXT.js_min(wrap_w)
                            } else {
                                FRAME_TEXT
                            })),
                            0,
                        )?
                    };
                    if label.width > room {
                        f.x1 += label.width - room
                    }
                    if k == 0 {
                        f.title = label
                    } else {
                        f.divider_labels.push(label)
                    }
                }
            }
            let (x0, x1) = (f.x0, f.x1);
            extend(&mut min_x, &mut max_x, x0, x1);
            if let Some(parent) = open.last() {
                frames[*parent].x0 = frames[*parent].x0.js_min(x0 - frame_nest);
                frames[*parent].x1 = frames[*parent].x1.js_max(x1 + frame_nest);
            }
        } else if let Some(v) = &mut views[i] {
            let a = &cols[v.a];
            let b = &cols[v.b];
            let (x0, x1);
            if v.kind == V::Note {
                if input.byte(&m, "placement")? == Some(0) {
                    v.x = a.x - NOTE_GAP - v.w
                } else if input.byte(&m, "placement")? == Some(1) {
                    v.x = a.x + NOTE_GAP
                } else if v.a == v.b {
                    v.x = a.x - v.w / 2.
                } else {
                    let lo = a.x.js_min(b.x) - note_over;
                    let hi = a.x.js_max(b.x) + note_over;
                    if tight || input.flag(&m, "wrap")? {
                        let raw = input.field(&m, "message")?;
                        input.keep_text(raw.as_deref().unwrap_or("").encode_utf16().count())?;
                        v.label = ctx.text(
                            raw.as_deref(),
                            small,
                            true,
                            Some(wrap_w.js_max(hi - lo - 2. * note_pad)),
                            0,
                        )?
                    }
                    v.w = (hi - lo).js_max(v.label.width + 2. * note_pad);
                    v.x = (lo + hi - v.w) / 2.;
                }
                x0 = v.x;
                x1 = v.x + v.w;
            } else if v.kind == V::SelfLoop {
                x0 = a.x;
                x1 = a.x
                    + ACT_W
                    + SELF_W
                    + 8.
                    + v.label.width
                    + badge_width(i, &numbering, tiny, &mut ctx)?;
            } else {
                let mut lo = a.x.js_min(b.x);
                let mut hi = a.x.js_max(b.x);
                if tight || input.flag(&m, "wrap")? {
                    let raw = input.field(&m, "message")?;
                    input.keep_text(raw.as_deref().unwrap_or("").encode_utf16().count())?;
                    v.label = ctx.text(
                        raw.as_deref(),
                        small,
                        true,
                        Some(wrap_w.js_max(hi - lo - 2. * msg_pad - ACT_W)),
                        0,
                    )?
                }
                if b.create_at == i as isize {
                    if b.x > a.x {
                        hi += b.size.w / 2.
                    } else {
                        lo -= b.size.w / 2.
                    }
                }
                x0 = lo;
                x1 = hi;
            }
            extend(&mut min_x, &mut max_x, x0, x1);
            if let Some(index) = open.last() {
                let f = &mut frames[*index];
                f.x0 = f.x0.js_min(x0 - frame_pad);
                f.x1 = f.x1.js_max(x1 + frame_pad);
            }
        }
    }
    let box_title_h = runs.iter().fold(0_f64, |h, r| h.js_max(r.title.height));
    let box_top = if !runs.is_empty() {
        box_pad
            + if box_title_h > 0. {
                box_title_h + 4.
            } else {
                0.
            }
    } else {
        0.
    };
    let row_h = cols
        .iter()
        .filter(|c| c.create_at < 0)
        .fold(0_f64, |h, c| h.js_max(c.size.h));
    let head_bottom = box_top + row_h;
    for c in &mut cols {
        c.top = head_bottom - c.size.h;
        c.life_start = head_bottom;
    }
    let xs: Vec<_> = cols.iter().map(|c| c.x).collect();
    let mut rects_out = vec![];
    let mut activations_out = vec![];
    let mut frames_out = String::new();
    let mut edges_out = String::new();
    let mut labels_out = String::new();
    let mut notes_out = String::new();
    let mut actors_out = String::new();
    let mut crosses_out = String::new();
    let mut y = head_bottom + 6.;
    let mut stack: Vec<usize> = vec![];
    for i in 0..count {
        input.step()?;
        let m = input.index(&messages, i)?;
        let kind = input.byte(&m, "type")?;
        let view = &views[i];
        let frame = frame_of[i];
        if kind == Some(ACTIVE_START) {
            if let Some(&c) = input
                .field(&m, "from")?
                .as_ref()
                .and_then(|s| col_of.get(s))
            {
                cols[c].activations.push(y)
            }
        } else if kind == Some(ACTIVE_END) {
            if let Some(&c) = input
                .field(&m, "from")?
                .as_ref()
                .and_then(|s| col_of.get(s))
            {
                if !cols[c].activations.is_empty() {
                    y = y.js_max(end_activation(
                        &mut cols[c],
                        y,
                        &mut min_x,
                        &mut max_x,
                        &mut activations_out,
                    )?);
                }
            }
        } else if let Some(index) = frame {
            let f = &mut frames[index];
            if f.dividers[0] == i {
                stack.push(index);
                if f.kind == "rect" {
                    f.top = y + 8.;
                    y = f.top;
                } else {
                    f.top = y + 12.;
                    y = f.top + TAB_H.js_max(f.title.height + 2.);
                }
            } else {
                stack.pop();
                let bottom = y + 12.;
                let x = num(f.x0);
                let w = num(f.x1 - f.x0);
                if f.kind == "rect" {
                    let fill = if f.fill.is_empty() {
                        String::new()
                    } else {
                        resolve_style(&[format!("fill:{}", f.fill)]).shape
                    };
                    rects_out.push(format!("<rect class=\"pele-rect\" x=\"{x}\" y=\"{}\" width=\"{w}\" height=\"{}\" rx=\"{RADIUS}\"{}{fill}/>",num(f.top),num(bottom-f.top),if fill.is_empty(){" fill-opacity=\"0.5\""}else{""}));
                } else {
                    let mut inner=format!("<rect x=\"{x}\" y=\"{}\" width=\"{w}\" height=\"{}\" rx=\"{RADIUS}\" fill=\"none\"/>",num(f.top),num(bottom-f.top));
                    for k in 0..f.divider_y.len() {
                        let dy = f.divider_y[k];
                        let label = &f.divider_labels[k];
                        inner.push_str(&format!(
                            "<path d=\"M{x},{}H{}\"{DASH}/>{}",
                            num(dy),
                            num(f.x1),
                            backed(
                                label,
                                f.x0 + 8. + label.width / 2.,
                                dy + 2. + label.height / 2.,
                                " class=\"pele-frame-label\" fill=\"var(--_m)\" stroke=\"none\"",
                                &xs,
                                &mut ctx
                            )?
                        ));
                    }
                    inner.push_str(&format!("<rect x=\"{x}\" y=\"{}\" width=\"{}\" height=\"{TAB_H}\" rx=\"{RADIUS}\" fill=\"var(--_s)\"/><text class=\"pele-frame-kind\" x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"{}\" font-weight=\"var(--_hw)\" fill=\"var(--_m)\" stroke=\"none\">{}</text>{}",num(f.top),num(f.tab_w),num(f.x0+f.tab_w/2.),num(f.top+TAB_H/2.+tiny*0.35),js_num(tiny),f.kind,backed(&f.title,f.x0+f.tab_w+8.+f.title.width/2.,f.top+TAB_H.js_max(f.title.height)/2.," class=\"pele-frame-label\" fill=\"var(--_m)\" stroke=\"none\"",&xs,&mut ctx)?));
                    frames_out.push_str(&format!(
                        "<g class=\"pele-frame pele-frame-{}\">{inner}</g>",
                        f.kind
                    ));
                }
                y = bottom;
            }
        } else if divider(kind) {
            if let Some(index) = stack.last() {
                let f = &mut frames[*index];
                let label = &f.divider_labels[f.divider_y.len()];
                f.divider_y.push(y + 12.);
                y += 12. + if label.height > 0. { label.height } else { 0. };
            }
        } else if let Some(v) = view {
            if v.kind == V::Note {
                let a = &cols[v.a];
                let h = v.label.height + 2. * 4_f64.js_max(note_pad - 4.);
                let top = y + 10.;
                let mut x = v.x;
                if input.byte(&m, "placement")? == Some(1) {
                    x += right_edge(a, 0) - a.x
                } else if input.byte(&m, "placement")? == Some(0) {
                    x += left_edge(a, 0) - a.x
                }
                extend(&mut min_x, &mut max_x, x, x + v.w);
                notes_out.push_str(&format!("<g class=\"pele-note\" data-id=\"i{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{RADIUS}\"/>{}</g>",esc_text(&input.field(&m,"id")?.unwrap_or_default())?,num(x),num(top),num(v.w),num(h),ctx.svg(&v.label,x+v.w/2.,top+h/2.," class=\"pele-label\" stroke=\"none\" fill=\"var(--_fg)\"")?));
                y = top + h;
                continue;
            }
            let st = kind.and_then(style).ok_or_else(|| {
                PeleError::new(
                    "Sequence live message kind changed to an unsupported line style",
                    crate::compat::ErrorCode::Semantic,
                )
            })?;
            let label = &v.label;
            let bw = badge_width(i, &numbering, tiny, &mut ctx)?;
            let mut starting = 0;
            for j in i + 1..count {
                input.step()?;
                let message = input.index(&messages, j)?;
                let t = input.byte(&message, "type")?;
                if t == Some(ACTIVE_START) {
                    if input.field(&message, "from")? == input.field(&m, "to")? {
                        starting += 1
                    }
                } else if !matches!(
                    t,
                    Some(ACTIVE_END | CENTRAL_CONNECTION | CENTRAL_CONNECTION_REVERSE)
                ) {
                    break;
                }
            }
            let up = if st.bottom { 1. } else { -1. };
            let attrs = if st.dotted { DASH } else { "" };
            let circle_to = matches!(
                input.byte(&m, "centralConnection")?,
                Some(CENTRAL_CONNECTION | CENTRAL_CONNECTION_DUAL)
            );
            let circle_from = matches!(
                input.byte(&m, "centralConnection")?,
                Some(CENTRAL_CONNECTION_REVERSE | CENTRAL_CONNECTION_DUAL)
            );
            let tail_at_end = st.at_start && !st.at_end;
            let head_trim = trim(st.head);
            let mut inner = String::new();
            let line_y;
            if v.kind == V::SelfLoop {
                let a = &cols[v.a];
                let top = y + msg_gap + ((label.height - SELF_H) / 2.).js_max(0.);
                let bottom = top + SELF_H;
                let mut x0 = if circle_from {
                    a.x + CENTRAL_R
                } else {
                    right_edge(a, 0)
                };
                let mut x1 = if circle_to {
                    a.x + CENTRAL_R
                } else {
                    right_edge(a, starting)
                };
                let out = x0.js_max(x1) + SELF_W + bw / 2.;
                if circle_from {
                    inner.push_str(&circle(a.x, top))
                }
                if circle_to {
                    inner.push_str(&circle(a.x, bottom))
                }
                if bw > 0. {
                    inner.push_str(&badge(
                        i,
                        if tail_at_end { x1 } else { x0 },
                        if tail_at_end { bottom } else { top },
                        bw,
                        tiny,
                        &numbering,
                    ));
                    if tail_at_end {
                        x1 += bw / 2.
                    } else {
                        x0 += bw / 2.
                    }
                }
                let heads = if st.at_start {
                    head(st.head, x0, top, -1., up)
                } else {
                    String::new()
                } + &if st.at_end {
                    head(st.head, x1, bottom, -1., -up)
                } else {
                    String::new()
                };
                let from = num(x0 + if st.at_start { head_trim } else { 0. });
                let to = num(x1 + if st.at_end { head_trim } else { 0. });
                let d = if right_angles {
                    format!("M{from},{}H{}V{}H{to}", num(top), num(out), num(bottom))
                } else {
                    format!(
                        "M{from},{}H{}A6,6 0 0 1 {},{}V{}A6,6 0 0 1 {},{}H{to}",
                        num(top),
                        num(out - 6.),
                        num(out),
                        num(top + 6.),
                        num(bottom - 6.),
                        num(out - 6.),
                        num(bottom)
                    )
                };
                inner = format!("<path d=\"{d}\"{attrs}/>{heads}{inner}");
                extend(&mut min_x, &mut max_x, a.x, out + 8. + label.width);
                if label.width > 0. {
                    labels_out.push_str(&format!(
                        "<g class=\"pele-edge-label\" data-id=\"i{}\">{}</g>",
                        esc_text(&input.field(&m, "id")?.unwrap_or_default())?,
                        ctx.svg(label, out + 8. + label.width / 2., (top + bottom) / 2., "")?
                    ))
                }
                line_y = bottom;
            } else {
                let a = &cols[v.a];
                let b = &cols[v.b];
                let dir = if b.x > a.x { 1. } else { -1. };
                let creating = b.create_at == i as isize;
                line_y = y
                    + msg_gap
                    + (if label.height > 0. {
                        label.height + 2.
                    } else {
                        6.
                    })
                    .js_max(if creating {
                        b.size.h / 2. - msg_gap + 6.
                    } else {
                        0.
                    });
                let mut x0 = if dir > 0. {
                    right_edge(a, 0)
                } else {
                    left_edge(a, 0)
                };
                let mut x1 = if dir > 0. {
                    left_edge(b, starting)
                } else {
                    right_edge(b, starting)
                };
                if creating {
                    x1 = b.x
                        - dir
                            * if is_figure(&b.kind) {
                                18.
                            } else {
                                b.size.w / 2. + 1.
                            };
                }
                if circle_from {
                    inner.push_str(&circle(a.x, line_y));
                    x0 = a.x + dir * CENTRAL_R;
                }
                if circle_to {
                    inner.push_str(&circle(b.x, line_y));
                    x1 = b.x - dir * CENTRAL_R;
                }
                if bw > 0. {
                    if tail_at_end {
                        let cx = if circle_to {
                            x1 - dir * (bw / 2. + 2.)
                        } else {
                            x1
                        };
                        inner.push_str(&badge(i, cx, line_y, bw, tiny, &numbering));
                        x1 = cx - dir * bw / 2.;
                    } else {
                        let cx = if circle_from {
                            x0 + dir * (bw / 2. + 2.)
                        } else {
                            x0
                        };
                        inner.push_str(&badge(i, cx, line_y, bw, tiny, &numbering));
                        x0 = cx + dir * bw / 2.;
                    }
                }
                let heads = if st.at_start {
                    head(st.head, x0, line_y, -dir, up * dir)
                } else {
                    String::new()
                } + &if st.at_end {
                    head(st.head, x1, line_y, dir, up * dir)
                } else {
                    String::new()
                };
                let from = x0 + if st.at_start { dir * head_trim } else { 0. };
                let to = x1 - if st.at_end { dir * head_trim } else { 0. };
                inner = format!(
                    "<path d=\"M{},{}H{}\"{attrs}/>{heads}{inner}",
                    num(from),
                    num(line_y),
                    num(to)
                );
                if label.width > 0. {
                    let l = x0.js_min(x1);
                    let r = x0.js_max(x1);
                    let cx = if align < 0 {
                        l + msg_pad + label.width / 2.
                    } else if align > 0 {
                        r - msg_pad - label.width / 2.
                    } else {
                        (l + r) / 2.
                    };
                    extend(
                        &mut min_x,
                        &mut max_x,
                        cx - label.width / 2.,
                        cx + label.width / 2.,
                    );
                    labels_out.push_str(&format!(
                        "<g class=\"pele-edge-label\" data-id=\"i{}\">{}</g>",
                        esc_text(&input.field(&m, "id")?.unwrap_or_default())?,
                        backed(
                            label,
                            cx,
                            line_y - 4. - label.height / 2.,
                            "",
                            &xs,
                            &mut ctx
                        )?
                    ))
                }
                if creating {
                    let b = &mut cols[v.b];
                    b.top = line_y - b.size.h / 2.;
                    b.life_start = b.top + b.size.h;
                    y = line_y + b.size.h / 2. - 6.;
                }
            }
            edges_out.push_str(&format!(
                "<g class=\"pele-edge pele-message\" data-id=\"i{}\">{inner}</g>",
                esc_text(&input.field(&m, "id")?.unwrap_or_default())?
            ));
            y = y.js_max(line_y);
            if cols[v.a].destroy_at == i as isize {
                y = y.js_max(destroy(&mut cols[v.a], line_y, &mut crosses_out)?)
            }
            if v.a != v.b && cols[v.b].destroy_at == i as isize {
                y = y.js_max(destroy(&mut cols[v.b], line_y, &mut crosses_out)?)
            }
        }
    }
    y += 20.;
    for col in &mut cols {
        while !col.activations.is_empty() {
            end_activation(col, y, &mut min_x, &mut max_x, &mut activations_out)?;
        }
    }
    let mut bottom = y;
    if mirror {
        let mut foot_h = 0_f64;
        for col in &cols {
            if col.life_end >= 0. {
                continue;
            }
            foot_h = foot_h.js_max(col.size.h);
            actors_out.push_str(&actor_svg(input, col, y, true, &mut ctx)?);
        }
        bottom = y + foot_h;
    }
    let mut lifelines = String::new();
    for col in &cols {
        actors_out.push_str(&actor_svg(input, col, col.top, false, &mut ctx)?);
        let end = if col.life_end >= 0. { col.life_end } else { y };
        if end > col.life_start {
            lifelines.push_str(&format!(
                "<path class=\"pele-lifeline\" data-id=\"{}\" d=\"M{},{}V{}\"/>",
                esc_text(&col.id)?,
                num(col.x),
                num(col.life_start),
                num(end)
            ))
        }
    }
    let mut boxes_out = String::new();
    for run in &runs {
        let b = &run.value;
        let x0 = cols[run.first].x - cols[run.first].size.w / 2. - box_pad - run.grow;
        let x1 = cols[run.last].x + cols[run.last].size.w / 2. + box_pad + run.grow;
        let fill = if input.field(b, "fill")?.as_deref() == Some("transparent") {
            String::new()
        } else {
            resolve_style(&[format!(
                "fill:{}",
                input.field(b, "fill")?.unwrap_or_default()
            )])
            .shape
        };
        let id = if let Some(s) = input.field(b, "name")?.as_deref().filter(|s| !s.is_empty()) {
            format!(" data-id=\"{}\"", esc_text(s)?)
        } else {
            String::new()
        };
        boxes_out.push_str(&format!("<g class=\"pele-cluster pele-box\"{id}><rect x=\"{}\" y=\"0\" width=\"{}\" height=\"{}\" rx=\"{RADIUS}\" fill=\"var(--_a)\" stroke=\"var(--_b)\"{}{fill}/>{}</g>",num(x0),num(x1-x0),num(bottom+box_pad),if fill.is_empty(){" fill-opacity=\"0.5\""}else{""},ctx.svg(&run.title,(x0+x1)/2.,box_pad/2.+2.+run.title.height/2.," class=\"pele-cluster-label\" fill=\"var(--_m)\"")?));
    }
    if !runs.is_empty() {
        bottom += box_pad;
    }
    let mut min_y = 0.;
    let mut title_out = String::new();
    let title_room = options.max_width.filter(|n| *n > 0.).map_or(4000., |n| {
        80_f64.js_max(n - 2. * options.padding.unwrap_or(8.))
    });
    let raw_title = input.metadata("title")?;
    input.keep_text(raw_title.as_deref().unwrap_or("").encode_utf16().count())?;
    let title = ctx.text(
        raw_title.as_deref(),
        size,
        false,
        Some(if tight { title_room } else { wrap_w }),
        1,
    )?;
    if min_x > max_x {
        extend(&mut min_x, &mut max_x, 0., title.width.js_max(40.))
    }
    if title.width > 0. {
        let cx = (min_x + max_x) / 2.;
        extend(
            &mut min_x,
            &mut max_x,
            cx - title.width / 2.,
            cx + title.width / 2.,
        );
        min_y = -title.height - 12.;
        title_out = ctx.svg(
            &title,
            cx,
            min_y + title.height / 2.,
            " class=\"pele-title\" font-weight=\"var(--_tw)\"",
        )?;
    }
    let width = (max_x - min_x + 2. * pad).ceil();
    let height = (bottom - min_y + 2. * pad).ceil();
    rects_out.reverse();
    activations_out.reverse();
    let content = format!(
        "<g transform=\"translate({},{})\">{title_out}{}{}{}{}{}{}{}{}{}{}{}</g>",
        num(pad - min_x),
        num(pad - min_y),
        group(
            "pele-boxes",
            &format!(" font-size=\"{}\"", js_num(small)),
            boxes_out
        ),
        group("pele-rects", " fill=\"var(--_a)\"", rects_out.join("")),
        group(
            "pele-lifelines",
            " fill=\"none\" stroke=\"var(--_b)\"",
            lifelines
        ),
        group(
            "pele-activations",
            " fill=\"var(--_s)\" stroke=\"var(--_b)\"",
            activations_out.join("")
        ),
        group(
            "pele-frames",
            &format!(" font-size=\"{}\" stroke=\"var(--_b)\"", js_num(small)),
            frames_out
        ),
        group(
            "pele-edges",
            " fill=\"none\" stroke=\"var(--_l)\" stroke-linecap=\"round\"",
            edges_out
        ),
        group(
            "pele-destroys",
            " fill=\"none\" stroke=\"var(--_l)\" stroke-width=\"1.5\" stroke-linecap=\"round\"",
            crosses_out
        ),
        group(
            "pele-edge-labels",
            &format!(" font-size=\"{}\"", js_num(small)),
            labels_out
        ),
        group(
            "pele-notes",
            &format!(
                " font-size=\"{}\" fill=\"var(--_a)\" stroke=\"var(--_b)\"",
                js_num(small)
            ),
            notes_out
        ),
        group("pele-nodes", "", actors_out),
        ""
    );
    let svg = svg::document(
        "sequence",
        width,
        height,
        size,
        options,
        input.metadata("accTitle")?.as_deref(),
        input.metadata("accDescr")?.as_deref(),
        &content,
    )?;
    Ok(Rendered {
        svg,
        width,
        height,
        links: vec![],
    })
}
fn group(class: &str, attrs: &str, body: String) -> String {
    if body.is_empty() {
        String::new()
    } else {
        format!("<g class=\"{class}\"{attrs}>{body}</g>")
    }
}
