//! Pele treemap/layout.ts squarified rows, hierarchy sums and section insets.
use crate::compat::PeleError;
pub(super) trait Input {
    type Node: Clone;
    type Error: From<PeleError>;
    fn roots(&mut self) -> Result<Vec<Self::Node>, Self::Error>;
    fn children(&mut self, node: &Self::Node) -> Result<Vec<Self::Node>, Self::Error>;
    fn initial_value(&mut self, node: &Self::Node) -> Result<f64, Self::Error>;
    fn reserve(&mut self, _count: usize) -> Result<(), Self::Error> {
        Ok(())
    }
}
use crate::compat::number_math::{round, NumberMath};
pub(super) struct Box<N> {
    pub node: N,
    parent: Option<usize>,
    pub series: usize,
    pub value: f64,
    children: Vec<usize>,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}
impl<N> Box<N> {
    fn new(node: N, parent: Option<usize>, series: usize, value: f64) -> Self {
        Self {
            node,
            parent,
            series,
            value,
            children: vec![],
            x: 0.,
            y: 0.,
            w: 0.,
            h: 0.,
        }
    }
}
fn max(a: f64, b: f64) -> f64 {
    a.js_max(b)
}
fn squarify<N>(
    all: &mut [Box<N>],
    boxes: &[usize],
    total: f64,
    mut x: f64,
    mut y: f64,
    mut w: f64,
    mut h: f64,
    gap: f64,
) {
    let mut rest = total;
    let mut from = 0;
    while from < boxes.len() {
        let scale = w * h / rest;
        let side = w.js_min(h);
        let mut sum = all[boxes[from]].value * scale;
        let largest = sum;
        let mut worst = max(
            side * side * largest / (sum * sum),
            sum * sum / (side * side * largest),
        );
        let mut to = from + 1;
        while to < boxes.len() {
            let smallest = all[boxes[to]].value * scale;
            let next = sum + smallest;
            let ratio = max(
                side * side * largest / (next * next),
                next * next / (side * side * smallest),
            );
            if ratio > worst {
                break;
            }
            worst = ratio;
            sum = next;
            to += 1;
        }
        let thickness = sum / side;
        let mut offset = 0.;
        for (i, &k) in boxes.iter().enumerate().take(to).skip(from) {
            let b = &mut all[k];
            let length = if i == to - 1 {
                side - offset
            } else {
                b.value * scale / thickness
            };
            let bx = if w >= h { x } else { x + offset };
            let by = if w >= h { y + offset } else { y };
            let x0 = round(bx + gap / 2.);
            let y0 = round(by + gap / 2.);
            b.x = x0;
            b.y = y0;
            b.w = round(bx + if w >= h { thickness } else { length } - gap / 2.) - x0;
            b.h = round(by + if w >= h { length } else { thickness } - gap / 2.) - y0;
            offset += length;
            rest -= b.value;
        }
        if w >= h {
            x += thickness;
            w -= thickness;
        } else {
            y += thickness;
            h -= thickness;
        }
        from = to;
    }
}
fn place<N>(
    all: &mut [Box<N>],
    children: &[usize],
    placed: &mut Vec<usize>,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    gap: f64,
) {
    if w <= 0. || h <= 0. {
        return;
    }
    let mut shown: Vec<_> = children
        .iter()
        .copied()
        .filter(|i| all[*i].value > 0.)
        .collect();
    shown.sort_by(|a, b| {
        all[*b]
            .value
            .partial_cmp(&all[*a].value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let total: f64 = shown.iter().map(|i| all[*i].value).sum();
    if shown.is_empty() || !total.is_finite() {
        return;
    }
    squarify(
        all,
        &shown,
        total,
        x - gap / 2.,
        y - gap / 2.,
        w + gap,
        h + gap,
        gap,
    );
    for i in shown {
        if all[i].w > 0. && all[i].h > 0. {
            placed.push(i)
        }
    }
}
pub(super) fn layout<P: Input>(
    input: &mut P,
    width: f64,
    height: f64,
    gap: f64,
) -> Result<Vec<Box<P::Node>>, P::Error> {
    let roots = input.roots()?;
    let mut all = Vec::new();
    for (series, node) in roots.into_iter().enumerate() {
        input.reserve(all.len())?;
        let value = input.initial_value(&node)?;
        all.push(Box::new(node, None, series, value));
    }
    let top: Vec<_> = (0..all.len()).collect();
    let mut i = 0;
    while i < all.len() {
        let parent = i;
        let series = all[i].series;
        let node = all[i].node.clone();
        for child in input.children(&node)? {
            input.reserve(all.len())?;
            let value = input.initial_value(&child)?;
            let k = all.len();
            all[parent].children.push(k);
            all.push(Box::new(child, Some(parent), series, value));
        }
        i += 1;
    }
    for i in (0..all.len()).rev() {
        if let Some(parent) = all[i].parent {
            all[parent].value += all[i].value;
        }
    }
    let mut placed = vec![];
    place(&mut all, &top, &mut placed, 0., 0., width, height, gap);
    let mut i = 0;
    while i < placed.len() {
        let b = &all[placed[i]];
        let (x, y, w, h, children) = (
            b.x + 6.,
            b.y + 28.,
            b.w - 12.,
            b.h - 34.,
            b.children.clone(),
        );
        if !children.is_empty() {
            place(&mut all, &children, &mut placed, x, y, w, h, gap)
        }
        i += 1;
    }
    let mut values: Vec<_> = all.into_iter().map(Some).collect();
    Ok(placed
        .into_iter()
        .map(|i| values[i].take().unwrap())
        .collect())
}
