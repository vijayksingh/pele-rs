//! Cynefin domains, ellipse clearance and clipped quadratic transitions, Pele 458322a5 (MIT).
use super::{boundaries as b, model::*};
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    svg::{
        self,
        edges::{marker, marker_trim},
        esc_text, label_svg, label_svg_with_resources, num,
    },
    text::{layout_label, Label},
    Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
use std::collections::HashMap;
use DomainName::*;
const QUADRANTS: [(DomainName, f64, f64, usize); 4] = [
    (Complex, 0., 0., 0),
    (Complicated, 1., 0., 1),
    (Chaotic, 0., 1., 3),
    (Clear, 1., 1., 2),
];
struct Badge {
    text: String,
    label: Label,
    w: f64,
    h: f64,
    overflow: bool,
}
struct Block {
    name: DomainName,
    title: Label,
    subtitles: Vec<Label>,
    badges: Vec<Badge>,
    w: f64,
    h: f64,
    cx: f64,
    cy: f64,
}
fn setting(c: &Object, key: &str, fallback: f64, min: f64, max: f64) -> f64 {
    match c
        .get("cynefin")
        .and_then(Value::as_object)
        .and_then(|c| c.get(key))
    {
        Some(Value::Number(n)) if *n >= min && *n <= max => *n,
        _ => fallback,
    }
}
fn meta(name: DomainName) -> [&'static str; 3] {
    match name {
        Complex => ["Complex", "Probe → Sense → Respond", "Emergent Practices"],
        Complicated => ["Complicated", "Sense → Analyse → Respond", "Good Practices"],
        Clear => ["Clear", "Sense → Categorise → Respond", "Best Practices"],
        Chaotic => ["Chaotic", "Act → Sense → Respond", "Novel Practices"],
        Confusion => ["Confusion", "", "Disorder"],
    }
}
struct Metrics {
    size: f64,
    small: f64,
    item_size: f64,
    item_line: f64,
    show_descriptions: bool,
}
fn badge(
    text: String,
    overflow: bool,
    m: &Metrics,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Badge, PeleError> {
    let label = layout_label(Some(&text), false, r, o, m.item_size, 200., 0)?;
    Ok(Badge {
        w: label.width + 20.,
        h: label.height.js_max(m.item_line) + 8.,
        text,
        label,
        overflow,
    })
}
fn block<I: PhaseInput>(
    name: DomainName,
    input: &mut I,
    m: &Metrics,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Block, I::Error> {
    let meta = meta(name);
    let title = layout_label(Some(meta[0]), false, r, o, m.size, 4000., 1)?;
    let mut subtitles = vec![];
    if m.show_descriptions {
        for s in &meta[1..] {
            if !s.is_empty() {
                subtitles.push(layout_label(Some(s), false, r, o, m.small, 4000., 2)?);
            }
        }
    }
    // Source captures the items receiver and initial shown length after the
    // domain heading callbacks, then reads each label as it reaches that slot.
    let items = input.items(name)?;
    let count = input.item_count(&items)?;
    let shown = if name == Confusion && count > 3 {
        3
    } else {
        count
    };
    let mut badges = vec![];
    for i in 0..shown {
        let text = input.item_label(&items, i)?;
        badges.push(badge(text, false, m, o, r)?);
    }
    let count = input.item_count(&items)?;
    if shown < count {
        badges.push(badge(format!("+{} more", count - shown), true, m, o, r)?);
    }
    let mut w = title.width;
    let mut h = title.height;
    for s in &subtitles {
        w = w.js_max(s.width);
        h += s.height;
    }
    if !badges.is_empty() {
        h += 4.;
    }
    for b in &badges {
        w = w.js_max(b.w);
        h += 6. + b.h;
    }
    Ok(Block {
        name,
        title,
        subtitles,
        badges,
        w,
        h,
        cx: 0.,
        cy: 0.,
    })
}
fn draw_block(b: &Block, m: &Metrics, r: &mut dyn RenderResources) -> Result<String, PeleError> {
    let mut y = b.cy - b.h / 2.;
    let mut out = label_svg(
        &b.title,
        b.cx,
        y + b.title.height / 2.,
        " class=\"pele-label\" font-weight=\"var(--_hw)\"",
    );
    y += b.title.height;
    for s in &b.subtitles {
        out+=&label_svg(s,b.cx,y+s.height/2.,&format!(" class=\"pele-cynefin-subtitle\" font-size=\"{}\" font-style=\"italic\" fill=\"var(--_m)\"",svg::js_num(m.small)));
        y += s.height;
    }
    y += 4.;
    for item in &b.badges {
        y += 6.;
        out+=&format!("<g class=\"pele-node{}\"{}><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"var(--pele-radius,4px)\" fill=\"var(--_bg)\" stroke=\"var(--_b)\"{}/>{}</g>",if item.overflow{" pele-cynefin-overflow"}else{""},if item.overflow{String::new()}else{format!(" data-id=\"{}\"",esc_text(&item.text)?)},num(b.cx-item.w/2.),num(y),num(item.w),num(item.h),if item.overflow{" stroke-dasharray=\"3 2\""}else{""},label_svg_with_resources(&item.label,b.cx,y+item.h/2.,&format!(" class=\"pele-label\" font-size=\"{}\"{}",svg::js_num(m.item_size),if item.overflow{" fill=\"var(--_m)\""}else{""}),r)?);
        y += item.h;
    }
    Ok(format!(
        "<g class=\"pele-cynefin-domain\" data-id=\"{}\">{out}</g>",
        b.name.as_str()
    ))
}
fn inside(b: &Block, x: f64, y: f64, cx: f64, cy: f64, rx: f64, ry: f64) -> bool {
    if b.name == Confusion {
        ((x - cx) / (rx + 4.)).powi(2) + ((y - cy) / (ry + 4.)).powi(2) <= 1.
    } else {
        (x - b.cx).abs() <= b.w / 2. + 8. && (y - b.cy).abs() <= b.h / 2. + 8.
    }
}
fn point(t: f64, from: &Block, to: &Block, px: f64, py: f64) -> (f64, f64) {
    let u = 1. - t;
    (
        u * u * from.cx + 2. * u * t * px + t * t * to.cx,
        u * u * from.cy + 2. * u * t * py + t * t * to.cy,
    )
}
pub(crate) fn render_phases<I: PhaseInput>(
    input: &mut I,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, I::Error> {
    let size = o.font_size.unwrap_or(16.);
    let small = round(size * 0.8125);
    let item_size = round(size * 0.875);
    let pad = o.padding.unwrap_or(8.);
    let show_descriptions = !matches!(
        c.get("cynefin")
            .and_then(Value::as_object)
            .and_then(|c| c.get("showDomainDescriptions")),
        Some(Value::Bool(false))
    );
    let amplitude = setting(c, "boundaryAmplitude", 8., 0., 50.);
    let seed = b::resolve_seed(
        setting(c, "seed", 0., -1e15, 1e15),
        o.id_prefix.as_deref().unwrap_or("pele"),
    );
    let m = Metrics {
        size,
        small,
        item_size,
        item_line: round(item_size * 1.5),
        show_descriptions,
    };
    let mut quadrants = vec![];
    for &(name, ..) in &QUADRANTS {
        quadrants.push(block(name, input, &m, o, r)?);
    }
    let mut confusion = block(Confusion, input, &m, o, r)?;
    let rx = 88f64.js_max(confusion.w * 0.71 + 8.);
    let ry = 64f64.js_max(confusion.h * 0.71 + 6.);
    let mut quad_w = setting(c, "width", 640., 100., 10000.) / 2.;
    let mut quad_h = setting(c, "height", 440., 100., 10000.) / 2.;
    for b in &quadrants {
        quad_w = quad_w.js_max(b.w + 40.);
        quad_h = quad_h.js_max(b.h + 40.)
    }
    for b in &quadrants {
        let dx = (quad_w - b.w) / 2.;
        let dy = (quad_h - b.h) / 2.;
        if (dx / (rx + 10.)).powi(2) + (dy / (ry + 10.)).powi(2) >= 1. {
            continue;
        }
        let wider = b.w + 2. * (rx + 10.);
        let taller = b.h + 2. * (ry + 10.);
        if wider * quad_h <= quad_w * taller {
            quad_w = quad_w.js_max(wider)
        } else {
            quad_h = quad_h.js_max(taller)
        }
    }
    let title_text = input.metadata("title")?;
    let title = layout_label(title_text.as_deref(), false, r, o, size, 4000., 1)?;
    let title_height = if title.height > 0. {
        title.height + 12.
    } else {
        0.
    };
    let width = 2. * quad_w;
    let height = 2. * quad_h;
    let x0 = pad;
    let y0 = pad + title_height;
    let cx = x0 + quad_w;
    let cy = y0 + quad_h;
    let right = x0 + width;
    let bottom = y0 + height;
    let fold = b::wave(y0, cy, cx, seed, amplitude, true, false, true);
    let west = b::wave(x0, cx, cy, seed + 100., amplitude, false, false, true);
    let east = b::wave(cx, right, cy, seed + 200., amplitude, false, true, false);
    let drop = b::cliff(cx, cy, bottom, width * 0.03);
    let regions = [
        format!(
            "M{},{}L{}{}{}Z",
            num(x0),
            num(y0),
            b::at(&fold, false),
            b::forward(&fold),
            b::backward(&west)
        ),
        format!(
            "M{}L{},{}L{}{}{}Z",
            b::at(&fold, false),
            num(right),
            num(y0),
            b::at(&east, true),
            b::backward(&east),
            b::backward(&fold)
        ),
        format!(
            "M{}{}{}L{},{}Z",
            b::at(&west, false),
            b::forward(&west),
            b::forward(&drop),
            num(x0),
            num(bottom)
        ),
        format!(
            "M{}{}L{},{}L{}{}Z",
            b::at(&east, false),
            b::forward(&east),
            num(right),
            num(bottom),
            b::at(&drop, true),
            b::backward(&drop)
        ),
    ];
    let mut fills = String::new();
    for (i, &(name, _, _, color)) in QUADRANTS.iter().enumerate() {
        fills+=&format!("<path class=\"pele-cynefin-region\" data-id=\"{}\" d=\"{}\" fill=\"{}\" fill-opacity=\"0.14\"/>",name.as_str(),regions[i],svg::series_color(color));
    }
    let boundaries=format!("<g class=\"pele-cynefin-boundaries\" fill=\"none\" stroke=\"var(--_b)\" stroke-dasharray=\"6 3\"><path d=\"{}\"/><path d=\"{}\"/><path d=\"{}\"/><path class=\"pele-cynefin-cliff\" d=\"{}\" stroke=\"var(--_l)\" stroke-width=\"2\" stroke-dasharray=\"none\" stroke-linecap=\"round\"/></g>",b::curve(&fold),b::curve(&west),b::curve(&east),b::curve(&drop));
    let oval = b::ellipse(cx, cy, rx, ry);
    let ellipse=format!("<path d=\"{oval}\" fill=\"var(--_bg)\"/><path class=\"pele-cynefin-region\" data-id=\"confusion\" d=\"{oval}\" fill=\"{}\" fill-opacity=\"0.18\" stroke=\"var(--_b)\" stroke-dasharray=\"4 2\"/>",svg::series_color(4));
    let mut domains = String::new();
    for (i, &(_, col, row, _)) in QUADRANTS.iter().enumerate() {
        let b = &mut quadrants[i];
        b.cx = x0 + quad_w * (col + 0.5);
        b.cy = y0 + quad_h * (row + 0.5);
        domains += &draw_block(b, &m, r)?;
    }
    confusion.cx = cx;
    confusion.cy = cy;
    domains += &draw_block(&confusion, &m, r)?;
    quadrants.push(confusion);
    let blocks = quadrants;
    let mut seen: HashMap<(DomainName, DomainName), usize> = HashMap::new();
    let mut edges = String::new();
    let transitions = input.transitions()?;
    let mut next = 0;
    while let Some(t) = input.next_transition(&transitions, next)? {
        next += 1;
        let (start, end) = input.endpoints(&t)?;
        let Some(from) = blocks.iter().find(|b| Some(b.name) == start) else {
            continue;
        };
        let Some(to) = blocks.iter().find(|b| Some(b.name) == end) else {
            continue;
        };
        if from.name == to.name {
            continue;
        }
        let key = if from.name.as_str() < to.name.as_str() {
            (from.name, to.name)
        } else {
            (to.name, from.name)
        };
        let repeat = seen.entry(key).or_default();
        let rep = *repeat;
        *repeat += 1;
        let dx = to.cx - from.cx;
        let dy = to.cy - from.cy;
        let len = dx.hypot(dy);
        let len = if len == 0. { 1. } else { len };
        let mx = (from.cx + to.cx) / 2.;
        let my = (from.cy + to.cy) / 2.;
        let outward = if -dy * (mx - cx) + dx * (my - cy) >= 0. {
            1.
        } else {
            -1.
        };
        let nx = (-dy / len) * outward;
        let ny = (dx / len) * outward;
        let diagonal =
            from.name != Confusion && to.name != Confusion && dx.abs() > 1. && dy.abs() > 1.;
        let bow = if diagonal {
            2. * ((rx * nx).hypot(ry * ny) + 16.)
        } else {
            len * 0.15
        } + rep as f64 * 56.;
        let px = mx + nx * bow;
        let py = my + ny * bow;
        let point = |t| point(t, from, to, px, py);
        let inside = |b: &Block, t| {
            let (x, y) = point(t);
            inside(b, x, y, cx, cy, rx, ry)
        };
        let crossing = |b: &Block, mut lo: f64, mut hi: f64, leaving: bool| {
            for _ in 0..16 {
                let mid = (lo + hi) / 2.;
                if inside(b, mid) == leaving {
                    lo = mid
                } else {
                    hi = mid
                }
            }
            (lo + hi) / 2.
        };
        let steps = 48.;
        let mut i = 1;
        while i <= 48 && inside(from, i as f64 / steps) {
            i += 1
        }
        if i > 48 {
            continue;
        }
        let t0 = crossing(from, (i - 1) as f64 / steps, i as f64 / steps, true);
        while i <= 48 && !inside(to, i as f64 / steps) {
            i += 1
        }
        let t1 = if i > 48 {
            1.
        } else {
            crossing(to, (i - 1) as f64 / steps, i as f64 / steps, false)
        };
        let (sx, sy) = point(t0);
        let (mut ex, mut ey) = point(t1);
        let blossom = |a: f64, b: f64, c: f64| {
            (1. - t0) * (1. - t1) * a + ((1. - t0) * t1 + t0 * (1. - t1)) * b + t0 * t1 * c
        };
        let qx = blossom(from.cx, px, to.cx);
        let qy = blossom(from.cy, py, to.cy);
        let el = (ex - qx).hypot(ey - qy);
        let el = if el == 0. { 1. } else { el };
        let ux = (ex - qx) / el;
        let uy = (ey - qy) / el;
        let head = marker("arrow_point", ex, ey, ux, uy, "var(--_l)");
        ex -= ux * marker_trim("arrow_point");
        ey -= uy * marker_trim("arrow_point");
        let mut text = String::new();
        let raw_label = input.transition_label(&t)?;
        if let Some(s) = raw_label.as_deref() {
            let label = layout_label(Some(s), false, r, o, small, 200., 0)?;
            let (lx, ly) = point((t0 + t1) / 2.);
            let away = (nx.abs() * label.width) / 2. + (ny.abs() * label.height) / 2. + 6.;
            let half = label.width / 2. + 4.;
            let tx = (lx + nx * away)
                .js_max(x0 + half)
                .js_min((right - half).js_max(x0 + half));
            text = label_svg_with_resources(
                &label,
                tx,
                ly + ny * away,
                &format!(
                    " class=\"pele-edge-label\" font-size=\"{}\"",
                    svg::js_num(small)
                ),
                r,
            )?;
        }
        let (from_id, to_id) = input.transition_id(&t)?;
        edges+=&format!("<g class=\"pele-edge\" data-id=\"{}-{}\"><path d=\"M{},{}Q{},{} {},{}\" fill=\"none\" stroke=\"var(--_l)\"/>{head}{text}</g>",from_id,to_id,num(sx),num(sy),num(qx),num(qy),num(ex),num(ey));
    }
    let total_width = (width.js_max(title.width) + 2. * pad).ceil();
    let total_height = (height + title_height + 2. * pad).ceil();
    let content = label_svg(
        &title,
        x0 + width / 2.,
        pad + title.height / 2.,
        " class=\"pele-title\" font-weight=\"var(--_tw)\"",
    ) + &format!(
        "<g class=\"pele-cynefin-regions\">{fills}</g>{boundaries}{ellipse}{domains}{}",
        if edges.is_empty() {
            String::new()
        } else {
            format!("<g class=\"pele-edges\">{edges}</g>")
        }
    );
    let acc_title = input.metadata("accTitle")?;
    let acc_descr = input.metadata("accDescr")?;
    Ok(Rendered {
        svg: svg::document(
            "cynefin",
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

/// Source observation phases shared by immutable Rust models and owned jobs.
pub(crate) trait PhaseInput {
    type Error: From<PeleError>;
    type Items;
    type Transitions;
    type Transition;
    fn items(&mut self, name: DomainName) -> Result<Self::Items, Self::Error>;
    fn item_count(&mut self, items: &Self::Items) -> Result<usize, Self::Error>;
    fn item_label(&mut self, items: &Self::Items, index: usize) -> Result<String, Self::Error>;
    fn metadata(&mut self, key: &'static str) -> Result<Option<String>, Self::Error>;
    fn transitions(&mut self) -> Result<Self::Transitions, Self::Error>;
    fn next_transition(
        &mut self,
        rows: &Self::Transitions,
        index: usize,
    ) -> Result<Option<Self::Transition>, Self::Error>;
    fn endpoints(
        &mut self,
        row: &Self::Transition,
    ) -> Result<(Option<DomainName>, Option<DomainName>), Self::Error>;
    fn transition_label(&mut self, row: &Self::Transition) -> Result<Option<String>, Self::Error>;
    fn transition_id(&mut self, row: &Self::Transition) -> Result<(String, String), Self::Error>;
}
struct TypedPhase<'a>(&'a CynefinModel);
impl<'a> PhaseInput for TypedPhase<'a> {
    type Error = PeleError;
    type Items = &'a [CynefinItem];
    type Transitions = &'a [CynefinTransition];
    type Transition = &'a CynefinTransition;
    fn items(&mut self, name: DomainName) -> Result<Self::Items, PeleError> {
        Ok(self.0.domain(name).map_or(&[], |d| d.items.as_slice()))
    }
    fn item_count(&mut self, items: &Self::Items) -> Result<usize, PeleError> {
        Ok(items.len())
    }
    fn item_label(&mut self, items: &Self::Items, index: usize) -> Result<String, PeleError> {
        Ok(items[index].label.clone())
    }
    fn metadata(&mut self, key: &'static str) -> Result<Option<String>, PeleError> {
        Ok(match key {
            "title" => self.0.title.clone(),
            "accTitle" => self.0.acc_title.clone(),
            _ => self.0.acc_descr.clone(),
        })
    }
    fn transitions(&mut self) -> Result<Self::Transitions, PeleError> {
        Ok(&self.0.transitions)
    }
    fn next_transition(
        &mut self,
        rows: &Self::Transitions,
        index: usize,
    ) -> Result<Option<Self::Transition>, PeleError> {
        Ok(rows.get(index))
    }
    fn endpoints(
        &mut self,
        row: &Self::Transition,
    ) -> Result<(Option<DomainName>, Option<DomainName>), PeleError> {
        Ok((Some(row.from), Some(row.to)))
    }
    fn transition_label(&mut self, row: &Self::Transition) -> Result<Option<String>, PeleError> {
        Ok(row.label.clone().filter(|s| !s.is_empty()))
    }
    fn transition_id(&mut self, row: &Self::Transition) -> Result<(String, String), PeleError> {
        Ok((row.from.as_str().into(), row.to.as_str().into()))
    }
}
pub(super) fn render(
    model: &CynefinModel,
    config: &Object,
    settings: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_phases(&mut TypedPhase(model), config, settings, resources)
}
