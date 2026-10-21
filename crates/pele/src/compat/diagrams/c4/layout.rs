//! Pele c4/layout.ts actual nested grid layout, 458322a5 (MIT, kepano).
use super::{
    model::ElementRef,
    phase::{Collection, PhaseInput},
    route::{Box, HEAD, LANE},
};
use crate::compat::number_math::NumberMath;
use crate::compat::{
    source::space,
    text::{layout_label, Label},
    PeleError, RenderResources, RenderSettings,
};
use std::collections::HashMap;
pub(super) const GROUP_PAD: f64 = LANE + 8.;
pub(super) const HEAD_LEFT: f64 = 16.;
pub(super) const HEAD_TOP: f64 = 10.;
pub(super) struct Node<R> {
    pub shape: R,
    pub bbox: Box,
    pub labels: Vec<Label>,
    pub text_h: f64,
}
pub(super) struct Group<R> {
    pub boundary: R,
    pub bbox: Box,
    pub kids: Vec<usize>,
    pub nodes: Vec<usize>,
    pub labels: Vec<Label>,
    pub head: f64,
    pub head_w: f64,
    pub head_x: f64,
}
pub(super) struct Layout<R> {
    pub root: usize,
    pub groups: Vec<Group<R>>,
    pub nodes: Vec<Node<R>>,
    pub order: Vec<usize>,
    pub boxes: HashMap<ElementRef, Box>,
}
fn form(name: &str) -> Option<&'static str> {
    Some(match name {
        "person" => "person",
        "box" | "rounded" | "component" => "rect",
        "cylinder" | "database" | "db" => "cyl",
        "queue" | "pipe" => "h-cyl",
        _ => return None,
    })
}
fn form_of<P: PhaseInput>(input: &mut P, shape: &P::Record) -> Result<String, P::Error> {
    if let Some(form) = form(
        &input
            .text(shape, "shape")?
            .unwrap_or_default()
            .to_lowercase(),
    ) {
        return Ok(form.into());
    }
    if let Some(form) = form(
        &input
            .text(shape, "sprite")?
            .unwrap_or_default()
            .to_lowercase(),
    ) {
        return Ok(form.into());
    }
    if input.flag(shape, "tags")? {
        let tags = input.text(shape, "tags")?.unwrap_or_default();
        for tag in tags.split(',') {
            if let Some(form) = form(&tag.trim_matches(space).to_lowercase()) {
                return Ok(form.into());
            }
        }
    }
    let kind = input.text(shape, "kind")?.unwrap_or_default();
    Ok(if kind.ends_with("person") {
        "person"
    } else if kind.ends_with("_db") {
        "cyl"
    } else if kind.ends_with("_queue") {
        "h-cyl"
    } else {
        "rect"
    }
    .into())
}
fn measure<P: PhaseInput>(
    input: &mut P,
    shape: P::Record,
    identity: ElementRef,
    resources: &mut dyn RenderResources,
    options: &RenderSettings,
    size: f64,
    small: f64,
    width: f64,
) -> Result<Node<P::Record>, P::Error> {
    let form = form_of(input, &shape)?;
    let cap_w = if form == "h-cyl" { 36. } else { 0. };
    let cap_h = if form == "person" {
        HEAD
    } else if form == "cyl" {
        24.
    } else {
        0.
    };
    let inner = width - 28. - cap_w;
    let kind = format!(
        "«{}»",
        input
            .text(&shape, "kind")?
            .unwrap_or_default()
            .replace('_', " ")
    );
    input.retain_label(Some(&kind))?;
    let kind = layout_label(Some(&kind), false, resources, options, small - 1., inner, 0)?;
    let label = input.text(&shape, "label")?;
    input.retain_label(label.as_deref())?;
    let label = layout_label(label.as_deref(), false, resources, options, size, inner, 1)?;
    let techn = if input.flag(&shape, "techn")? {
        Some(format!(
            "[{}]",
            input.text(&shape, "techn")?.unwrap_or_default()
        ))
    } else {
        None
    };
    input.retain_label(techn.as_deref())?;
    let techn = layout_label(techn.as_deref(), false, resources, options, small, inner, 0)?;
    let descr = input.text(&shape, "descr")?;
    input.retain_label(descr.as_deref())?;
    let descr = layout_label(descr.as_deref(), false, resources, options, small, inner, 0)?;
    let labels = vec![kind, label, techn, descr];
    let mut tw = 0f64;
    let mut text_h = if labels[3].height > 0. { 4. } else { 0. };
    for label in &labels {
        tw = tw.js_max(label.width);
        text_h += label.height
    }
    Ok(Node {
        shape,
        bbox: Box::new(
            identity,
            form,
            width.js_max(tw + 28. + cap_w),
            64f64.js_max(text_h + 24. + cap_h),
        ),
        labels,
        text_h,
    })
}
fn count(
    widths: impl Iterator<Item = f64>,
    limit: f64,
    gap: f64,
    room: Option<f64>,
    pad: f64,
) -> f64 {
    let Some(room) = room else { return limit };
    let widest = widths.fold(1_f64, |a, b| a.js_max(b));
    1_f64.js_max(limit.js_min(((room + gap - 2. * pad) / (widest + gap)).floor()))
}
fn rows(boxes: &mut [&mut Box], per_row: f64, gap_x: f64, gap_y: f64, stretch: bool) -> (f64, f64) {
    // The JS loop enters its first row, but `i < Math.min(n, NaN)`
    // places no boxes. Its next `from += NaN` ends the outer loop.
    if per_row.is_nan() {
        return if boxes.is_empty() {
            (0., 0.)
        } else {
            (0_f64.js_max(-gap_x), gap_y - gap_y)
        };
    }
    let mut width = 0_f64;
    let mut y = 0.;
    let per_row = (per_row.js_min(boxes.len().max(1) as f64)) as usize;
    for row in boxes.chunks_mut(per_row.max(1)) {
        let mut x = 0.;
        let row_h = row.iter().map(|b| b.h).fold(0_f64, |a, b| a.js_max(b));
        for b in row {
            b.x = x;
            b.y = y;
            if stretch {
                b.h = row_h
            }
            x += b.w + gap_x
        }
        width = width.js_max(x - gap_x);
        y += row_h + gap_y
    }
    (width, if boxes.is_empty() { 0. } else { y - gap_y })
}
pub(super) fn layout<P: PhaseInput>(
    input: &mut P,
    resources: &mut dyn RenderResources,
    options: &RenderSettings,
    size: f64,
    small: f64,
    room: Option<f64>,
    gap: f64,
) -> Result<Layout<P::Record>, P::Error> {
    let pad = if room.is_some() { 12. } else { GROUP_PAD };
    let mut groups: Vec<Group<P::Record>> = Vec::new();
    let mut by_identity = HashMap::new();
    let boundaries = input.list(Collection::Boundaries)?;
    let mut i = 0;
    while i < input.len(&boundaries)? {
        input.step()?;
        let boundary = input.at(&boundaries, i)?;
        i += 1;
        let label_room = room.map_or(4000., |room| {
            80f64.js_max(room - 2. * HEAD_LEFT - 2. * GROUP_PAD * 2.)
        });
        let labels = if input.parent(&boundary)?.is_some() {
            let label = input.text(&boundary, "label")?;
            input.retain_label(label.as_deref())?;
            let label = layout_label(
                label.as_deref(),
                false,
                resources,
                options,
                size,
                label_room,
                1,
            )?;
            let typ = if input.flag(&boundary, "type")? {
                Some(format!(
                    "[{}]",
                    input.text(&boundary, "type")?.unwrap_or_default()
                ))
            } else {
                None
            };
            input.retain_label(typ.as_deref())?;
            let typ = layout_label(
                typ.as_deref(),
                false,
                resources,
                options,
                small,
                label_room,
                0,
            )?;
            let descr = input.text(&boundary, "descr")?;
            input.retain_label(descr.as_deref())?;
            let descr = layout_label(
                descr.as_deref(),
                false,
                resources,
                options,
                small,
                label_room,
                0,
            )?;
            vec![label, typ, descr]
        } else {
            Vec::new()
        };
        let (mut head, mut head_w) = (0., 0f64);
        for label in &labels {
            head += label.height;
            head_w = head_w.js_max(label.width)
        }
        let identity = input.identity(&boundary)?;
        let group = Group {
            boundary,
            bbox: Box::new(identity, "rect".into(), 0., 0.),
            kids: Vec::new(),
            nodes: Vec::new(),
            labels,
            head: if head > 0. {
                head + HEAD_TOP + 6. + LANE
            } else {
                pad
            },
            head_w,
            head_x: HEAD_LEFT,
        };
        if let Some(&slot) = by_identity.get(&identity) {
            groups[slot] = group
        } else {
            let slot = groups.len();
            groups.push(group);
            by_identity.insert(identity, slot);
        }
    }
    let current_boundaries = input.list(Collection::Boundaries)?;
    let root = input.at(&current_boundaries, 0)?;
    let root_identity = input.identity(&root)?;
    let root = *by_identity.get(&root_identity).ok_or_else(|| {
        PeleError::new(
            "C4 root boundary is not present",
            crate::compat::ErrorCode::Semantic,
        )
    })?;
    for i in 0..groups.len() {
        if let Some(parent) = input.parent(&groups[i].boundary)? {
            if let Some(&p) = by_identity.get(&parent) {
                groups[p].kids.push(i)
            }
        }
    }
    let mut order = vec![root];
    let mut depths = vec![None; groups.len()];
    depths[root] = Some(0usize);
    let mut i = 0;
    while i < order.len() {
        input.step()?;
        let at = order[i];
        for &kid in &groups[at].kids {
            input.step()?;
            depths[kid] = Some(depths[at].unwrap() + 1);
            order.push(kid)
        }
        i += 1;
    }
    let mut nodes = Vec::new();
    let shapes = input.list(Collection::Shapes)?;
    let mut i = 0;
    while i < input.len(&shapes)? {
        input.step()?;
        let shape = input.at(&shapes, i)?;
        i += 1;
        if input.parent(&shape)?.is_some() {
            let parent = input.parent(&shape)?;
            let depth = parent
                .and_then(|p| by_identity.get(&p))
                .and_then(|p| depths[*p])
                .unwrap_or(0);
            let width = room.map_or(216., |room| {
                100f64.js_max(216f64.js_min(room - 2. * depth as f64 * pad - 32.))
            });
            let parent = input.parent(&shape)?;
            if let Some(&p) = parent.as_ref().and_then(|p| by_identity.get(p)) {
                let identity = input.identity(&shape)?;
                let index = nodes.len();
                nodes.push(measure(
                    input, shape, identity, resources, options, size, small, width,
                )?);
                groups[p].nodes.push(index);
            }
        }
    }
    let owner = input.root();
    let per_row = 1f64.js_max(input.number(&owner, "shapeInRow")?.unwrap_or(f64::NAN));
    let groups_per_row = 1f64.js_max(input.number(&owner, "boundaryInRow")?.unwrap_or(f64::NAN));
    let mut kids_top = vec![0.; groups.len()];
    for &i in order.iter().rev() {
        let gap_x = if room.is_some() { gap } else { 100. };
        let node_ids = &groups[i].nodes;
        let per = count(
            node_ids.iter().map(|&n| nodes[n].bbox.w),
            per_row,
            gap_x,
            room,
            pad,
        );
        let mut node_boxes: Vec<_> = node_ids.iter().map(|&n| nodes[n].bbox.clone()).collect();
        let mut boxes: Vec<_> = node_boxes.iter_mut().collect();
        let (grid_w, grid_h) = rows(&mut boxes, per, gap_x, 88., true);
        for (&n, b) in node_ids.iter().zip(node_boxes) {
            nodes[n].bbox = b
        }
        let kids = groups[i].kids.clone();
        let per = count(
            kids.iter().map(|&n| groups[n].bbox.w),
            groups_per_row,
            48.,
            room,
            pad,
        );
        let mut copies: Vec<Box> = kids.iter().map(|&n| groups[n].bbox.clone()).collect();
        let mut boxes: Vec<_> = copies.iter_mut().collect();
        let (kids_w, kids_h) = rows(&mut boxes, per, 48., 48., false);
        for (&n, b) in kids.iter().zip(copies) {
            groups[n].bbox = b
        }
        let between = if grid_h > 0. && kids_h > 0. { 48. } else { 0. };
        let frame = if i == root { 0. } else { pad };
        kids_top[i] = grid_h + between;
        groups[i].bbox.w = grid_w
            .js_max(kids_w)
            .js_max(groups[i].head_w + 2. * HEAD_LEFT - 2. * frame)
            + 2. * frame;
        groups[i].bbox.h =
            if frame != 0. { groups[i].head } else { 0. } + grid_h + between + kids_h + frame;
    }
    let mut boxes = HashMap::new();
    for &i in &order {
        let left = groups[i].bbox.x + if i == root { 0. } else { pad };
        let top = groups[i].bbox.y + if i == root { 0. } else { groups[i].head };
        for &n in &groups[i].nodes {
            nodes[n].bbox.x += left;
            nodes[n].bbox.y += top;
            boxes.insert(nodes[n].bbox.id, nodes[n].bbox.clone());
        }
        let kids = groups[i].kids.clone();
        for kid in kids {
            groups[kid].bbox.x += left;
            groups[kid].bbox.y += top + kids_top[i]
        }
        if i != root {
            boxes.insert(groups[i].bbox.id, groups[i].bbox.clone());
        }
    }
    Ok(Layout {
        root,
        groups,
        nodes,
        order,
        boxes,
    })
}
