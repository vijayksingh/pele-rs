//! Pele diagrams/mindmap/render.ts, 458322a5 (MIT, kepano).
use super::MindmapModel;
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    svg::{
        document,
        edges::theme::class_names,
        esc_text, label_svg, label_svg_with_resources, num, series_color,
        shapes::{draw_shape, shape_inset, shape_size},
    },
    text::{layout_label, Label},
    Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
pub(super) trait PhaseInput {
    type Error: From<PeleError>;
    type Node: Clone;
    type List;
    fn begin_draw(&mut self) {}
    fn retain_label(&mut self, _: Option<&str>) -> Result<(), Self::Error> {
        Ok(())
    }
    fn step(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn nodes(&mut self) -> Result<Self::List, Self::Error>;
    fn count(&mut self, list: &Self::List) -> Result<usize, Self::Error>;
    fn node(&mut self, list: &Self::List, index: usize) -> Result<Self::Node, Self::Error>;
    fn children(&mut self, node: &Self::Node) -> Result<Self::List, Self::Error>;
    fn index(&mut self, node: &Self::Node, bound: usize) -> Result<usize, Self::Error>;
    fn text(
        &mut self,
        node: &Self::Node,
        key: &'static str,
        optional: bool,
    ) -> Result<Option<String>, Self::Error>;
    fn node_type(&mut self, node: &Self::Node) -> Result<u8, Self::Error>;
    fn section(&mut self, node: &Self::Node) -> Result<Option<usize>, Self::Error>;
    fn title(&mut self) -> Result<Option<String>, Self::Error>;
}
enum TypedList<'a> {
    All(usize),
    Children(&'a [usize]),
}
struct Typed<'a>(&'a MindmapModel);
impl<'a> PhaseInput for Typed<'a> {
    type Error = PeleError;
    type Node = usize;
    type List = TypedList<'a>;
    fn nodes(&mut self) -> Result<Self::List, PeleError> {
        Ok(TypedList::All(self.0.nodes.len()))
    }
    fn count(&mut self, list: &Self::List) -> Result<usize, PeleError> {
        Ok(match list {
            TypedList::All(n) => *n,
            TypedList::Children(v) => v.len(),
        })
    }
    fn node(&mut self, list: &Self::List, index: usize) -> Result<usize, PeleError> {
        Ok(match list {
            TypedList::All(_) => index,
            TypedList::Children(v) => v[index],
        })
    }
    fn children(&mut self, node: &usize) -> Result<Self::List, PeleError> {
        Ok(TypedList::Children(&self.0.nodes[*node].children))
    }
    fn index(&mut self, node: &usize, _: usize) -> Result<usize, PeleError> {
        Ok(*node)
    }
    fn text(
        &mut self,
        node: &usize,
        key: &'static str,
        _: bool,
    ) -> Result<Option<String>, PeleError> {
        let node = &self.0.nodes[*node];
        Ok(match key {
            "descr" => Some(node.descr.clone()),
            "nodeId" => Some(node.node_id.clone()),
            "class" => node.class.clone(),
            "icon" => node.icon.clone(),
            _ => unreachable!(),
        })
    }
    fn node_type(&mut self, node: &usize) -> Result<u8, PeleError> {
        Ok(self.0.nodes[*node].node_type)
    }
    fn section(&mut self, node: &usize) -> Result<Option<usize>, PeleError> {
        Ok(self.0.nodes[*node].section)
    }
    fn title(&mut self) -> Result<Option<String>, PeleError> {
        Ok(self.0.title.clone())
    }
}
const GAP_X: f64 = 36.;
const GAP_Y: f64 = 8.;
const GROUP_GAP: f64 = 18.;
const BRANCH_GAP_X: f64 = 52.;
const BRANCH_GAP_Y: f64 = 28.;
const TEXT_PAD: f64 = 6.;
const PILL_PAD: f64 = 12.;
const ICON_GAP: f64 = 6.;
const INDENT: f64 = 28.;
const STEM: f64 = 12.;
const BEND: f64 = 8.;
const ROW_GAP: f64 = 10.;
fn left_label(
    l: &Label,
    left: f64,
    cy: f64,
    r: &mut dyn RenderResources,
) -> Result<String, PeleError> {
    if l.lines.len() < 2 {
        return label_svg_with_resources(l, left + l.width / 2., cy, " class=\"pele-label\"", r);
    }
    let mut out = String::new();
    for (k, line) in l.lines.iter().enumerate() {
        let w = l.widths[k];
        let one = Label {
            lines: vec![line.clone()],
            widths: vec![w],
            width: w,
            height: l.line_height,
            size: l.size,
            line_height: l.line_height,
        };
        out += &label_svg_with_resources(
            &one,
            left + w / 2.,
            cy + (k as f64 - (l.lines.len() as f64 - 1.) / 2.) * l.line_height,
            " class=\"pele-label\"",
            r,
        )?;
    }
    Ok(out)
}
fn stack<P: PhaseInput>(
    list: &P::List,
    input: &mut P,
    from: usize,
    to: usize,
    gap: f64,
    extent: &[f64],
    own: &[f64],
    anchor: &[f64],
    rel: &mut [f64],
) -> Result<(f64, f64), P::Error> {
    let mut y = 0.;
    let mut last = 0.;
    for k in from..to {
        input.step()?;
        let child = input.node(list, k)?;
        let c = input.index(&child, extent.len())?;
        rel[c] = y;
        let child = input.node(list, k)?;
        let children = input.children(&child)?;
        let grouped = if input.count(&children)? > 0 {
            true
        } else if k + 1 < to {
            let next = input.node(list, k + 1)?;
            let children = input.children(&next)?;
            input.count(&children)? > 0
        } else {
            false
        };
        last = if grouped { gap.js_max(GROUP_GAP) } else { gap };
        y += extent[c] + last;
    }
    let first = input.node(list, from)?;
    let a = input.index(&first, extent.len())?;
    let last_node = input.node(list, to - 1)?;
    let b = input.index(&last_node, extent.len())?;
    Ok((
        (rel[a] + own[a] + anchor[a] + rel[b] + own[b] + anchor[b]) / 2.,
        y - last,
    ))
}
pub(super) fn render(
    m: &MindmapModel,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_phases(&mut Typed(m), c, o, r)
}
pub(super) fn render_phases<P: PhaseInput>(
    input: &mut P,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    let across = draw(input, c, o, r, false)?;
    let Some(available) = o.max_width.filter(|w| *w > 0.) else {
        return Ok(across);
    };
    if o.auto_direction == Some(false)
        || across.width <= available
        || available >= o.direction_breakpoint.filter(|v| *v > 0.).unwrap_or(640.)
    {
        return Ok(across);
    }
    let down = draw(input, c, o, r, true)?;
    Ok(if down.width < across.width {
        down
    } else {
        across
    })
}
fn draw<P: PhaseInput>(
    input: &mut P,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    down: bool,
) -> Result<Rendered, P::Error> {
    input.begin_draw();
    let size = o.font_size.unwrap_or(16.);
    let pad = o.padding.unwrap_or(8.);
    let wrap = c
        .get("mindmap")
        .and_then(Value::as_object)
        .and_then(|m| m.get("maxNodeWidth"))
        .and_then(|v| {
            if let Value::Number(n) = v {
                Some(*n)
            } else {
                None
            }
        })
        .filter(|v| *v > 0.)
        .map_or(200., |v| v.js_min(2000.));
    let glyph = size * 1.25;
    let nodes = input.nodes()?;
    let n = input.count(&nodes)?;
    let mut parent = vec![0usize; n];
    let mut depth = vec![0usize; n];
    for i in 0..n {
        let node = input.node(&nodes, i)?;
        let children = input.children(&node)?;
        let mut k = 0;
        while k < input.count(&children)? {
            input.step()?;
            let child = input.node(&children, k)?;
            let child = input.index(&child, n)?;
            parent[child] = i;
            depth[child] = depth[i] + 1;
            k += 1;
        }
    }
    let room = if down {
        o.max_width
            .filter(|v| *v > 0.)
            .map_or(f64::INFINITY, |v| v - 2. * pad)
    } else {
        f64::INFINITY
    };
    let mut labels = Vec::with_capacity(n);
    let mut shapes = Vec::with_capacity(n);
    let mut w = vec![0.; n];
    let mut h = vec![0.; n];
    let mut dy = vec![0.; n];
    let mut anchor = vec![0.; n];
    let mut drawn = vec![false; n];
    for i in 0..n {
        let node = input.node(&nodes, i)?;
        drawn[i] = if input.text(&node, "icon", true)?.is_some() {
            let icon = input.text(&node, "icon", true)?.unwrap_or_default();
            r.icon(&icon)?.is_some_and(|s| !s.markup().is_empty())
        } else {
            false
        };
        let fit = wrap.js_min(72f64.js_max(
            room - depth[i] as f64 * INDENT
                - 2. * TEXT_PAD
                - if drawn[i] { glyph + ICON_GAP } else { 0. },
        ));
        let descr = input.text(&node, "descr", true)?;
        input.retain_label(descr.as_deref())?;
        let label = layout_label(descr.as_deref(), true, r, o, size, fit, 0)?;
        let tw = label.width
            + if drawn[i] {
                glyph + if label.width > 0. { ICON_GAP } else { 0. }
            } else {
                0.
            };
        let th = label.height.js_max(round(size * 1.5));
        let mut shape = match input.node_type(&node)? {
            1 => "rounded",
            2 => "rect",
            3 => "circle",
            4 => "cloud",
            5 => "bang",
            6 => "hex",
            _ => {
                if i == 0 {
                    "stadium"
                } else {
                    ""
                }
            }
        };
        if down && i == 0 && shape != "rect" && shape != "rounded" {
            shape = "stadium";
        }
        shapes.push(shape);
        if shape.is_empty() {
            let children = input.children(&node)?;
            let pill = input.count(&children)? > 0;
            w[i] = tw + 2. * if pill { PILL_PAD } else { TEXT_PAD };
            h[i] = th + if pill { 6. } else { 0. };
            anchor[i] = h[i] / 2.;
        } else {
            let s = shape_size(shape, tw, th);
            w[i] = s.w;
            h[i] = s.h;
            dy[i] = s.dy;
            anchor[i] = s.h / 2.;
        }
        labels.push(label);
    }
    let mut extent = vec![0.; n];
    let mut rel = vec![0.; n];
    let mut own = vec![0.; n];
    let mut kids = vec![0.; n];
    for i in (1..n).rev() {
        let node = input.node(&nodes, i)?;
        let children = input.children(&node)?;
        if input.count(&children)? == 0 {
            extent[i] = h[i];
            continue;
        }
        let count = input.count(&children)?;
        let (middle, stacked) = stack(
            &children, input, 0, count, GAP_Y, &extent, &own, &anchor, &mut rel,
        )?;
        let top = middle - anchor[i];
        let low = 0f64.js_min(top);
        extent[i] = stacked.js_max(top + h[i]) - low;
        own[i] = top - low;
        kids[i] = -low;
    }
    let mut x = vec![0.; n];
    let mut y = vec![0.; n];
    let mut band = vec![0.; n];
    let mut side = vec![0.; n];
    let mut trunk = vec![0.; n];
    if down {
        let mut row = 0.;
        for i in 0..n {
            if i > 0 && parent[i] == 0 {
                row += ROW_GAP;
            }
            x[i] = depth[i] as f64 * INDENT + w[i] / 2.;
            y[i] = row;
            row += h[i] + ROW_GAP;
        }
    } else if n > 0 {
        let root = input.node(&nodes, 0)?;
        let branches = input.children(&root)?;
        let count = input.count(&branches)?;
        let mut total = 0.;
        let mut k = 0;
        while k < input.count(&branches)? {
            input.step()?;
            let child = input.node(&branches, k)?;
            total += extent[input.index(&child, n)?];
            k += 1;
        }
        let mut split = count;
        let mut best = f64::INFINITY;
        let mut right = 0.;
        for k in 1..count {
            input.step()?;
            let child = input.node(&branches, k - 1)?;
            right += extent[input.index(&child, n)?];
            let diff = (2. * right - total + (2. * k as f64 - count as f64) * BRANCH_GAP_Y).abs();
            if diff < best {
                best = diff;
                split = k;
            }
        }
        let mut from = 0;
        let mut to = split;
        let mut dir = 1.;
        while from < count {
            let (middle, _) = stack(
                &branches,
                input,
                from,
                to,
                BRANCH_GAP_Y,
                &extent,
                &own,
                &anchor,
                &mut rel,
            )?;
            for k in from..to {
                input.step()?;
                let child = input.node(&branches, k)?;
                let child = input.index(&child, n)?;
                band[child] = rel[child] - middle;
                side[child] = dir;
            }
            from = to;
            to = count;
            dir = -1.;
        }
        y[0] = -h[0] / 2.;
        for i in 0..n {
            if i > 0 {
                y[i] = band[i] + own[i];
            }
            let node = input.node(&nodes, i)?;
            let children = input.children(&node)?;
            let mut k = 0;
            while k < input.count(&children)? {
                input.step()?;
                let child = input.node(&children, k)?;
                let child = input.index(&child, n)?;
                k += 1;
                if i > 0 {
                    side[child] = side[i];
                    band[child] = band[i] + kids[i] + rel[child];
                }
                x[child] = x[i]
                    + side[child]
                        * (w[i] / 2. + if i > 0 { GAP_X } else { BRANCH_GAP_X } + w[child] / 2.);
            }
        }
    }
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (0f64, 0f64, 0f64, 0f64);
    let mut out = String::new();
    let mut edges = String::new();
    let mut body = String::new();
    let mut color = String::new();
    for k in 1..=n {
        let i = k % n;
        let node = input.node(&nodes, i)?;
        let shape = shapes[i];
        let label = &labels[i];
        let cx = x[i];
        let top = y[i];
        min_x = min_x.js_min(cx - w[i] / 2.);
        max_x = max_x.js_max(cx + w[i] / 2.);
        min_y = min_y.js_min(top);
        max_y = max_y.js_max(top + h[i]);
        if (i == 0 || parent[i] == 0) && !body.is_empty() {
            let previous = input.node(&nodes, k - 1)?;
            let section = input.section(&previous)?.unwrap_or(0);
            out+=&format!("<g class=\"pele-branch pele-section-{}\"><g class=\"pele-edges\" fill=\"none\" stroke=\"{color}\" stroke-width=\"1.5\">{edges}</g>{body}</g>",section);
            edges.clear();
            body.clear();
        }
        if i > 0 && down {
            if parent[i] == 0 {
                color = series_color(input.section(&node)?.unwrap_or(0));
            }
            let p = parent[i];
            let stem = if p == 0 {
                STEM.js_max(h[0] / 2.).js_min(INDENT - BEND)
            } else {
                STEM
            };
            let x1 = x[p] - w[p] / 2. + stem.js_min(w[p] / 2.);
            let y1 = if trunk[p] != 0. {
                trunk[p]
            } else {
                y[p] + h[p]
            };
            let left = cx - w[i] / 2.;
            let x2 = left
                + if !shape.is_empty() {
                    shape_inset(shape, w[i], h[i], 3, 0.)
                } else {
                    0.
                };
            let y2 = top + anchor[i];
            let bend = 0f64.js_max(BEND.js_min(x2 - x1).js_min(y2 - y1));
            trunk[p] = y2 - bend;
            edges += &format!(
                "<path class=\"pele-edge\" d=\"M{},{}V{}Q{},{} {},{}H{}\"/>",
                num(x1),
                num(y1),
                num(y2 - bend),
                num(x1),
                num(y2),
                num(x1 + bend),
                num(y2),
                num(x2)
            );
        } else if i > 0 {
            if parent[i] == 0 {
                color = series_color(input.section(&node)?.unwrap_or(0));
            }
            let p = parent[i];
            let dir = side[i];
            let from = if dir > 0. { 1 } else { 3 };
            let x1 = x[p]
                + dir
                    * (w[p] / 2.
                        - if !shapes[p].is_empty() {
                            shape_inset(shapes[p], w[p], h[p], from, 0.)
                        } else {
                            0.
                        });
            let y1 = y[p] + anchor[p];
            let near = cx - dir * w[i] / 2.;
            let x2 = near
                + if !shape.is_empty() {
                    dir * shape_inset(shape, w[i], h[i], 4 - from, 0.)
                } else {
                    0.
                };
            let y2 = top + anchor[i];
            let mid = num((x1 + x2) / 2.);
            edges += &format!(
                "<path class=\"pele-edge\" d=\"M{},{}C{mid},{} {mid},{} {},{}\"/>",
                num(x1),
                num(y1),
                num(y1),
                num(y2),
                num(x2),
                num(y2)
            );
        }
        let pill = if shape.is_empty() {
            let children = input.children(&node)?;
            input.count(&children)? > 0
        } else {
            false
        };
        let tint = format!(" fill=\"{color}\" fill-opacity=\"0.22\" stroke=\"{color}\"");
        let mut inner = String::new();
        if !shape.is_empty() {
            inner = if i > 0 {
                draw_shape(shape, w[i], h[i], &tint, &format!(" stroke=\"{color}\""))
            } else {
                draw_shape(
                    shape,
                    w[i],
                    h[i],
                    " fill=\"var(--_s)\" stroke=\"var(--_b)\"",
                    " stroke=\"var(--_b)\"",
                )
            };
        } else if pill {
            inner = format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\"{tint}/>",
                num(-w[i] / 2.),
                num(-h[i] / 2.),
                num(w[i]),
                num(h[i]),
                num(h[i] / 2.)
            );
        }
        let mut shift = 0.;
        if input
            .text(&node, "icon", true)?
            .is_some_and(|s| !s.is_empty())
            && drawn[i]
        {
            let icon = input.text(&node, "icon", true)?.unwrap_or_default();
            shift = (glyph + if label.width > 0. { ICON_GAP } else { 0. }) / 2.;
            inner += &super::super::common::icon::icon_svg(
                &icon,
                -shift - label.width / 2.,
                dy[i] - glyph / 2.,
                glyph,
                r,
            )?;
        }
        let class = input.text(&node, "class", true)?.unwrap_or_default();
        let node_id = input.text(&node, "nodeId", false)?.unwrap_or_default();
        let group=format!("<g class=\"pele-node pele-shape-{}{}{}\" data-id=\"{}\" transform=\"translate({},{})\">{inner}{}</g>",if shape.is_empty(){if pill{"branch"}else{"text"}}else{shape},if i>0{""}else{" pele-root"},class_names(&class),esc_text(&node_id)?,num(cx),num(top+h[i]/2.),if down&&shape.is_empty(){left_label(label,shift-label.width/2.,dy[i],r)?}else{label_svg_with_resources(label,shift,dy[i]," class=\"pele-label\"",r)?});
        if i > 0 {
            body += &group;
        } else {
            out += &group;
        }
    }
    let title_room = o
        .max_width
        .filter(|v| *v > 0.)
        .map_or(4000., |v| 80f64.js_max(v - 2. * pad));
    let title_text = input.title()?;
    let title = layout_label(title_text.as_deref(), false, r, o, size, title_room, 1)?;
    let title_height = if title.height > 0. {
        title.height + 16.
    } else {
        0.
    };
    let width = (max_x - min_x).js_max(title.width);
    let total_w = (width + 2. * pad).ceil();
    let total_h = (max_y - min_y + title_height + 2. * pad).ceil();
    let content = label_svg(
        &title,
        total_w / 2.,
        pad + title.height / 2.,
        " class=\"pele-title\" font-weight=\"var(--_tw)\"",
    ) + &format!(
        "<g transform=\"translate({},{})\">{out}</g>",
        num(pad - min_x + (width - max_x + min_x) / 2.),
        num(pad + title_height - min_y)
    );
    Ok(Rendered {
        svg: document("mindmap", total_w, total_h, size, o, None, None, &content)?,
        width: total_w,
        height: total_h,
        links: Vec::new(),
    })
}
