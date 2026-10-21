//! Pele state/render.ts compound state renderer, 458322a5 (MIT, kepano).
use super::{
    model::StateModel,
    phase::{PhaseInput, Typed},
};
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    layout::{
        compound::{compound_layout, direction, CEdge, CNode, Dir},
        layered::LayeredOptions,
    },
    source::{
        decode_entities,
        url::{linked, safe_url},
    },
    svg::{
        self,
        edges::{
            cluster::{mark_crossings, Crossings},
            edge_label_svg, marker, marker_trim, route_path,
            theme::{class_names, resolve_style, ResolvedStyle},
            EdgePath, SideReach,
        },
        esc, esc_text, label_svg_with_resources as label_svg, num,
        shapes::end_side,
        title::with_title,
    },
    text::{layout_label, Label},
    Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
use std::collections::HashMap;
const SURFACE: &str = " fill=\"var(--_s)\" stroke=\"var(--_b)\"";
const RULE: &str = " fill=\"none\" stroke=\"var(--_b)\"";
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum K {
    State,
    Described,
    Start,
    End,
    Choice,
    Fork,
    Join,
    Note,
    Composite,
    Region,
    Hidden,
}
const CLASSES: [&str; 9] = [
    "pele-node pele-state",
    "pele-node pele-state",
    "pele-node pele-state-start",
    "pele-node pele-state-end",
    "pele-node pele-state-choice",
    "pele-node pele-state-fork",
    "pele-node pele-state-join",
    "pele-note",
    "pele-cluster pele-state-composite",
];
struct View<V> {
    node: Option<V>,
    kind: K,
    style: Option<ResolvedStyle>,
    label: Label,
    lines: Vec<Label>,
    w: f64,
    h: f64,
    title_h: f64,
}
impl<V> View<V> {
    fn new(node: Option<V>, kind: K, label: &Label) -> Self {
        Self {
            node,
            kind,
            style: None,
            label: label.clone(),
            lines: Vec::new(),
            w: 0.,
            h: 0.,
            title_h: 0.,
        }
    }
}
// The upstream clamp uses comparisons, not Math.min/max: NaN bounds leave
// the incoming coordinate alone, and NaN values remain NaN without a panic.
fn clamp(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}
fn number_option(config: &Object, key: &str, fallback: f64) -> f64 {
    match config
        .get("state")
        .and_then(Value::as_object)
        .and_then(|v| v.get(key))
    {
        Some(Value::Number(v)) if *v > 0. => v.js_min(2000.),
        _ => fallback,
    }
}
fn rect(x: f64, y: f64, w: f64, h: f64, rest: &str) -> String {
    format!(
        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{rest}/>",
        num(x),
        num(y),
        num(w),
        num(h)
    )
}
fn group(dir: Dir, pad_x: f64, pad_top: f64, pad_bottom: f64) -> CNode {
    let mut c = CNode::new(0., 0., -1);
    c.is_group = true;
    c.dir = Some(dir);
    c.pad_x = pad_x;
    c.pad_top = pad_top;
    c.pad_bottom = pad_bottom;
    c
}
fn across<P: PhaseInput>(
    phase: &mut P,
    i: usize,
    logical: &[isize],
    views: &[View<P::Value>],
    root: Dir,
) -> Result<bool, P::Error> {
    Ok(!if logical[i] < 0 {
        root
    } else {
        direction(
            phase
                .field(views[logical[i] as usize].node.as_ref().unwrap(), "dir")?
                .as_deref(),
        )
    }
    .sideways())
}

fn encloses(composite: usize, item: usize, logical: &[isize], nodes: &[CNode]) -> bool {
    if !nodes[composite].is_group {
        return false;
    }
    let mut p = logical[item];
    while p >= 0 {
        if p as usize == composite {
            return true;
        }
        p = logical[p as usize]
    }
    false
}
fn title_x(c: &CNode, width: f64, crossings: &Crossings) -> f64 {
    if width == 0. {
        return c.x;
    }
    let y = round(c.y - c.h / 2.);
    let key = if y == 0. { 0. } else { y }.to_bits();
    let Some(xs) = crossings.get(&key) else {
        return c.x;
    };
    let left = c.x - c.w / 2.;
    let half = width / 2.;
    let mut best = c.x;
    let mut best_gap = -1.;
    for x in [c.x, left + 16. + half, left + c.w - 16. - half] {
        let mut gap = f64::INFINITY;
        for &cx in xs {
            if cx > left && cx < left + c.w {
                gap = gap.js_min((cx - x).abs() - half)
            }
        }
        if gap >= 6. {
            return x;
        }
        if gap > best_gap + 0.5 {
            best_gap = gap;
            best = x
        }
    }
    best
}
fn cleaned_classes(text: &str) -> String {
    let mut out = String::new();
    let mut p = 0;
    while p < text.len() {
        let start = p;
        if p == 0 || text.as_bytes()[p] == b' ' {
            let at = if text.as_bytes()[p] == b' ' { p + 1 } else { p };
            let mut removed = false;
            for name in [
                "statediagram-state",
                "statediagram-cluster-alt",
                "statediagram-cluster",
                "statediagram-note",
            ] {
                if text[at..].starts_with(name)
                    && matches!(text.as_bytes().get(at + name.len()), None | Some(b' '))
                {
                    p = at + name.len();
                    removed = true;
                    break;
                }
            }
            if removed {
                continue;
            }
        }
        let c = text[start..].chars().next().unwrap();
        out.push(c);
        p = start + c.len_utf8()
    }
    out
}
fn divide<V>(
    items: &[usize],
    views: &[View<V>],
    nodes: &[CNode],
    pad: f64,
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
) -> String {
    let regions: Vec<usize> = items
        .iter()
        .copied()
        .filter(|&i| views[i].kind == K::Region)
        .collect();
    if regions.len() < 2 {
        return String::new();
    }
    let lo = |i: usize, x: bool| {
        if x {
            nodes[i].x - nodes[i].w / 2.
        } else {
            nodes[i].y - nodes[i].h / 2.
        }
    };
    let hi = |i: usize, x: bool| {
        if x {
            nodes[i].x + nodes[i].w / 2.
        } else {
            nodes[i].y + nodes[i].h / 2.
        }
    };
    for x in [true, false] {
        let mut sorted = regions.clone();
        sorted.sort_by(|&a, &b| {
            lo(a, x)
                .partial_cmp(&lo(b, x))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.cmp(&b))
        });
        if sorted.windows(2).any(|v| hi(v[0], x) > lo(v[1], x) + 0.5) {
            continue;
        }
        let mut out = String::new();
        for v in sorted.windows(2) {
            let at = num((hi(v[0], x) + lo(v[1], x)) / 2. + pad);
            let d = if x {
                format!("{at},{}V{}", num(top), num(bottom))
            } else {
                format!("{},{at}H{}", num(left), num(right))
            };
            out += &format!("<path class=\"pele-divider\" d=\"M{d}\"/>")
        }
        return out;
    }
    regions
        .iter()
        .map(|&i| {
            let c = &nodes[i];
            rect(
                c.x - c.w / 2. + pad,
                c.y - c.h / 2. + pad,
                c.w,
                c.h,
                " class=\"pele-divider\" rx=\"10\"",
            )
        })
        .collect()
}
fn attach<V>(route: &mut [f64], at: usize, i: usize, nodes: &[CNode], views: &[View<V>], pad: f64) {
    let c = &nodes[i];
    if c.is_group {
        return;
    }
    let view = &views[i];
    let bar = matches!(view.kind, K::Fork | K::Join);
    let other = if at == 0 { 3 } else { at - 3 };
    for axis in [1, 0] {
        let (extent, drawn_extent, breadth, center, cross) = if axis == 1 {
            (c.h, view.h, view.w, c.y + pad, c.x + pad)
        } else {
            (c.w, view.w, view.h, c.x + pad, c.y + pad)
        };
        let delta = route[at + axis] - center;
        if (delta.abs() - extent / 2.).abs() >= 0.5 {
            continue;
        }
        let sign = if delta == 0. { delta } else { delta.signum() };
        route[at + axis] -= sign * (extent - drawn_extent) / 2.;
        if bar && breadth > drawn_extent {
            route[at + 1 - axis] = clamp(
                route[other + 1 - axis],
                cross - breadth / 2. + 6.,
                cross + breadth / 2. - 6.,
            )
        }
        return;
    }
}
pub(super) fn render(
    db: &StateModel,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_phases(&mut Typed(db), config, options, resources)
}
pub(super) fn render_phases<P: PhaseInput>(
    phase: &mut P,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    let responsive = matches!(phase.direction()?.as_str(), "LR" | "RL");
    let first = draw(phase, config, options, resources, false)?;
    if !responsive {
        return Ok(first);
    }
    let Some(available) = options.max_width.filter(|w| *w > 0.) else {
        return Ok(first);
    };
    if options.auto_direction == Some(false)
        || first.width <= available
        || available
            >= options
                .direction_breakpoint
                .filter(|b| *b > 0.)
                .unwrap_or(640.)
    {
        return Ok(first);
    }
    let down = draw(phase, config, options, resources, true)?;
    Ok(if down.width < first.width {
        down
    } else {
        first
    })
}
fn draw<P: PhaseInput>(
    phase: &mut P,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
    turned: bool,
) -> Result<Rendered, P::Error> {
    phase.begin_draw();
    let graph = phase.get(&phase.root(), "graph")?;
    let size = options.font_size.unwrap_or(16.);
    let small = round(size * 0.875);
    let wrap = number_option(config, "wrappingWidth", 200.);
    let pad = options.padding.unwrap_or(8.);
    let root_dir = if turned {
        Dir::TB
    } else {
        direction(Some(&phase.direction()?))
    };
    let no_label = layout_label(None, false, resources, options, size, 0., 0)?;
    let mut index = HashMap::new();
    let mut views = Vec::new();
    let mut nodes = Vec::new();
    let node_values = phase.get(&graph, "nodes")?;
    let mut cursor = 0;
    while cursor < phase.count(&node_values)? {
        phase.step()?;
        let node = phase.index(&node_values, cursor)?;
        cursor += 1;
        let shape = phase.field(&node, "shape")?;
        let group = phase.get(&node, "isGroup")?;
        if shape.as_deref() == Some("noteGroup") {
            continue;
        }
        let kind = if phase.truthy(&group) {
            if shape.as_deref() == Some("divider") {
                K::Region
            } else {
                K::Composite
            }
        } else {
            match shape.as_deref() {
                Some("rectWithTitle") => K::Described,
                Some("stateStart") => K::Start,
                Some("stateEnd") => K::End,
                Some("choice") => K::Choice,
                Some("fork") => K::Fork,
                Some("join") => K::Join,
                Some("note") => K::Note,
                Some("divider") => K::Hidden,
                _ => K::State,
            }
        };
        index.insert(phase.string(&node, "id")?, views.len());
        views.push(View::new(Some(node), kind, &no_label));
    }
    let states = views.len();
    let mut logical = Vec::new();
    for i in 0..states {
        let parent = phase.field(views[i].node.as_ref().unwrap(), "parentId")?;
        let p = parent.as_ref().and_then(|id| index.get(id)).copied();
        let p = if let Some(p) = p.filter(|p| *p != i) {
            let value = phase.get(views[p].node.as_ref().unwrap(), "isGroup")?;
            if phase.truthy(&value) {
                p as isize
            } else {
                -1
            }
        } else {
            -1
        };
        logical.push(p);
    }

    {
        let mut kids = vec![Vec::new(); states];
        let mut queue = Vec::new();
        let mut placed = vec![false; states];
        for i in 0..states {
            if logical[i] < 0 {
                placed[i] = true;
                queue.push(i)
            } else {
                kids[logical[i] as usize].push(i)
            }
        }
        let mut done = 0;
        let settle = |queue: &mut Vec<usize>, placed: &mut [bool], done: &mut usize| {
            while *done < queue.len() {
                for &kid in &kids[queue[*done]] {
                    if !placed[kid] {
                        placed[kid] = true;
                        queue.push(kid)
                    }
                }
                *done += 1
            }
        };
        settle(&mut queue, &mut placed, &mut done);
        let mut seen = vec![usize::MAX; states];
        for i in 0..states {
            if placed[i] {
                continue;
            }
            let mut ring = i;
            while seen[ring] != i {
                seen[ring] = i;
                ring = logical[ring] as usize
            }
            logical[ring] = -1;
            placed[ring] = true;
            queue.push(ring);
            settle(&mut queue, &mut placed, &mut done)
        }
    }
    let mut owners = vec![-1isize; states];
    for i in 0..states {
        let owner = phase.field(views[i].node.as_ref().unwrap(), "noteFor")?;
        if let Some(&owner) = owner.as_ref().and_then(|id| index.get(id)) {
            owners[i] = owner as isize;
            logical[i] = logical[owner];
        }
    }
    let mut degree = vec![0usize; states];
    let mut noted = vec![false; states];
    let mut appear = vec![f64::INFINITY; states];
    let mut appeared = 0.;
    let mut counts = vec![0usize; 2 * states];
    let edge_values = phase.get(&graph, "edges")?;
    let mut cursor = 0;
    while cursor < phase.count(&edge_values)? {
        phase.step()?;
        let edge = phase.index(&edge_values, cursor)?;
        cursor += 1;
        let start = phase.string(&edge, "start")?;
        let end = phase.string(&edge, "end")?;
        let (Some(&src), Some(&dst)) = (index.get(start.as_str()), index.get(end.as_str())) else {
            continue;
        };
        if phase.field(&edge, "pattern")?.as_deref() == Some("dashed") {
            noted[if views[src].kind == K::Note { dst } else { src }] = true;
            continue;
        }
        counts[2 * src] += 1;
        degree[src] = degree[src].max(counts[2 * src]);
        counts[2 * dst + 1] += 1;
        degree[dst] = degree[dst].max(counts[2 * dst + 1]);
        for i in [src, dst] {
            if appear[i] == f64::INFINITY {
                appear[i] = appeared;
                appeared += 1.
            }
        }
    }
    for i in 0..states {
        let node = views[i].node.as_ref().unwrap().clone();
        let styles = phase.get(&node, "cssStyles")?;
        let count = phase.count(&styles)?;
        let compiled = phase.get(&node, "cssCompiledStyles")?;
        let mut styles_all = phase.strings(&compiled)?;
        if count > 0 {
            styles_all.extend(phase.strings(&styles)?)
        }
        let side = if matches!(views[i].kind, K::Fork | K::Join) {
            across(phase, i, &logical, &views, root_dir)?
        } else {
            false
        };
        let view = &mut views[i];
        let styles = styles_all;
        let style = resolve_style(&styles);
        let bits = u8::from(style.bold) | if style.italic { 2 } else { 0 };
        let font = style.font_size.unwrap_or(size);
        let mut c = None;
        match view.kind {
            K::Composite => {
                view.label = layout_label(
                    phase.field(&node, "label")?.as_deref(),
                    true,
                    resources,
                    options,
                    font,
                    4000.,
                    bits,
                )?;
                view.title_h = if view.label.height > 0. {
                    view.label.height + 12.
                } else {
                    0.
                };
                let mut g = group(
                    direction(phase.field(&node, "dir")?.as_deref()),
                    16.,
                    view.title_h + 16.,
                    16.,
                );
                g.min_w = view.label.width + 32.;
                c = Some(g)
            }
            K::Region => {
                c = Some(group(
                    direction(phase.field(&node, "dir")?.as_deref()),
                    10.,
                    10.,
                    10.,
                ))
            }
            K::Start => {
                view.w = 14.;
                view.h = 14.
            }
            K::End => {
                view.w = 18.;
                view.h = 18.
            }
            K::Choice => {
                view.w = 30.;
                view.h = 30.
            }
            K::Fork | K::Join => {
                let length = clamp(40. * degree[i] as f64 + 16., 72., 320.);
                view.w = if side { length } else { 8. };
                view.h = if side { 8. } else { length }
            }
            K::Hidden => {}
            K::Note => {
                view.label = layout_label(
                    phase.field(&node, "label")?.as_deref(),
                    true,
                    resources,
                    options,
                    small,
                    wrap.js_max(320.),
                    0,
                )?;
                view.w = view.label.width + 24.;
                view.h = view.label.height + 16.
            }
            _ => {
                view.label = layout_label(
                    phase.field(&node, "label")?.as_deref(),
                    true,
                    resources,
                    options,
                    font,
                    wrap,
                    bits,
                )?;
                let mut width = view.label.width;
                let mut height = view.label.height + 20.;
                if view.kind == K::Described {
                    height += 14.;
                    let descriptions = phase.get(&node, "description")?;
                    let mut cursor = 0;
                    while cursor < phase.count(&descriptions)? {
                        phase.step()?;
                        let value = phase.index(&descriptions, cursor)?;
                        cursor += 1;
                        let line = phase.text(&value)?.unwrap_or_default();
                        let label = layout_label(
                            Some(if line.is_empty() { " " } else { &line }),
                            true,
                            resources,
                            options,
                            small,
                            wrap,
                            bits,
                        )?;
                        if label.width > width {
                            width = label.width;
                        }
                        height += label.height;
                        view.lines.push(label)
                    }
                }
                view.w = (width + 32.).js_max(56.);
                view.h = height
            }
        }
        let mut c = c.unwrap_or_else(|| CNode::new(view.w, view.h, -1));
        let dimensions = if matches!(view.kind, K::State | K::Described) {
            Some((view.w, view.h))
        } else {
            None
        };
        view.style = Some(style);
        if let Some((w, h)) = dimensions {
            c.span = if across(phase, i, &logical, &views, root_dir)? {
                w
            } else {
                h
            } - 24.;
        }
        nodes.push(c)
    }
    let mut edges = Vec::new();
    let mut labels = Vec::new();
    let mut drawn = Vec::new();
    let mut loop_count = vec![0usize; states];
    let mut loop_width = vec![0_f64; states];
    let mut loop_height = vec![0.; states];
    let mut anchors = HashMap::new();
    let edge_values = phase.get(&graph, "edges")?;
    let mut cursor = 0;
    while cursor < phase.count(&edge_values)? {
        phase.step()?;
        let edge = phase.index(&edge_values, cursor)?;
        cursor += 1;
        let start = phase.string(&edge, "start")?;
        let end = phase.string(&edge, "end")?;
        let (Some(&a), Some(&b)) = (index.get(start.as_str()), index.get(end.as_str())) else {
            continue;
        };
        let (mut src, mut dst) = (a, b);
        let label = layout_label(
            phase.field(&edge, "label")?.as_deref(),
            true,
            resources,
            options,
            small,
            wrap,
            0,
        )?;
        let label_w = if label.width > 0. {
            label.width + 12.
        } else {
            0.
        };
        if src == dst {
            loop_count[src] += 1;
            loop_width[src] = loop_width[src].js_max(label_w);
            loop_height[src] += label.height
        } else {
            let anchor = if encloses(src, dst, &logical, &nodes) {
                Some((src, true))
            } else if encloses(dst, src, &logical, &nodes) {
                Some((dst, false))
            } else {
                None
            };
            if let Some((composite, first)) = anchor {
                anchors.insert(edges.len(), composite);
                appear.push(if first { -1. } else { f64::INFINITY });
                let at = views.len();
                views.push(View::new(None, K::Hidden, &no_label));
                nodes.push(CNode::new(0., 0., composite as isize));
                if first {
                    src = at
                } else {
                    dst = at
                }
            }
        }
        edges.push(CEdge {
            src,
            dst,
            minlen: 1.,
            label_w,
            label_h: if label.height > 0. {
                label.height + 4.
            } else {
                0.
            },
            route: Vec::new(),
            label_x: 0.,
            label_y: 0.,
        });
        drawn.push(edge);
        labels.push(label)
    }
    for i in 0..states {
        if owners[i] >= 0 {
            continue;
        }
        let side = across(phase, i, &logical, &views, root_dir)?;
        let item = &mut nodes[i];
        let mut room = 0.;
        if loop_count[i] > 0 {
            let reach = 26. + (loop_count[i] - 1) as f64 * 8.;
            room = if side {
                reach + loop_width[i]
            } else {
                reach * 0.75 + 4. + loop_height[i]
            };
            if !item.is_group {
                if side {
                    item.w += 2. * room;
                    item.h = item.h.js_max(loop_height[i])
                } else {
                    item.h += 2. * room;
                    item.w = item.w.js_max(loop_width[i])
                }
                room = 0.
            }
        }
        if !noted[i] && room == 0. {
            item.parent = logical[i];
            continue;
        }
        let mut c = if side {
            group(Dir::LR, room, 0., 0.)
        } else {
            group(Dir::TB, 0., room, room)
        };
        c.parent = logical[i];
        appear.push(appear[i]);
        nodes[i].parent = views.len() as isize;
        views.push(View::new(None, K::Hidden, &no_label));
        nodes.push(c)
    }
    for i in 0..states {
        if owners[i] >= 0 {
            nodes[i].parent = nodes[owners[i] as usize].parent
        }
    }
    {
        let mut members = vec![Vec::new(); nodes.len()];
        let mut top = Vec::new();
        for (i, c) in nodes.iter().enumerate() {
            if c.parent >= 0 {
                members[c.parent as usize].push(i)
            } else {
                top.push(i)
            }
        }
        let by_flow = |a: &usize, b: &usize| {
            u8::from(views[*a].kind != K::Start)
                .cmp(&u8::from(views[*b].kind != K::Start))
                .then_with(|| {
                    appear[*a]
                        .partial_cmp(&appear[*b])
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then(a.cmp(b))
        };
        top.sort_by(by_flow);
        top.reverse();
        let mut seq = 0;
        while let Some(i) = top.pop() {
            nodes[i].seq = seq;
            seq += 1;
            members[i].sort_by(by_flow);
            top.extend(members[i].iter().rev().copied())
        }
    }
    let layout = compound_layout(
        &mut nodes,
        &mut edges,
        root_dir,
        &LayeredOptions {
            node_sep: number_option(config, "nodeSpacing", 40.),
            edge_sep: 16.,
            rank_sep: number_option(config, "rankSpacing", 44.),
            port_sep: 20.,
            head_room: marker_trim("arrow_point"),
            tracks: false,
            straight: false,
            apart: false,
        },
    );
    let width = (layout.width + 2. * pad).ceil();
    let height = (layout.height + 2. * pad).ceil();
    let mut links = Vec::new();
    let mut crossings = Crossings::default();
    for e in &edges {
        if e.route.len() > 6 {
            mark_crossings(&mut crossings, &e.route[3..e.route.len() - 3])
        }
    }
    let mut children = vec![Vec::new(); nodes.len()];
    let mut order = Vec::new();
    for (i, c) in nodes.iter().enumerate() {
        if c.parent >= 0 {
            children[c.parent as usize].push(i)
        } else {
            order.push(i)
        }
    }
    let roots = order.clone();
    let mut q = 0;
    while q < order.len() {
        order.extend(children[order[q]].iter().copied());
        q += 1
    }
    let mut clusters = String::new();
    let mut dividers = divide(
        &roots,
        &views,
        &nodes,
        pad,
        pad / 2.,
        pad / 2.,
        width - pad / 2.,
        height - pad / 2.,
    );
    let mut nodes_out = String::new();
    let mut notes_out = String::new();
    for i in order {
        let view = &views[i];
        let (Some(node), Some(style)) = (view.node.as_ref(), view.style.as_ref()) else {
            continue;
        };
        if view.kind >= K::Region {
            continue;
        }
        let c = &nodes[i];
        let (x, y, w, h) = (c.x + pad, c.y + pad, view.w, view.h);
        let attrs = format!(" class=\"pele-label\"{}", style.text);
        let mut body=match view.kind{K::Composite=>{let(left,top)=(x-c.w/2.,y-c.h/2.);let mut body=rect(left,top,c.w,c.h,&format!(" rx=\"10\" fill=\"var(--_a)\" fill-opacity=\"0.5\" stroke=\"var(--_b)\"{}",style.shape));if view.title_h>0.{body+=&format!("<path d=\"M{},{}H{}\"{RULE}{}/>",num(left),num(top+view.title_h),num(left+c.w),style.line);body+=&label_svg(&view.label,title_x(c,view.label.width,&crossings)+pad,top+view.title_h/2.,&format!(" class=\"pele-cluster-label\"{}",style.text),resources)?}dividers+=&divide(&children[i],&views,&nodes,pad,left,top+view.title_h,left+c.w,top+c.h);body},K::Start=>format!("<circle r=\"7\" fill=\"var(--_l)\"{}/>",style.shape),K::End=>format!("<circle r=\"8.5\" fill=\"var(--_bg)\" stroke=\"var(--_l)\"{}/><circle r=\"5\" fill=\"var(--_l)\"{}/>",style.line,style.shape),K::Choice=>format!("<polygon points=\"0,{} {},0 0,{} {},0\"{SURFACE}{}/>",num(-h/2.),num(w/2.),num(h/2.),num(-w/2.),style.shape),K::Fork|K::Join=>rect(-w/2.,-h/2.,w,h,&format!(" rx=\"2\" fill=\"var(--_l)\"{}",style.shape)),K::Note=>{let(r,b,fold,corner)=(num(w/2.),num(h/2.),num(w/2.-8.),num(8.-h/2.));format!("<path d=\"M-{r},-{b}H{fold}L{r},{corner}V{b}H-{r}Z\" fill=\"var(--_bg)\" stroke=\"var(--_b)\"/><path d=\"M{fold},-{b}V{corner}H{r}\"{RULE}/>{}",label_svg(&view.label,0.,0.,&format!(" class=\"pele-label\" font-size=\"{}\" fill=\"var(--_m)\"",svg::js_num(small)),resources)?)},_=>{let mut body=rect(-w/2.,-h/2.,w,h,&format!(" rx=\"{}\"{SURFACE}{}",num(10_f64.js_min(h/2.)),style.shape));if view.kind==K::State{body+=&label_svg(&view.label,0.,0.,&attrs,resources)?}else{let mut at=10.-h/2.+view.label.height;body+=&format!("<path d=\"M{},{}H{}\"{RULE}{}/>",num(-w/2.),num(at+6.),num(w/2.),style.line);body+=&label_svg(&view.label,0.,at-view.label.height/2.,&attrs,resources)?;at+=14.;for line in &view.lines{body+=&label_svg(line,0.,at+line.height/2.,&format!(" class=\"pele-state-description\" font-size=\"{}\"{}",svg::js_num(small),style.text),resources)?;at+=line.height}}body}};
        let id = phase.string(node, "id")?;
        if let Some((url, tooltip)) = phase.link(&id)? {
            let tip = decode_entities(tooltip.trim_matches('"'))?;
            if !tip.is_empty() {
                body = format!("<title>{}</title>{body}", esc(&tip))
            }
            body = linked(
                &body,
                &safe_url(url.trim_matches('"'))?,
                &phase.string(node, "id")?,
                &mut links,
                options,
                " target=\"_blank\"",
            )
        }
        let open = format!(
            "<g class=\"{}{}\" data-id=\"{}\"",
            CLASSES[view.kind as usize],
            class_names(&cleaned_classes(&phase.string(node, "cssClasses")?)),
            esc_text(&phase.string(node, "id")?)?
        );
        if view.kind == K::Composite {
            clusters += &format!("{open}>{body}</g>")
        } else {
            let out = format!(
                "{open} transform=\"translate({},{})\">{body}</g>",
                num(x),
                num(y)
            );
            if view.kind == K::Note {
                notes_out += &out
            } else {
                nodes_out += &out
            }
        }
    }
    let mut reach = SideReach::default();
    for (i, ce) in edges.iter().enumerate() {
        if ce.src == ce.dst
            || ce.route.len() < 6
            || phase.field(&drawn[i], "pattern")?.as_deref() == Some("dashed")
        {
            continue;
        }
        let c = &nodes[ce.dst];
        reach.add(
            ce.dst,
            end_side(&ce.route, ce.route.len() - 3, c.x, c.y, c.w, c.h),
            "arrow_point",
        )
    }
    let mut edges_out = String::new();
    let mut labels_out = String::new();
    let mut loop_seen = vec![0usize; states];
    let mut loop_text = vec![0.; states];
    for (i, ce) in edges.iter_mut().enumerate() {
        let edge = &drawn[i];
        let label = &labels[i];
        let note = phase.field(&edge, "pattern")?.as_deref() == Some("dashed");
        let end_type = if note { "none" } else { "arrow_point" };
        let path = if ce.src == ce.dst {
            let a = &nodes[ce.src];
            let view = &views[ce.src];
            let k = loop_seen[ce.src] as f64;
            loop_seen[ce.src] += 1;
            let side = across(phase, ce.src, &logical, &views, root_dir)?;
            let (w, h) = if a.is_group {
                (a.w, a.h)
            } else {
                (view.w, view.h)
            };
            let out = if side { w } else { h } / 2.;
            let spread = (if side { h } else { w } / 2. - 4.)
                .js_min(8. + k * 6.)
                .js_max(2.);
            let reach = 26. + k * 8.;
            let trim = marker_trim(end_type) / reach.hypot(spread);
            let px = |dx: f64, dy: f64| a.x + pad + if side { out + dx } else { -dy };
            let py = |dx: f64, dy: f64| a.y + pad + if side { dy } else { out + dx };
            let at = |dx, dy| format!("{},{}", num(px(dx, dy)), num(py(dx, dy)));
            let mut path = EdgePath {
                d: format!(
                    "M{}C{} {} {}",
                    at(0., -spread),
                    at(reach, -2. * spread),
                    at(reach, 2. * spread),
                    at(reach * trim, spread + spread * trim)
                ),
                sx: 0.,
                sy: 0.,
                sdx: 0.,
                sdy: 0.,
                ex: px(0., spread),
                ey: py(0., spread),
                edx: px(-reach, -spread) - px(0., 0.),
                edy: py(-reach, -spread) - py(0., 0.),
            };
            let len = path.edx.hypot(path.edy);
            path.edx /= if len == 0. { 1. } else { len };
            path.edy /= if len == 0. { 1. } else { len };
            let furthest = 26. + (loop_count[ce.src] - 1) as f64 * 8.;
            let below = loop_text[ce.src] + label.height / 2.;
            loop_text[ce.src] += label.height;
            ce.label_x = if side {
                a.x + out + furthest + 4. + ce.label_w / 2.
            } else {
                a.x
            };
            ce.label_y = if side {
                a.y - loop_height[ce.src] / 2. + below
            } else {
                a.y + out + furthest * 0.75 + 4. + below
            };
            path
        } else {
            let mut route = ce.route.clone();
            let a = &nodes[ce.src];
            let b = &nodes[ce.dst];
            let leaves = reach.of(ce.src, end_side(&route, 0, a.x, a.y, a.w, a.h));
            let arrives = reach.of(
                ce.dst,
                end_side(&route, route.len() - 3, b.x, b.y, b.w, b.h),
            );
            for point in route.chunks_exact_mut(3) {
                point[0] += pad;
                point[1] += pad
            }
            attach(&mut route, 0, ce.src, &nodes, &views, pad);
            let last = route.len() - 3;
            attach(&mut route, last, ce.dst, &nodes, &views, pad);
            if let Some(&composite) = anchors.get(&i) {
                if route.len() >= 6 {
                    let g = &nodes[composite];
                    let inward = views[ce.src].kind == K::Hidden;
                    let at = if inward { 0 } else { route.len() - 3 };
                    let near = matches!(g.dir, Some(Dir::TB | Dir::LR)) == inward;
                    let point = if !g.dir.is_some_and(Dir::sideways) {
                        [
                            route[at],
                            g.y + pad
                                + if near {
                                    views[composite].title_h - g.h / 2.
                                } else {
                                    g.h / 2.
                                },
                            0.,
                        ]
                    } else {
                        [
                            g.x + pad + if near { -g.w / 2. } else { g.w / 2. },
                            route[at + 1],
                            1.,
                        ]
                    };
                    if inward {
                        route.splice(0..0, point);
                    } else {
                        route.extend(point)
                    }
                }
            }
            route_path(
                &route,
                None,
                0.,
                marker_trim(end_type),
                Some(leaves),
                Some(arrives),
            )
        };
        edges_out += &format!(
            "<g class=\"pele-edge {}\" data-id=\"{}\"><path d=\"{}\"/>{}</g>",
            if note {
                "pele-note-edge\" stroke=\"var(--_b)\" stroke-dasharray=\"3 4"
            } else {
                "pele-transition"
            },
            esc_text(&phase.string(edge, "id")?)?,
            path.d,
            marker(end_type, path.ex, path.ey, path.edx, path.edy, "var(--_l)")
        );
        if label.width > 0. {
            labels_out += &edge_label_svg(
                &phase.string(edge, "id")?,
                label,
                ce.label_x + pad,
                ce.label_y + pad,
                "",
                resources,
            )?
        }
    }
    let mut content = String::new();
    if !clusters.is_empty() {
        content += &format!("<g class=\"pele-clusters\">{clusters}</g>")
    }
    if !dividers.is_empty() {
        content +=
            &format!("<g class=\"pele-dividers\" stroke-dasharray=\"4 4\"{RULE}>{dividers}</g>")
    }
    if !edges_out.is_empty() {
        content+=&format!("<g class=\"pele-edges\" fill=\"none\" stroke=\"var(--_l)\" stroke-linecap=\"round\">{edges_out}</g>")
    }
    if !labels_out.is_empty() {
        content += &format!(
            "<g class=\"pele-edge-labels\" font-size=\"{}\">{labels_out}</g>",
            svg::js_num(small)
        )
    }
    content += &format!("<g class=\"pele-nodes\">{nodes_out}{notes_out}</g>");
    let titled = with_title(
        phase.metadata("title")?.as_deref(),
        resources,
        options,
        size,
        pad,
        width,
        height,
        content,
    )?;
    let svg = svg::document(
        "state",
        titled.width,
        titled.height,
        size,
        options,
        phase.metadata("accTitle")?.as_deref(),
        phase.metadata("accDescr")?.as_deref(),
        &titled.content,
    )?;
    Ok(Rendered {
        svg,
        width: titled.width,
        height: titled.height,
        links,
    })
}
