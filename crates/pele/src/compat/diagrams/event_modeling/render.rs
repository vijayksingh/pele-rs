//! Pele eventmodeling/render.ts, 458322a5 (MIT, kepano).
use super::{
    model::*,
    phase::{PhaseInput, Typed},
};
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    diagrams::common::literal_label::{break_label, left_label_svg, literal_label},
    source::decode_entities,
    svg::{
        document,
        edges::{marker, marker_trim},
        esc, esc_text, js_num, label_svg, num, series_color,
    },
    text::{layout_label, measure, Label},
    PeleError, RenderResources, RenderSettings, Rendered,
};
use std::collections::{HashMap, HashSet};
fn series(kind: &str) -> Option<usize> {
    match kind {
        "command" => Some(0),
        "event" => Some(1),
        "readmodel" => Some(2),
        "processor" => Some(6),
        _ => None,
    }
}
fn invalid() -> PeleError {
    PeleError::new(
        "Eventmodeling live box refers to a missing lane",
        crate::compat::ErrorCode::Semantic,
    )
}
// Pinned JS slices UTF-16 and can bisect a surrogate pair. Rust's UTF-8 SVG
// projection substitutes U+FFFD at those non-scalar boundaries. Exact code-unit
// outputs remain explicit in the fixture ledger; they are not passing parity cases.
fn hard_wrap(text: &str, columns: usize) -> Result<Vec<String>, PeleError> {
    let mut out = vec![];
    for line in decode_entities(text)?.split('\n') {
        let units = line.encode_utf16().collect::<Vec<_>>();
        if units.len() <= columns {
            out.push(line.into());
        } else {
            for chunk in units.chunks(columns) {
                out.push(String::from_utf16_lossy(chunk));
            }
        }
    }
    Ok(out)
}
fn lines(rows: &[String], x: f64, y: f64, step: f64, attrs: &str) -> String {
    let mut body = String::new();
    for (i, row) in rows.iter().enumerate() {
        if !row.is_empty() {
            body += &format!(
                "<tspan x=\"{}\" y=\"{}\">{}</tspan>",
                num(x),
                num(y + i as f64 * step),
                esc(row)
            );
        }
    }
    if body.is_empty() {
        body
    } else {
        format!("<text{attrs} xml:space=\"preserve\">{body}</text>")
    }
}
fn widest(
    rows: &[String],
    size: f64,
    style: u8,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<f64, PeleError> {
    let mut width = 0_f64;
    for row in rows {
        width = width.js_max(measure(r, o, row, size, style)?);
    }
    Ok(width)
}
fn heading(heading: &str, frame: &str, names: &HashMap<String, String>) -> String {
    format!(
        "{heading} · {frame}{}",
        names.get(frame).map_or(String::new(), |v| format!(" {v}"))
    )
}
fn names<P: PhaseInput>(m: &mut P) -> Result<HashMap<String, String>, P::Error> {
    let boxes = m.root_field("boxes")?;
    let mut names = HashMap::new();
    let mut i = 0;
    while i < m.count(&boxes)? {
        m.step()?;
        let b = m.index(&boxes, i)?;
        i += 1;
        let id = m.field(&b, "id")?;
        let name = m.field(&b, "name")?;
        names.insert(id, name);
    }
    Ok(names)
}
fn spec_rows<P: PhaseInput>(m: &mut P, spec: &P::Value) -> Result<Vec<String>, P::Error> {
    let given = m.get(spec, "given")?;
    let mut rows = vec![format!("Given  {}", m.join(&given)?)];
    let when = m.get(spec, "when")?;
    if m.count(&when)? > 0 {
        let when = m.get(spec, "when")?;
        rows.push(format!("When  {}", m.join(&when)?));
    }
    let then = m.get(spec, "then")?;
    rows.push(format!("Then  {}", m.join(&then)?));
    Ok(rows)
}
pub(super) fn render(
    m: &EventModelingModel,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_phases(&mut Typed(m), o, r)
}
pub(super) fn render_phases<P: PhaseInput>(
    m: &mut P,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    let across = horizontal(m, o, r)?;
    let Some(available) = o.max_width.filter(|w| *w > 0.) else {
        return Ok(across);
    };
    if o.auto_direction == Some(false)
        || across.width <= available
        || available >= o.direction_breakpoint.filter(|b| *b > 0.).unwrap_or(640.)
    {
        return Ok(across);
    }
    let down = vertical(m, o, r)?;
    Ok(if down.width < across.width {
        down
    } else {
        across
    })
}
struct Placed<V> {
    value: V,
    name: Label,
    data: Vec<String>,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}
struct Band<V> {
    lane: V,
    label: Label,
    right: f64,
    y: f64,
    h: f64,
}
fn horizontal<P: PhaseInput>(
    m: &mut P,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    m.begin_draw();
    let size = o.font_size.unwrap_or(16.);
    let name_size = round(size * 0.875);
    let small = round(size * 0.8125);
    let mono = round(size * 0.75);
    let mono_step = round(mono * 1.4);
    let small_step = round(small * 1.5);
    let pad = o.padding.unwrap_or(8.);
    let mut bands = vec![];
    let mut band_of = HashMap::new();
    let mut label_column = 0_f64;
    let lane_values = m.root_field("lanes")?;
    let mut i = 0;
    while i < m.count(&lane_values)? {
        m.step()?;
        let lane = m.index(&lane_values, i)?;
        i += 1;
        let label_text = m.field(&lane, "label")?;
        let label = layout_label(Some(&label_text), false, r, o, small, 4000., 1)?;
        label_column = label_column.js_max(label.width);
        let id = m.identity(&lane)?;
        let band = Band {
            lane,
            label,
            right: f64::NEG_INFINITY,
            y: 0.,
            h: 0.,
        };
        if let Some(&at) = band_of.get(&id) {
            bands[at] = band
        } else {
            band_of.insert(id, bands.len());
            bands.push(band)
        }
    }
    label_column += 24.;
    let mut placed = Vec::<Placed<P::Value>>::new();
    let mut by_box = HashMap::new();
    let box_values = m.root_field("boxes")?;
    let mut i = 0;
    while i < m.count(&box_values)? {
        m.step()?;
        let b = m.index(&box_values, i)?;
        i += 1;
        let lane = m.get(&b, "lane")?;
        let lane_id = m.identity(&lane)?;
        let band_index = *band_of.get(&lane_id).ok_or_else(invalid)?;
        let name_text = m.field(&b, "name")?;
        let name = layout_label(Some(&name_text), false, r, o, name_size, 4000., 1)?;
        let raw = m.get(&b, "data")?;
        let data = if m.truthy(&raw) {
            let raw = m.get(&b, "data")?;
            hard_wrap(&m.text(&raw)?.unwrap_or_default(), 48)?
        } else {
            vec![]
        };
        let band = &mut bands[band_index];
        let w = 96_f64.js_max(name.width.js_max(widest(&data, mono, 4, o, r)?) + 24.);
        let h = 20.
            + name.height
            + if data.is_empty() {
                0.
            } else {
                6. + data.len() as f64 * mono_step
            };
        let mut x = 0.;
        if let Some(previous) = placed.last() {
            let stagger = previous.x + (previous.w / 2.).js_max(previous.w - 80.);
            x = if {
                let a = m.get(&previous.value, "lane")?;
                let b = m.get(&b, "lane")?;
                m.identity(&a)? == m.identity(&b)?
            } {
                band.right + 16.
            } else {
                stagger.js_max(band.right + 16.)
            };
        }
        band.right = x + w;
        band.h = band.h.js_max(h);
        by_box.insert(m.identity(&b)?, placed.len());
        placed.push(Placed {
            value: b,
            name,
            data,
            x,
            y: 0.,
            w,
            h,
        });
    }
    let content_right = placed
        .iter()
        .fold(0_f64, |right, p| right.js_max(p.x + p.w));
    let lane_width = label_column + content_right + 12.;
    let title_text = m.metadata("title")?;
    let title = layout_label(title_text.as_deref(), false, r, o, size, 4000., 1)?;
    let title_height = if title.height > 0. {
        title.height + 12.
    } else {
        0.
    };
    let left = pad;
    let top = pad + title_height;
    let mut y = top;
    let mut lanes = String::new();
    for band in &mut bands {
        band.y = y;
        band.h += 24.;
        lanes+=&format!("<g class=\"pele-cluster pele-em-lane\" data-id=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"var(--pele-radius,4px)\" fill=\"var(--_s)\"/>{}</g>",esc_text(&m.field(&band.lane,"label")?)?,num(left),num(y),num(lane_width),num(band.h),label_svg(&band.label,left+12.+band.label.width/2.,y+band.h/2.,&format!(" class=\"pele-cluster-label\" font-size=\"{}\" font-weight=\"var(--_hw)\" fill=\"var(--_m)\"",js_num(small))));
        y += band.h + 8.;
    }
    let mut bottom = if bands.is_empty() { top } else { y - 8. };
    let mut nodes = String::new();
    for item in &mut placed {
        let b = &item.value;
        let lane = m.get(b, "lane")?;
        let lane_id = m.identity(&lane)?;
        let band = &bands[*band_of.get(&lane_id).ok_or_else(invalid)?];
        item.x += left + label_column;
        item.y = band.y + (band.h - item.h) / 2.;
        let frame = format!(
            "x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"var(--pele-radius,4px)\"",
            num(item.x),
            num(item.y),
            num(item.w),
            num(item.h)
        );
        let shape = if let Some(series) = series(&m.field(b, "kind")?) {
            let color = series_color(series);
            format!("<rect {frame} fill=\"var(--_bg)\"/><rect {frame} fill=\"{color}\" fill-opacity=\"0.22\" stroke=\"{color}\"/>")
        } else {
            format!("<rect {frame} fill=\"var(--_bg)\" stroke=\"var(--_b)\"/>")
        };
        let text = format!(
            "{}{}",
            label_svg(
                &item.name,
                item.x + item.w / 2.,
                item.y + 10. + item.name.height / 2.,
                &format!(
                    " class=\"pele-label\" font-size=\"{}\" font-weight=\"var(--_hw)\"",
                    js_num(name_size)
                )
            ),
            lines(
                &item.data,
                item.x + 12.,
                item.y + 10. + item.name.height + 6. + mono_step / 2. + mono * 0.35,
                mono_step,
                &format!(
                    " class=\"pele-em-data\" font-family=\"var(--_fm)\" font-size=\"{}\"",
                    js_num(mono)
                )
            )
        );
        nodes += &format!(
            "<g class=\"pele-node pele-em-{}{}\" data-id=\"{}\">{shape}{text}</g>",
            m.field(b, "kind")?,
            {
                let reset = m.get(b, "reset")?;
                if m.truthy(&reset) {
                    " pele-em-reset"
                } else {
                    ""
                }
            },
            esc_text(&m.field(b, "id")?)?
        );
    }
    let mut edges = String::new();
    let trim = marker_trim("arrow_point");
    let relations = m.root_field("relations")?;
    let mut i = 0;
    while i < m.count(&relations)? {
        m.step()?;
        let relation = m.index(&relations, i)?;
        i += 1;
        let source = m.get(&relation, "source")?;
        let target = m.get(&relation, "target")?;
        let (Some(&si), Some(&ti)) = (
            by_box.get(&m.identity(&source)?),
            by_box.get(&m.identity(&target)?),
        ) else {
            continue;
        };
        if si == ti {
            continue;
        }
        let s = &placed[si];
        let t = &placed[ti];
        let (d, head);
        if {
            let a = m.get(&placed[si].value, "lane")?;
            let b = m.get(&placed[ti].value, "lane")?;
            m.identity(&a)? == m.identity(&b)?
        } {
            let forward = t.x >= s.x;
            let sx = if forward { s.x + s.w } else { s.x };
            let tx = if forward { t.x } else { t.x + t.w };
            let sy = s.y + s.h / 2.;
            let ty = t.y + t.h / 2.;
            let dir = if tx >= sx { 1. } else { -1. };
            d = format!(
                "M{},{}L{},{}",
                num(sx),
                num(sy),
                num(tx - dir * trim),
                num(ty)
            );
            head = marker("arrow_point", tx, ty, dir, 0., "var(--_l)");
        } else {
            let down = t.y > s.y;
            let from = s.x.js_max(t.x);
            let to = (s.x + s.w).js_min(t.x + t.w);
            let inset = |w: f64| 24_f64.js_min(w / 4.);
            let sx = if to - from >= 16. {
                (from + to) / 2.
            } else if t.x > s.x {
                s.x + s.w - inset(s.w)
            } else {
                s.x + inset(s.w)
            };
            let tx = if to - from >= 16. {
                sx
            } else if t.x > s.x {
                t.x + inset(t.w)
            } else {
                t.x + t.w - inset(t.w)
            };
            let sy = if down { s.y + s.h } else { s.y };
            let ty = if down { t.y } else { t.y + t.h };
            let dir = if down { 1. } else { -1. };
            let end = ty - dir * trim;
            let mid = (sy + end) / 2.;
            d = if sx == tx {
                format!("M{},{}V{}", num(sx), num(sy), num(end))
            } else {
                format!(
                    "M{},{}C{},{} {},{} {},{}",
                    num(sx),
                    num(sy),
                    num(sx),
                    num(mid),
                    num(tx),
                    num(mid),
                    num(tx),
                    num(end)
                )
            };
            head = marker("arrow_point", tx, ty, 0., dir, "var(--_l)");
        }
        edges+=&format!("<g class=\"pele-edge\" data-id=\"{}-{}\"><path d=\"{d}\" fill=\"none\" stroke=\"var(--_l)\"/>{head}</g>",{let source=m.get(&relation,"source")?;esc_text(&m.field(&source,"id")?)?},{let target=m.get(&relation,"target")?;esc_text(&m.field(&target,"id")?)?});
    }
    let names = names(m)?;
    let mut cards = String::new();
    let mut cards_right = 0_f64;
    let mut card = |kind: &str,
                    heading_text: &str,
                    frame: &str,
                    rows: Vec<String>|
     -> Result<(), PeleError> {
        let head = layout_label(
            Some(&heading(heading_text, frame, &names)),
            false,
            r,
            o,
            small,
            4000.,
            1,
        )?;
        let w = head.width.js_max(widest(&rows, small, 0, o, r)?) + 24.;
        let h = 20. + head.height + rows.len() as f64 * small_step;
        let cy = bottom
            + if cards.is_empty() && bands.is_empty() {
                0.
            } else {
                8.
            };
        cards+=&format!("<g class=\"pele-em-{kind}\" data-id=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"var(--pele-radius,4px)\" fill=\"var(--_bg)\" stroke=\"var(--_b)\"/>{}{}</g>",esc_text(frame)?,num(left),num(cy),num(w),num(h),label_svg(&head,left+12.+head.width/2.,cy+10.+head.height/2.,&format!(" font-size=\"{}\" font-weight=\"var(--_hw)\"",js_num(small))),lines(&rows,left+12.,cy+10.+head.height+small_step/2.+small*0.35,small_step,&format!(" font-size=\"{}\"",js_num(small))));
        bottom = cy + h;
        cards_right = cards_right.js_max(w);
        Ok(())
    };
    let notes = m.root_field("notes")?;
    let mut i = 0;
    while i < m.count(&notes)? {
        m.step()?;
        let note = m.index(&notes, i)?;
        i += 1;
        let frame = m.field(&note, "frame")?;
        let text = m.field(&note, "text")?;
        card("note", "Note", &frame, hard_wrap(&text, 80)?)?;
    }
    let specs = m.root_field("specifications")?;
    let mut i = 0;
    while i < m.count(&specs)? {
        m.step()?;
        let spec = m.index(&specs, i)?;
        i += 1;
        let rows = spec_rows(m, &spec)?
            .iter()
            .map(|s| hard_wrap(s, 80))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();
        let frame = m.field(&spec, "frame")?;
        card("specification", "Given / When / Then", &frame, rows)?;
    }
    let width = (if bands.is_empty() { 0. } else { lane_width })
        .js_max(cards_right)
        .js_max(title.width);
    let total_width = (width + 2. * pad).ceil();
    let total_height = (bottom + pad).ceil();
    let content = format!(
        "{}{lanes}{edges}{nodes}{cards}",
        label_svg(
            &title,
            total_width / 2.,
            pad + title.height / 2.,
            " class=\"pele-title\" font-weight=\"var(--_tw)\""
        )
    );
    let acc_title = m.metadata("accTitle")?;
    let acc_descr = m.metadata("accDescr")?;
    Ok(Rendered {
        svg: document(
            "eventmodeling",
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
struct VerticalPlaced {
    y: f64,
    h: f64,
    w: f64,
    index: usize,
    lane: f64,
}
fn label_lines(label: &Label) -> Vec<String> {
    label
        .lines
        .iter()
        .map(|line| line.iter().map(|span| span.text.as_str()).collect())
        .collect()
}
fn vertical<P: PhaseInput>(
    m: &mut P,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    m.begin_draw();
    let size = o.font_size.unwrap_or(16.);
    let small = round(size * 0.875);
    let mono = round(size * 0.75);
    let pad = o.padding.unwrap_or(8.);
    let room = 160_f64.js_max(o.max_width.unwrap_or(320.) - 2. * pad);
    let initial_boxes = m.root_field("boxes")?;
    let n = m.count(&initial_boxes)?;
    let mut order = HashMap::new();
    for i in 0..n {
        m.step()?;
        if m.present(&initial_boxes, i)? {
            let b = m.index(&initial_boxes, i)?;
            order.insert(m.identity(&b)?, i);
        }
    }
    let mut runs = Vec::<(u64, usize, usize)>::new();
    let mut run_of = HashMap::new();
    let mut passed = HashSet::new();
    let relations = m.root_field("relations")?;
    let mut i = 0;
    while i < m.count(&relations)? {
        m.step()?;
        let relation = m.index(&relations, i)?;
        i += 1;
        let source = m.get(&relation, "source")?;
        let target = m.get(&relation, "target")?;
        let target_id = m.identity(&target)?;
        let (Some(&s), Some(&t)) = (order.get(&m.identity(&source)?), order.get(&target_id)) else {
            continue;
        };
        if s == t {
            continue;
        }
        if s.abs_diff(t) < 2 {
            let current = m.root_field("boxes")?;
            let b = m.index(&current, s.max(t))?;
            passed.insert(m.identity(&b)?);
            continue;
        }
        if let Some(&i) = run_of.get(&target_id) {
            let run: &mut (u64, usize, usize) = &mut runs[i];
            run.1 = run.1.min(s);
            run.2 = run.2.max(s);
        } else {
            run_of.insert(target_id, runs.len());
            runs.push((target_id, s.min(t), s.max(t)));
        }
    }
    runs.sort_by_key(|(_, from, _)| *from);
    let mut rail_of = HashMap::new();
    let mut rail_ends = Vec::<usize>::new();
    for (target, from, to) in runs {
        let rail = rail_ends
            .iter()
            .position(|&end| end < from)
            .unwrap_or(rail_ends.len());
        if rail == rail_ends.len() {
            rail_ends.push(to);
        } else {
            rail_ends[rail] = to;
        }
        rail_of.insert(target, rail);
    }
    let gutter = 24. + 8. * (rail_ends.len() as f64 - 1.).js_max(0.);
    let card_width = 120_f64.js_max(room - gutter);
    let text_room = card_width - 24.;
    let title_text = m.metadata("title")?;
    let title = layout_label(title_text.as_deref(), false, r, o, size, room, 1)?;
    let mut y = pad
        + if title.height != 0. {
            title.height + 12.
        } else {
            0.
        };
    let mut bottom = pad + title.height;
    let mut width = title.width.js_max(card_width + gutter);
    let mut nodes = String::new();
    let mut edges = String::new();
    let mut cards = String::new();
    let mut placed = HashMap::new();
    let boxes = m.root_field("boxes")?;
    let n = m.count(&boxes)?;
    for index in 0..n {
        m.step()?;
        if !m.present(&boxes, index)? {
            continue;
        }
        let b = m.index(&boxes, index)?;
        let identity = m.identity(&b)?;
        let lane_room = if passed.contains(&identity) {
            text_room - 20.
        } else {
            text_room
        };
        let lane_handle = m.get(&b, "lane")?;
        let lane_text = m.field(&lane_handle, "label")?;
        let lane = layout_label(Some(&lane_text), false, r, o, small, lane_room, 1)?;
        let lane = break_label(lane, r, o, lane_room)?;
        let name_text = m.field(&b, "name")?;
        let name = layout_label(Some(&name_text), false, r, o, small, text_room - 36., 1)?;
        let raw = m.get(&b, "data")?;
        let data = if m.truthy(&raw) {
            let raw = m.get(&b, "data")?;
            let data = m.text(&raw)?.unwrap_or_default();
            Some(literal_label(
                &decode_entities(&data)?,
                r,
                o,
                mono,
                text_room,
                4,
            )?)
        } else {
            None
        };
        let w = card_width
            .js_max(lane.width + 24.)
            .js_max(name.width + 24. + 36.)
            .js_max(data.as_ref().map_or(0., |d| d.width) + 24.);
        let head_h = lane.height + 12.;
        let h = 20. + name.height + data.as_ref().map_or(0., |d| d.height + 6.);
        let cy = y + head_h;
        let frame = format!(
            "x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"var(--pele-radius,4px)\"",
            num(pad),
            num(cy),
            num(w),
            num(h)
        );
        let shade = series(&m.field(&b, "kind")?).map_or(String::new(), |series| {
            let color = series_color(series);
            format!("<rect {frame} fill=\"{color}\" fill-opacity=\"0.22\" stroke=\"{color}\"/>")
        });
        let data_svg = data.as_ref().map_or(String::new(), |data| {
            lines(
                &label_lines(data),
                pad + 12.,
                cy + 10. + name.height + 6. + data.line_height / 2. + mono * 0.35,
                data.line_height,
                &format!(
                    " class=\"pele-em-data\" font-family=\"var(--_fm)\" font-size=\"{}\"",
                    js_num(mono)
                ),
            )
        });
        nodes+=&format!("<g class=\"pele-cluster pele-em-lane\" data-id=\"{}\">{}<g class=\"pele-node pele-em-{}{}\" data-id=\"{}\"><rect {frame} fill=\"var(--_bg)\" stroke=\"var(--_b)\"/>{shade}{}<text class=\"pele-em-frame\" x=\"{}\" y=\"{}\" text-anchor=\"end\" font-size=\"{}\" fill=\"var(--_m)\">{}</text>{data_svg}</g></g>",{let lane=m.get(&b,"lane")?;esc_text(&m.field(&lane,"label")?)?},left_label_svg(&lane,pad+12.,y,&format!(" class=\"pele-cluster-label\" font-size=\"{}\" font-weight=\"var(--_hw)\" fill=\"var(--_m)\"",js_num(small))),m.field(&b,"kind")?,{let reset=m.get(&b,"reset")?;if m.truthy(&reset){" pele-em-reset"}else{""}},esc_text(&m.field(&b,"id")?)?,label_svg(&name,pad+12.+name.width/2.,cy+10.+name.height/2.,&format!(" class=\"pele-label\" font-size=\"{}\" font-weight=\"var(--_hw)\"",js_num(small))),num(pad+w-12.),num(cy+10.+small),js_num(mono),esc(&m.field(&b,"id")?));
        placed.insert(
            identity,
            VerticalPlaced {
                y: cy,
                h,
                w,
                index,
                lane: lane.width,
            },
        );
        width = width.js_max(w + gutter);
        bottom = cy + h;
        y = bottom + 28.;
    }
    let relations = m.root_field("relations")?;
    let mut i = 0;
    while i < m.count(&relations)? {
        m.step()?;
        let relation = m.index(&relations, i)?;
        i += 1;
        let source = m.get(&relation, "source")?;
        let target = m.get(&relation, "target")?;
        let si = m.identity(&source)?;
        let ti = m.identity(&target)?;
        if !placed.contains_key(&si) || !placed.contains_key(&ti) {
            continue;
        }
        if si == ti {
            continue;
        }
        let s = &placed[&si];
        let t = &placed[&ti];
        let down = t.y > s.y;
        let dir = if down { 1. } else { -1. };
        let sy = if down { s.y + s.h } else { s.y };
        let ty = if down { t.y } else { t.y + t.h };
        let (d, head);
        if s.index.abs_diff(t.index) == 1 {
            let cx =
                pad + (s.w.js_min(t.w) / 2.).js_max(12. + if down { t.lane } else { s.lane } + 16.);
            d = format!(
                "M{},{}V{}",
                num(cx),
                num(sy),
                num(ty - dir * marker_trim("arrow_point"))
            );
            head = marker("arrow_point", cx, ty, 0., dir, "var(--_l)");
        } else {
            let rail =
                pad + width - gutter + 12. + 8. * rail_of.get(&ti).copied().unwrap_or(0) as f64;
            let from = s.y + s.h / 2. - 5.;
            let to = t.y + t.h / 2. + 5.;
            d = format!(
                "M{},{}H{}V{}H{}",
                num(pad + s.w),
                num(from),
                num(rail),
                num(to),
                num(pad + t.w + marker_trim("arrow_point"))
            );
            head = marker("arrow_point", pad + t.w, to, -1., 0., "var(--_l)");
        }
        edges+=&format!("<g class=\"pele-edge\" data-id=\"{}-{}\"><path d=\"{d}\" fill=\"none\" stroke=\"var(--_l)\"/>{head}</g>",{let source=m.get(&relation,"source")?;esc_text(&m.field(&source,"id")?)?},{let target=m.get(&relation,"target")?;esc_text(&m.field(&target,"id")?)?});
    }
    let names = names(m)?;
    let mut card = |kind: &str,
                    heading_text: &str,
                    frame: &str,
                    raw: &str|
     -> Result<(), PeleError> {
        let head = layout_label(
            Some(&heading(heading_text, frame, &names)),
            false,
            r,
            o,
            small,
            text_room,
            1,
        )?;
        let body = literal_label(&decode_entities(raw)?, r, o, small, text_room, 0)?;
        let w = card_width.js_max(head.width + 24.).js_max(body.width + 24.);
        let h = 20. + head.height + body.height;
        cards+=&format!("<g class=\"pele-em-{kind}\" data-id=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"var(--pele-radius,4px)\" fill=\"var(--_bg)\" stroke=\"var(--_b)\"/>{}{}</g>",esc_text(frame)?,num(pad),num(y),num(w),num(h),label_svg(&head,pad+12.+head.width/2.,y+10.+head.height/2.,&format!(" font-size=\"{}\" font-weight=\"var(--_hw)\"",js_num(small))),lines(&label_lines(&body),pad+12.,y+10.+head.height+body.line_height/2.+small*0.35,body.line_height,&format!(" font-size=\"{}\"",js_num(small))));
        bottom = y + h;
        y = bottom + 8.;
        width = width.js_max(w);
        Ok(())
    };
    let notes = m.root_field("notes")?;
    let mut i = 0;
    while i < m.count(&notes)? {
        m.step()?;
        let note = m.index(&notes, i)?;
        i += 1;
        let frame = m.field(&note, "frame")?;
        let text = m.field(&note, "text")?;
        card("note", "Note", &frame, &text)?;
    }
    let specs = m.root_field("specifications")?;
    let mut i = 0;
    while i < m.count(&specs)? {
        m.step()?;
        let spec = m.index(&specs, i)?;
        i += 1;
        let text = spec_rows(m, &spec)?.join("\n");
        let frame = m.field(&spec, "frame")?;
        card("specification", "Given / When / Then", &frame, &text)?;
    }
    let total_width = (width + 2. * pad).ceil();
    let height = (bottom + pad).ceil();
    let content = format!(
        "{}{edges}{nodes}{cards}",
        label_svg(
            &title,
            total_width / 2.,
            pad + title.height / 2.,
            " class=\"pele-title\" font-weight=\"var(--_tw)\""
        )
    );
    let acc_title = m.metadata("accTitle")?;
    let acc_descr = m.metadata("accDescr")?;
    Ok(Rendered {
        svg: document(
            "eventmodeling",
            total_width,
            height,
            size,
            o,
            acc_title.as_deref(),
            acc_descr.as_deref(),
            &content,
        )?,
        width: total_width,
        height,
        links: vec![],
    })
}
