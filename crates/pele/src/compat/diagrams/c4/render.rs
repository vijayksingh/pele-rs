//! Pele c4/render.ts nested diagrams, routed relations and collision-aware labels, 458322a5 (MIT, kepano).
use super::{
    layout::{self, HEAD_LEFT, HEAD_TOP},
    model::{C4Model, ElementRef},
    phase::{Collection, PhaseInput, Typed},
    route::{self, crosses, cyl_ry, queue_rx, Box, Lanes, HEAD, LANE},
};
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    diagrams::common::literal_label::left_label_svg,
    source::url::{linked, sanitize_url},
    svg::{
        self,
        edges::{
            marker, marker_trim,
            theme::{resolve_style, ResolvedStyle},
            ARROW,
        },
        esc, esc_text, label_svg, num,
        shapes::draw_shape,
    },
    text::layout_label,
    ErrorCode, LinkInfo, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
use std::collections::{HashMap, HashSet};
const RADIUS: &str = "var(--pele-radius,4px)";
fn user_style(fill: Option<&str>, stroke: Option<&str>, color: Option<&str>) -> ResolvedStyle {
    let mut declarations = Vec::new();
    for (key, value) in [("fill", fill), ("stroke", stroke), ("color", color)] {
        if let Some(v) = value.filter(|v| !v.is_empty()) {
            declarations.push(format!("{key}:{v}"))
        }
    }
    resolve_style(&declarations)
}
fn link(
    body: &str,
    url: Option<&str>,
    id: &str,
    links: &mut Vec<LinkInfo>,
    options: &RenderSettings,
) -> String {
    let Some(url) = url.filter(|s| !s.is_empty()) else {
        return body.into();
    };
    let restored = if url.contains("¶ß") {
        url.replace("ﬂ°°", "#")
            .replace("ﬂ°", "#")
            .replace("¶ß", ";")
    } else {
        url.into()
    };
    linked(body, &sanitize_url(&restored), id, links, options, "")
}
fn title_room(options: &RenderSettings) -> f64 {
    options.max_width.filter(|v| *v > 0.).map_or(4000., |v| {
        80_f64.js_max(v - 2. * options.padding.unwrap_or(8.))
    })
}
pub(super) fn render(
    db: &C4Model,
    _config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_phases(
        &mut Typed {
            model: db,
            shape_lookup: true,
        },
        options,
        resources,
    )
}
pub(super) fn render_phases<P: PhaseInput>(
    input: &mut P,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    let loose = draw(input, options, resources, false, 36.)?.0;
    let Some(available) = options.max_width.filter(|v| *v > 0.) else {
        return Ok(loose);
    };
    if loose.width <= available {
        return Ok(loose);
    }
    let (mut best, mut squeezed) = draw(input, options, resources, true, 36.)?;
    for gap in [60., 84., 108.] {
        if squeezed == 0 {
            break;
        }
        let (wider, left) = draw(input, options, resources, true, gap)?;
        if left < squeezed {
            best = wider;
            squeezed = left
        }
    }
    Ok(if best.width < loose.width {
        best
    } else {
        loose
    })
}
struct Bounds {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}
impl Bounds {
    fn grow(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) {
        self.min_x = self.min_x.js_min(x0);
        self.min_y = self.min_y.js_min(y0);
        self.max_x = self.max_x.js_max(x1);
        self.max_y = self.max_y.js_max(y1)
    }
}
fn covered(taken: &[f64], x0: f64, y0: f64, x1: f64, y1: f64) -> f64 {
    let mut area = 0.;
    for b in taken.chunks_exact(4) {
        let w = x1.js_min(b[2]) - x0.js_max(b[0]);
        if w <= 0. {
            continue;
        }
        let h = y1.js_min(b[3]) - y0.js_max(b[1]);
        if h > 0. {
            area += w * h
        }
    }
    area
}
fn offset(value: Option<f64>) -> f64 {
    match value {
        Some(v) if v != 0. && !v.is_nan() => v.clamp(-2000., 2000.),
        _ => 0.,
    }
}
fn draw<P: PhaseInput>(
    input: &mut P,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
    tight: bool,
    gap: f64,
) -> Result<(Rendered, usize), P::Error> {
    input.begin_draw();
    let size = options.font_size.unwrap_or(16.);
    let small = round(size * 0.8125);
    let pad = options.padding.unwrap_or(8.);
    let mut links = Vec::new();
    let mut layout = layout::layout(
        input,
        resources,
        options,
        size,
        small,
        if tight {
            Some(80_f64.js_max(options.max_width.unwrap_or(320.) - 2. * pad))
        } else {
            None
        },
        gap,
    )?;
    let mut bounds = Bounds {
        min_x: 0.,
        min_y: 0.,
        max_x: layout.groups[layout.root].bbox.w,
        max_y: layout.groups[layout.root].bbox.h,
    };
    let mut ends = Vec::new();
    let mut pairs: HashMap<ElementRef, HashSet<ElementRef>> = HashMap::new();
    let rels = input.list(Collection::Relations)?;
    let mut index = 0;
    while index < input.len(&rels)? {
        input.step()?;
        let rel = input.at(&rels, index)?;
        index += 1;
        let from = if input.flag(&rel, "fromAttr")? {
            None
        } else {
            let key = input.text(&rel, "from")?.unwrap_or_default();
            input.lookup(&key)?
        };
        let to = if input.flag(&rel, "toAttr")? {
            None
        } else {
            let key = input.text(&rel, "to")?.unwrap_or_default();
            input.lookup(&key)?
        };
        let a = from.and_then(|id| layout.boxes.get(&id));
        let b = to.and_then(|id| layout.boxes.get(&id));
        let (Some(a), Some(b)) = (a, b) else {
            let from = input.text(&rel, "from")?.unwrap_or_default();
            let to = input.text(&rel, "to")?.unwrap_or_default();
            let mut e = PeleError::new(
                format!("C4 rel \"{from}\" -> \"{to}\" references an unknown shape or boundary"),
                ErrorCode::Semantic,
            );
            e.diagram_type = Some("c4".into());
            return Err(e.into());
        };
        ends.push((a.clone(), b.clone()));
        pairs.entry(a.id).or_default().insert(b.id);
    }
    let mut nodes: Vec<Box> = Vec::new();
    let mut taken = Vec::new();
    for &i in &layout.order {
        for &n in &layout.groups[i].nodes {
            let b = &layout.nodes[n].bbox;
            nodes.push(b.clone());
            taken.extend([b.x + 8., b.y + 8., b.x + b.w - 8., b.y + b.h - 8.])
        }
    }
    let current_rels = input.list(Collection::Relations)?;
    let rel_count = input.len(&current_rels)? as u64;
    let careful =
        rel_count * (nodes.len() as u64 + layout.order.len() as u64 + 3 * rel_count) <= 2_000_000;
    let mut edges_out = String::new();
    let trim = marker_trim("arrow_point");
    let mut lanes = Lanes::default();
    for &i in &layout.order {
        if i != layout.root {
            lanes.frame(&layout.groups[i].bbox)
        }
    }
    let mut routes = Vec::new();
    let route_rels = input.list(Collection::Relations)?;
    let route_count = input.len(&route_rels)?;
    for index in 0..route_count {
        input.step()?;
        let rel = input.at(&route_rels, index)?;
        let (a, b) = ends.get(index).ok_or_else(|| {
            PeleError::new(
                "C4 live relation count changed before routing",
                ErrorCode::Semantic,
            )
        })?;
        let line_color = input.text(&rel, "lineColor")?;
        let style = user_style(None, line_color.as_deref(), None);
        let color = style
            .stroke
            .as_deref()
            .map(esc)
            .unwrap_or_else(|| "var(--_l)".into());
        let start_arrow = input.text(&rel, "type")?.as_deref() == Some("birel")
            || input.text(&rel, "type")?.as_deref() == Some("rel_b");
        let end_arrow = input.text(&rel, "type")?.as_deref() != Some("rel_b");
        let reverse = a.id != b.id
            && pairs
                .get(&b.id)
                .is_some_and(|targets| targets.contains(&a.id));
        let r = route::route(
            a,
            b,
            if reverse { 7. } else { 0. },
            if careful { Some(&nodes) } else { None },
            &mut lanes,
            if start_arrow { trim } else { 0. },
            if end_arrow { trim } else { 0. },
        );
        bounds.grow(
            r.x0.js_min(r.x1),
            r.y0.js_min(r.y1),
            r.x0.js_max(r.x1),
            r.y0.js_max(r.y1),
        );
        if start_arrow && careful {
            taken.extend([r.sx - ARROW, r.sy - ARROW, r.sx + ARROW, r.sy + ARROW])
        }
        if end_arrow && careful {
            taken.extend([r.ex - ARROW, r.ey - ARROW, r.ex + ARROW, r.ey + ARROW])
        }
        edges_out += &format!(
            "<g class=\"pele-edge\" data-id=\"{}\"{}><path d=\"{}\"/>{}{}</g>",
            esc_text(&format!(
                "{}-{}",
                input.text(&rel, "from")?.unwrap_or_default(),
                input.text(&rel, "to")?.unwrap_or_default()
            ))?,
            style.line,
            r.d,
            if start_arrow {
                marker("arrow_point", r.sx, r.sy, r.sdx, r.sdy, &color)
            } else {
                String::new()
            },
            if end_arrow {
                marker("arrow_point", r.ex, r.ey, r.edx, r.edy, &color)
            } else {
                String::new()
            }
        );
        routes.push(r)
    }
    for &i in &layout.order {
        let group = &mut layout.groups[i];
        if !careful || group.head_w == 0. {
            continue;
        }
        let top = group.bbox.y + HEAD_TOP;
        let bottom = group.bbox.y + group.head - LANE;
        let mut fewest = usize::MAX;
        for x in [
            HEAD_LEFT,
            (group.bbox.w - group.head_w) / 2.,
            group.bbox.w - HEAD_LEFT - group.head_w,
        ] {
            let count = routes
                .iter()
                .filter(|r| {
                    crosses(
                        &r.points,
                        group.bbox.x + x - 4.,
                        top,
                        group.bbox.x + x + group.head_w + 4.,
                        bottom,
                    )
                })
                .count();
            if count < fewest {
                fewest = count;
                group.head_x = x
            }
            if count == 0 {
                break;
            }
        }
        taken.extend([
            group.bbox.x + group.head_x - 4.,
            top,
            group.bbox.x + group.head_x + group.head_w + 4.,
            bottom,
        ]);
    }
    let mut labels_out = String::new();
    let label_rels = input.list(Collection::Relations)?;
    let label_count = input.len(&label_rels)?;
    for index in 0..label_count {
        input.step()?;
        if !input.present(&label_rels, index)? {
            continue;
        }
        let rel = input.at(&label_rels, index)?;
        let r = routes.get(index).ok_or_else(|| {
            PeleError::new(
                "C4 live relation count changed before labels",
                ErrorCode::Semantic,
            )
        })?;
        let id = format!(
            "{}-{}",
            input.text(&rel, "from")?.unwrap_or_default(),
            input.text(&rel, "to")?.unwrap_or_default()
        );
        let text_color = input.text(&rel, "textColor")?;
        let style = user_style(None, None, text_color.as_deref());
        let owner = input.root();
        let dynamic = input.text(&owner, "c4Type")?.as_deref() == Some("C4Dynamic");
        let text = input.text(&rel, "label")?.unwrap_or_default();
        let text = if dynamic {
            format!("{}: {}", index + 1, text)
        } else {
            text
        };
        input.retain_label(Some(&text))?;
        let label = layout_label(Some(&text), false, resources, options, small, r.room, 0)?;
        let techn = if input.flag(&rel, "techn")? {
            Some(format!(
                "[{}]",
                input.text(&rel, "techn")?.unwrap_or_default()
            ))
        } else {
            None
        };
        input.retain_label(techn.as_deref())?;
        let techn = layout_label(
            techn.as_deref(),
            false,
            resources,
            options,
            small,
            r.room,
            2,
        )?;
        let lw = label.width.js_max(techn.width) + 8.;
        let lh = label.height + techn.height;
        if lh == 0. {
            continue;
        }
        let beside = r.nx.abs() * (lw / 2.) + r.ny.abs() * (lh / 2.) + 2.;
        let aside = if r.lean != 0. {
            0_f64.js_max(r.lean.js_min(beside - 14.))
        } else {
            beside
        };
        let mut dx = r.nx * aside
            + if tight {
                0.
            } else {
                offset(input.number(&rel, "offsetX")?)
            };
        let mut dy = r.ny * aside
            + if tight {
                0.
            } else {
                offset(input.number(&rel, "offsetY")?)
            };
        let mut at = 0.5;
        if careful {
            let mut least = f64::INFINITY;
            let moved = dx != r.nx * aside || dy != r.ny * aside;
            for pass in 0..if moved { 2 } else { 1 } {
                if least <= 0. {
                    break;
                }
                let px = if pass > 0 { r.nx * aside } else { dx };
                let py = if pass > 0 { r.ny * aside } else { dy };
                for t in [0.5, 0.4, 0.6, 0.3, 0.7, 0.2, 0.8] {
                    let cx = r.x0 + (r.x1 - r.x0) * t + px;
                    let cy = r.y0 + (r.y1 - r.y0) * t + py;
                    let area = covered(
                        &taken,
                        cx - lw / 2.,
                        cy - lh / 2.,
                        cx + lw / 2.,
                        cy + lh / 2.,
                    );
                    if area < least {
                        least = area;
                        at = t;
                        dx = px;
                        dy = py
                    }
                    if area == 0. {
                        break;
                    }
                }
            }
        }
        let cx = r.x0 + (r.x1 - r.x0) * at + dx;
        let top = r.y0 + (r.y1 - r.y0) * at + dy - lh / 2.;
        if careful {
            taken.extend([cx - lw / 2., top, cx + lw / 2., top + lh])
        }
        bounds.grow(cx - lw / 2., top, cx + lw / 2., top + lh);
        let text = format!(
            "{}{}",
            label_svg(&label, cx, top + label.height / 2., &style.text),
            label_svg(
                &techn,
                cx,
                top + label.height + techn.height / 2.,
                &format!(" class=\"pele-c4-techn\" fill=\"var(--_m)\"{}", style.text)
            )
        );
        labels_out+=&format!("<g class=\"pele-edge-label\" data-id=\"{}\">{}<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"3\" fill=\"var(--_bg)\"/>{}</g>",esc_text(&id)?,if input.flag(&rel,"descr")?{let descr=input.text(&rel,"descr")?.unwrap_or_default();format!("<title>{}</title>",esc_text(&descr)?)}else{String::new()},num(cx-lw/2.),num(top),num(lw),num(lh),link(&text,input.text(&rel,"link")?.as_deref(),&id,&mut links,options));
    }
    let mut clusters = String::new();
    let mut nodes_out = String::new();
    for &i in &layout.order {
        let group = &layout.groups[i];
        if i != layout.root {
            let boundary = &group.boundary;
            let bg = input.text(boundary, "bgColor")?;
            let border = input.text(boundary, "borderColor")?;
            let font = input.text(boundary, "fontColor")?;
            let style = user_style(bg.as_deref(), border.as_deref(), font.as_deref());
            let x = group.bbox.x + group.head_x;
            let mut y = group.bbox.y + HEAD_TOP;
            let mut text = left_label_svg(
                &group.labels[0],
                x,
                y,
                &format!(
                    " class=\"pele-cluster-label\" font-weight=\"var(--_hw)\"{}",
                    style.text
                ),
            );
            y += group.labels[0].height;
            text += &left_label_svg(
                &group.labels[1],
                x,
                y,
                &format!(
                    " class=\"pele-c4-type\" font-size=\"{}\" fill=\"var(--_m)\"{}",
                    svg::js_num(small),
                    style.text
                ),
            );
            y += group.labels[1].height;
            text += &left_label_svg(
                &group.labels[2],
                x,
                y,
                &format!(
                    " class=\"pele-c4-descr\" font-size=\"{}\" fill=\"var(--_m)\"{}",
                    svg::js_num(small),
                    style.text
                ),
            );
            clusters+=&format!("<g class=\"pele-cluster pele-c4-{}\" data-id=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{RADIUS}\" fill=\"none\" stroke=\"var(--_b)\"{}{}/>{}</g>",if input.flag(boundary,"nodeType")?{"node"}else{"boundary"},esc_text(&input.text(boundary,"alias")?.unwrap_or_default())?,num(group.bbox.x),num(group.bbox.y),num(group.bbox.w),num(group.bbox.h),if input.flag(boundary,"nodeType")?{""}else{" stroke-dasharray=\"6 4\""},style.shape,link(&text,input.text(boundary,"link")?.as_deref(),&input.text(boundary,"alias")?.unwrap_or_default(),&mut links,options));
        }
        for &n in &group.nodes {
            let node = &layout.nodes[n];
            let shape = &node.shape;
            let external = input
                .text(shape, "kind")?
                .unwrap_or_default()
                .starts_with("external");
            let bg = input.text(shape, "bgColor")?;
            let border = input.text(shape, "borderColor")?;
            let font = input.text(shape, "fontColor")?;
            let style = user_style(bg.as_deref(), border.as_deref(), font.as_deref());
            let stroke = format!(
                " stroke=\"var(--_b)\"{}",
                if external {
                    " stroke-dasharray=\"4 3\""
                } else {
                    ""
                }
            );
            let (w, h, form, labels) = (
                node.bbox.w,
                node.bbox.h,
                node.bbox.form.as_str(),
                &node.labels,
            );
            let dx = if form == "h-cyl" {
                -queue_rx(h) / 2.
            } else {
                0.
            };
            let dy = if form == "person" {
                HEAD / 2.
            } else if form == "cyl" {
                cyl_ry(w) / 2.
            } else {
                0.
            };
            let muted = format!(" fill=\"var(--_m)\"{}", style.text);
            let mut y = dy - node.text_h / 2.;
            let mut text = label_svg(
                &labels[0],
                dx,
                y + labels[0].height / 2.,
                &format!(
                    " class=\"pele-c4-type\" font-size=\"{}\"{muted}",
                    svg::js_num(small - 1.)
                ),
            );
            y += labels[0].height;
            text += &label_svg(
                &labels[1],
                dx,
                y + labels[1].height / 2.,
                &format!(
                    " class=\"pele-label\" font-weight=\"var(--_hw)\"{}",
                    style.text
                ),
            );
            y += labels[1].height;
            text += &label_svg(
                &labels[2],
                dx,
                y + labels[2].height / 2.,
                &format!(
                    " class=\"pele-c4-techn\" font-size=\"{}\"{muted}",
                    svg::js_num(small)
                ),
            );
            y += labels[2].height + 4.;
            text += &label_svg(
                &labels[3],
                dx,
                y + labels[3].height / 2.,
                &format!(
                    " class=\"pele-c4-descr\" font-size=\"{}\"{}",
                    svg::js_num(small),
                    style.text
                ),
            );
            let body = draw_shape(
                form,
                w,
                h,
                &format!(
                    " fill=\"var({})\"{stroke}{}",
                    if external { "--_bg" } else { "--_s" },
                    style.shape
                ),
                &format!("{stroke}{}", style.line),
            ) + &text;
            nodes_out+=&format!("<g class=\"pele-node pele-shape-{form} pele-c4-{}{}\" data-id=\"{}\" transform=\"translate({},{})\">{}</g>",input.text(shape,"kind")?.unwrap_or_default(),if external{" pele-c4-external"}else{""},esc_text(&input.text(shape,"alias")?.unwrap_or_default())?,num(node.bbox.x+w/2.),num(node.bbox.y+h/2.),link(&body,input.text(shape,"link")?.as_deref(),&input.text(shape,"alias")?.unwrap_or_default(),&mut links,options));
        }
    }
    let owner = input.root();
    let title_text = input.text(&owner, "title")?;
    input.retain_label(title_text.as_deref())?;
    let title = layout_label(
        title_text.as_deref(),
        false,
        resources,
        options,
        size,
        if tight { title_room(options) } else { 4000. },
        1,
    )?;
    let title_height = if title.height > 0. {
        title.height + 12.
    } else {
        0.
    };
    let inner = (bounds.max_x - bounds.min_x).js_max(title.width);
    let width = (inner + 2. * pad).ceil();
    let height = (bounds.max_y - bounds.min_y + title_height + 2. * pad).ceil();
    let ox = pad - bounds.min_x + (inner - (bounds.max_x - bounds.min_x)) / 2.;
    let oy = pad + title_height - bounds.min_y;
    let mut content = label_svg(
        &title,
        width / 2.,
        pad + title.height / 2.,
        " class=\"pele-title\" font-weight=\"var(--_tw)\"",
    );
    content += &format!("<g transform=\"translate({},{})\">", num(ox), num(oy));
    if !clusters.is_empty() {
        content += &format!("<g class=\"pele-clusters\">{clusters}</g>")
    }
    if !edges_out.is_empty() {
        content+=&format!("<g class=\"pele-edges\" fill=\"none\" stroke=\"var(--_l)\" stroke-linecap=\"round\">{edges_out}</g>")
    }
    if !nodes_out.is_empty() {
        content += &format!("<g class=\"pele-nodes\">{nodes_out}</g>")
    }
    if !labels_out.is_empty() {
        content += &format!(
            "<g class=\"pele-edge-labels\" font-size=\"{}\">{labels_out}</g>",
            svg::js_num(small)
        )
    }
    content += "</g>";
    let acc_title = input.text(&owner, "accTitle")?;
    let acc_descr = input.text(&owner, "accDescr")?;
    Ok((
        Rendered {
            svg: svg::document(
                "c4",
                width,
                height,
                size,
                options,
                acc_title.as_deref(),
                acc_descr.as_deref(),
                &content,
            )?,
            width,
            height,
            links,
        },
        lanes.squeezed,
    ))
}

/// A declared plain C4Model has no DB getC4Shape helper. Its first relation
/// lookup follows layout measurements, so do not reject before layout.
pub(super) fn render_plain(
    db: &C4Model,
    config: &Object,
    options: &RenderSettings,
    host: &mut dyn RenderResources,
) -> Result<Rendered, crate::compat::public::PublicError> {
    if !db
        .rels
        .iter()
        .any(|r| r.from_attr.is_none() || r.to_attr.is_none())
    {
        return render(db, config, options, host).map_err(Into::into);
    }
    match draw(
        &mut Typed {
            model: db,
            shape_lookup: false,
        },
        options,
        host,
        false,
        36.,
    ) {
        Err(e) if e.message == "t.getC4Shape is not a function" => {
            Err(crate::compat::public::PublicError::JavaScript {
                name: crate::compat::public::JavaScriptErrorKind::TypeError,
                message: e.message.as_str().into(),
            })
        }
        Err(e) => Err(e.into()),
        Ok((result, _)) => Ok(result),
    }
}
