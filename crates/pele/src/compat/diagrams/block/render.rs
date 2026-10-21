//! Dedicated Block grid renderer, Pele block/render.ts, 458322a5 (MIT, kepano).
use super::{
    arrows, layout,
    phase::{PhaseInput, Typed},
    route, BlockModel,
};
use crate::compat::{
    source::space,
    svg::{
        document,
        edges::{
            edge_label_svg, marker, marker_trim,
            theme::{class_names, resolve_style, ResolvedStyle},
        },
        esc_text, js_num, label_svg, label_svg_with_resources, num,
        shapes::{draw_shape, shape_size},
    },
    text::{layout_label, Label},
    Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
use std::collections::{HashMap, HashSet};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Leaf,
    Arrow,
    Composite,
    Space,
}
pub(super) struct View {
    pub children: Vec<usize>,
    pub block: usize,
    pub kind: Kind,
    pub shape: &'static str,
    pub label: Label,
    pub style: ResolvedStyle,
    pub mask: u8,
    pub dx: f64,
    pub dy: f64,
    pub bw: f64,
    pub bh: f64,
    pub columns: f64,
    pub span: f64,
    pub space: bool,
    pub head: f64,
    pub nw: f64,
    pub nh: f64,
    pub col: f64,
    pub row: usize,
    pub cols: f64,
    pub heights: Vec<f64>,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}
fn title_room(o: &RenderSettings) -> f64 {
    o.max_width
        .filter(|w| *w > 0.)
        .map_or(4000., |w| 80_f64.max(w - 2. * o.padding.unwrap_or(8.)))
}
fn shape(kind: Option<&str>) -> &'static str {
    match kind {
        Some("square") => "rect",
        Some("round") => "rounded",
        Some("circle") => "circle",
        Some("doublecircle") => "dbl-circ",
        Some("diamond") => "diam",
        Some("hexagon") => "hex",
        Some("stadium") => "stadium",
        Some("subroutine") => "fr-rect",
        Some("cylinder") => "cyl",
        Some("rect_left_inv_arrow") => "odd",
        Some("lean_right") => "lean-r",
        Some("lean_left") => "lean-l",
        Some("trapezoid") => "trap-b",
        Some("inv_trapezoid") => "trap-t",
        _ => "rect",
    }
}
fn style<I: PhaseInput>(input: &mut I, block: &I::Value) -> Result<ResolvedStyle, I::Error> {
    let mut declarations = vec![];
    let classes = input.get(block, "classes")?;
    let names = if input.truthy(&classes) && input.count(&classes)? > 0 {
        input.strings(&classes)?
    } else {
        vec!["default".into()]
    };
    let root = input.root();
    for name in names {
        let classes = input.get(&root, "classes")?;
        let class = input.map_get_text(&classes, &name)?;
        if input.truthy(&class) {
            let styles = input.get(&class, "styles")?;
            for value in input.strings(&styles)? {
                input.keep_text(value.len())?;
                declarations.extend(value.split(';').map(str::to_owned));
            }
        }
    }
    let styles = input.get(block, "styles")?;
    if input.truthy(&styles) {
        for value in input.strings(&styles)? {
            input.keep_text(value.len())?;
            declarations.extend(value.split(';').map(str::to_owned));
        }
    }
    Ok(resolve_style(&declarations))
}
fn view<I: PhaseInput>(
    input: &mut I,
    index: usize,
    b: &I::Value,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    gap: f64,
    tight: bool,
) -> Result<View, I::Error> {
    let raw_kind = input.field_text(b, "type")?;
    let kind = match raw_kind.as_deref() {
        Some("space") => Kind::Space,
        Some("composite") => Kind::Composite,
        Some("block_arrow") => Kind::Arrow,
        _ => Kind::Leaf,
    };
    let style = if kind == Kind::Space {
        resolve_style(&[])
    } else {
        style(input, b)?
    };
    let text_style = u8::from(style.bold) | u8::from(style.italic) * 2;
    let size = style.font_size.unwrap_or(o.font_size.unwrap_or(16.));
    let wrap = if tight { 80. } else { 200. };
    let mut v = View {
        children: vec![],
        block: index,
        kind,
        shape: "",
        label: Label::default(),
        style,
        mask: 0,
        dx: 0.,
        dy: 0.,
        bw: 0.,
        bh: 0.,
        columns: -1.,
        span: 1.,
        space: kind == Kind::Space,
        head: 0.,
        nw: 0.,
        nh: 0.,
        col: 0.,
        row: 0,
        cols: 0.,
        heights: vec![],
        x: 0.,
        y: 0.,
        w: 0.,
        h: 0.,
    };
    let label = if kind == Kind::Space {
        None
    } else {
        input.field_text(b, "label")?
    };
    if let Some(text) = &label {
        input.keep_text(text.len())?;
    }
    match kind {
        Kind::Space => v.label = layout_label(None, false, r, o, size, wrap, 0)?,
        Kind::Composite => {
            v.label = layout_label(
                label.as_deref(),
                false,
                r,
                o,
                size,
                if tight { title_room(o) } else { 4000. },
                text_style,
            )?;
            v.head = if v.label.height > 0. {
                v.label.height + 4.
            } else {
                0.
            };
            v.nw = v.label.width + 2. * gap;
            v.nh = v.head + 2. * gap;
        }
        Kind::Arrow => {
            v.label = layout_label(label.as_deref(), false, r, o, size, wrap, text_style)?;
            let blank = v
                .label
                .lines
                .iter()
                .flatten()
                .all(|s| s.icon.is_none() && s.text.trim_matches(space).is_empty());
            let directions = input.get(b, "directions")?;
            let directions = input.strings(&directions)?;
            v.mask = arrows::mask(Some(&directions));
            let (w, h, dx, dy) = if blank {
                arrows::size(v.mask, 0., 0.)
            } else {
                arrows::size(v.mask, v.label.width, v.label.height)
            };
            v.nw = w;
            v.nh = h;
            v.dx = dx;
            v.dy = dy;
        }
        Kind::Leaf => {
            v.shape = shape(raw_kind.as_deref());
            v.label = layout_label(label.as_deref(), false, r, o, size, wrap, text_style)?;
            let s = shape_size(v.shape, v.label.width, v.label.height);
            v.nw = s.w;
            v.nh = if v.shape == "cyl" {
                s.h.max(v.label.height + 40.)
            } else {
                s.h
            };
            v.dy = s.dy;
        }
    }
    // Ot reads grid operands after label resources have returned.
    v.columns = if kind == Kind::Composite {
        input.field_number(b, "columns")?.unwrap_or(-1.)
    } else {
        -1.
    };
    v.span = if kind == Kind::Space {
        1.
    } else {
        input.field_number(b, "widthInColumns")?.unwrap_or(1.)
    };
    Ok(v)
}
fn grow(bounds: &mut (f64, f64, f64, f64), x: f64, y: f64, w: f64, h: f64) {
    if x - w / 2. < bounds.0 {
        bounds.0 = x - w / 2.;
    }
    if y - h / 2. < bounds.1 {
        bounds.1 = y - h / 2.;
    }
    if x + w / 2. > bounds.2 {
        bounds.2 = x + w / 2.;
    }
    if y + h / 2. > bounds.3 {
        bounds.3 = y + h / 2.;
    }
}
struct Route {
    edge: usize,
    points: Vec<f64>,
    loop_index: usize,
    label: Label,
    lx: f64,
    ly: f64,
    s: usize,
}
pub(super) fn render(
    m: &BlockModel,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_accessible(m, c, o, r, None, None)
}
pub(super) fn render_accessible(
    m: &BlockModel,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    acc_title: Option<&str>,
    acc_descr: Option<&str>,
) -> Result<Rendered, PeleError> {
    render_phases(
        &mut Typed {
            model: m,
            acc_title,
            acc_descr,
        },
        c,
        o,
        r,
    )
}
pub(super) fn render_phases<I: PhaseInput>(
    input: &mut I,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, I::Error> {
    let loose = draw(input, c, o, r, false)?;
    if o.max_width.is_none_or(|w| !(w > 0.) || loose.width <= w) {
        return Ok(loose);
    }
    let tight = draw(input, c, o, r, true)?;
    Ok(if tight.width < loose.width {
        tight
    } else {
        loose
    })
}
fn draw<I: PhaseInput>(
    input: &mut I,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    tight: bool,
) -> Result<Rendered, I::Error> {
    input.begin_draw();
    let model = input.root();
    let size = o.font_size.unwrap_or(16.);
    let edge_size = (size * 0.875 + 0.5).floor();
    let pad = o.padding.unwrap_or(8.);
    let configured = c
        .get("block")
        .and_then(Value::as_object)
        .and_then(|c| c.get("padding"));
    let gap = match configured {
        Some(Value::Number(n)) if *n >= 0. => n.min(200.),
        _ => 12.,
    }
    .min(if tight { 6. } else { f64::INFINITY });
    let root_record = input.get(&model, "root")?;
    let mut root = view(input, 0, &root_record, o, r, gap, tight)?;
    let mut records = vec![root_record];
    root.nw = 0.;
    root.nh = 0.;
    let mut order = vec![root];
    let mut by_id = HashMap::<String, usize>::new();
    let mut i = 0;
    while i < order.len() {
        if order[i].kind == Kind::Composite {
            let record = records[order[i].block].clone();
            let children = input.get(&record, "children")?;
            let mut at = 0;
            while !input.nullish(&children) && at < input.count(&children)? {
                input.work(1)?;
                let child = input.index(&children, at)?;
                let j = order.len();
                if j >= 1_000_000 {
                    let mut error = PeleError::new(
                        "Block expanded rendering exceeds native work budget",
                        crate::compat::ErrorCode::Limit,
                    );
                    error.diagram_type = Some("block".into());
                    return Err(error.into());
                }
                let v = view(input, j, &child, o, r, gap, tight)?;
                if v.kind != Kind::Space {
                    if let Some(id) = input.field_text(&child, "id")? {
                        by_id.insert(id, j);
                    }
                }
                records.push(child);
                order[i].children.push(j);
                order.push(v);
                at += 1;
            }
        }
        i += 1;
    }
    let (w, h) = layout::layout(&mut order, gap)?;
    let mut obstacles = vec![];
    for (i, v) in order.iter_mut().enumerate() {
        if v.kind == Kind::Leaf && matches!(v.shape, "circle" | "dbl-circ" | "diam") {
            v.bw = v.nw;
            v.bh = v.nh;
        } else if v.kind == Kind::Arrow {
            v.bw = if v.span > 1. { v.w } else { v.nw };
            v.bh = v.nh;
        } else {
            v.bw = v.w;
            v.bh = if v.kind == Kind::Leaf
                && matches!(
                    v.shape,
                    "stadium" | "hex" | "odd" | "lean-r" | "lean-l" | "trap-b" | "trap-t"
                ) {
                v.nh
            } else {
                v.h
            };
        }
        if v.kind == Kind::Leaf && v.shape == "cyl" {
            v.dy = 12_f64.min(6_f64.max(v.bw / 14.)) / 2.;
        }
        if matches!(v.kind, Kind::Leaf | Kind::Arrow) {
            obstacles.push(i);
        }
    }
    let mut pairs = HashMap::<(usize, usize), usize>::new();
    let mut labelled = HashSet::new();
    let mut ends = vec![];
    let mut edge_records = vec![];
    let edge_receiver = input.get(&model, "edges")?;
    let mut at = 0;
    while at < input.count(&edge_receiver)? {
        input.work(1)?;
        let e = input.index(&edge_receiver, at)?;
        at += 1;
        let start = input.field_text(&e, "start")?.unwrap_or_default();
        let end = input.field_text(&e, "end")?.unwrap_or_default();
        let (Some(&s), Some(&t)) = (by_id.get(&start), by_id.get(&end)) else {
            continue;
        };
        let edge = edge_records.len();
        let key = (s.min(t), s.max(t));
        *pairs.entry(key).or_default() += 1;
        if input
            .field_text(&e, "label")?
            .is_some_and(|l| !l.is_empty())
        {
            labelled.insert(key);
        }
        edge_records.push(e);
        ends.push((edge, s, t));
    }
    let check =
        (ends.len().saturating_mul(obstacles.len()) <= 2_000_000).then_some(obstacles.as_slice());
    let mut seen = HashMap::<(usize, usize), usize>::new();
    let mut routes = vec![];
    let mut bounds = (0., 0., w, h);
    for (edge, s, t) in ends {
        let key = (s.min(t), s.max(t));
        let total = pairs[&key];
        let k = seen.entry(key).or_default();
        let nth = *k;
        *k += 1;
        let label_text = input.field_text(&edge_records[edge], "label")?;
        if let Some(text) = &label_text {
            input.keep_text(text.len())?;
        }
        let label = layout_label(label_text.as_deref(), false, r, o, edge_size, 200., 0)?;
        let points;
        let (lx, ly);
        if s == t {
            let (px, py, r, _, _) = route::loop_arc(&order[s], nth + 1);
            points = vec![];
            grow(&mut bounds, px, py, 2. * r + 2., 2. * r + 2.);
            lx = px + r + 6. + label.width / 2.;
            ly = py - r / 2.;
        } else {
            let step = if labelled.contains(&key) {
                (edge_size * 1.5 + 0.5).floor() + 2.
            } else {
                14.
            };
            points = route::route(
                s,
                t,
                (nth as f64 - (total as f64 - 1.) / 2.) * step,
                check,
                &order,
            );
            let n = points.len();
            let first = (points[2] - points[0]).abs() + (points[3] - points[1]).abs();
            let last =
                (points[n - 2] - points[n - 4]).abs() + (points[n - 1] - points[n - 3]).abs();
            let at = if n == 6 && last > first { 2 } else { 0 };
            let spot = if label.width > 0. {
                route::label_spot(
                    &points,
                    at,
                    label.width + 8.,
                    label.height,
                    s,
                    t,
                    check,
                    &order,
                )
            } else {
                0.5
            };
            lx = points[at] + (points[at + 2] - points[at]) * spot;
            ly = points[at + 1] + (points[at + 3] - points[at + 1]) * spot;
        }
        if label.width > 0. {
            grow(&mut bounds, lx, ly, label.width + 8., label.height);
        }
        routes.push(Route {
            edge,
            points,
            loop_index: if s == t { nth + 1 } else { 0 },
            label,
            lx,
            ly,
            s,
        });
    }
    let title_text = input.field_text(&model, "title")?;
    if let Some(text) = &title_text {
        input.keep_text(text.len())?;
    }
    let title = layout_label(
        title_text.as_deref(),
        false,
        r,
        o,
        size,
        if tight { title_room(o) } else { 4000. },
        1,
    )?;
    let title_height = if title.height > 0. {
        title.height + 12.
    } else {
        0.
    };
    let inner = (bounds.2 - bounds.0).max(title.width);
    let ox = pad - bounds.0 + (inner - (bounds.2 - bounds.0)) / 2.;
    let oy = pad + title_height - bounds.1;
    for v in &mut order {
        v.x += ox;
        v.y += oy;
    }
    let width = (inner + 2. * pad).ceil();
    let height = (bounds.3 - bounds.1 + title_height + 2. * pad).ceil();
    let mut clusters = String::new();
    let mut nodes = String::new();
    for v in order.iter().skip(1) {
        if v.kind == Kind::Space {
            continue;
        }
        let block = &records[v.block];
        let raw_classes = input.get(block, "classes")?;
        let classes = if input.nullish(&raw_classes) {
            String::new()
        } else {
            class_names(&input.join(&raw_classes)?)
        };
        let id = esc_text(&input.field_text(block, "id")?.unwrap_or_default())?;
        if v.kind == Kind::Composite {
            let top = v.y - v.h / 2.;
            clusters+=&format!("<g class=\"pele-cluster{classes}\" data-id=\"{id}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"var(--pele-radius,4px)\" fill=\"var(--_a)\" fill-opacity=\"0.5\" stroke=\"var(--_b)\"{}/>{}</g>",num(v.x-v.w/2.),num(top),num(v.w),num(v.h),v.style.shape,label_svg_with_resources(&v.label,v.x,top+(v.head+gap)/2.+2.,&format!(" class=\"pele-cluster-label\" fill=\"var(--_m)\"{}",v.style.text),r)?);
            continue;
        }
        let arrow = v.kind == Kind::Arrow;
        let attrs = format!(" fill=\"var(--_s)\" stroke=\"var(--_b)\"{}", v.style.shape);
        let drawn = if arrow {
            arrows::draw(v.mask, v.bw, v.bh, &attrs)
        } else {
            draw_shape(
                v.shape,
                v.bw,
                v.bh,
                &attrs,
                &format!(" stroke=\"var(--_b)\"{}", v.style.line),
            )
        };
        nodes+=&format!("<g class=\"pele-node pele-shape-{}{classes}\" data-id=\"{id}\" transform=\"translate({},{})\">{}{}</g>",if arrow{"block-arrow"}else{v.shape},num(v.x),num(v.y),drawn,label_svg_with_resources(&v.label,v.dx,v.dy,&format!(" class=\"pele-label\"{}",v.style.text),r)?);
    }
    let mut edges = String::new();
    let mut labels = String::new();
    for route in &mut routes {
        let edge = &edge_records[route.edge];
        let start_value = input
            .field_text(edge, "arrowTypeStart")?
            .unwrap_or_default();
        let end_value = input.field_text(edge, "arrowTypeEnd")?.unwrap_or_default();
        let start = start_value.as_str();
        let end = end_value.as_str();
        let path = if route.loop_index > 0 {
            route::loop_path(
                &order[route.s],
                route.loop_index,
                marker_trim(start),
                marker_trim(end),
            )
        } else {
            for p in route.points.chunks_exact_mut(2) {
                p[0] += ox;
                p[1] += oy;
            }
            route::trace(&route.points, marker_trim(start), marker_trim(end))
        };
        let id = esc_text(&input.field_text(edge, "id")?.unwrap_or_default())?;
        edges += &format!(
            "<g class=\"pele-edge\" data-id=\"{id}\"><path d=\"{}\"{}{}/>{}{}</g>",
            path.d,
            if input.field_text(edge, "thickness")?.as_deref() == Some("thick") {
                " stroke-width=\"2.5\""
            } else {
                ""
            },
            if input.field_text(edge, "pattern")?.as_deref() == Some("dotted") {
                " stroke-dasharray=\"3 4\""
            } else {
                ""
            },
            marker(start, path.sx, path.sy, path.sdx, path.sdy, "var(--_l)"),
            marker(end, path.ex, path.ey, path.edx, path.edy, "var(--_l)")
        );
        if route.label.width > 0. {
            labels += &edge_label_svg(
                &input.field_text(edge, "id")?.unwrap_or_default(),
                &route.label,
                route.lx + ox,
                route.ly + oy,
                "",
                r,
            )?;
        }
    }
    let content = format!(
        "{}{}{}<g class=\"pele-nodes\">{nodes}</g>{}",
        label_svg(
            &title,
            width / 2.,
            pad + title.height / 2.,
            " class=\"pele-title\" font-weight=\"var(--_tw)\""
        ),
        if clusters.is_empty() {
            String::new()
        } else {
            format!("<g class=\"pele-clusters\">{clusters}</g>")
        },
        if edges.is_empty() {
            String::new()
        } else {
            format!("<g class=\"pele-edges\" fill=\"none\" stroke=\"var(--_l)\" stroke-linecap=\"round\">{edges}</g>")
        },
        if labels.is_empty() {
            String::new()
        } else {
            format!(
                "<g class=\"pele-edge-labels\" font-size=\"{}\">{labels}</g>",
                js_num(edge_size)
            )
        }
    );
    Ok(Rendered {
        svg: document(
            "block",
            width,
            height,
            size,
            o,
            input.field_text(&model, "accTitle")?.as_deref(),
            input.field_text(&model, "accDescr")?.as_deref(),
            &content,
        )?,
        width,
        height,
        links: vec![],
    })
}
