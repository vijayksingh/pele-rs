//! Pele requirement/render.ts measured requirement tables, compound graph routing,
//! port spreading, containment marks and responsive rotation, 458322a5 (MIT, kepano).
use super::model::RequirementModel;
use super::phase::{PhaseInput, Typed};
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    layout::{
        compound::{compound_layout, direction, CEdge, CNode, Dir},
        layered::LayeredOptions,
    },
    source::space,
    svg::{
        self,
        edges::{
            edge_label_svg, route_path,
            theme::{class_names, resolve_style, ResolvedStyle},
            EdgePath,
        },
        esc, esc_text, js_num, label_svg, num,
    },
    text::{layout_label, measure, Label},
    Object, PeleError, RenderResources, RenderSettings, Rendered,
};
use std::collections::HashMap;
struct Row {
    key: Label,
    value: Label,
    height: f64,
}
struct View {
    id: String,
    requirement: bool,
    classes: String,
    style: ResolvedStyle,
    kind: Label,
    name: Label,
    rows: Vec<Row>,
    key_width: f64,
    head_height: f64,
    w: f64,
    h: f64,
}
fn restyle(
    mut label: Label,
    bits: u8,
    r: &mut dyn RenderResources,
    o: &RenderSettings,
) -> Result<Label, PeleError> {
    if bits == 0 || label.lines.is_empty() {
        return Ok(label);
    }
    let mut width = 0_f64;
    let mut widths = vec![];
    for line in &mut label.lines {
        let mut total = 0.;
        for span in line {
            span.style |= bits;
            if span.icon.is_none() {
                span.width = measure(r, o, &span.text, label.size, span.style)?;
            }
            total += span.width;
        }
        widths.push(total);
        if total > width {
            width = total
        }
    }
    label.widths = widths;
    label.width = width;
    Ok(label)
}
fn text(
    s: &str,
    markdown: bool,
    size: f64,
    room: f64,
    bits: u8,
    r: &mut dyn RenderResources,
    o: &RenderSettings,
) -> Result<Label, PeleError> {
    let label = layout_label(Some(s), markdown, r, o, size, room, 0)?;
    restyle(label, bits, r, o)
}
fn span_style(bits: u8) -> String {
    format!(
        "{}{}",
        if bits & 1 != 0 {
            " font-weight=\"var(--_w)\""
        } else {
            ""
        },
        if bits & 2 != 0 {
            " font-style=\"italic\""
        } else {
            ""
        }
    )
}
fn start_text(label: &Label, x: f64, top: f64, attrs: &str) -> String {
    let mut shared = 3;
    let mut any = false;
    for span in label.lines.iter().flatten() {
        if span.icon.is_none() && !span.text.is_empty() {
            shared &= span.style;
            any = true;
        }
    }
    if !any {
        shared = 0
    }
    let mut body = String::new();
    let mut y = top + label.line_height / 2. + label.size * 0.35;
    for line in &label.lines {
        let mut first = true;
        for span in line {
            if span.icon.is_some() || span.text.is_empty() {
                continue;
            }
            body.push_str(&format!(
                "<tspan{}{}>{}</tspan>",
                if first {
                    format!(" x=\"{}\" y=\"{}\"", num(x), num(y))
                } else {
                    String::new()
                },
                span_style(span.style & !shared),
                esc(&span.text)
            ));
            first = false;
        }
        y += label.line_height;
    }
    if body.is_empty() {
        String::new()
    } else {
        format!(
            "<text{attrs}{}{} text-anchor=\"start\" xml:space=\"preserve\">{body}</text>",
            if shared & 1 != 0 && !attrs.contains(" font-weight=\"") {
                " font-weight=\"var(--_w)\""
            } else {
                ""
            },
            if shared & 2 != 0 && !attrs.contains(" font-style=\"") {
                " font-style=\"italic\""
            } else {
                ""
            }
        )
    }
}
fn contains(x: f64, y: f64, dx: f64, dy: f64) -> String {
    let r = 6.;
    let cx = x - dx * r;
    let cy = y - dy * r;
    format!("<circle class=\"pele-marker\" cx=\"{}\" cy=\"{}\" r=\"6\"/><path class=\"pele-marker\" d=\"M{},{}L{},{}M{},{}L{},{}\"/>",num(cx),num(cy),num(cx-dx*r),num(cy-dy*r),num(cx+dx*r),num(cy+dy*r),num(cx+dy*r),num(cy-dx*r),num(cx-dy*r),num(cy+dx*r))
}
fn arrow(x: f64, y: f64, dx: f64, dy: f64) -> String {
    let bx = x - dx * 9.;
    let by = y - dy * 9.;
    format!(
        "<path class=\"pele-marker\" d=\"M{},{}L{},{}L{},{}\" stroke-linejoin=\"round\"/>",
        num(bx - dy * 4.5),
        num(by + dx * 4.5),
        num(x),
        num(y),
        num(bx + dy * 4.5),
        num(by - dx * 4.5)
    )
}
struct End {
    edge: usize,
    at: usize,
    toward: f64,
}
fn spread(nodes: &[CNode], edges: &mut [CEdge], views: &[View]) {
    let mut sides: Vec<(usize, Vec<End>)> = vec![];
    for (i, e) in edges.iter().enumerate() {
        if e.src == e.dst || e.route.len() < 6 {
            continue;
        }
        let last = e.route.len() - 3;
        for (node, at, next) in [(e.src, 0, 3), (e.dst, last, last - 3)] {
            let c = &nodes[node];
            let dx = e.route[at] - c.x;
            let dy = e.route[at + 1] - c.y;
            let side = if (dy.abs() - c.h / 2.).abs() < 0.5 {
                if dy < 0. {
                    0
                } else {
                    2
                }
            } else if (dx.abs() - c.w / 2.).abs() < 0.5 {
                if dx < 0. {
                    3
                } else {
                    1
                }
            } else {
                continue;
            };
            let key = node * 4 + side;
            let end = End {
                edge: i,
                at,
                toward: e.route[next + if side % 2 == 0 { 0 } else { 1 }],
            };
            if let Some((_, list)) = sides.iter_mut().find(|(k, _)| *k == key) {
                list.push(end)
            } else {
                sides.push((key, vec![end]));
            }
        }
    }
    for (key, mut list) in sides {
        if list.len() < 2 {
            continue;
        }
        let node = key >> 2;
        let vertical = key % 2 == 1;
        let span = if vertical {
            views[node].h
        } else {
            views[node].w
        } - 28.;
        let step = (span / (list.len() - 1) as f64).js_min(28.).js_max(0.);
        list.sort_by(|a, b| {
            a.toward
                .partial_cmp(&b.toward)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let len = list.len();
        for (j, e) in list.into_iter().enumerate() {
            let offset = (j as f64 - (len - 1) as f64 / 2.) * step;
            edges[e.edge].route[e.at + usize::from(vertical)] = if vertical {
                nodes[node].y
            } else {
                nodes[node].x
            } + offset;
        }
    }
}
pub(super) fn render(
    m: &RequirementModel,
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
    let horizontal = matches!(m.direction()?.as_str(), "LR" | "RL");
    let across = draw(m, c, o, r, false)?;
    if !horizontal {
        return Ok(across);
    }
    let Some(available) = o.max_width.filter(|v| *v > 0.) else {
        return Ok(across);
    };
    let breakpoint = o.direction_breakpoint.filter(|v| *v > 0.).unwrap_or(640.);
    if o.auto_direction == Some(false) || across.width <= available || available >= breakpoint {
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
    _c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    turned: bool,
) -> Result<Rendered, P::Error> {
    let scene = m.capture()?;
    let size = o.font_size.unwrap_or(16.);
    let small = round(size * 0.875);
    let pad = o.padding.unwrap_or(8.);
    let mut index = HashMap::new();
    let mut views = vec![];
    let mut cnodes = vec![];
    // Descriptor identity/classes/edges are captured before resource callbacks.
    for node in scene.nodes {
        let id = node.id;
        let requirement = node.requirement;
        let styles = m.styles(&node.styles)?;
        let style = resolve_style(&styles);
        let base = if style.bold { 1 } else { 0 } | if style.italic { 2 } else { 0 };
        let name_size = style.font_size.unwrap_or(size);
        let body_size = style.font_size.unwrap_or(small);
        let node_kind = m.kind(&node.record)?;
        let kind = text(
            &format!("«{node_kind}»"),
            false,
            body_size,
            4000.,
            base,
            r,
            o,
        )?;
        let name = text(&id, true, name_size, 280., base | 1, r, o)?;
        let fields = m.fields(&node.record)?;
        let mut rows = vec![];
        let (mut key_width, mut value_width, mut body_height) = (0_f64, 0_f64, 0.);
        for (label, raw) in fields {
            if raw.is_empty() {
                continue;
            }
            let key = text(label, false, body_size, 4000., base, r, o)?;
            let value = text(&raw, true, body_size, 240., base, r, o)?;
            let height = key.height.js_max(value.height);
            key_width = key_width.js_max(key.width);
            value_width = value_width.js_max(value.width);
            body_height += height + 2.;
            rows.push(Row { key, value, height });
        }
        if !rows.is_empty() {
            body_height += 18.;
        }
        let head_height = 20. + kind.height + name.height;
        let w = 120_f64
            .js_max(kind.width.js_max(name.width) + 28.)
            .js_max(if rows.is_empty() {
                0.
            } else {
                key_width + 12. + value_width + 28.
            });
        let h = head_height + body_height;
        index.entry(id.clone()).or_insert(views.len());
        let mut c = CNode::new(w, h, -1);
        c.seq = views.len();
        views.push(View {
            id: id.clone(),
            requirement,
            classes: node.classes,
            style,
            kind,
            name,
            rows,
            key_width,
            head_height,
            w,
            h,
        });
        cnodes.push(c);
    }
    let root_dir = if turned {
        Dir::TB
    } else {
        direction(Some(&scene.direction))
    };
    let sideways = matches!(root_dir, Dir::LR | Dir::RL);
    let mut cedges = vec![];
    let mut edge_labels = vec![];
    let mut drawn = vec![];
    for edge in scene.edges {
        let (Some(start), Some(end)) = (&edge.start, &edge.end) else {
            continue;
        };
        let (Some(&src), Some(&dst)) = (index.get(start), index.get(end)) else {
            continue;
        };
        let label = layout_label(
            Some(&format!("«{}»", edge.kind)),
            false,
            r,
            o,
            small,
            4000.,
            0,
        )?;
        if src == dst {
            if sideways {
                cnodes[src].h += 2. * (26. + label.height + 8.);
            } else {
                cnodes[src].w += 2. * (26. + label.width + 12.);
            }
        }
        drawn.push((edge.id, edge.contains));
        cedges.push(CEdge {
            src,
            dst,
            minlen: 1.,
            label_w: label.width + 12.,
            label_h: label.height + 4.,
            route: vec![],
            label_x: 0.,
            label_y: 0.,
        });
        edge_labels.push(label);
    }
    let layout = compound_layout(
        &mut cnodes,
        &mut cedges,
        root_dir,
        &LayeredOptions {
            node_sep: 40.,
            edge_sep: 16.,
            rank_sep: 56.,
            port_sep: 20.,
            tracks: false,
            straight: false,
            apart: false,
            head_room: 0.,
        },
    );
    spread(&cnodes, &mut cedges, &views);
    let title_text = m.metadata("title")?;
    let title = layout_label(title_text.as_deref(), false, r, o, size, 4000., 1)?;
    let title_height = if title.height > 0. {
        title.height + 12.
    } else {
        0.
    };
    let inner = layout.width.js_max(title.width);
    let ox = pad + (inner - layout.width) / 2.;
    let oy = pad + title_height;
    let width = (inner + 2. * pad).ceil();
    let height = (layout.height + title_height + 2. * pad).ceil();
    let mut nodes_out = String::new();
    for (i, v) in views.iter().enumerate() {
        let left = -v.w / 2.;
        let top = -v.h / 2.;
        let mut body=format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"var(--pele-radius,4px)\" fill=\"var(--_s)\" stroke=\"var(--_b)\"{}/>{}{}",num(left),num(top),num(v.w),num(v.h),v.style.shape,label_svg(&v.kind,0.,top+10.+v.kind.height/2.,&format!(" class=\"pele-stereotype\" font-size=\"{}\" fill=\"var(--_m)\"{}",js_num(v.kind.size),v.style.text)),label_svg(&v.name,0.,top+10.+v.kind.height+v.name.height/2.,&format!(" class=\"pele-label\" font-size=\"{}\" font-weight=\"var(--_hw)\"{}",js_num(v.name.size),v.style.text)));
        if !v.rows.is_empty() {
            let divider = top + v.head_height;
            body.push_str(&format!(
                "<path class=\"pele-divider\" d=\"M{},{}H{}\" stroke=\"var(--_b)\"{}/>",
                num(left),
                num(divider),
                num(-left),
                v.style.line
            ));
            let mut y = divider + 10.;
            let mut rows = String::new();
            for row in &v.rows {
                rows.push_str(&start_text(
                    &row.key,
                    left + 14.,
                    y,
                    &format!(" class=\"pele-field\" fill=\"var(--_m)\"{}", v.style.text),
                ));
                rows.push_str(&start_text(
                    &row.value,
                    left + 14. + v.key_width + 12.,
                    y,
                    &format!(" class=\"pele-value\"{}", v.style.text),
                ));
                y += row.height + 2.;
            }
            body.push_str(&format!(
                "<g class=\"pele-fields\" font-size=\"{}\">{rows}</g>",
                js_num(v.rows[0].key.size)
            ));
        }
        let classes = if let Some(rest) = v.classes.strip_prefix("default") {
            let mut chars = rest.chars();
            if chars.next().is_some_and(space) {
                &rest[rest.chars().next().unwrap().len_utf8()..]
            } else {
                rest
            }
        } else {
            &v.classes
        };
        nodes_out.push_str(&format!(
            "<g class=\"pele-node {}{}\" data-id=\"{}\" transform=\"translate({},{})\">{body}</g>",
            if v.requirement {
                "pele-requirement"
            } else {
                "pele-element"
            },
            class_names(classes),
            esc_text(&v.id)?,
            num(cnodes[i].x + ox),
            num(cnodes[i].y + oy)
        ));
    }
    let (mut edges_out, mut labels_out) = (String::new(), String::new());
    let mut loop_seen: HashMap<usize, usize> = HashMap::new();
    for (i, ce) in cedges.iter().enumerate() {
        let (id, has_contains) = &drawn[i];
        let label = &edge_labels[i];
        let start_trim = if *has_contains { 12. } else { 0. };
        let (mut label_x, mut label_y) = (ce.label_x + ox, ce.label_y + oy);
        let path;
        if ce.src == ce.dst {
            let v = &views[ce.src];
            let cx = cnodes[ce.src].x + ox;
            let cy = cnodes[ce.src].y + oy;
            let k = *loop_seen.entry(ce.src).or_default();
            *loop_seen.get_mut(&ce.src).unwrap() += 1;
            let half = if sideways { v.w } else { v.h } / 2.;
            let spread = (half - 4.).js_min(12. + k as f64 * 6.);
            let reach = 26. + k as f64 * 8.;
            let at = |out: f64, along: f64| {
                if sideways {
                    format!("{},{}", num(cx + along), num(cy + v.h / 2. + out))
                } else {
                    format!("{},{}", num(cx + v.w / 2. + out), num(cy + along))
                }
            };
            let pull = reach / 0.75;
            path = EdgePath {
                d: format!(
                    "M{}C{} {} {}",
                    at(start_trim, -spread),
                    at(pull, -spread),
                    at(pull, spread),
                    at(0., spread)
                ),
                sx: if sideways { cx - spread } else { cx + v.w / 2. },
                sy: if sideways { cy + v.h / 2. } else { cy - spread },
                sdx: if sideways { 0. } else { -1. },
                sdy: if sideways { -1. } else { 0. },
                ex: if sideways { cx + spread } else { cx + v.w / 2. },
                ey: if sideways { cy + v.h / 2. } else { cy + spread },
                edx: if sideways { 0. } else { -1. },
                edy: if sideways { -1. } else { 0. },
            };
            if sideways {
                label_x = cx;
                label_y = cy + v.h / 2. + reach + 4. + ce.label_h / 2.;
            } else {
                label_x = cx + v.w / 2. + reach + 4. + ce.label_w / 2.;
                label_y = cy;
            }
        } else {
            let mut route = ce.route.clone();
            for p in route.chunks_exact_mut(3) {
                p[0] += ox;
                p[1] += oy;
            }
            path = route_path(&route, None, start_trim, 0., None, None);
        }
        edges_out.push_str(&format!(
            "<g class=\"pele-edge pele-relationship-{}\" data-id=\"{}\"><path d=\"{}\"{}/>{}</g>",
            if *has_contains { "contains" } else { "arrow" },
            esc_text(id)?,
            path.d,
            if *has_contains {
                ""
            } else {
                " stroke-dasharray=\"5 4\""
            },
            if *has_contains {
                contains(path.sx, path.sy, path.sdx, path.sdy)
            } else {
                arrow(path.ex, path.ey, path.edx, path.edy)
            }
        ));
        labels_out.push_str(&edge_label_svg(id, label, label_x, label_y, "", r)?);
    }
    let content = format!(
        "{}{}{}<g class=\"pele-nodes\">{nodes_out}</g>",
        label_svg(
            &title,
            width / 2.,
            pad + title.height / 2.,
            " class=\"pele-title\" font-weight=\"var(--_tw)\""
        ),
        if edges_out.is_empty() {
            String::new()
        } else {
            format!("<g class=\"pele-edges\" fill=\"none\" stroke=\"var(--_l)\" stroke-linecap=\"round\">{edges_out}</g>")
        },
        if labels_out.is_empty() {
            String::new()
        } else {
            format!("<g class=\"pele-edge-labels\" font-size=\"{}\" fill=\"var(--_m)\">{labels_out}</g>",js_num(small))
        }
    );
    let acc_title = m.metadata("accTitle")?;
    let acc_descr = m.metadata("accDescr")?;
    let svg = svg::document(
        "requirement",
        width,
        height,
        size,
        o,
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
