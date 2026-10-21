//! Pele architecture/{render,icons}.ts, 458322a5 (MIT, kepano).
use super::{
    layout::{place, side_index, DX, DY},
    model::*,
};
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    svg::{
        document,
        edges::{edge_label_svg, marker, marker_trim},
        esc_text, label_svg, label_svg_with_resources, num,
    },
    text::{layout_label, measure, Label},
    Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
use std::collections::HashMap;
const STUB: f64 = 12.;
const DOT: f64 = 3.;
const RADIUS: &str = "var(--pele-radius,4px)";
fn setting(c: &Object, key: &str, fallback: f64, min: f64, max: f64) -> f64 {
    match c
        .get("architecture")
        .and_then(Value::as_object)
        .and_then(|c| c.get(key))
    {
        Some(Value::Number(n)) if n.is_finite() => n.clamp(min, max),
        _ => fallback,
    }
}
fn rank(values: &mut [i32]) -> HashMap<i32, usize> {
    values.sort();
    let mut out = HashMap::new();
    for value in values {
        let next = out.len();
        out.entry(*value).or_insert(next);
    }
    out
}
fn builtin(name: &str) -> String {
    let d=match name{"database"=>"M4 6a8 3 0 1 0 16 0a8 3 0 1 0-16 0M4 6v12a8 3 0 0 0 16 0V6M4 12a8 3 0 0 0 16 0","server"=>"M3 4h18v7H3zM3 13h18v7H3zM7 7.5h.01M7 16.5h.01M11 7.5h6M11 16.5h6","disk"=>"M5 3h14v18H5zM12 6.5a4 4 0 1 0 0 8a4 4 0 1 0 0-8M12 10.5h.01M8 18h3","internet"=>"M3 12a9 9 0 1 0 18 0a9 9 0 1 0-18 0M3 12h18M12 3a13 13 0 0 0 0 18M12 3a13 13 0 0 1 0 18","cloud"=>"M7 18a4 4 0 0 1-.5-7.97A6 6 0 0 1 18 9.5a4.25 4.25 0 0 1-.5 8.5z",_=>return String::new()};
    format!("<path d=\"{d}\" stroke=\"var(--_fg)\" stroke-width=\"1.5\"/>")
}
fn icon_slot(
    name: &str,
    x: f64,
    y: f64,
    side: f64,
    r: &mut dyn RenderResources,
) -> Result<String, PeleError> {
    let inner = r
        .direct_icon(name)?
        .map_or_else(|| builtin(name), |s| s.markup().to_string());
    Ok(format!("<svg class=\"pele-icon\" data-icon=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{inner}</svg>",esc_text(name)?,num(x),num(y),num(side),num(side)))
}
fn fit(
    mut label: Label,
    max_width: f64,
    max_lines: usize,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Label, PeleError> {
    let cut = label.lines.len() > max_lines;
    if !cut && label.width <= max_width {
        return Ok(label);
    }
    let dots = measure(r, o, "…", label.size, 0)?;
    let mut lines = vec![];
    let mut widths = vec![];
    let mut widest = 0f64;
    for i in 0..max_lines.min(label.lines.len()) {
        let mut line = label.lines[i].clone();
        let mut width = label.widths[i];
        if width > max_width || cut && i == max_lines - 1 {
            loop {
                let Some(span) = line.last_mut() else {
                    line.push(crate::compat::text::Span {
                        text: "…".into(),
                        style: 0,
                        icon: None,
                        width: dots,
                    });
                    width = dots;
                    break;
                };
                let rest = width - span.width;
                if span.icon.is_none() && rest + dots < max_width {
                    let chars: Vec<char> = span.text.chars().collect();
                    let (mut lo, mut hi) = (0, chars.len());
                    while lo < hi {
                        let mid = (lo + hi + 1) >> 1;
                        let text: String = chars[..mid].iter().collect();
                        if rest + measure(r, o, &text, label.size, span.style)? + dots <= max_width
                        {
                            lo = mid
                        } else {
                            hi = mid - 1
                        }
                    }
                    span.text = format!(
                        "{}…",
                        chars[..lo]
                            .iter()
                            .collect::<String>()
                            .trim_end_matches(crate::compat::source::space)
                    );
                    span.width = measure(r, o, &span.text, label.size, span.style)?;
                    width = rest + span.width;
                    break;
                }
                line.pop();
                width = rest
            }
        }
        lines.push(line);
        widths.push(width);
        widest = widest.js_max(width)
    }
    label.height = lines.len() as f64 * label.line_height;
    label.lines = lines;
    label.widths = widths;
    label.width = widest;
    Ok(label)
}
#[derive(Clone, Copy)]
struct Port {
    x: f64,
    y: f64,
    dx: f64,
    dy: f64,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    gap: f64,
    wire: bool,
}
fn route(s: Port, e: Port) -> Vec<f64> {
    let (sx, sy, sdx, sdy) = (s.x, s.y, s.dx, s.dy);
    let (ex, ey, edx, edy) = (e.x, e.y, e.dx, e.dy);
    let (s1x, s1y, e1x, e1y) = (
        sx + sdx * STUB,
        sy + sdy * STUB,
        ex + edx * STUB,
        ey + edy * STUB,
    );
    if sdx * edx + sdy * edy == 0. {
        let cx = if sdx != 0. { ex } else { sx };
        let cy = if sdx != 0. { sy } else { ey };
        if (cx - sx) * sdx + (cy - sy) * sdy > 0. && (cx - ex) * edx + (cy - ey) * edy > 0. {
            return vec![sx, sy, cx, cy, ex, ey];
        }
        return if sdx != 0. {
            vec![sx, sy, s1x, s1y, s1x, e1y, e1x, e1y, ex, ey]
        } else {
            vec![sx, sy, s1x, s1y, e1x, s1y, e1x, e1y, ex, ey]
        };
    }
    let sideways = sdx != 0.;
    let offset = if sideways { ey - sy } else { ex - sx };
    if sdx == -edx && sdy == -edy {
        let ahead = (ex - sx) * sdx + (ey - sy) * sdy;
        if ahead > 0. {
            if offset.abs() < 0.5 {
                return vec![sx, sy, ex, ey];
            }
            if sideways {
                let lo = if sdx > 0. { s.x1 } else { e.x1 };
                let hi = if sdx > 0. { e.x0 } else { s.x0 };
                let mx = if lo < hi {
                    (lo + hi) / 2.
                } else {
                    (sx + ex) / 2.
                };
                return vec![sx, sy, mx, sy, mx, ey, ex, ey];
            }
            let lo = if sdy > 0. { s.y1 } else { e.y1 };
            let hi = if sdy > 0. { e.y0 } else { s.y0 };
            let my = if lo < hi {
                (lo + hi) / 2.
            } else {
                (sy + ey) / 2.
            };
            return vec![sx, sy, sx, my, ex, my, ex, ey];
        }
    } else if offset.abs() >= 1. {
        if sideways {
            let x = if sdx > 0. {
                sx.js_max(ex) + STUB
            } else {
                sx.js_min(ex) - STUB
            };
            return vec![sx, sy, x, sy, x, ey, ex, ey];
        }
        let y = if sdy > 0. {
            sy.js_max(ey) + STUB
        } else {
            sy.js_min(ey) - STUB
        };
        return vec![sx, sy, sx, y, ex, y, ex, ey];
    }
    if sideways {
        let lane = if s.y1 + STUB <= e.y0 {
            (s.y1 + e.y0) / 2.
        } else if e.y1 + STUB <= s.y0 {
            (e.y1 + s.y0) / 2.
        } else {
            s.y1.js_max(e.y1) + STUB
        };
        vec![sx, sy, s1x, sy, s1x, lane, e1x, lane, e1x, ey, ex, ey]
    } else {
        let lane = if s.x1 + STUB <= e.x0 {
            (s.x1 + e.x0) / 2.
        } else if e.x1 + STUB <= s.x0 {
            (e.x1 + s.x0) / 2.
        } else {
            s.x1.js_max(e.x1) + STUB
        };
        vec![sx, sy, sx, s1y, lane, s1y, lane, e1y, ex, e1y, ex, ey]
    }
}
fn simplify(p: &[f64]) -> Vec<f64> {
    let mut out = p[..2].to_vec();
    for point in p[2..].chunks_exact(2) {
        let (x, y) = (point[0], point[1]);
        let n = out.len();
        let same_x = (x - out[n - 2]).abs() < 0.01;
        let same_y = (y - out[n - 1]).abs() < 0.01;
        if same_x && same_y {
            continue;
        }
        if n >= 4
            && (same_x && (out[n - 4] - out[n - 2]).abs() < 0.01
                || same_y && (out[n - 3] - out[n - 1]).abs() < 0.01)
        {
            out[n - 2] = x;
            out[n - 1] = y
        } else {
            out.push(x);
            out.push(y)
        }
    }
    out
}
fn path_data(p: &[f64], start_trim: f64, end_trim: f64) -> String {
    let n = p.len() / 2;
    let mut xs: Vec<f64> = p.chunks_exact(2).map(|p| p[0]).collect();
    let mut ys: Vec<f64> = p.chunks_exact(2).map(|p| p[1]).collect();
    for (at, toward, amount) in [(0, 1, start_trim), (n - 1, n - 2, end_trim)] {
        let len = (xs[toward] - xs[at]).hypot(ys[toward] - ys[at]);
        let t = amount.js_min((len - 1.).js_max(0.));
        if len > 0. && t > 0. {
            xs[at] += (xs[toward] - xs[at]) / len * t;
            ys[at] += (ys[toward] - ys[at]) / len * t
        }
    }
    let mut d = format!("M{},{}", num(xs[0]), num(ys[0]));
    for i in 1..n - 1 {
        let before = (xs[i] - xs[i - 1]).hypot(ys[i] - ys[i - 1]);
        let after = (xs[i + 1] - xs[i]).hypot(ys[i + 1] - ys[i]);
        let rad = 6f64.js_min(before / 2.).js_min(after / 2.);
        if rad < 0.5 {
            d += &format!("L{},{}", num(xs[i]), num(ys[i]));
            continue;
        }
        let ix = xs[i] - (xs[i] - xs[i - 1]) / before * rad;
        let iy = ys[i] - (ys[i] - ys[i - 1]) / before * rad;
        let ox = xs[i] + (xs[i + 1] - xs[i]) / after * rad;
        let oy = ys[i] + (ys[i + 1] - ys[i]) / after * rad;
        d += &format!(
            "L{},{}Q{},{} {},{}",
            num(ix),
            num(iy),
            num(xs[i]),
            num(ys[i]),
            num(ox),
            num(oy)
        )
    }
    d + &format!("L{},{}", num(xs[n - 1]), num(ys[n - 1]))
}
pub(super) struct Geometry {
    grid: super::layout::Grid,
    index: HashMap<String, usize>,
    nodes: usize,
    groups: usize,
}
pub(super) fn geometry(m: &ArchitectureModel) -> Geometry {
    Geometry {
        grid: place(&m.nodes, &m.groups, &m.edges, &m.layout_hints),
        index: m
            .nodes
            .iter()
            .enumerate()
            .map(|(i, (id, _))| (id.clone(), i))
            .collect(),
        nodes: m.nodes.len(),
        groups: m.groups.len(),
    }
}
pub(super) trait PhaseInput {
    type Error: From<PeleError>;
    fn begin(&mut self) -> Result<Geometry, Self::Error>;
    fn node_label(&mut self, i: usize) -> Result<Option<String>, Self::Error>;
    fn edge_count(&mut self) -> Result<usize, Self::Error>;
    fn edge(&mut self, i: usize, label: bool) -> Result<ArchEdge, Self::Error>;
    fn node_type(&mut self, i: usize) -> Result<String, Self::Error>;
    fn group_title(&mut self, i: usize) -> Result<Option<String>, Self::Error>;
    fn group_icon(&mut self, i: usize) -> Result<Option<String>, Self::Error>;
    fn group_id(&mut self, i: usize) -> Result<String, Self::Error>;
    fn node_paint(&mut self, i: usize) -> Result<ArchNode, Self::Error>;
    fn title(&mut self) -> Result<Option<String>, Self::Error>;
    fn accessibility(&mut self) -> Result<(Option<String>, Option<String>), Self::Error>;
}
struct Typed<'a>(&'a ArchitectureModel);
impl PhaseInput for Typed<'_> {
    type Error = PeleError;
    fn begin(&mut self) -> Result<Geometry, PeleError> {
        Ok(geometry(self.0))
    }
    fn node_label(&mut self, i: usize) -> Result<Option<String>, PeleError> {
        Ok(if self.0.nodes[i].1.node_type == "service" {
            self.0.nodes[i].1.title.clone()
        } else {
            None
        })
    }
    fn edge_count(&mut self) -> Result<usize, PeleError> {
        Ok(self.0.edges.len())
    }
    fn edge(&mut self, i: usize, _label: bool) -> Result<ArchEdge, PeleError> {
        Ok(self.0.edges[i].clone())
    }
    fn node_type(&mut self, i: usize) -> Result<String, PeleError> {
        Ok(self.0.nodes[i].1.node_type.clone())
    }
    fn group_title(&mut self, i: usize) -> Result<Option<String>, PeleError> {
        Ok(self.0.groups[i].1.title.clone())
    }
    fn group_icon(&mut self, i: usize) -> Result<Option<String>, PeleError> {
        Ok(self.0.groups[i].1.icon.clone())
    }
    fn group_id(&mut self, i: usize) -> Result<String, PeleError> {
        Ok(self.0.groups[i].1.id.clone())
    }
    fn node_paint(&mut self, i: usize) -> Result<ArchNode, PeleError> {
        Ok(self.0.nodes[i].1.clone())
    }
    fn title(&mut self) -> Result<Option<String>, PeleError> {
        Ok(self.0.title.clone())
    }
    fn accessibility(&mut self) -> Result<(Option<String>, Option<String>), PeleError> {
        Ok((self.0.acc_title.clone(), self.0.acc_descr.clone()))
    }
}
pub fn render(
    m: &ArchitectureModel,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_phase(&mut Typed(m), c, o, r)
}
pub(super) fn render_phase<P: PhaseInput>(
    m: &mut P,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    let loose = draw(m, c, o, r, false)?;
    if o.max_width.is_none_or(|w| !(w > 0.) || loose.width <= w) {
        return Ok(loose);
    }
    let tight = draw(m, c, o, r, true)?;
    Ok(if tight.width < loose.width {
        tight
    } else {
        loose
    })
}
fn title_room(o: &RenderSettings) -> f64 {
    o.max_width
        .filter(|w| *w > 0.)
        .map_or(4000., |w| (w - 2. * o.padding.unwrap_or(8.)).js_max(80.))
}
fn draw<P: PhaseInput>(
    m: &mut P,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    tight: bool,
) -> Result<Rendered, P::Error> {
    let icon_size = setting(c, "iconSize", 48., 8., 512.);
    let size = setting(c, "fontSize", o.font_size.unwrap_or(16.), 4., 128.);
    let group_pad =
        setting(c, "padding", 20., 0., 256.).js_min(if tight { 12. } else { f64::INFINITY });
    let small = round(size * 0.875);
    let pad = o.padding.unwrap_or(8.);
    let half = icon_size / 2.;
    let Geometry {
        grid,
        index,
        nodes: n,
        groups: g,
    } = m.begin()?;
    let mut labels = vec![];
    let mut box_w = vec![0f64; n];
    let mut box_h = vec![0f64; n];
    let wrap = if tight {
        icon_size.js_max(64.)
    } else {
        (icon_size * 2.25).js_max(96.)
    };
    for i in 0..n {
        let title = m.node_label(i)?;
        let label = layout_label(title.as_deref(), true, r, o, size, wrap, 0)?;
        box_w[i] = icon_size.js_max(2. * (label.width / 2.).ceil());
        box_h[i] = icon_size
            + if label.height > 0. {
                4. + label.height
            } else {
                0.
            };
        labels.push(label)
    }
    let mut cv = grid.col.clone();
    let mut rv = grid.row.clone();
    for k in 0..g {
        if grid.group_empty[k] {
            cv.push(grid.group_col[k]);
            rv.push(grid.group_row[k])
        }
    }
    let co = rank(&mut cv);
    let ro = rank(&mut rv);
    let (cols, rows) = (co.len(), ro.len());
    let mut cw = vec![0f64; cols];
    let mut rh = vec![0f64; rows];
    let gap = if tight {
        24f64.js_max(2. * STUB)
    } else {
        40f64.js_max(round(icon_size * 0.75))
    };
    let mut gx = vec![gap; cols];
    let mut gy = vec![gap; rows];
    let nc: Vec<usize> = grid.col.iter().map(|v| co[v]).collect();
    let nr: Vec<usize> = grid.row.iter().map(|v| ro[v]).collect();
    for i in 0..n {
        cw[nc[i]] = cw[nc[i]].js_max(box_w[i]);
        rh[nr[i]] = rh[nr[i]].js_max(box_h[i]);
    }
    for k in 0..g {
        if grid.group_empty[k] {
            let c = co[&grid.group_col[k]];
            let rr = ro[&grid.group_row[k]];
            cw[c] = cw[c].js_max(icon_size);
            rh[rr] = rh[rr].js_max(half)
        }
    }
    let mut edge_labels = vec![];
    let mut ends = vec![];
    let edge_count = m.edge_count()?;
    for edge_index in 0..edge_count {
        let edge = m.edge(edge_index, true)?;
        let a = index.get(edge.lhs_id.as_str()).copied();
        let b = index.get(edge.rhs_id.as_str()).copied();
        ends.push((a, b));
        let label = layout_label(edge.title.as_deref(), true, r, o, small, 160., 0)?;
        if label.width > 0. {
            if let (Some(a), Some(b)) = (a, b) {
                if nc[a] != nc[b] {
                    let at = nc[a].min(nc[b]);
                    gx[at] = gx[at].js_max(label.width.ceil() + 28.)
                } else {
                    cw[nc[a]] = cw[nc[a]].js_max(2. * (label.width / 2.).ceil() + 8.);
                    if nr[a] != nr[b] {
                        let at = nr[a].min(nr[b]);
                        gy[at] = gy[at].js_max(label.height.ceil() + 24.)
                    }
                }
            }
        }
        edge_labels.push(label)
    }
    let (mut gl, mut c0, mut c1, mut r0, mut r1) =
        (vec![], vec![0; g], vec![0; g], vec![0; g], vec![0; g]);
    let (mut il, mut ir, mut it, mut ib, mut hw, mut hh) = (
        vec![0f64; g],
        vec![0f64; g],
        vec![0f64; g],
        vec![0f64; g],
        vec![0f64; g],
        vec![0f64; g],
    );
    let (mut cl, mut cr, mut rt, mut rb) = (
        vec![0f64; cols],
        vec![0f64; cols],
        vec![0f64; rows],
        vec![0f64; rows],
    );
    let badge = round(size * 1.25);
    for k in 0..g {
        c0[k] = co.get(&grid.group_col[k]).copied().unwrap_or(0);
        c1[k] = co
            .get(&(grid.group_col[k] + grid.group_cols[k] - 1))
            .copied()
            .unwrap_or(0);
        r0[k] = ro.get(&grid.group_row[k]).copied().unwrap_or(0);
        r1[k] = ro
            .get(&(grid.group_row[k] + grid.group_rows[k] - 1))
            .copied()
            .unwrap_or(0);
        let title = m.group_title(k)?;
        let label = layout_label(
            title.as_deref(),
            true,
            r,
            o,
            size,
            if tight {
                80f64.js_max(title_room(o) / 2.)
            } else {
                320.
            },
            0,
        )?;
        let icon = if m.group_icon(k)?.as_deref().is_some_and(|s| !s.is_empty()) {
            badge
        } else {
            0.
        };
        hw[k] = label.width
            + icon
            + if label.width > 0. && icon > 0. {
                6.
            } else {
                0.
            };
        hh[k] = label.height.js_max(icon);
        gl.push(label)
    }
    for k in (0..g).rev() {
        let left = group_pad + il[k];
        let mut right = group_pad + ir[k];
        let top = if hh[k] > 0. {
            group_pad.js_max(hh[k] + 12.)
        } else {
            group_pad
        } + it[k];
        let bottom = group_pad + ib[k];
        if hw[k] > 0. {
            let mut inner = left + right;
            for cc in c0[k]..=c1[k] {
                inner += cw[cc]
                    + if cc < c1[k] {
                        cr[cc] + gx[cc] + cl[cc + 1]
                    } else {
                        0.
                    }
            }
            if hw[k] + 20. > inner {
                right += (hw[k] + 20. - inner).ceil()
            }
        }
        il[k] = left;
        ir[k] = right;
        it[k] = top;
        ib[k] = bottom;
        cl[c0[k]] = cl[c0[k]].js_max(left);
        cr[c1[k]] = cr[c1[k]].js_max(right);
        rt[r0[k]] = rt[r0[k]].js_max(top);
        rb[r1[k]] = rb[r1[k]].js_max(bottom);
        if grid.group_parent[k] >= 0 {
            let p = grid.group_parent[k] as usize;
            if c0[p] == c0[k] {
                il[p] = il[p].js_max(left)
            }
            if c1[p] == c1[k] {
                ir[p] = ir[p].js_max(right)
            }
            if r0[p] == r0[k] {
                it[p] = it[p].js_max(top)
            }
            if r1[p] == r1[k] {
                ib[p] = ib[p].js_max(bottom)
            }
        }
    }
    let mut xs = vec![0.; cols];
    let mut ys = vec![0.; rows];
    let mut inner_w = 0.;
    let mut inner_h = 0.;
    for cc in 0..cols {
        xs[cc] = inner_w + cl[cc];
        inner_w = xs[cc] + cw[cc] + cr[cc] + if cc + 1 < cols { gx[cc] } else { 0. }
    }
    for rr in 0..rows {
        ys[rr] = inner_h + rt[rr];
        inner_h = ys[rr] + rh[rr] + rb[rr] + if rr + 1 < rows { gy[rr] } else { 0. }
    }
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (0f64, 0f64, inner_w, inner_h);
    let left = |k: usize| xs[c0[k]] - il[k];
    let right = |k: usize| xs[c1[k]] + cw[c1[k]] + ir[k];
    let top = |k: usize| ys[r0[k]] - it[k];
    let bottom = |k: usize| ys[r1[k]] + rh[r1[k]] + ib[k];
    let center = |i: usize| xs[nc[i]] + cw[nc[i]] / 2.;
    let port = |i: usize, side: usize, via: bool, junction: bool| {
        let (dx, dy) = (DX[side] as f64, DY[side] as f64);
        let cx = center(i);
        let y = ys[nr[i]];
        let mut p = Port {
            x: cx,
            y: y + half,
            dx,
            dy,
            x0: cx - box_w[i] / 2.,
            y0: y,
            x1: cx + box_w[i] / 2.,
            y1: y + box_h[i],
            gap: 0.,
            wire: false,
        };
        if junction {
            p.x0 = cx;
            p.x1 = cx;
            p.y0 = p.y;
            p.y1 = p.y;
            p.gap = DOT
        } else if dx != 0. {
            p.x = cx + dx * half
        } else {
            p.y = if dy < 0. {
                y
            } else {
                y + box_h[i] + if labels[i].height > 0. { 2. } else { 0. }
            }
        }
        let k = if via { grid.node_parent[i] } else { -1 };
        if k >= 0 {
            let k = k as usize;
            p.x0 = left(k);
            p.x1 = right(k);
            p.y0 = top(k);
            p.y1 = bottom(k);
            p.gap = 0.;
            p.wire = junction;
            if dx != 0. {
                p.x = if dx < 0. { p.x0 } else { p.x1 }
            } else {
                p.y = if dy < 0. { p.y0 } else { p.y1 }
            }
        }
        p
    };
    let (mut edges_out, mut labels_out) = (String::new(), String::new());
    let mut runs = vec![];
    let trim = marker_trim("arrow_point");
    for k in 0..edge_count {
        let edge = m.edge(k, false)?;
        let (Some(a), Some(b)) = ends[k] else {
            continue;
        };
        let mut s = port(
            a,
            side_index(edge.lhs_dir),
            edge.lhs_group,
            m.node_type(a)? == "junction",
        );
        let mut e = port(
            b,
            side_index(edge.rhs_dir),
            edge.rhs_group,
            m.node_type(b)? == "junction",
        );
        if s.x == e.x && s.y == e.y && s.dx == e.dx {
            s.x -= 8. * s.dy.abs();
            s.y -= 8. * s.dx.abs();
            e.x += 8. * s.dy.abs();
            e.y += 8. * s.dx.abs()
        }
        let mut routed = route(s, e);
        if s.wire {
            routed.splice(0..0, [center(a), ys[nr[a]] + half]);
        }
        if e.wire {
            routed.extend([center(b), ys[nr[b]] + half])
        }
        let points = simplify(&routed);
        if points.len() < 4 {
            continue;
        }
        let id = format!("{}-{}", edge.lhs_id, edge.rhs_id);
        edges_out += &format!(
            "<g class=\"pele-edge\" data-id=\"{}\"><path d=\"{}\"/>{}{}</g>",
            esc_text(&id)?,
            path_data(
                &points,
                if edge.lhs_into && !s.wire {
                    trim + s.gap
                } else {
                    0.
                },
                if edge.rhs_into && !e.wire {
                    trim + e.gap
                } else {
                    0.
                }
            ),
            if edge.lhs_into {
                marker(
                    "arrow_point",
                    s.x + s.dx * s.gap,
                    s.y + s.dy * s.gap,
                    -s.dx,
                    -s.dy,
                    "var(--_l)",
                )
            } else {
                String::new()
            },
            if edge.rhs_into {
                marker(
                    "arrow_point",
                    e.x + e.dx * e.gap,
                    e.y + e.dy * e.gap,
                    -e.dx,
                    -e.dy,
                    "var(--_l)",
                )
            } else {
                String::new()
            }
        );
        let (mut best, mut flat) = (0, 0);
        let (mut best_len, mut flat_len) = (-1f64, -1f64);
        let label = &edge_labels[k];
        for p in points.chunks_exact(2) {
            min_x = min_x.js_min(p[0] - 4.);
            max_x = max_x.js_max(p[0] + 4.);
            min_y = min_y.js_min(p[1] - 4.);
            max_y = max_y.js_max(p[1] + 4.)
        }
        for at in (0..points.len() - 3).step_by(2) {
            let horizontal = (points[at + 1] - points[at + 3]).abs() < 0.01;
            let len = (points[at] - points[at + 2]).abs() + (points[at + 1] - points[at + 3]).abs();
            if !horizontal {
                runs.push((
                    points[at],
                    points[at + 1].js_min(points[at + 3]),
                    points[at + 1].js_max(points[at + 3]),
                ))
            }
            if len > best_len {
                best = at;
                best_len = len
            }
            if horizontal && len > flat_len {
                flat = at;
                flat_len = len
            }
        }
        if label.width > 0. {
            let at = if flat_len >= label.width + 8. {
                flat
            } else {
                best
            };
            let mut x = (points[at] + points[at + 2]) / 2.;
            let mut y = (points[at + 1] + points[at + 3]) / 2.;
            let w = label.width + 8.;
            let upright = (points[at] - points[at + 2]).abs() < 0.01;
            for p in [s, e] {
                if x + w / 2. <= p.x0
                    || x - w / 2. >= p.x1
                    || y + label.height / 2. <= p.y0
                    || y - label.height / 2. >= p.y1
                {
                    continue;
                }
                if upright {
                    x = if x >= (p.x0 + p.x1) / 2. {
                        p.x1 + w / 2. + 4.
                    } else {
                        p.x0 - w / 2. - 4.
                    }
                } else {
                    y = if y >= (p.y0 + p.y1) / 2. {
                        p.y1 + label.height / 2. + 4.
                    } else {
                        p.y0 - label.height / 2. - 4.
                    }
                }
            }
            min_x = min_x.js_min(x - w / 2.);
            max_x = max_x.js_max(x + w / 2.);
            min_y = min_y.js_min(y - label.height / 2.);
            max_y = max_y.js_max(y + label.height / 2.);
            labels_out += &edge_label_svg(&id, label, x, y, "", r)?
        }
    }
    let mut clusters = String::new();
    for k in 0..g {
        let (x, y) = (left(k), top(k));
        let w = right(k) - x;
        let label = &gl[k];
        let mut head = String::new();
        if hw[k] > 0. {
            let band = y + hh[k] + 12.;
            let mut crossings: Vec<f64> = runs
                .iter()
                .filter(|(rx, y0, y1)| *y0 < band && *y1 > y && *rx > x && *rx < x + w)
                .map(|v| v.0)
                .collect();
            crossings.sort_by(f64::total_cmp);
            crossings.push(x + w - 4.);
            let mut start = x + 10.;
            for cross in crossings {
                if cross - 6. - start >= hw[k] {
                    break;
                }
                start = start.js_max(cross + 6.)
            }
            if start + hw[k] > x + w - 4. {
                start = x + 10.
            }
            let cy = y + 6. + hh[k] / 2.;
            let mut tx = start;
            let icon = m.group_icon(k)?;
            if let Some(icon) = icon.as_deref().filter(|s| !s.is_empty()) {
                head += &icon_slot(icon, start, cy - badge / 2., badge, r)?;
                tx += badge + 6.
            }
            head += &label_svg_with_resources(
                label,
                tx + label.width / 2.,
                cy,
                " class=\"pele-cluster-label\" fill=\"var(--_m)\"",
                r,
            )?
        }
        clusters+=&format!("<g class=\"pele-cluster\" data-id=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{RADIUS}\" fill=\"var(--_a)\" fill-opacity=\"0.5\" stroke=\"var(--_b)\"/>{head}</g>",esc_text(&m.group_id(k)?)?,num(x),num(y),num(w),num(bottom(k)-y))
    }
    let mut nodes_out = String::new();
    let glyph = icon_size * 0.5;
    let text_size = 6f64.js_max(round(size * 0.75));
    for i in 0..n {
        let node = m.node_paint(i)?;
        let at = format!(
            "data-id=\"{}\" transform=\"translate({},{})\"",
            esc_text(&node.id)?,
            num(center(i)),
            num(ys[nr[i]])
        );
        if node.node_type == "junction" {
            nodes_out+=&format!("<g class=\"pele-node pele-junction\" {at}><circle cy=\"{}\" r=\"{DOT}\" fill=\"var(--_l)\"/></g>",num(half));
            continue;
        }
        let mut inner = String::new();
        if let Some(icon) = node.icon.as_deref().filter(|s| !s.is_empty()) {
            inner = icon_slot(icon, -glyph / 2., (icon_size - glyph) / 2., glyph, r)?
        } else if let Some(text) = node.icon_text.as_deref().filter(|s| !s.is_empty()) {
            let text = layout_label(Some(text), false, r, o, text_size, icon_size - 8., 0)?;
            let lines = ((icon_size - 4.)
                / if text.line_height != 0. {
                    text.line_height
                } else {
                    1.
                })
            .floor()
            .js_max(1.) as usize;
            inner = label_svg_with_resources(
                &fit(text, icon_size - 8., lines, o, r)?,
                0.,
                half,
                &format!(" class=\"pele-icon-text\" font-size=\"{}\"", num(text_size)),
                r,
            )?
        }
        nodes_out+=&format!("<g class=\"pele-node pele-service\" {at}><rect x=\"{}\" width=\"{}\" height=\"{}\" rx=\"{RADIUS}\" fill=\"var(--_s)\" stroke=\"var(--_b)\"/>{inner}{}</g>",num(-half),num(icon_size),num(icon_size),label_svg_with_resources(&labels[i],0.,icon_size+4.+labels[i].height/2.," class=\"pele-label\"",r)?)
    }
    let title_text = m.title()?;
    let title = layout_label(
        title_text.as_deref(),
        false,
        r,
        o,
        size,
        if tight { title_room(o) } else { 4000. },
        1,
    )?;
    let th = if title.height > 0. {
        title.height + 12.
    } else {
        0.
    };
    let width = ((max_x - min_x).js_max(title.width) + 2. * pad).ceil();
    let height = (max_y - min_y + th + 2. * pad).ceil();
    let ox = round((width - (max_x - min_x)) / 2. - min_x);
    let oy = round(pad + th - min_y);
    let mut body = String::new();
    for(content,pre,post)in[(&clusters,"<g class=\"pele-clusters\">","</g>"),(&edges_out,"<g class=\"pele-edges\" fill=\"none\" stroke=\"var(--_l)\" stroke-linecap=\"round\" stroke-linejoin=\"round\">","</g>"),(&labels_out,&format!("<g class=\"pele-edge-labels\" font-size=\"{}\">",num(small)),"</g>"),(&nodes_out,"<g class=\"pele-nodes\">","</g>")]{if !content.is_empty(){body+=pre;body+=content;body+=post}}
    let content = label_svg(
        &title,
        width / 2.,
        pad + title.height / 2.,
        " class=\"pele-title\" font-weight=\"var(--_tw)\"",
    ) + &if body.is_empty() {
        String::new()
    } else {
        format!(
            "<g transform=\"translate({},{})\">{body}</g>",
            num(ox),
            num(oy)
        )
    };
    let (acc_title, acc_descr) = m.accessibility()?;
    Ok(Rendered {
        svg: document(
            "architecture",
            width,
            height,
            size,
            o,
            acc_title.as_deref(),
            acc_descr.as_deref(),
            &content,
        )?,
        width,
        height,
        links: vec![],
    })
}
