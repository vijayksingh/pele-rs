//! Actual fishbone rows/twigs and narrow outline layout, Pele ishikawa/render.ts (MIT).
use super::IshikawaModel;
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    diagrams::common::literal_label::break_label,
    svg::{document, edges::marker, esc_text, js_num, label_svg, num},
    text::{layout_label, Label},
    PeleError, RenderResources, RenderSettings, Rendered,
};
pub(super) trait PhaseInput {
    type Error: From<PeleError>;
    type Node: Clone;
    type Children;
    fn begin_draw(&mut self) {}
    fn title(&mut self) -> Result<Option<String>, Self::Error>;
    fn root(&mut self, repeat: bool) -> Result<Option<Self::Node>, Self::Error>;
    fn text(&mut self, node: &Self::Node) -> Result<String, Self::Error>;
    fn children(&mut self, node: &Self::Node) -> Result<Self::Children, Self::Error>;
    fn count(&mut self, children: &Self::Children) -> Result<usize, Self::Error>;
    fn has_child(&mut self, children: &Self::Children, index: usize) -> Result<bool, Self::Error>;
    fn child(&mut self, children: &Self::Children, index: usize)
        -> Result<Self::Node, Self::Error>;
}
struct Typed<'a>(&'a IshikawaModel);
impl<'a> PhaseInput for Typed<'a> {
    type Error = PeleError;
    type Node = usize;
    type Children = &'a [usize];
    fn title(&mut self) -> Result<Option<String>, PeleError> {
        Ok(self.0.title.clone())
    }
    fn root(&mut self, _: bool) -> Result<Option<usize>, PeleError> {
        Ok(self.0.root)
    }
    fn text(&mut self, node: &usize) -> Result<String, PeleError> {
        Ok(self.0.nodes[*node].text.clone())
    }
    fn children(&mut self, node: &usize) -> Result<Self::Children, PeleError> {
        Ok(&self.0.nodes[*node].children)
    }
    fn count(&mut self, children: &Self::Children) -> Result<usize, PeleError> {
        Ok(children.len())
    }
    fn has_child(&mut self, _: &Self::Children, _: usize) -> Result<bool, PeleError> {
        Ok(true)
    }
    fn child(&mut self, children: &Self::Children, index: usize) -> Result<usize, PeleError> {
        Ok(children[index])
    }
}
fn work_limit() -> PeleError {
    let mut e = PeleError::new(
        "Ishikawa public operation exceeds native work budget",
        crate::compat::ErrorCode::Limit,
    );
    e.diagram_type = Some("ishikawa".into());
    e
}
struct Bone {
    label: Label,
    depth: usize,
    parent: Option<usize>,
    slot: usize,
    kids: usize,
    rise: f64,
    at: f64,
    text_end: f64,
}
struct Category<N> {
    node: N,
    label: Label,
    box_w: f64,
    box_h: f64,
    bones: Vec<Bone>,
    rise: f64,
    reach: f64,
}
fn arrange<P: PhaseInput>(
    node: P::Node,
    up: bool,
    input: &mut P,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    size: f64,
    small: f64,
    work: &mut usize,
) -> Result<Category<P::Node>, P::Error> {
    *work += 1;
    if *work > 1_000_000 {
        return Err(work_limit().into());
    }
    let node_text = input.text(&node)?;
    let label = layout_label(Some(&node_text), false, r, o, size, 180., 0)?;
    let line_height = round(small * 1.5);
    let mut bones = Vec::<Bone>::new();
    let mut rows = vec![];
    let mut stack = vec![];
    let push = |input: &mut P,
                children: &P::Children,
                depth: usize,
                parent: Option<usize>,
                stack: &mut Vec<(P::Node, usize, Option<usize>, usize, Option<usize>)>|
     -> Result<(), P::Error> {
        let n = input.count(children)?;
        if n > 1_000_000usize.saturating_sub(stack.len()) {
            return Err(work_limit().into());
        }
        for slot in (0..n).rev() {
            stack.push((
                input.child(children, if up { n - 1 - slot } else { slot })?,
                depth,
                parent,
                slot,
                None,
            ));
        }
        Ok(())
    };
    let children = input.children(&node)?;
    push(input, &children, 2, None, &mut stack)?;
    while let Some((node, depth, parent, slot, bone)) = stack.pop() {
        if let Some(bone) = bone {
            rows.push(bone);
            continue;
        }
        *work += 1;
        if *work > 1_000_000 {
            return Err(work_limit().into());
        }
        let index = bones.len();
        let text = input.text(&node)?;
        bones.push(Bone {
            label: layout_label(Some(&text), false, r, o, small, 200., 0)?,
            depth,
            parent,
            slot,
            kids: {
                let children = input.children(&node)?;
                input.count(&children)?
            },
            rise: 0.,
            at: 0.,
            text_end: 0.,
        });
        if depth % 2 == 0 {
            rows.push(index);
        } else {
            stack.push((node.clone(), depth, parent, slot, Some(index)));
        }
        let children = input.children(&node)?;
        push(input, &children, depth + 1, Some(index), &mut stack)?;
    }
    let mut rise = 14.;
    for index in rows {
        let height = bones[index].label.height.js_max(line_height) + 6.;
        bones[index].rise = rise + height / 2.;
        rise += height;
    }
    let box_w = label.width + 20.;
    let mut reach = box_w / 2.;
    for i in 0..bones.len() {
        let at = if bones[i].depth % 2 == 0 {
            bones[i].parent.map_or(0., |parent| bones[parent].at)
        } else {
            let parent = &bones[bones[i].parent.unwrap()];
            parent.at - (parent.kids - bones[i].slot) as f64 * 22.
        };
        let text_end = if bones[i].depth % 2 == 0 {
            at - if bones[i].kids > 0 {
                (bones[i].kids + 1) as f64 * 22.
            } else {
                18.
            } - 5.
        } else {
            at - 5.
        };
        bones[i].at = at;
        bones[i].text_end = text_end;
        reach = reach.js_max(bones[i].label.width - text_end);
    }
    let box_h = label.height + 10.;
    Ok(Category {
        node,
        label,
        box_w,
        box_h,
        bones,
        rise: rise + 8.,
        reach,
    })
}
pub(super) fn render(
    m: &IshikawaModel,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_phases(&mut Typed(m), o, r)
}
pub(super) fn render_phases<P: PhaseInput>(
    input: &mut P,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    let across = horizontal(input, o, r)?;
    let Some(available) = o.max_width.filter(|w| *w > 0.) else {
        return Ok(across);
    };
    if o.auto_direction == Some(false)
        || across.width <= available
        || available >= o.direction_breakpoint.filter(|b| *b > 0.).unwrap_or(640.)
    {
        return Ok(across);
    }
    let down = vertical(input, o, r)?;
    Ok(if down.width < across.width {
        down
    } else {
        across
    })
}
fn horizontal<P: PhaseInput>(
    input: &mut P,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    input.begin_draw();
    let size = o.font_size.unwrap_or(16.);
    let small = round(size * 0.875);
    let pad = o.padding.unwrap_or(8.);
    let root = input.root(false)?;
    let title_text = input.title()?;
    let title = layout_label(title_text.as_deref(), false, r, o, size, 4000., 1)?;
    let title_height = if title.height > 0. {
        title.height + 12.
    } else {
        0.
    };
    let mut body = String::new();
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (0_f64, 0_f64, 0_f64, 0_f64);
    if let Some(root) = root {
        let head_text = input.text(&root)?;
        let head = layout_label(Some(&head_text), false, r, o, size, 200., 1)?;
        let head_h = (head.height + 24.).js_max(48.);
        let head_w = head.width + 24. + head_h / 2.;
        let children = input.children(&root)?;
        let count = input.count(&children)?;
        let mut categories = Vec::with_capacity(count);
        let mut work = 1usize;
        let mut absent = false;
        for i in 0..count {
            if !input.has_child(&children, i)? {
                absent = true;
                continue;
            }
            let node = input.child(&children, i)?;
            categories.push(arrange(
                node,
                i % 2 == 0,
                input,
                o,
                r,
                size,
                small,
                &mut work,
            )?);
        }
        if absent {
            return Err(PeleError::new(
                "Ishikawa category arrays must not contain holes",
                crate::compat::ErrorCode::Semantic,
            )
            .into());
        }
        for c in &mut categories {
            c.rise = c.rise.js_max(head_h / 2. + 14.);
        }
        let mut xs = vec![];
        let mut x = -34.;
        for i in (0..categories.len()).step_by(2) {
            if i > 0 {
                let mut room = 0_f64;
                for side in 0..2 {
                    if let Some(before) = categories.get(i - 2 + side) {
                        let next = categories.get(i + side);
                        room = room.js_max(before.reach + next.map_or(0., |next| next.box_w / 2.));
                    }
                }
                x -= room + 26.;
            }
            xs.extend([x, x]);
        }
        let tail = if categories.is_empty() { -34. } else { x - 28. };
        max_x = head_w;
        min_x = tail;
        min_y = -head_h / 2.;
        max_y = head_h / 2.;
        let mut groups = String::new();
        let length = 0.5_f64.hypot(1.);
        for (i, c) in categories.iter().enumerate() {
            let sign = if i % 2 == 0 { -1. } else { 1. };
            let ax = xs[i];
            let px = |at: f64, rise: f64| ax + at - rise * 0.5;
            let end_x = px(0., c.rise);
            let end_y = sign * c.rise;
            let trim = 7.;
            let mut d = format!(
                "M{},{}L{},{}",
                num(ax - (0.5 / length) * trim),
                num(sign * trim / length),
                num(end_x),
                num(end_y)
            );
            let mut texts = String::new();
            for bone in &c.bones {
                let y = sign * bone.rise;
                if bone.depth % 2 == 0 {
                    d += &format!(
                        "M{},{}H{}",
                        num(px(bone.at, bone.rise)),
                        num(y),
                        num(px(bone.text_end + 5., bone.rise))
                    );
                } else {
                    let from = c.bones[bone.parent.unwrap()].rise;
                    d += &format!(
                        "M{},{}L{},{}",
                        num(px(bone.at, from)),
                        num(sign * from),
                        num(px(bone.at, bone.rise)),
                        num(y)
                    );
                }
                let right = px(bone.text_end, bone.rise);
                texts += &label_svg(
                    &bone.label,
                    right - bone.label.width / 2.,
                    y,
                    if bone.depth > 2 {
                        " fill=\"var(--_m)\""
                    } else {
                        ""
                    },
                );
                min_x = min_x.js_min(right - bone.label.width);
            }
            let box_y = end_y + sign * (c.box_h / 2.);
            min_x = min_x.js_min(end_x - c.box_w / 2.);
            max_x = max_x.js_max(end_x + c.box_w / 2.);
            min_y = min_y.js_min(box_y - c.box_h / 2.);
            max_y = max_y.js_max(box_y + c.box_h / 2.);
            groups+=&format!("<g class=\"pele-ishikawa-category\" data-id=\"{}\"><path class=\"pele-ishikawa-bones\" d=\"{d}\" fill=\"none\" stroke=\"var(--_l)\"/>{}<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"var(--pele-radius,4px)\" fill=\"var(--_s)\" stroke=\"var(--_b)\"/>{}{}</g>",esc_text(&input.text(&c.node)?)?,marker("arrow_point",ax,0.,0.5/length,-sign/length,"var(--_l)"),num(end_x-c.box_w/2.),num(box_y-c.box_h/2.),num(c.box_w),num(c.box_h),label_svg(&c.label,end_x,box_y," class=\"pele-label\""),if texts.is_empty(){String::new()}else{format!("<g class=\"pele-ishikawa-causes\" font-size=\"{}\">{texts}</g>",js_num(small))});
        }
        let radius = head_h / 2.;
        body=format!("<path class=\"pele-ishikawa-spine\" d=\"M{},0H-7\" fill=\"none\" stroke=\"var(--_l)\"/>{}{groups}<g class=\"pele-ishikawa-head\" data-id=\"{}\"><path d=\"M0,{}H{}A{},{} 0 0 1 {},{}H0Z\" fill=\"var(--_s)\" stroke=\"var(--_b)\" stroke-linejoin=\"round\"/>{}</g>",num(tail),marker("arrow_point",0.,0.,1.,0.,"var(--_l)"),esc_text(&input.text(&root)?)?,num(-radius),num(head_w-radius),num(radius),num(radius),num(head_w-radius),num(radius),label_svg(&head,12.+head.width/2.,0.," class=\"pele-label\" font-weight=\"var(--_hw)\""));
    }
    let inner = (max_x - min_x).js_max(title.width);
    let width = (inner + 2. * pad).ceil();
    let height = (max_y - min_y + title_height + 2. * pad).ceil();
    let ox = pad + (inner - (max_x - min_x)) / 2. - min_x;
    let oy = pad + title_height - min_y;
    let content = format!(
        "{}{}",
        label_svg(
            &title,
            width / 2.,
            pad + title.height / 2.,
            " class=\"pele-title\" font-weight=\"var(--_tw)\""
        ),
        if body.is_empty() {
            String::new()
        } else {
            format!(
                "<g transform=\"translate({},{})\">{body}</g>",
                num(ox),
                num(oy)
            )
        }
    );
    Ok(Rendered {
        svg: document("ishikawa", width, height, size, o, None, None, &content)?,
        width,
        height,
        links: vec![],
    })
}
struct Entry<N> {
    node: N,
    depth: usize,
    parent: Option<usize>,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}
fn vertical<P: PhaseInput>(
    input: &mut P,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    input.begin_draw();
    let size = o.font_size.unwrap_or(16.);
    let small = round(size * 0.875);
    let pad = o.padding.unwrap_or(8.);
    let room = 140_f64.js_max(o.max_width.unwrap_or(320.) - 2. * pad);
    let title_room = o
        .max_width
        .filter(|w| *w > 0.)
        .map_or(4000., |w| 80_f64.js_max(w - 2. * pad));
    let title_text = input.title()?;
    let title = layout_label(title_text.as_deref(), false, r, o, size, title_room, 1)?;
    let mut order = vec![];
    let mut pending = input
        .root(true)?
        .map_or(vec![], |node| vec![(node, 0usize, None)]);
    let mut max_depth = 0;
    while let Some((node, depth, parent)) = pending.pop() {
        if order.len() >= 1_000_000 {
            return Err(work_limit().into());
        }
        let index = order.len();
        order.push(Entry {
            node: node.clone(),
            depth,
            parent,
            x: 0.,
            y: 0.,
            w: 0.,
            h: 0.,
        });
        max_depth = max_depth.max(depth);
        let children = input.children(&node)?;
        let count = input.count(&children)?;
        if count > 1_000_000usize.saturating_sub(pending.len()) {
            return Err(work_limit().into());
        }
        for i in (0..count).rev() {
            let children = input.children(&node)?;
            let child = input.child(&children, i)?;
            pending.push((child, depth + 1, Some(index)));
        }
    }
    let indent = 20_f64.js_min(2_f64.js_max((room - 120.) / max_depth.max(1) as f64));
    let mut y = pad
        + if title.height != 0. {
            title.height + 12.
        } else {
            0.
        };
    let mut width = title.width;
    let mut nodes = String::new();
    let mut edges = String::new();
    for i in 0..order.len() {
        let entry = &mut order[i];
        entry.x = pad + entry.depth as f64 * indent;
        let label_room = 60_f64.js_max(room - entry.depth as f64 * indent - 20.);
        let text = input.text(&entry.node)?;
        let label = layout_label(
            Some(&text),
            false,
            r,
            o,
            if entry.depth < 2 { size } else { small },
            label_room,
            if entry.depth < 2 { 1 } else { 0 },
        )?;
        let label = break_label(label, r, o, label_room)?;
        entry.w = label.width + 20.;
        entry.h = label.height + 12.;
        entry.y = y;
        y += entry.h + 12.;
        width = width.js_max(entry.x - pad + entry.w);
        let shape = if entry.depth < 2 {
            format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"var(--_s)\" stroke=\"var(--_b)\"/>",num(entry.x),num(entry.y),num(entry.w),num(entry.h),if entry.depth==0{12}else{4})
        } else {
            String::new()
        };
        nodes += &format!(
            "<g class=\"pele-node {}\" data-id=\"{}\">{shape}{}</g>",
            if entry.depth == 0 {
                "pele-ishikawa-head"
            } else if entry.depth == 1 {
                "pele-ishikawa-category"
            } else {
                "pele-ishikawa-cause"
            },
            esc_text(&input.text(&entry.node)?)?,
            label_svg(
                &label,
                entry.x + entry.w / 2.,
                entry.y + entry.h / 2.,
                &format!(
                    " class=\"pele-label\"{}{}",
                    if entry.depth < 2 {
                        " font-weight=\"var(--_hw)\""
                    } else {
                        ""
                    },
                    if entry.depth > 1 {
                        format!(" font-size=\"{}\"", js_num(small))
                    } else {
                        String::new()
                    }
                )
            )
        );
        if let Some(parent) = entry.parent {
            let (ex, ey, eh) = (entry.x, entry.y, entry.h);
            let parent = &order[parent];
            let gx = parent.x + indent / 2.;
            edges += &format!(
                "<path class=\"pele-edge\" d=\"M{},{}V{}H{}\" fill=\"none\" stroke=\"var(--_b)\"/>",
                num(gx),
                num(parent.y + parent.h),
                num(ey + eh / 2.),
                num(ex)
            );
        }
    }
    let total_width = (width + 2. * pad).ceil();
    let height = ((pad + title.height).js_max(y - 12.) + pad).ceil();
    let content = format!(
        "{}{edges}{nodes}",
        label_svg(
            &title,
            total_width / 2.,
            pad + title.height / 2.,
            " class=\"pele-title\" font-weight=\"var(--_tw)\""
        )
    );
    Ok(Rendered {
        svg: document(
            "ishikawa",
            total_width,
            height,
            size,
            o,
            None,
            None,
            &content,
        )?,
        width: total_width,
        height,
        links: vec![],
    })
}
