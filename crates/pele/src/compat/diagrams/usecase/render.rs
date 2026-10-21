//! Pele usecase/render.ts, 458322a5 (MIT, kepano). Actual actor geometry,
//! ellipse ports, package boundaries, JSON tables and self-relationship curves.
use super::{
    graph::{self, Node},
    model::UsecaseModel,
};
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    layout::{
        compound::{compound_layout, direction, shift_layout, CEdge, CNode},
        layered::LayeredOptions,
    },
    svg::{
        self,
        edges::{
            cluster::{mark_crossings, Crossings},
            edge_label_svg, marker, marker_trim, route_path,
            theme::{class_names, resolve_style, ResolvedStyle},
            EdgePath,
        },
        esc, esc_text, js_num, label_svg, label_svg_with_resources, num,
    },
    text::{layout_label, Label},
    Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
use std::collections::HashMap;
const ATTRS: &str = " fill=\"var(--_s)\" stroke=\"var(--_b)\"";
const RADIUS: &str = "var(--pele-radius,4px)";
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Actor,
    Ellipse,
    Box,
    Json,
    Boundary,
}
struct View {
    node: Node,
    kind: Kind,
    style: ResolvedStyle,
    label: Label,
    stereotype: Label,
    w: f64,
    h: f64,
    rows: Vec<(Label, Label)>,
    key_w: f64,
    head_h: f64,
    loops: usize,
}
fn number(c: &Object, key: &str, fallback: f64) -> f64 {
    match c
        .get("usecase")
        .and_then(Value::as_object)
        .and_then(|c| c.get(key))
    {
        Some(Value::Number(n)) if *n > 0. => n.js_min(2000.),
        _ => fallback,
    }
}
fn text(
    raw: Option<&str>,
    markdown: bool,
    r: &mut dyn RenderResources,
    o: &RenderSettings,
    size: f64,
    wrap: f64,
    base: u8,
) -> Result<Label, PeleError> {
    let literal = raw
        .unwrap_or_default()
        .chars()
        .map(|c| match c {
            '&' | '<' | '>' | '\\' => format!("&#{};", c as u32),
            _ => c.to_string(),
        })
        .collect::<String>();
    layout_label(
        Some(if markdown {
            raw.unwrap_or_default()
        } else {
            &literal
        }),
        markdown,
        r,
        o,
        size,
        wrap,
        base,
    )
}
fn anchor(v: &View, c: &CNode, side: usize, along: f64) -> (f64, f64) {
    let sign = if side == 1 || side == 2 { 1. } else { -1. };
    let upright = side == 1 || side == 3;
    let mut reach = if upright { v.w } else { v.h } / 2.;
    let mut middle = 0.;
    if v.kind == Kind::Actor {
        if upright {
            reach = 23.;
            middle = (60. - v.h) / 2.
        } else {
            reach += 2.
        }
    } else if v.kind == Kind::Ellipse {
        let span = if upright { v.h } else { v.w } / 2.;
        reach *= (1. - (along / span).powi(2)).js_max(0.).sqrt()
    }
    if upright {
        (c.x + sign * reach, c.y + middle + along)
    } else {
        (c.x + along, c.y + sign * reach)
    }
}
fn side_length(v: &View, side: usize) -> f64 {
    let upright = side == 1 || side == 3;
    if v.kind == Kind::Actor {
        return if upright { 30. } else { 24. };
    }
    let length = if upright { v.h } else { v.w };
    if v.kind == Kind::Ellipse {
        length * 0.6
    } else {
        length - 16.
    }
}
fn attach(edges: &mut [CEdge], nodes: &[CNode], views: &[View]) {
    let mut sides: Vec<(usize, Vec<(usize, usize, f64)>)> = vec![];
    let mut index = HashMap::new();
    for (i, e) in edges.iter().enumerate() {
        let Some(last) = e.route.len().checked_sub(3) else {
            continue;
        };
        if e.src == e.dst || last < 3 {
            continue;
        }
        for (node, at, next) in [(e.src, 0, 3), (e.dst, last, last - 3)] {
            let c = &nodes[node];
            if c.is_group {
                continue;
            }
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
            let toward = e.route[next + if side & 1 != 0 { 1 } else { 0 }];
            let pos = *index.entry(key).or_insert_with(|| {
                sides.push((key, vec![]));
                sides.len() - 1
            });
            sides[pos].1.push((i, at, toward))
        }
    }
    for (key, mut ends) in sides {
        let i = key >> 2;
        let side = key & 3;
        let step = if ends.len() > 1 {
            14_f64.js_min(side_length(&views[i], side) / (ends.len() - 1) as f64)
        } else {
            0.
        };
        ends.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
        let count = ends.len();
        for (k, (edge, at, _)) in ends.into_iter().enumerate() {
            let (x, y) = anchor(
                &views[i],
                &nodes[i],
                side,
                (k as f64 - (count - 1) as f64 / 2.) * step,
            );
            edges[edge].route[at] = x;
            edges[edge].route[at + 1] = y;
        }
    }
}
fn extension(x: f64, y: f64, dx: f64, dy: f64, color: &str) -> String {
    let bx = x - dx * 12.;
    let by = y - dy * 12.;
    format!("<path class=\"pele-marker\" d=\"M{},{}L{},{}L{},{}Z\" fill=\"var(--_bg)\" stroke=\"{color}\" stroke-linejoin=\"round\"/>",num(x),num(y),num(bx-dy*6.),num(by+dx*6.),num(bx+dy*6.),num(by-dx*6.))
}
fn trim(t: &str) -> f64 {
    if t == "extension" {
        12.
    } else {
        marker_trim(t)
    }
}
fn mark(t: &str, x: f64, y: f64, dx: f64, dy: f64, color: &str) -> String {
    if t == "extension" {
        extension(x, y, dx, dy, color)
    } else {
        marker(t, x, y, dx, dy, color)
    }
}
fn figure(v: &View, r: &mut dyn RenderResources) -> Result<String, PeleError> {
    let n = &v.node;
    let shape = &v.style.shape;
    let line = format!(" fill=\"none\" stroke=\"var(--_b)\"{}", v.style.line);
    let mut head = -21.;
    let mut radius = 9.;
    let mut out=match n.actor_type.as_deref(){Some("hollow")=>{radius=8.;format!("<circle cy=\"-21\" r=\"8\" fill=\"var(--_bg)\" stroke=\"var(--_b)\"{shape}/><path d=\"M-20,-9H20V0H5.5L20,15.5L12,25.5L0,12L-12,25.5L-20,15.5L-5.5,0H-20Z\" fill=\"var(--_bg)\" stroke=\"var(--_b)\" stroke-linejoin=\"round\"{shape}/>")},Some("awesome")=>format!("<path d=\"M0,-30A11,11 0 1 1 0,-8A11,11 0 1 1 0,-30ZM-20,25C-20,9 -12,-3 0,-3C12,-3 20,9 20,25C20,28 18,30 15,30H-15C-18,30 -20,28 -20,25Z\" fill=\"var(--_b)\"{shape}/>"),Some("icon")=>{head=-14.;radius=12.;let icon=n.icon.as_deref().unwrap_or_default();let body=r.direct_icon(icon)?.map_or(String::new(),|s|s.markup().into());format!("<rect x=\"-22\" y=\"-26\" width=\"44\" height=\"44\" rx=\"{RADIUS}\"{ATTRS}{shape}/><svg class=\"pele-icon\" data-icon=\"{}\" x=\"-14\" y=\"-18\" width=\"28\" height=\"28\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{body}</svg>",esc_text(icon)?)},_=>format!("<circle cy=\"-21\" r=\"9\"{ATTRS}{shape}/><path d=\"M0,-12V10M-14,-3H14M0,10L-12,30M0,10L12,30\"{line}/>")};
    if n.business {
        out.push_str(&format!(
            "<path class=\"pele-business\" d=\"M{},{}L{},{}\"{line}/>",
            num(0.12 * radius),
            num(head + 0.99 * radius),
            num(0.92 * radius),
            num(head - 0.39 * radius)
        ))
    }
    Ok(out)
}
pub(super) trait PhaseInput {
    type Error: From<PeleError>;
    fn direction(&mut self) -> Result<String, Self::Error>;
    fn graph(&mut self) -> Result<(Vec<graph::Node>, Vec<graph::Edge>), Self::Error>;
    fn title(&mut self) -> Result<Option<String>, Self::Error>;
    fn accessibility(&mut self) -> Result<(String, String), Self::Error>;
}
struct TypedPhase<'a>(&'a UsecaseModel);
impl PhaseInput for TypedPhase<'_> {
    type Error = PeleError;
    fn direction(&mut self) -> Result<String, PeleError> {
        Ok(self.0.direction.clone())
    }
    fn graph(&mut self) -> Result<(Vec<graph::Node>, Vec<graph::Edge>), PeleError> {
        Ok(graph::build(self.0))
    }
    fn title(&mut self) -> Result<Option<String>, PeleError> {
        Ok(self.0.title.clone())
    }
    fn accessibility(&mut self) -> Result<(String, String), PeleError> {
        Ok((self.0.acc_title.clone(), self.0.acc_descr.clone()))
    }
}
pub(super) fn render(
    m: &UsecaseModel,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_phase(&mut TypedPhase(m), c, o, r)
}
pub(super) fn render_phase<P: PhaseInput>(
    phase: &mut P,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    // Upstream chooses responsive policy from entry direction before callbacks.
    let sideways = matches!(phase.direction()?.as_str(), "LR" | "RL");
    let across = draw(phase, c, o, r, false)?;
    if !sideways {
        return Ok(across);
    }
    let Some(available) = o.max_width.filter(|v| *v > 0.) else {
        return Ok(across);
    };
    if o.auto_direction == Some(false)
        || across.width <= available
        || available >= o.direction_breakpoint.filter(|v| *v > 0.).unwrap_or(640.)
    {
        return Ok(across);
    }
    let down = draw(phase, c, o, r, true)?;
    Ok(if down.width < across.width {
        down
    } else {
        across
    })
}
fn draw<P: PhaseInput>(
    phase: &mut P,
    config: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    turned: bool,
) -> Result<Rendered, P::Error> {
    let (graph_nodes, graph_edges) = phase.graph()?;
    let size = o.font_size.unwrap_or(16.);
    let small = round(size * 0.875);
    let tiny = round(size * 0.8125);
    let pad = o.padding.unwrap_or(8.);
    let root = if turned {
        direction(Some("TB"))
    } else {
        direction(Some(&phase.direction()?))
    };
    let sideways = root.sideways();
    let index: HashMap<_, _> = graph_nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.clone(), i))
        .collect();
    let mut views = vec![];
    let mut nodes = vec![];
    for node in graph_nodes {
        let mut styles = node.compiled.clone();
        styles.extend(node.styles.iter().cloned());
        let style = resolve_style(&styles);
        let base = if style.bold { 1 } else { 0 } | if style.italic { 2 } else { 0 };
        let font_size = style.font_size.unwrap_or(size);
        let markdown = node.label_type == "markdown";
        let st = node
            .stereotype
            .as_ref()
            .filter(|s| !s.is_empty())
            .map(|s| format!("«{s}»"));
        let stereotype = text(st.as_deref(), false, r, o, tiny, 200., base)?;
        let mut v = View {
            node,
            kind: Kind::Box,
            style,
            label: stereotype.clone(),
            stereotype,
            w: 0.,
            h: 0.,
            rows: vec![],
            key_w: 0.,
            head_h: 0.,
            loops: 0,
        };
        let mut cn;
        if v.node.group {
            v.kind = Kind::Boundary;
            v.label = text(Some(&v.node.label), markdown, r, o, font_size, 4000., base)?;
            cn = CNode::new(0., 0., -1);
            cn.is_group = true;
            cn.pad_x = 20.;
            cn.pad_bottom = 20.;
            cn.pad_top = if v.node.boundary_type.as_deref() == Some("package") {
                v.label.height + 28.
            } else if v.label.height > 0. {
                v.label.height + 16.
            } else {
                20.
            };
            cn.min_w = v.label.width + 40.;
        } else {
            if let Some(rows) = &v.node.rows {
                v.kind = Kind::Json;
                v.label = text(Some(&v.node.label), false, r, o, font_size, 4000., base | 1)?;
                let mut value_w: f64 = 0.;
                for row in rows {
                    let key = text(Some(&row.key), false, r, o, small, 4000., base)?;
                    let value = text(Some(&row.value), false, r, o, small, 260., base)?;
                    v.key_w = v.key_w.js_max(key.width);
                    value_w = value_w.js_max(value.width);
                    v.h += value.height.js_max(round(small * 1.5)) + 8.;
                    v.rows.push((key, value));
                }
                if v.key_w > 0. {
                    v.key_w += 20.
                }
                v.head_h = v.label.height.js_max(round(font_size * 1.5)) + 8.;
                v.w = (v.label.width + 20.)
                    .js_max(if v.rows.is_empty() {
                        0.
                    } else {
                        v.key_w + value_w + 20.
                    })
                    .js_max(72.);
                v.h += v.head_h;
            } else if v.node.actor_type.is_some() {
                v.kind = Kind::Actor;
                v.label = text(Some(&v.node.label), markdown, r, o, font_size, 140., base)?;
                v.w = 40_f64.js_max(v.label.width).js_max(v.stereotype.width);
                v.h = 66. + v.stereotype.height + v.label.height;
            } else {
                v.label = text(
                    Some(&v.node.label),
                    markdown,
                    r,
                    o,
                    if v.node.shape == "note" {
                        small
                    } else {
                        font_size
                    },
                    180.,
                    base,
                )?;
                let tw = v.label.width.js_max(v.stereotype.width);
                let th = v.label.height + v.stereotype.height;
                if v.node.shape == "usecaseEllipse" || v.node.shape == "usecaseBusiness" {
                    v.kind = Kind::Ellipse;
                    v.w = (tw + 2. * v.node.padding + if v.node.business { 16. } else { 4. })
                        .js_max(88.);
                    v.h = (th + 28.).js_max(48.)
                } else if v.node.shape == "note" {
                    v.w = tw + 2. * v.node.padding + 6.;
                    v.h = th + 18.
                } else {
                    v.w = (tw + 2. * v.node.padding + 12.).js_max(80.);
                    v.h = (th + 24.).js_max(44.)
                }
            }
            cn = CNode::new(v.w, v.h, -1)
        }
        cn.seq = views.len();
        views.push(v);
        nodes.push(cn);
    }
    for (i, v) in views.iter().enumerate() {
        nodes[i].parent = v
            .node
            .parent
            .as_ref()
            .and_then(|id| index.get(id))
            .map_or(-1, |i| *i as isize)
    }
    let mut edges = vec![];
    let mut labels = vec![];
    let mut drawn = vec![];
    for edge in graph_edges {
        let (Some(&src), Some(&dst)) = (index.get(&edge.start), index.get(&edge.end)) else {
            continue;
        };
        let semantic = edge.kind == "include" || edge.kind == "extend";
        let semantic_label = edge.label.as_ref().map(|s| format!("«{s}»"));
        let label = text(
            if semantic {
                semantic_label.as_deref()
            } else {
                edge.label.as_deref()
            },
            edge.label_type.as_deref() == Some("markdown"),
            r,
            o,
            small,
            200.,
            0,
        )?;
        if src == dst {
            views[src].loops += 1;
            if sideways {
                nodes[src].h += 2.
                    * (26.
                        + if label.height > 0. {
                            label.height + 8.
                        } else {
                            0.
                        })
            } else {
                nodes[src].w += 2.
                    * (26.
                        + if label.width > 0. {
                            label.width + 12.
                        } else {
                            0.
                        })
            }
        }
        edges.push(CEdge {
            src,
            dst,
            minlen: edge.minlen as f64,
            label_w: if label.width > 0. {
                label.width + 12.
            } else {
                0.
            },
            label_h: if label.height > 0. {
                label.height + 4.
            } else {
                0.
            },
            route: vec![],
            label_x: 0.,
            label_y: 0.,
        });
        labels.push(label);
        drawn.push(edge);
    }
    let layout = compound_layout(
        &mut nodes,
        &mut edges,
        root,
        &LayeredOptions {
            node_sep: number(config, "nodeSpacing", 32.),
            edge_sep: 16.,
            rank_sep: number(config, "rankSpacing", 72.),
            port_sep: 20.,
            tracks: false,
            straight: false,
            apart: false,
            head_room: 0.,
        },
    );
    let title_text = phase.title()?;
    let title = layout_label(title_text.as_deref(), false, r, o, size, 4000., 1)?;
    let title_height = if title.height > 0. {
        title.height + 12.
    } else {
        0.
    };
    let inner = layout.width.js_max(title.width);
    shift_layout(
        &mut nodes,
        &mut edges,
        pad + (inner - layout.width) / 2.,
        pad + title_height,
    );
    let width = (inner + 2. * pad).ceil();
    let height = (layout.height + title_height + 2. * pad).ceil();
    let mut crossings = Crossings::new();
    for e in &edges {
        mark_crossings(&mut crossings, &e.route)
    }
    attach(&mut edges, &nodes, &views);
    let mut clusters = String::new();
    let mut nodes_out = String::new();
    for (i, v) in views.iter().enumerate() {
        let c = &nodes[i];
        let n = &v.node;
        let classes = class_names(strip_default(&n.classes));
        let id = esc_text(&n.id)?;
        let attrs = &v.style.text;
        let shape = &v.style.shape;
        let (w, h) = (v.w, v.h);
        if v.kind == Kind::Boundary {
            let x = c.x - c.w / 2.;
            let y = c.y - c.h / 2.;
            let b = format!(" fill=\"var(--_a)\" fill-opacity=\"0.5\" stroke=\"var(--_b)\"{shape}");
            let ta = format!(" class=\"pele-cluster-label\" fill=\"var(--_m)\"{attrs}");
            let body = if n.boundary_type.as_deref() == Some("package") {
                let tw = (v.label.width + 20.).js_min(c.w);
                let th = v.label.height + 12.;
                format!("<rect class=\"pele-boundary-tab\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{b}/><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{b}/>{}",num(x),num(y),num(tw),num(th),num(x),num(y+th),num(c.w),num(c.h-th),label_svg_with_resources(&v.label,x+tw/2.,y+th/2.,&ta,r)?)
            } else {
                format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{RADIUS}\"{b}/>{}",
                    num(x),
                    num(y),
                    num(c.w),
                    num(c.h),
                    label_svg_with_resources(
                        &v.label,
                        title_x(c, v.label.width, &crossings),
                        y + 8. + v.label.height / 2.,
                        &ta,
                        r
                    )?
                )
            };
            clusters.push_str(&format!(
                "<g class=\"pele-cluster pele-boundary{classes}\" data-id=\"{id}\">{body}</g>"
            ));
            continue;
        }
        let (kind, body) = if v.kind == Kind::Actor {
            let top = -h / 2.;
            (
                "pele-actor",
                format!(
                    "<g class=\"pele-actor-figure\" transform=\"translate(0,{})\">{}</g>{}{}",
                    num(top + 30.),
                    figure(v, r)?,
                    label_svg_with_resources(
                        &v.stereotype,
                        0.,
                        top + 66. + v.stereotype.height / 2.,
                        &format!(
                            " class=\"pele-stereotype\" font-size=\"{}\" fill=\"var(--_m)\"",
                            js_num(tiny)
                        ),
                        r
                    )?,
                    label_svg_with_resources(
                        &v.label,
                        0.,
                        h / 2. - v.label.height / 2.,
                        &format!(" class=\"pele-label\"{attrs}"),
                        r
                    )?
                ),
            )
        } else if v.kind == Kind::Json {
            let x0 = -w / 2.;
            let mut y = -h / 2. + v.head_h;
            let mut cells = String::new();
            let mut lines = format!("M{},{}H{}", num(x0), num(y), num(x0 + w));
            if v.key_w > 0. {
                lines.push_str(&format!(
                    "M{},{}V{}",
                    num(x0 + v.key_w),
                    num(y),
                    num(h / 2.)
                ))
            }
            for (key, value) in &v.rows {
                let rh = value.height.js_max(round(small * 1.5)) + 8.;
                cells.push_str(&label_svg_with_resources(
                    key,
                    x0 + 10. + key.width / 2.,
                    y + rh / 2.,
                    " class=\"pele-json-key\" fill=\"var(--_m)\"",
                    r,
                )?);
                cells.push_str(&left_label(
                    value,
                    x0 + v.key_w + 10.,
                    y + rh / 2.,
                    &format!(" class=\"pele-json-value\"{attrs}"),
                    r,
                )?);
                y += rh;
            }
            ("pele-json",format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{RADIUS}\"{ATTRS}{shape}/><path class=\"pele-json-lines\" d=\"{lines}\" fill=\"none\" stroke=\"var(--_b)\"{}/>{}<g font-size=\"{}\">{cells}</g>",num(x0),num(-h/2.),num(w),num(h),v.style.line,label_svg_with_resources(&v.label,0.,-h/2.+v.head_h/2.,&format!(" class=\"pele-label\" font-weight=\"var(--_hw)\"{attrs}"),r)?,js_num(small)))
        } else {
            let (kind, mut outline) = if v.kind == Kind::Ellipse {
                (
                    "pele-usecase",
                    format!(
                        "<ellipse rx=\"{}\" ry=\"{}\"{ATTRS}{shape}/>",
                        num(w / 2.),
                        num(h / 2.)
                    ),
                )
            } else if n.shape == "note" {
                let right = w / 2.;
                let bottom = h / 2.;
                ("pele-note",format!("<path d=\"M{},{}H{}L{},{}V{}H{}Z\" fill=\"var(--_a)\" stroke=\"var(--_b)\" stroke-linejoin=\"round\"{shape}/><path d=\"M{},{}V{}H{}\" fill=\"none\" stroke=\"var(--_b)\" stroke-linejoin=\"round\"{}/>",num(-right),num(-bottom),num(right-9.),num(right),num(9.-bottom),num(bottom),num(-right),num(right-9.),num(-bottom),num(9.-bottom),num(right),v.style.line))
            } else {
                ("pele-usecase",format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{RADIUS}\"{ATTRS}{shape}/>",num(-w/2.),num(-h/2.),num(w),num(h)))
            };
            if v.kind == Kind::Ellipse && n.business {
                let from = w / 2. - 15.;
                let to = w / 2. - 5.;
                let arc = |a: f64| h / 2. * (1. - (a / (w / 2.)).powi(2)).sqrt();
                outline.push_str(&format!("<path class=\"pele-business\" d=\"M{},{}L{},{}\" fill=\"none\" stroke=\"var(--_b)\"{}/>",num(from),num(arc(from)),num(to),num(-arc(to)),v.style.line))
            }
            let body = format!(
                "{outline}{}{}",
                label_svg_with_resources(
                    &v.stereotype,
                    0.,
                    -v.label.height / 2.,
                    &format!(
                        " class=\"pele-stereotype\" font-size=\"{}\" fill=\"var(--_m)\"",
                        js_num(tiny)
                    ),
                    r
                )?,
                label_svg_with_resources(
                    &v.label,
                    0.,
                    v.stereotype.height / 2.,
                    &format!(
                        " class=\"pele-label\"{}{attrs}",
                        if n.shape == "note" {
                            format!(" font-size=\"{}\"", js_num(small))
                        } else {
                            String::new()
                        }
                    ),
                    r
                )?
            );
            (kind, body)
        };
        nodes_out.push_str(&format!("<g class=\"pele-node {kind}{classes}\" data-id=\"{id}\" transform=\"translate({},{})\">{body}</g>",num(c.x),num(c.y)));
    }
    let mut edges_out = String::new();
    let mut labels_out = String::new();
    let mut loops: HashMap<usize, usize> = HashMap::new();
    for (i, ce) in edges.iter_mut().enumerate() {
        let e = &drawn[i];
        let label = &labels[i];
        let mut declarations = e.compiled.clone();
        declarations.extend(e.styles.iter().cloned());
        let style = if declarations.is_empty() {
            None
        } else {
            Some(resolve_style(&declarations))
        };
        let path = if ce.src == ce.dst {
            let c = &nodes[ce.src];
            let v = &views[ce.src];
            let k = loops.entry(ce.src).or_insert(0);
            let side = if sideways { 2 } else { 1 };
            let spread = (side_length(v, side) / 2.).js_min(8. + *k as f64 * 6.);
            let reach = 26. + *k as f64 * 8.;
            *k += 1;
            let (ax, ay) = anchor(v, c, side, -spread);
            let (bx, by) = anchor(v, c, side, spread);
            let len = reach.hypot(spread);
            let len = if len == 0. { 1. } else { len };
            let ux = if sideways { spread } else { reach } / len;
            let uy = if sideways { reach } else { spread } / len;
            let st = trim(&e.arrow_start);
            let et = trim(&e.arrow_end);
            let out = |x: f64, y: f64, sign: f64, far: f64| {
                if sideways {
                    format!(
                        "{},{}",
                        num(x + sign * spread * if far != 0. { 1. } else { 0. }),
                        num(y + far)
                    )
                } else {
                    format!(
                        "{},{}",
                        num(x + far),
                        num(y + sign * spread * if far != 0. { 1. } else { 0. })
                    )
                }
            };
            let sx = if sideways { ax - ux * st } else { ax + ux * st };
            let sy = if sideways { ay + uy * st } else { ay - uy * st };
            let ex = bx + ux * et;
            let ey = by + uy * et;
            if sideways {
                ce.label_x = c.x;
                ce.label_y = ay.js_max(by) + reach * 0.75 + 6. + label.height / 2.
            } else {
                ce.label_x = ax.js_max(bx) + reach * 0.75 + 8. + label.width / 2.;
                ce.label_y = c.y
            }
            EdgePath {
                d: format!(
                    "M{},{}C{} {} {},{}",
                    num(sx),
                    num(sy),
                    out(ax, ay, -1., reach),
                    out(bx, by, 1., reach),
                    num(ex),
                    num(ey)
                ),
                sx: ax,
                sy: ay,
                sdx: if sideways { ux } else { -ux },
                sdy: if sideways { -uy } else { uy },
                ex: bx,
                ey: by,
                edx: -ux,
                edy: -uy,
            }
        } else {
            if ce.route.len() < 6 {
                continue;
            }
            route_path(
                &ce.route,
                None,
                trim(&e.arrow_start),
                trim(&e.arrow_end),
                None,
                None,
            )
        };
        let color = style
            .as_ref()
            .and_then(|s| s.stroke.as_ref())
            .filter(|s| !s.is_empty())
            .map_or_else(|| "var(--_l)".into(), |s| esc(s));
        let dashes = if e.internal {
            "2 3"
        } else if e.dotted {
            "5 4"
        } else if e.animate {
            "9 5"
        } else {
            ""
        };
        let classes = class_names(strip_relationship(&e.classes));
        edges_out.push_str(&format!(
            "<g class=\"pele-edge pele-{}{classes}\" data-id=\"{}\"{}><path d=\"{}\"{}/>{}{}</g>",
            e.kind,
            esc_text(&e.id)?,
            style.as_ref().map_or("", |s| s.line.as_str()),
            path.d,
            if dashes.is_empty() {
                String::new()
            } else {
                format!(" stroke-dasharray=\"{dashes}\"")
            },
            mark(&e.arrow_start, path.sx, path.sy, path.sdx, path.sdy, &color),
            mark(&e.arrow_end, path.ex, path.ey, path.edx, path.edy, &color)
        ));
        if label.width > 0. {
            let semantic = e.kind == "include" || e.kind == "extend";
            labels_out.push_str(&edge_label_svg(
                &e.id,
                label,
                ce.label_x,
                ce.label_y,
                &format!(
                    "{}{}",
                    if semantic { " fill=\"var(--_m)\"" } else { "" },
                    style.as_ref().map_or("", |s| s.text.as_str())
                ),
                r,
            )?);
        }
    }
    let content = format!(
        "{}{}{}{}<g class=\"pele-nodes\">{nodes_out}</g>",
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
        if edges_out.is_empty() {
            String::new()
        } else {
            format!("<g class=\"pele-edges\" fill=\"none\" stroke=\"var(--_l)\" stroke-linecap=\"round\">{edges_out}</g>")
        },
        if labels_out.is_empty() {
            String::new()
        } else {
            format!(
                "<g class=\"pele-edge-labels\" font-size=\"{}\">{labels_out}</g>",
                js_num(small)
            )
        }
    );
    let (acc_title, acc_descr) = phase.accessibility()?;
    let svg = svg::document(
        "usecase",
        width,
        height,
        size,
        o,
        Some(&acc_title),
        Some(&acc_descr),
        &content,
    )?;
    Ok(Rendered {
        svg,
        width,
        height,
        links: vec![],
    })
}
fn strip_default(s: &str) -> &str {
    s.strip_prefix("default").map_or(s, |s| {
        if s.starts_with(crate::compat::source::space) {
            &s[s.chars().next().unwrap().len_utf8()..]
        } else {
            s
        }
    })
}
fn strip_relationship(s: &str) -> &str {
    if let Some(s) = s.strip_prefix("default relationship relationship-") {
        let end = s.find(crate::compat::source::space).unwrap_or(s.len());
        let rest = &s[end..];
        if let Some(c) = rest.chars().next() {
            &rest[c.len_utf8()..]
        } else {
            rest
        }
    } else {
        s
    }
}
fn title_x(c: &CNode, width: f64, crossings: &Crossings) -> f64 {
    let y = round(c.y - c.h / 2.);
    let key = if y == 0. {
        0_f64.to_bits()
    } else {
        y.to_bits()
    };
    let Some(xs) = crossings.get(&key) else {
        return c.x;
    };
    if width == 0. {
        return c.x;
    }
    let left = c.x - c.w / 2. + 6.;
    let right = c.x + c.w / 2. - 6.;
    let mut cuts: Vec<_> = xs
        .iter()
        .copied()
        .filter(|x| *x > left && *x < right)
        .collect();
    cuts.sort_by(f64::total_cmp);
    cuts.push(right);
    let need = width / 2. + 6.;
    let mut best = c.x;
    let mut best_off = f64::INFINITY;
    let mut widest = 0.;
    let mut from = left;
    for cut in cuts {
        if cut - from >= 2. * need {
            let x = c.x.js_max(from + need).js_min(cut - need);
            if (x - c.x).abs() < best_off {
                best_off = (x - c.x).abs();
                best = x
            }
        } else if best_off == f64::INFINITY && cut - from > widest {
            widest = cut - from;
            best = (from + cut) / 2.
        }
        from = cut
    }
    best
}
fn left_label(
    label: &Label,
    x: f64,
    cy: f64,
    attrs: &str,
    r: &mut dyn RenderResources,
) -> Result<String, PeleError> {
    if label.lines.len() < 2 {
        return label_svg_with_resources(label, x + label.width / 2., cy, attrs, r);
    }
    let mut out = String::new();
    for (i, line) in label.lines.iter().enumerate() {
        let own = Label {
            lines: vec![line.clone()],
            widths: vec![label.widths[i]],
            height: label.line_height,
            ..label.clone()
        };
        out.push_str(&label_svg_with_resources(
            &own,
            x + label.widths[i] / 2.,
            cy - label.height / 2. + (i as f64 + 0.5) * label.line_height,
            attrs,
            r,
        )?)
    }
    Ok(out)
}
