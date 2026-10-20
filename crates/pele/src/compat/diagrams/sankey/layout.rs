//! Pele sankey/layout.ts topological columns, flow scaling and six relaxation passes.
//! 458322a5 (MIT, kepano). Sorts retain insertion ties, matching JS stable sort.
use super::indexed::Input;
use crate::compat::number_math::NumberMath;
use crate::compat::{ErrorCode, PeleError};
#[derive(Clone, Copy)]
pub(super) enum Alignment {
    Justify,
    Left,
    Right,
    Center,
}
pub(super) struct Options {
    pub width: f64,
    pub height: f64,
    pub node_width: f64,
    pub node_padding: f64,
    pub label_height: f64,
    pub alignment: Alignment,
}
pub(super) struct Node {
    pub layer: usize,
    pub value: f64,
    pub x: f64,
    pub y: f64,
    pub height: f64,
}
pub(super) struct Link {
    pub source: usize,
    pub target: usize,
    pub value: f64,
    pub thickness: f64,
    pub source_y: f64,
    pub target_y: f64,
}
pub(super) struct Layout {
    pub nodes: Vec<Node>,
    pub links: Vec<Link>,
}
// Match Math.min/max NaN propagation and signed-zero ties.
pub(super) fn min(a: f64, b: f64) -> f64 {
    a.js_min(b)
}
pub(super) fn max(a: f64, b: f64) -> f64 {
    a.js_max(b)
}
fn compare(a: f64, b: f64, ia: usize, ib: usize) -> std::cmp::Ordering {
    let d = a - b;
    if d == 0. || d.is_nan() {
        ia.cmp(&ib)
    } else if d < 0. {
        std::cmp::Ordering::Less
    } else {
        std::cmp::Ordering::Greater
    }
}
struct Work {
    nodes: Vec<Node>,
    links: Vec<Link>,
    outgoing: Vec<Vec<usize>>,
    incoming: Vec<Vec<usize>>,
    source_offset: Vec<f64>,
    target_offset: Vec<f64>,
    margin: Vec<f64>,
    gap: f64,
    height: f64,
}
impl Work {
    fn center(&self, i: usize) -> f64 {
        self.nodes[i].y + self.nodes[i].height / 2.
    }
    fn stack_out(&mut self, i: usize) {
        let nodes = &self.nodes;
        let links = &self.links;
        self.outgoing[i].sort_by(|a, b| {
            let na = &nodes[links[*a].target];
            let nb = &nodes[links[*b].target];
            compare(na.y + na.height / 2., nb.y + nb.height / 2., *a, *b)
        });
        let mut y = 0.;
        for &k in &self.outgoing[i] {
            self.source_offset[k] = y + self.links[k].thickness / 2.;
            y += self.links[k].thickness;
        }
    }
    fn stack_in(&mut self, i: usize) {
        let nodes = &self.nodes;
        let links = &self.links;
        self.incoming[i].sort_by(|a, b| {
            let na = &nodes[links[*a].source];
            let nb = &nodes[links[*b].source];
            compare(na.y + na.height / 2., nb.y + nb.height / 2., *a, *b)
        });
        let mut y = 0.;
        for &k in &self.incoming[i] {
            self.target_offset[k] = y + self.links[k].thickness / 2.;
            y += self.links[k].thickness;
        }
    }
    fn stack_all(&mut self) {
        for i in 0..self.nodes.len() {
            self.stack_out(i);
            self.stack_in(i);
        }
    }
    fn settle(&mut self, column: &mut [usize], pull: f64, forward: bool) {
        for &i in column.iter() {
            if if forward {
                self.incoming[i].is_empty()
            } else {
                self.outgoing[i].is_empty()
            } {
                continue;
            }
            if forward {
                self.stack_in(i)
            } else {
                self.stack_out(i)
            }
            let list = if forward {
                &self.incoming[i]
            } else {
                &self.outgoing[i]
            };
            let (mut weighted, mut weight, mut plain) = (0., 0., 0.);
            for &k in list {
                let link = &self.links[k];
                let level = if forward {
                    self.nodes[link.source].y + self.source_offset[k] - self.target_offset[k]
                } else {
                    self.nodes[link.target].y + self.target_offset[k] - self.source_offset[k]
                };
                weighted += level * link.thickness;
                weight += link.thickness;
                plain += self.center(if forward { link.source } else { link.target })
                    - self.nodes[i].height / 2.;
            }
            let wanted = if weight > 0. {
                weighted / weight
            } else {
                plain / list.len() as f64
            };
            self.nodes[i].y += (wanted - self.nodes[i].y) * pull;
        }
        column.sort_by(|a, b| compare(self.center(*a), self.center(*b), *a, *b));
        let mut y = 0.;
        for &i in column.iter() {
            if self.nodes[i].y - self.margin[i] < y {
                self.nodes[i].y = y + self.margin[i]
            }
            y = self.nodes[i].y + self.nodes[i].height + self.margin[i] + self.gap;
        }
        y = self.height;
        for &i in column.iter().rev() {
            if self.nodes[i].y + self.nodes[i].height + self.margin[i] > y {
                self.nodes[i].y = y - self.nodes[i].height - self.margin[i]
            }
            y = self.nodes[i].y - self.margin[i] - self.gap;
        }
    }
}
pub(super) fn layout(model: &impl Input, o: Options) -> Result<Layout, PeleError> {
    let n = model.nodes().len();
    let mut links: Vec<Link> = (0..model.link_count())
        .map(|i| {
            let (source, target, value) = model.link(i);
            Link {
                source,
                target,
                value: if value.is_finite() && value > 0. {
                    value
                } else {
                    0.
                },
                thickness: 0.,
                source_y: 0.,
                target_y: 0.,
            }
        })
        .collect();
    let mut outgoing = vec![vec![]; n];
    let mut incoming = vec![vec![]; n];
    for (k, l) in links.iter().enumerate() {
        outgoing[l.source].push(k);
        incoming[l.target].push(k)
    }
    let mut depth = vec![0; n];
    let mut pending: Vec<usize> = incoming.iter().map(Vec::len).collect();
    let mut order: Vec<usize> = (0..n).filter(|i| pending[*i] == 0).collect();
    let mut q = 0;
    while q < order.len() {
        let i = order[q];
        for &k in &outgoing[i] {
            let t = links[k].target;
            if depth[t] < depth[i] + 1 {
                depth[t] = depth[i] + 1
            }
            pending[t] -= 1;
            if pending[t] == 0 {
                order.push(t)
            }
        }
        q += 1;
    }
    if order.len() < n {
        let mut stuck = (0..n).find(|i| pending[*i] != 0).unwrap();
        let mut seen = vec![false; n];
        while !seen[stuck] {
            seen[stuck] = true;
            for &k in &incoming[stuck] {
                if pending[links[k].source] > 0 {
                    stuck = links[k].source;
                    break;
                }
            }
        }
        let mut error = PeleError::new(
            format!(
                "Sankey diagrams cannot have circular links, and \"{}\" is part of one.",
                model.nodes()[stuck].id
            ),
            ErrorCode::Semantic,
        );
        error.diagram_type = Some("sankey".into());
        return Err(error);
    }
    let layers = depth.iter().map(|d| d + 1).max().unwrap_or(1);
    let mut reach = vec![0; n];
    for &i in order.iter().rev() {
        for &k in &outgoing[i] {
            let r = reach[links[k].target] + 1;
            if reach[i] < r {
                reach[i] = r
            }
        }
    }
    let largest = links.iter().fold(0_f64, |v, l| max(v, l.value));
    let unit = if largest > 0. { 1. / largest } else { 0. };
    let mut nodes = vec![];
    let mut flow = vec![0.; n];
    let mut columns = vec![vec![]; layers];
    let step = if layers > 1 {
        (o.width - o.node_width) / (layers - 1) as f64
    } else {
        0.
    };
    for i in 0..n {
        let layer = match o.alignment {
            Alignment::Left => depth[i],
            Alignment::Right => layers - 1 - reach[i],
            Alignment::Center => {
                if incoming[i].is_empty() && !outgoing[i].is_empty() {
                    outgoing[i]
                        .iter()
                        .map(|k| depth[links[*k].target] - 1)
                        .min()
                        .unwrap_or(layers)
                } else {
                    depth[i]
                }
            }
            Alignment::Justify => {
                if !outgoing[i].is_empty() {
                    depth[i]
                } else {
                    layers - 1
                }
            }
        };
        let (mut into, mut out, mut value, mut value_out) = (0., 0., 0., 0.);
        for &k in &incoming[i] {
            into += links[k].value * unit;
            value += links[k].value;
        }
        for &k in &outgoing[i] {
            out += links[k].value * unit;
            value_out += links[k].value;
        }
        flow[i] = max(into, out);
        nodes.push(Node {
            layer,
            value: max(value, value_out),
            x: layer as f64 * step,
            y: 0.,
            height: 0.,
        });
        columns[layer].push(i);
    }
    let tallest = columns.iter().map(Vec::len).max().unwrap_or(1).max(1);
    let gap = if tallest > 1 {
        min(o.node_padding, o.height / (tallest - 1) as f64)
    } else {
        o.node_padding
    };
    let mut scale = f64::INFINITY;
    for col in &columns {
        let sum: f64 = col.iter().map(|i| flow[*i]).sum();
        if sum > 0. {
            scale = min(scale, (o.height - (col.len() as f64 - 1.) * gap) / sum)
        }
    }
    if !scale.is_finite() || scale < 0. {
        scale = 0.
    }
    let mut claim = max(0., o.label_height - gap);
    let fits = |s: f64, claim: f64| {
        for col in &columns {
            let mut extent = (col.len() as f64 - 1.) * gap;
            for &i in col {
                extent += max(flow[i] * s, claim)
            }
            if extent > o.height + 1e-6 {
                return false;
            }
        }
        true
    };
    if claim > 0. && !fits(scale, claim) {
        if fits(0., claim) {
            let mut low = 0.;
            for _ in 0..40 {
                let mid = (low + scale) / 2.;
                if fits(mid, claim) {
                    low = mid
                } else {
                    scale = mid
                }
            }
            scale = low;
        } else {
            claim = 0.
        }
    }
    let mut margin = vec![0.; n];
    for link in &mut links {
        link.thickness = link.value * unit * scale;
    }
    for col in &columns {
        let mut y = 0.;
        for &i in col {
            nodes[i].height = flow[i] * scale;
            margin[i] = max(0., claim - nodes[i].height) / 2.;
            nodes[i].y = y + margin[i];
            y += nodes[i].height + 2. * margin[i] + gap;
        }
        let spare = (o.height - y + gap) / (col.len() + 1) as f64;
        for (k, &i) in col.iter().enumerate() {
            nodes[i].y += spare * (k + 1) as f64;
        }
    }
    let len = links.len();
    let mut w = Work {
        nodes,
        links,
        outgoing,
        incoming,
        source_offset: vec![0.; len],
        target_offset: vec![0.; len],
        margin,
        gap,
        height: o.height,
    };
    w.stack_all();
    // JS Math.pow(0.99, pass), fixed integral exponents 0..5. Literal IEEE values
    // avoid platform libm differences on WASM and retain the pinned arithmetic.
    for pull in [
        1.,
        0.99,
        0.9801,
        0.970299,
        0.9605960099999999,
        0.9509900498999999,
    ] {
        for l in (0..layers.saturating_sub(1)).rev() {
            w.settle(&mut columns[l], pull, false)
        }
        w.stack_all();
        for col in columns.iter_mut().skip(1) {
            w.settle(col, pull, true)
        }
        w.stack_all();
    }
    for (k, link) in w.links.iter_mut().enumerate() {
        link.source_y = w.nodes[link.source].y + w.source_offset[k];
        link.target_y = w.nodes[link.target].y + w.target_offset[k];
    }
    Ok(Layout {
        nodes: w.nodes,
        links: w.links,
    })
}
