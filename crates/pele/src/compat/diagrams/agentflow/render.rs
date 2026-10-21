//! Pele agentflow/render.ts, 458322a5 (MIT, kepano). Actual compound layout,
//! shape geometry, collapsed frames, border-held routes, kind colors and resource labels.
use super::phase::{PhaseInput, Typed};
use super::{
    graph::{AgentGraph, GraphEdge, GraphNode},
    model::{AgentflowModel, LabelType},
    shapes::{canonical, kind_slot},
};
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    layout::{
        compound::{compound_layout, direction, shift_layout, CEdge, CNode, Dir},
        layered::LayeredOptions,
    },
    source::{
        decode_entities,
        url::{link_url, linked, rel_attr, sanitize_url},
    },
    svg::{
        self,
        edges::{
            cluster::{cluster_title_x, mark_crossings, ClusterBox, Crossings},
            edge_label_svg, loop_path, marker, marker_trim, route_path,
            theme::{class_names, resolve_style, ResolvedStyle},
            EdgePath, LoopLabel,
        },
        esc, esc_text, js_num, label_svg, label_svg_with_resources, num, series_color,
        shapes::{draw_shape, inset_route, shape_size, PAD_X, PAD_Y},
    },
    text::{layout_label, Label},
    LinkInfo, Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
use std::collections::HashMap;
fn number(c: &Object, key: &str, fallback: f64) -> f64 {
    match c
        .get("agentflow")
        .and_then(Value::as_object)
        .and_then(|v| v.get(key))
    {
        Some(Value::Number(n)) if *n > 0. => n.js_min(2000.),
        _ => fallback,
    }
}
fn tint(slot: usize, wash: &str, style: &ResolvedStyle) -> String {
    let color = series_color(slot);
    format!(
        " fill=\"{color}\"{} stroke=\"{color}\"",
        if style.shape.contains("\"fill:") || style.shape.contains(";fill:") {
            String::new()
        } else {
            format!(" fill-opacity=\"{wash}\"")
        }
    )
}
struct View {
    node: GraphNode,
    style: ResolvedStyle,
    label: Label,
    shape: String,
    w: f64,
    h: f64,
    dy: f64,
    loops: usize,
}
fn flow_of(nodes: &[CNode], i: usize, root: Dir) -> Dir {
    let mut p = nodes[i].parent;
    for _ in 0..10000 {
        if p < 0 {
            break;
        }
        if let Some(d) = nodes[p as usize].dir {
            return d;
        }
        p = nodes[p as usize].parent;
    }
    root
}
fn inset(route: &mut [f64], at: usize, view: &View, c: &CNode) {
    inset_route(
        route,
        at,
        &view.shape,
        view.w,
        view.h,
        c.x,
        c.y,
        c.w,
        c.h,
        c.is_group,
    )
}
fn dot(x: f64, y: f64) -> String {
    format!(
        "M{},{}a1.75,1.75 0 1 0 3.5,0a1.75,1.75 0 1 0 -3.5,0Z",
        num(x - 1.75),
        num(y)
    )
}
pub(super) fn render(
    db: &AgentflowModel,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_phases(
        &mut Typed(db),
        db.graph
            .as_ref()
            .expect("parsed agentflow has its normalized graph"),
        c,
        o,
        r,
    )
}
pub(super) fn render_phases<P: PhaseInput>(
    db: &mut P,
    graph: &AgentGraph,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    let across_direction = matches!(db.metadata("direction")?.as_deref(), Some("LR" | "RL"));
    let selected = (|| -> Result<_, P::Error> {
        let across = draw(db, graph, c, o, r, false)?;
        if !across_direction {
            return Ok(across);
        }
        let Some(available) = o.max_width.filter(|v| *v > 0.) else {
            return Ok(across);
        };
        if o.auto_direction == Some(false)
            || across.0.width <= available
            || available >= o.direction_breakpoint.filter(|v| *v > 0.).unwrap_or(640.)
        {
            return Ok(across);
        }
        let down = draw(db, graph, c, o, r, true)?;
        Ok(if down.0.width < across.0.width {
            down
        } else {
            across
        })
    })()?;
    if let Some(frame) = selected.1 {
        r.capture_presentation(frame);
    }
    Ok(selected.0)
}
fn contains(nodes: &[CNode], depth: &[isize], group: usize, i: usize) -> bool {
    let mut j = i as isize;
    for _ in depth[group]..depth[i] {
        j = nodes[j as usize].parent;
    }
    j == group as isize && i != group
}
fn put(
    edge: &GraphEdge,
    label: &Label,
    path: &EdgePath,
    x: f64,
    y: f64,
    edges: &mut String,
    labels: &mut String,
    r: &mut dyn RenderResources,
) -> Result<(), PeleError> {
    let mut declarations = edge.css_compiled_styles.clone();
    declarations.extend(edge.style.iter().cloned());
    let style = (!declarations.is_empty()).then(|| resolve_style(&declarations));
    let color = style
        .as_ref()
        .and_then(|s| s.stroke.as_ref())
        .filter(|s| !s.is_empty())
        .map_or_else(|| "var(--_l)".into(), |s| esc(s));
    let kind = if edge.thickness.as_deref() == Some("dotted") {
        "reference"
    } else if edge.arrow_type_end == "arrow_cross" {
        "failure"
    } else {
        "sequence"
    };
    let classes = class_names(
        &edge
            .classes
            .replace("edge-thickness-normal", "")
            .replace("edge-pattern-solid", "")
            .replace("flowchart-link", ""),
    );
    edges.push_str(&format!("<g class=\"pele-edge pele-edge-{kind}{classes}\" data-id=\"{}\"{}><path d=\"{}\"{}/>{}</g>",esc_text(&edge.id)?,style.as_ref().map_or("",|s|s.line.as_str()),path.d,if edge.thickness.as_deref()==Some("dotted"){" stroke-dasharray=\"3 4\""}else{""},marker(&edge.arrow_type_end,path.ex,path.ey,path.edx,path.edy,&color)));
    if label.width > 0. {
        labels.push_str(&edge_label_svg(
            &edge.id,
            label,
            x,
            y,
            style.as_ref().map_or("", |s| s.text.as_str()),
            r,
        )?);
    }
    Ok(())
}
fn draw<P: PhaseInput>(
    db: &mut P,
    graph: &AgentGraph,
    config: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    turned: bool,
) -> Result<(Rendered, Option<crate::compat::PresentationOutcome>), P::Error> {
    let size = o.font_size.unwrap_or(16.);
    let edge_size = round(size * 0.875);
    let wrap = number(config, "wrappingWidth", 120.);
    let min_width = number(config, "minNodeWidth", 120.);
    let pad = o.padding.unwrap_or(8.);
    let index: HashMap<_, _> = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.clone(), i))
        .collect();
    let mut views = vec![];
    let mut nodes = vec![];
    for (i, original) in graph.nodes.iter().enumerate() {
        let mut node = original.clone();
        db.resolve_styles(i, &mut node)?;
        let mut declarations = node.css_compiled_styles.clone();
        declarations.extend(node.css_styles.iter().cloned());
        let style = resolve_style(&declarations);
        let bits = if style.bold { 1 } else { 0 } | if style.italic { 2 } else { 0 };
        let markdown = node.label_type == LabelType::Markdown;
        let font = style.font_size.unwrap_or(size);
        let (mut c, view) = if node.is_group {
            let label = layout_label(node.label.as_deref(), markdown, r, o, font, 4000., bits)?;
            let mut c = CNode::new(0., 0., -1);
            c.is_group = true;
            c.dir = node
                .dir
                .as_deref()
                .filter(|s| !s.is_empty())
                .map(|s| direction(Some(s)));
            c.pad_x = 20.;
            c.pad_top = if label.height > 0. {
                label.height + 16.
            } else {
                20.
            };
            c.pad_bottom = 20.;
            c.min_w = label.width + 28.;
            (
                c,
                View {
                    node,
                    style,
                    label,
                    shape: "group".into(),
                    w: 0.,
                    h: 0.,
                    dy: 0.,
                    loops: 0,
                },
            )
        } else if node.shape == "collapsedGroup" {
            let label = layout_label(node.label.as_deref(), markdown, r, o, font, wrap, bits)?;
            let w = label.width.js_max(min_width) + 2. * PAD_X;
            let h = label.height + 2. * PAD_Y + 14.;
            (
                CNode::new(w, h, -1),
                View {
                    node,
                    style,
                    label,
                    shape: "collapsed".into(),
                    w,
                    h,
                    dy: -7.,
                    loops: 0,
                },
            )
        } else {
            let shape = canonical(Some(&node.shape)).to_string();
            let label = layout_label(node.label.as_deref(), markdown, r, o, font, wrap, bits)?;
            let s = shape_size(
                &shape,
                if shape == "diam" {
                    label.width
                } else {
                    label.width.js_max(min_width)
                },
                label.height,
            );
            (
                CNode::new(s.w, s.h, -1),
                View {
                    node,
                    style,
                    label,
                    shape,
                    w: s.w,
                    h: s.h,
                    dy: s.dy,
                    loops: 0,
                },
            )
        };
        c.seq = views.len();
        views.push(view);
        nodes.push(c);
    }
    for (i, v) in views.iter().enumerate() {
        nodes[i].parent = v
            .node
            .parent_id
            .as_ref()
            .and_then(|p| index.get(p))
            .map_or(-1, |i| *i as isize);
    }
    let root = if turned {
        Dir::TB
    } else {
        direction(db.metadata("direction")?.as_deref())
    };
    let mut depth = vec![-1_isize; nodes.len()];
    for i in 0..nodes.len() {
        let mut chain = vec![];
        let mut j = i as isize;
        while j >= 0 && depth[j as usize] < 0 && chain.len() <= nodes.len() {
            chain.push(j as usize);
            j = nodes[j as usize].parent;
        }
        let mut d = if j >= 0 { depth[j as usize] + 1 } else { 0 };
        for k in chain.into_iter().rev() {
            depth[k] = d;
            d += 1;
        }
    }
    let mut edges = vec![];
    let mut labels = vec![];
    let mut drawn = vec![];
    let mut held = vec![];
    for edge in &graph.edges {
        let (Some(&src), Some(&dst)) = (index.get(&edge.start), index.get(&edge.end)) else {
            continue;
        };
        let label = layout_label(
            Some(&edge.label),
            edge.label_type == LabelType::Markdown,
            r,
            o,
            edge_size,
            wrap,
            0,
        )?;
        if contains(&nodes, &depth, src, dst) || contains(&nodes, &depth, dst, src) {
            held.push((edge, label, src, dst));
            continue;
        }
        if src == dst {
            views[src].loops += 1;
            if flow_of(&nodes, src, root).sideways() {
                nodes[src].h += 2.
                    * (26.
                        + if label.height > 0. {
                            label.height + 8.
                        } else {
                            0.
                        });
            } else {
                nodes[src].w += 2.
                    * (26.
                        + if label.width > 0. {
                            label.width + 12.
                        } else {
                            0.
                        });
            }
        }
        drawn.push(edge);
        edges.push(CEdge {
            src,
            dst,
            minlen: match &edge.minlen {
                Some(Value::Number(n)) => *n,
                _ => 1.,
            },
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
    }
    let layout = compound_layout(
        &mut nodes,
        &mut edges,
        root,
        &LayeredOptions {
            node_sep: number(config, "nodeSpacing", 40.),
            edge_sep: 16.,
            rank_sep: number(config, "rankSpacing", 48.),
            port_sep: 20.,
            tracks: false,
            straight: false,
            apart: false,
            head_room: 0.,
        },
    );
    let title_text = db.metadata("title")?;
    let title = layout_label(title_text.as_deref(), false, r, o, size, 4000., 1)?;
    let title_height = if title.height > 0. {
        title.height + 12.
    } else {
        0.
    };
    let inner = layout.width.js_max(title.width);
    let ox = pad + (inner - layout.width) / 2.;
    let oy = pad + title_height;
    shift_layout(&mut nodes, &mut edges, ox, oy);
    let width = (inner + 2. * pad).ceil();
    let height = (layout.height + title_height + 2. * pad).ceil();
    let frame_access = if r.presentation_budget().is_some() {
        Some((
            db.metadata("accTitle")?,
            db.metadata("title")?,
            db.metadata("accDescr")?,
        ))
    } else {
        None
    };
    let mut presentation = r.presentation_budget().map(|budget| {
        crate::compat::presentation::FrameBuilder::new(
            budget,
            width,
            height,
            frame_access
                .as_ref()
                .and_then(|(a, t, _)| a.as_deref().or(t.as_deref())),
            frame_access.as_ref().and_then(|(_, _, d)| d.as_deref()),
        )
    });
    let mut links = vec![];
    let mut held_paths = vec![];
    let mut held_labels = vec![];
    let mut held_routes = vec![];
    let careful = held.len().saturating_mul(nodes.len()) < 2000000;
    for (edge, _, src, dst) in &held {
        let out = contains(&nodes, &depth, *dst, *src);
        let outer = if out { *dst } else { *src };
        let i = if out { *src } else { *dst };
        let g = &nodes[outer];
        let n = &nodes[i];
        let end = match g.dir.unwrap_or_else(|| flow_of(&nodes, outer, root)) {
            Dir::TB => 2,
            Dir::RL => 3,
            Dir::BT => 0,
            Dir::LR => 1,
        };
        let first = if out { end } else { (end + 2) & 3 };
        let toward = |side: usize| {
            let upright = side & 1 == 0;
            let sign = if side == 0 || side == 3 { -1. } else { 1. };
            let nx = if upright { n.x } else { n.x + sign * n.w / 2. };
            let ny = if upright { n.y + sign * n.h / 2. } else { n.y };
            let bx = if upright { n.x } else { g.x + sign * g.w / 2. };
            let by = if upright { g.y + sign * g.h / 2. } else { n.y };
            let axis = if upright { 0. } else { 1. };
            if out {
                vec![nx, ny, axis, bx, by, axis]
            } else {
                vec![bx, by, axis, nx, ny, axis]
            }
        };
        let blocked = |route: &[f64]| {
            nodes.iter().enumerate().any(|(k, c)| {
                k != i
                    && !c.is_group
                    && c.x + c.w / 2. > route[0].js_min(route[3]) - 1.
                    && c.x - c.w / 2. < route[0].js_max(route[3]) + 1.
                    && c.y + c.h / 2. > route[1].js_min(route[4]) - 1.
                    && c.y - c.h / 2. < route[1].js_max(route[4]) + 1.
            })
        };
        let mut route = toward(first);
        if careful && blocked(&route) {
            for side in [(first + 1) & 3, (first + 3) & 3, (first + 2) & 3] {
                let other = toward(side);
                if !blocked(&other) {
                    route = other;
                    break;
                }
            }
        }
        held_labels.push(((route[0] + route[3]) / 2., (route[1] + route[4]) / 2.));
        let mut drawn_route = route.clone();
        inset(&mut drawn_route, if out { 0 } else { 3 }, &views[i], n);
        held_paths.push(route_path(
            &drawn_route,
            Some("linear"),
            0.,
            marker_trim(&edge.arrow_type_end),
            None,
            None,
        ));
        held_routes.push(route);
    }
    let mut crossings = Crossings::new();
    if nodes.iter().any(|n| n.is_group) {
        for e in &edges {
            mark_crossings(&mut crossings, &e.route)
        }
        for route in &held_routes {
            mark_crossings(&mut crossings, route)
        }
    }
    let (mut clusters, mut nodes_out) = (String::new(), String::new());
    for (i, v) in views.iter().enumerate() {
        let c = &nodes[i];
        let node = &v.node;
        if let Some(frame) = &mut presentation {
            frame.label(&node.id, &v.label);
        }
        let classes = class_names(
            node.css_classes
                .strip_prefix("default")
                .map_or(node.css_classes.as_str(), |s| {
                    s.strip_prefix(' ').unwrap_or(s)
                }),
        );
        let id = esc_text(&node.id)?;
        if c.is_group {
            let title_x = cluster_title_x(
                &ClusterBox {
                    x: c.x,
                    y: c.y,
                    w: c.w,
                    h: c.h,
                },
                v.label.width,
                &crossings,
                20.,
            );
            clusters.push_str(&format!("<g class=\"pele-cluster pele-flow{classes}\" data-id=\"{id}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"var(--pele-radius,4px)\"{}{}/>{}</g>",num(c.x-c.w/2.),num(c.y-c.h/2.),num(c.w),num(c.h),tint(node.color_index.unwrap_or(7),"0.06",&v.style),v.style.shape,label_svg_with_resources(&v.label,title_x,c.y-c.h/2.+8.+v.label.height/2.,&format!(" class=\"pele-cluster-label\" fill=\"var(--_m)\"{}",v.style.text),r)?));
            continue;
        }
        let mut kind = String::new();
        let shape = if v.shape == "collapsed" {
            let color = series_color(node.color_index.unwrap_or(7));
            let line = v.h / 2. - 16.;
            let dots = line + 8.;
            format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"var(--pele-radius,4px)\"{}{}/><path d=\"M{},{}H{}\" fill=\"none\" stroke=\"{color}\" stroke-dasharray=\"3 3\"{}/><path class=\"pele-collapsed-mark\" d=\"{}{}{}\" fill=\"{color}\" stroke=\"none\"/>",num(-v.w/2.),num(-v.h/2.),num(v.w),num(v.h),tint(node.color_index.unwrap_or(7),"0.06",&v.style),v.style.shape,num(-v.w/2.+10.),num(line),num(v.w/2.-10.),v.style.line,dot(-9.,dots),dot(0.,dots),dot(9.,dots))
        } else {
            let node_kind = node.kind.as_deref().unwrap_or("task");
            kind = format!(" pele-kind-{node_kind}");
            let slot = kind_slot(node_kind).unwrap_or(1);
            draw_shape(
                &v.shape,
                v.w,
                v.h,
                &format!("{}{}", tint(slot, "0.14", &v.style), v.style.shape),
                &format!(" stroke=\"{}\"{}", series_color(slot), v.style.line),
            )
        };
        let mut text = label_svg_with_resources(
            &v.label,
            0.,
            v.dy,
            &format!(" class=\"pele-label\"{}", v.style.text),
            r,
        )?;
        if classes.split_whitespace().any(|s| s == "internal-link")
            && node.label.as_ref().is_some_and(|s| !s.is_empty())
            && o.links != Some(false)
        {
            let name = decode_entities(node.label.as_deref().unwrap())?;
            let safe = link_url(&sanitize_url(&name), o).is_some();
            let target = esc(&name);
            text = format!(
                "<a class=\"internal-link\"{} data-href=\"{target}\">{text}</a>",
                if safe {
                    format!(" href=\"{target}\"{}", rel_attr(o))
                } else {
                    String::new()
                }
            );
            if safe {
                if let Some(frame) = &mut presentation {
                    frame.link(
                        &node.id,
                        &name,
                        true,
                        crate::compat::PresentationRect::centered(
                            c.x,
                            c.y + v.dy,
                            v.label.width,
                            v.label.height,
                        ),
                    );
                }
            }
            links.push(LinkInfo {
                id: node.id.clone(),
                href: name,
                internal: true,
            });
        }
        let mut body = shape + &text;
        if let Some(tooltip) = node.tooltip.as_ref().filter(|s| !s.is_empty()) {
            body = format!("<title>{}</title>{body}", esc_text(tooltip)?)
        }
        if let Some(link) = node.link.as_ref().filter(|s| !s.is_empty()) {
            let attrs = node
                .link_target
                .as_ref()
                .filter(|s| !s.is_empty())
                .map_or_else(String::new, |s| format!(" target=\"{}\"", esc(s)));
            let link_index = links.len();
            body = linked(&body, link, &node.id, &mut links, o, &attrs);
            if let Some(frame) = &mut presentation {
                if let Some(approved) = links.get(link_index) {
                    frame.link(
                        &node.id,
                        &approved.href,
                        false,
                        crate::compat::PresentationRect::centered(c.x, c.y, v.w, v.h),
                    );
                }
            }
        }
        nodes_out.push_str(&format!("<g class=\"pele-node pele-shape-{}{kind}{classes}\" data-id=\"{id}\" transform=\"translate({},{})\">{body}</g>",v.shape,num(c.x),num(c.y)));
    }
    let (mut edges_out, mut labels_out) = (String::new(), String::new());
    let mut loop_seen = HashMap::new();
    for (i, ce) in edges.iter_mut().enumerate() {
        let edge = drawn[i];
        let label = &labels[i];
        let path = if ce.src == ce.dst {
            let v = &views[ce.src];
            let k = loop_seen.entry(ce.src).or_insert(0);
            let mut lab = LoopLabel {
                label_w: ce.label_w,
                label_h: ce.label_h,
                label_x: ce.label_x,
                label_y: ce.label_y,
            };
            let path = loop_path(
                nodes[ce.src].x,
                nodes[ce.src].y,
                v.w,
                v.h,
                *k,
                flow_of(&nodes, ce.src, root).sideways(),
                26.,
                marker_trim(&edge.arrow_type_end),
                &mut lab,
            );
            *k += 1;
            ce.label_x = lab.label_x;
            ce.label_y = lab.label_y;
            path
        } else {
            let mut route = ce.route.clone();
            let last = route.len() - 3;
            inset(&mut route, 0, &views[ce.src], &nodes[ce.src]);
            inset(&mut route, last, &views[ce.dst], &nodes[ce.dst]);
            let curve = match &edge.curve {
                Some(Value::String(s)) => Some(s.as_str()),
                _ => None,
            };
            route_path(
                &route,
                curve,
                0.,
                marker_trim(&edge.arrow_type_end),
                None,
                None,
            )
        };
        if label.width > 0. {
            if let Some(frame) = &mut presentation {
                frame.label(&edge.id, label);
            }
        }
        put(
            edge,
            label,
            &path,
            ce.label_x,
            ce.label_y,
            &mut edges_out,
            &mut labels_out,
            r,
        )?;
    }
    for (i, (edge, label, _, _)) in held.iter().enumerate() {
        if label.width > 0. {
            if let Some(frame) = &mut presentation {
                frame.label(&edge.id, label);
            }
        }
        put(
            edge,
            label,
            &held_paths[i],
            held_labels[i].0,
            held_labels[i].1,
            &mut edges_out,
            &mut labels_out,
            r,
        )?;
    }
    let mut content = label_svg(
        &title,
        width / 2.,
        pad + title.height / 2.,
        " class=\"pele-title\" font-weight=\"var(--_tw)\"",
    );
    if !clusters.is_empty() {
        content.push_str(&format!("<g class=\"pele-clusters\">{clusters}</g>"))
    }
    if !edges_out.is_empty() {
        content.push_str(&format!("<g class=\"pele-edges\" fill=\"none\" stroke=\"var(--_l)\" stroke-linecap=\"round\">{edges_out}</g>"))
    }
    if !labels_out.is_empty() {
        content.push_str(&format!(
            "<g class=\"pele-edge-labels\" font-size=\"{}\">{labels_out}</g>",
            js_num(edge_size)
        ))
    }
    content.push_str(&format!("<g class=\"pele-nodes\">{nodes_out}</g>"));
    let acc_title = db.metadata("accTitle")?;
    let acc_descr = db.metadata("accDescr")?;
    let svg = svg::document(
        "agentflow",
        width,
        height,
        size,
        o,
        acc_title.as_deref(),
        acc_descr.as_deref(),
        &content,
    )?;
    Ok((
        Rendered {
            svg,
            width,
            height,
            links,
        },
        presentation.map(|frame| frame.finish()),
    ))
}
