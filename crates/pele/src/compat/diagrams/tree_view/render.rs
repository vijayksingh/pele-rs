//! Dedicated TreeView row/guide renderer, Pele treeview/render.ts, 458322a5 (MIT).
use super::{icons, NodeType, TreeViewModel};
use crate::compat::number_math::{round, NumberMath};
use crate::compat::ErrorCode;
use crate::compat::{
    diagrams::common::literal_label::{left_label_svg, literal_label},
    source::{decode_entities, space},
    svg::{document, edges::theme::class_names, esc, esc_text, label_svg, num},
    text::{layout_label, measure, Label},
    Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
use std::collections::HashMap;
fn setting(config: &Object, key: &str, fallback: f64, min: f64, max: f64) -> f64 {
    match config
        .get("treeView")
        .and_then(Value::as_object)
        .and_then(|c| c.get(key))
    {
        Some(Value::Number(v)) if *v >= min && *v <= max => *v,
        _ => fallback,
    }
}
struct Row<N> {
    node: N,
    x: f64,
    cy: f64,
    icon: Option<String>,
    text: String,
    label_x: f64,
    highlighted: bool,
    label: Option<Label>,
    description: Option<Label>,
    top: f64,
    height: f64,
}
pub(super) fn render(
    m: &TreeViewModel,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_phases(&mut TypedPhase(m), c, o, r)
}
pub(super) fn render_phases<P: PhaseInput>(
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
fn draw<P: PhaseInput>(
    m: &mut P,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    tight: bool,
) -> Result<Rendered, P::Error> {
    let size = o.font_size.unwrap_or(16.);
    let pad = o.padding.unwrap_or(8.);
    let icons = icons::Config::new(c);
    let padding_x = setting(c, "paddingX", 5., 0., 100.);
    let padding_y = setting(c, "paddingY", 2., 0., 100.);
    let row_indent = setting(c, "rowIndent", 15., 0., 400.);
    let thickness = setting(c, "lineThickness", 1., 0.25, 12.);
    let mut max_depth = 0;
    let root_children = m.root_children()?;
    let mut depths = Vec::new();
    for i in 0..m.length(&root_children)? {
        if m.has_index(&root_children, i)? {
            depths.push((m.index(&root_children, i)?, 0usize));
        }
    }
    let mut i = 0;
    while tight && i < depths.len() {
        let (node, depth) = depths[i].clone();
        max_depth = max_depth.max(depth);
        let children = m.children(&node)?;
        let mut j = 0;
        while j < m.length(&children)? {
            reserve::<P::Error>(depths.len())?;
            depths.push((m.index(&children, j)?, depth + 1));
            j += 1;
        }
        i += 1;
    }
    let step = if tight {
        (row_indent + padding_x)
            .js_min(2_f64.js_max((o.max_width.unwrap_or(320.) - 120.) / max_depth.max(1) as f64))
    } else {
        row_indent + padding_x
    };
    let row_height = round(size * 1.5) + 2. * padding_y;
    let room = if tight {
        o.max_width
            .filter(|v| *v > 0.)
            .map_or(4000., |v| 80_f64.js_max(v - 2. * pad))
    } else {
        4000.
    };
    let title = m.title()?;
    let title = layout_label(title.as_deref(), false, r, o, size, room, 1)?;
    let title_height = if title.height > 0. {
        title.height + 12.
    } else {
        0.
    };
    let mut rows = vec![];
    let mut row_of = HashMap::new();
    let mut pending = vec![];
    let children = m.root_children()?;
    push(m, &children, &icons, 0, &mut pending)?;
    let mut label_right = 0.;
    let mut any_highlight = false;
    let mut rows_height = 0.;
    let mut retained_text = 0usize;
    while let Some((node, depth, slot)) = pending.pop() {
        reserve::<P::Error>(rows.len())?;
        let x = depth as f64 * step;
        let text = decode_entities(&m.name(&node)?)?;
        retain_text(&mut retained_text, &text)?;
        let label_x = x + if slot { size + 6. } else { 0. };
        let room = 40_f64.js_max(o.max_width.unwrap_or(320.) - 2. * pad - 8. - label_x);
        let label = if tight {
            Some(literal_label(
                &text,
                r,
                o,
                size,
                room,
                if m.directory(&node)? { 1 } else { 0 },
            )?)
        } else {
            None
        };
        let description = if tight {
            if let Some(s) = m.description(&node, true)? {
                let s = decode_entities(&s)?;
                retain_text(&mut retained_text, &s)?;
                Some(literal_label(&s, r, o, size, room, 2)?)
            } else {
                None
            }
        } else {
            None
        };
        let width = if let Some(label) = &label {
            label.width
        } else {
            measure(r, o, &text, size, if m.directory(&node)? { 1 } else { 0 })?
        };
        if label_x + width > label_right {
            label_right = label_x + width;
        }
        let highlighted = m
            .css_class(&node)?
            .is_some_and(|class| class.split(space).any(|name| name == "highlight"));
        if highlighted {
            any_highlight = true;
        }
        let label_height = label.as_ref().map_or(0., |l| l.height);
        let height = if tight {
            row_height.js_max(
                label_height + description.as_ref().map_or(0., |l| l.height) + 2. * padding_y,
            )
        } else {
            row_height
        };
        let cy = rows_height
            + if tight {
                padding_y + size.js_max(label_height) / 2.
            } else {
                height / 2.
            };
        let icon = icons::get_from(m, &node, &icons)?;
        if let Some(icon) = &icon {
            retain_text(&mut retained_text, icon)?;
        }
        let row = Row {
            node: node.clone(),
            x,
            cy,
            icon,
            text,
            label_x,
            highlighted,
            label,
            description,
            top: rows_height,
            height,
        };
        rows_height += height;
        row_of.insert(m.identity(&node)?, rows.len());
        rows.push(row);
        let children = m.children(&node)?;
        push(m, &children, &icons, depth + 1, &mut pending)?;
    }
    let desc_x = label_right + 16.;
    let mut width = label_right;
    for row in &rows {
        let Some(description) = m.description(&row.node, false)? else {
            continue;
        };
        let w = if let Some(label) = &row.description {
            label.width
        } else {
            measure(r, o, &decode_entities(&description)?, size, 2)?
        };
        width = width.js_max(if tight { row.label_x } else { desc_x } + w);
    }
    width = width.js_max(title.width);
    let inset = pad + if any_highlight { 4. } else { 0. };
    let top = pad + title_height;
    let total_width = (width + 2. * inset).ceil();
    let total_height = (top + rows_height + pad).ceil();
    let mut guides = String::new();
    for row in &rows {
        let children = m.children(&row.node)?;
        if m.length(&children)? == 0 {
            continue;
        }
        let gx = inset + row.x + 7.;
        let tick = gx.js_max(inset + row.x + step - padding_x);
        let count = m.length(&children)?;
        let last_child = m.index(&children, count - 1)?;
        let last_row = lookup(m, &last_child, &row_of)?;
        let last = top + rows[last_row].cy;
        let radius = 4_f64.js_min(tick - gx);
        let mut d = format!(
            "M{},{}V{}",
            num(gx),
            num(top + row.top + row.height - 2.),
            num(last - radius)
        );
        if radius > 0. {
            d += &format!(
                "Q{},{} {},{}H{}",
                num(gx),
                num(last),
                num(gx + radius),
                num(last),
                num(tick)
            );
        }
        if tick > gx {
            let mut child_index = 0;
            while child_index + 1 < m.length(&children)? {
                let child = m.index(&children, child_index)?;
                let child_row = lookup(m, &child, &row_of)?;
                d += &format!(
                    "M{},{}H{}",
                    num(gx),
                    num(top + rows[child_row].cy),
                    num(tick)
                );
                child_index += 1;
            }
        }
        guides += &format!("<path d=\"{d}\"/>");
    }
    let baseline = size * 0.35;
    let mut body = String::new();
    for row in &rows {
        let node = &row.node;
        let cy = top + row.cy;
        let directory = m.directory(node)?;
        let mut inner = String::new();
        if row.highlighted {
            let x = inset + row.x - 4.;
            inner+=&format!("<rect class=\"pele-tree-highlight\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"var(--pele-radius,4px)\" fill=\"var(--_c)\" fill-opacity=\"0.14\"/>",num(x),num(top+row.top+1.),num(total_width-inset+4.-x),num(row.height-2.));
        }
        if let Some(icon) = &row.icon {
            let drawn = r
                .direct_icon(icon)?
                .map(|v| v.markup().to_string())
                .unwrap_or_else(|| {
                    format!(
                        "<path d=\"{}\" stroke=\"var(--_m)\" stroke-width=\"1.5\"/>",
                        icons::glyph(icon, directory)
                    )
                });
            inner+=&format!("<svg class=\"pele-icon\" data-icon=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{drawn}</svg>",esc_text(icon)?,num(inset+row.x),num(cy-size/2.),num(size),num(size));
        }
        let bold = if directory {
            " font-weight=\"var(--_hw)\""
        } else {
            ""
        };
        if let Some(label) = &row.label {
            inner += &left_label_svg(
                label,
                inset + row.label_x,
                top + row.top + padding_y,
                &format!(" class=\"pele-label\"{bold}"),
            );
        } else if !row.text.is_empty() {
            inner+=&format!("<text class=\"pele-label\" x=\"{}\" y=\"{}\"{bold} xml:space=\"preserve\">{}</text>",num(inset+row.label_x),num(cy+baseline),esc(&row.text));
        }
        if let Some(description) = &row.description {
            inner += &left_label_svg(
                description,
                inset + row.label_x,
                top + row.top + padding_y + row.label.as_ref().map_or(0., |l| l.height),
                " class=\"pele-tree-description\" font-style=\"italic\" fill=\"var(--_m)\"",
            );
        } else if let Some(description) = m.description(node, true)? {
            inner+=&format!("<text class=\"pele-tree-description\" x=\"{}\" y=\"{}\" font-style=\"italic\" fill=\"var(--_m)\" xml:space=\"preserve\">{}</text>",num(inset+desc_x),num(cy+baseline),esc(&decode_entities(&description)?));
        }
        body += &format!(
            "<g class=\"pele-node pele-tree-{}{}\" data-id=\"{}\">{inner}</g>",
            if directory { "dir" } else { "file" },
            class_names(m.css_class(node)?.as_deref().unwrap_or("")),
            esc_text(&m.name(node)?)?
        );
    }
    let content = format!(
        "{}{}{body}",
        label_svg(
            &title,
            total_width / 2.,
            pad + title.height / 2.,
            " class=\"pele-title\" font-weight=\"var(--_tw)\""
        ),
        if guides.is_empty() {
            String::new()
        } else {
            format!("<g class=\"pele-tree-guides\" fill=\"none\" stroke=\"var(--_b)\" stroke-width=\"{}\">{guides}</g>",num(thickness))
        }
    );
    let acc_title = m.accessibility(false)?;
    let acc_descr = m.accessibility(true)?;
    Ok(Rendered {
        svg: document(
            "treeView",
            total_width,
            total_height,
            size,
            o,
            acc_title.as_deref(),
            acc_descr.as_deref(),
            &content,
        )?,
        width: total_width,
        height: total_height,
        links: vec![],
    })
}

pub(super) trait PhaseInput: icons::IconInput {
    type Children;
    fn root_children(&mut self) -> Result<Self::Children, Self::Error>;
    fn children(&mut self, node: &Self::Node) -> Result<Self::Children, Self::Error>;
    fn length(&mut self, children: &Self::Children) -> Result<usize, Self::Error>;
    fn has_index(&mut self, children: &Self::Children, index: usize) -> Result<bool, Self::Error>;
    fn index(&mut self, children: &Self::Children, index: usize)
        -> Result<Self::Node, Self::Error>;
    fn identity(&mut self, node: &Self::Node) -> Result<u64, Self::Error>;
    fn title(&mut self) -> Result<Option<String>, Self::Error>;
    fn description(
        &mut self,
        node: &Self::Node,
        reread: bool,
    ) -> Result<Option<String>, Self::Error>;
    fn css_class(&mut self, node: &Self::Node) -> Result<Option<String>, Self::Error>;
    fn accessibility(&mut self, description: bool) -> Result<Option<String>, Self::Error>;
}
fn limit() -> PeleError {
    PeleError::new(
        "TreeView public graph exceeds native work budget",
        ErrorCode::Limit,
    )
}
// Repeated handles can expand into many owned rows. Charge each retained
// occurrence, independently from the host reader's per-object read budget.
fn retain_text(total: &mut usize, text: &str) -> Result<(), PeleError> {
    *total = total.saturating_add(text.encode_utf16().count());
    if *total > 16_000_000 {
        Err(limit())
    } else {
        Ok(())
    }
}
fn reserve<E: From<PeleError>>(count: usize) -> Result<(), E> {
    if count >= 1_000_000 {
        Err(limit().into())
    } else {
        Ok(())
    }
}
fn push<P: PhaseInput>(
    m: &mut P,
    children: &P::Children,
    icons: &icons::Config<'_>,
    depth: usize,
    pending: &mut Vec<(P::Node, usize, bool)>,
) -> Result<(), P::Error> {
    let count = m.length(children)?;
    let mut slot = false;
    for i in 0..count {
        if m.has_index(children, i)? {
            let node = m.index(children, i)?;
            if icons::get_from(m, &node, icons)?.is_some() {
                slot = true;
                break;
            }
        }
    }
    let count = m.length(children)?;
    if count > 1_000_000usize.saturating_sub(pending.len()) {
        return Err(limit().into());
    }
    for i in (0..count).rev() {
        pending.push((m.index(children, i)?, depth, slot));
    }
    Ok(())
}
fn lookup<P: PhaseInput>(
    m: &mut P,
    node: &P::Node,
    rows: &HashMap<u64, usize>,
) -> Result<usize, P::Error> {
    rows.get(&m.identity(node)?).copied().ok_or_else(|| {
        PeleError::new(
            "TreeView child was not visited during layout",
            ErrorCode::Semantic,
        )
        .into()
    })
}
struct TypedPhase<'a>(&'a TreeViewModel);
impl icons::IconInput for TypedPhase<'_> {
    type Node = usize;
    type Error = PeleError;
    fn icon(&mut self, n: &usize) -> Result<Option<String>, PeleError> {
        Ok(self.0.nodes[*n].icon.clone())
    }
    fn name(&mut self, n: &usize) -> Result<String, PeleError> {
        Ok(self.0.nodes[*n].name.clone())
    }
    fn file(&mut self, n: &usize) -> Result<bool, PeleError> {
        Ok(self.0.nodes[*n].node_type == NodeType::File)
    }
    fn directory(&mut self, n: &usize) -> Result<bool, PeleError> {
        Ok(self.0.nodes[*n].node_type == NodeType::Directory)
    }
}
impl PhaseInput for TypedPhase<'_> {
    type Children = usize;
    fn root_children(&mut self) -> Result<usize, PeleError> {
        Ok(self.0.root)
    }
    fn children(&mut self, n: &usize) -> Result<usize, PeleError> {
        Ok(*n)
    }
    fn length(&mut self, n: &usize) -> Result<usize, PeleError> {
        Ok(self.0.nodes[*n].children.len())
    }
    fn has_index(&mut self, n: &usize, i: usize) -> Result<bool, PeleError> {
        Ok(i < self.0.nodes[*n].children.len())
    }
    fn index(&mut self, n: &usize, i: usize) -> Result<usize, PeleError> {
        Ok(self.0.nodes[*n].children[i])
    }
    fn identity(&mut self, n: &usize) -> Result<u64, PeleError> {
        Ok(*n as u64)
    }
    fn title(&mut self) -> Result<Option<String>, PeleError> {
        Ok(self.0.title.clone())
    }
    fn description(&mut self, n: &usize, _: bool) -> Result<Option<String>, PeleError> {
        Ok(self.0.nodes[*n]
            .description
            .clone()
            .filter(|s| !s.is_empty()))
    }
    fn css_class(&mut self, n: &usize) -> Result<Option<String>, PeleError> {
        Ok(self.0.nodes[*n].css_class.clone())
    }
    fn accessibility(&mut self, description: bool) -> Result<Option<String>, PeleError> {
        Ok(if description {
            self.0.acc_descr.clone()
        } else {
            self.0.acc_title.clone()
        })
    }
}
