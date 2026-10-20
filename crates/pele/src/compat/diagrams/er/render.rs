//! Direct Pele er/render.ts entity tables and relationships, 458322a5 (MIT, kepano).
use super::super::common::generics::parse_generic_types;
use super::{
    graph::Node,
    live_read::{CellField, ErRead, TypedRead},
    markers::{marker, MARKER_LENGTH},
    model::ErModel,
};
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    layout::{
        compound::{compound_layout, direction, shift_layout, CEdge, CNode, Dir},
        layered::LayeredOptions,
    },
    source::{
        decode_entities,
        url::{link_url, rel_attr, sanitize_url},
    },
    svg::{
        self,
        edges::{
            cluster::{cluster_title_x, mark_crossings, ClusterBox, Crossings},
            edge_label_svg, route_path,
            theme::{class_names, resolve_style, ResolvedStyle},
        },
        esc, esc_text, js_num, label_svg, label_svg_with_resources, num,
    },
    text::{layout_label, Label},
    LinkInfo, Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
use std::collections::HashMap;
const GROUP_PAD: f64 = 20.;
const LOOP: f64 = 28.;
const LOOP_BAND: f64 = 38.;
const MIN_WIDTH: f64 = 96.;
const BOX_PAD_X: f64 = 18.;
const BOX_PAD_Y: f64 = 12.;
const HEAD_PAD_Y: f64 = 9.;
const CELL_PAD_X: f64 = 12.;
const CELL_GAP: f64 = 16.;
const ROW_PAD: f64 = 9.;
const RADIUS: &str = "var(--pele-radius,4px)";
struct Row {
    cells: [Label; 4],
    h: f64,
}
struct View<A: Clone, S: Clone, V: Clone> {
    node: Node<A, S, V>,
    style: ResolvedStyle,
    label: Label,
    w: f64,
    h: f64,
    head_h: f64,
    rows: Vec<Row>,
    cols: Vec<f64>,
    row_size: f64,
    key_size: f64,
    below: bool,
}
struct Loop {
    label: Label,
    band: f64,
    at: f64,
}
fn number_option(config: &Object, key: &str, fallback: f64) -> f64 {
    match config
        .get("er")
        .and_then(Value::as_object)
        .and_then(|o| o.get(key))
    {
        Some(Value::Number(n)) if *n > 0. => n.js_min(2000.),
        _ => fallback,
    }
}
fn cell_text(raw: &str) -> String {
    let converted = parse_generic_types(raw);
    if converted == raw {
        converted
    } else {
        converted.replace('<', "&lt;").replace('>', "&gt;")
    }
}
fn lab(
    s: &str,
    size: f64,
    max: f64,
    base: u8,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Label, PeleError> {
    layout_label(Some(s), true, resources, options, size, max, base)
}
fn span_style(s: u8) -> String {
    format!(
        "{}{}",
        if s & 1 != 0 {
            " font-weight=\"var(--_w)\""
        } else {
            ""
        },
        if s & 2 != 0 {
            " font-style=\"italic\""
        } else {
            ""
        }
    )
}
fn text_left(
    label: &Label,
    x: f64,
    mut y: f64,
    attrs: &str,
    resources: &mut dyn RenderResources,
) -> Result<String, PeleError> {
    if label.lines.is_empty() {
        return Ok(String::new());
    }
    if label.lines.len() == 1
        && label.lines[0].len() == 1
        && label.lines[0][0].icon.is_none()
        && label.lines[0][0].style == 0
    {
        return Ok(format!(
            "<text{attrs} x=\"{}\" y=\"{}\">{}</text>",
            num(x),
            num(y),
            esc(&label.lines[0][0].text)
        ));
    }
    let mut shared = 3;
    let mut any = false;
    for s in label.lines.iter().flatten() {
        if s.icon.is_some() || s.text.is_empty() {
            continue;
        }
        shared &= s.style;
        any = true
    }
    if !any {
        shared = 0
    }
    let mut body = String::new();
    let mut extra = String::new();
    for line in &label.lines {
        let has_icon = line.iter().any(|s| s.icon.is_some());
        let mut sx = x;
        let mut placed = false;
        for span in line {
            if let Some(icon) = &span.icon {
                let side = label.size * 1.1;
                let markup = resources
                    .direct_icon(icon)?
                    .map(|s| s.markup().to_owned())
                    .unwrap_or_default();
                extra.push_str(&format!("<svg class=\"pele-icon\" data-icon=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{markup}</svg>",esc_text(icon)?,num(sx),num(y-label.size*0.35-side/2.),num(side),num(side)))
            } else if !span.text.is_empty() {
                let pos = if has_icon || !placed {
                    format!(" x=\"{}\" y=\"{}\"", num(sx), num(y))
                } else {
                    String::new()
                };
                body.push_str(&format!(
                    "<tspan{pos}{}>{}</tspan>",
                    span_style(span.style & !shared),
                    esc(&span.text)
                ));
                placed = true
            }
            sx += span.width;
        }
        y += label.line_height;
    }
    let attrs = format!(
        "{attrs}{}{}",
        if shared & 1 != 0 && !attrs.contains(" font-weight=\"") {
            " font-weight=\"var(--_w)\""
        } else {
            ""
        },
        if shared & 2 != 0 && !attrs.contains(" font-style=\"") {
            " font-style=\"italic\""
        } else {
            ""
        }
    );
    Ok(format!(
        "<text{attrs} xml:space=\"preserve\">{body}</text>{extra}"
    ))
}
pub(super) fn render(
    db: &ErModel,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_read(&TypedRead(db), config, options, resources)
}
pub(super) fn render_read<R: ErRead>(
    input: &R,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    // yt chooses the auto-direction wrapper before any callback in either draw.
    let horizontal = matches!(input.direction()?.as_ref(), "LR" | "RL");
    let selected = (|| -> Result<_, PeleError> {
        let across = draw(input, config, options, resources, false)?;
        if !horizontal {
            return Ok(across);
        }
        let Some(available) = options.max_width.filter(|n| *n > 0.) else {
            return Ok(across);
        };
        if options.auto_direction == Some(false)
            || across.0.width <= available
            || available
                >= options
                    .direction_breakpoint
                    .filter(|n| *n > 0.)
                    .unwrap_or(640.)
        {
            return Ok(across);
        }
        let down = draw(input, config, options, resources, true)?;
        Ok(if down.0.width < across.0.width {
            down
        } else {
            across
        })
    })()?;
    if let Some(frame) = selected.1 {
        resources.capture_presentation(frame);
    }
    Ok(selected.0)
}
fn draw<R: ErRead>(
    input: &R,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
    turned: bool,
) -> Result<(Rendered, Option<crate::compat::PresentationOutcome>), PeleError> {
    let (nodes, edges) = input.capture()?;
    let size = options.font_size.unwrap_or(16.);
    let small = round(size * 0.875);
    let pad = options.padding.unwrap_or(8.);
    let mut index = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        index.insert(input.key(&node.id)?.into_owned(), i);
    }
    let mut views: Vec<View<R::Attributes, R::Styles, R::Scalar>> = vec![];
    let mut cnodes = vec![];
    let mut cedges = vec![];
    let mut labels = vec![];
    let mut drawn = vec![];
    let mut loops: Vec<Loop> = vec![];
    let mut loops_by_node: Vec<Vec<usize>> = vec![vec![]; nodes.len()];
    let mut loop_of = vec![];
    for edge in &edges {
        let (Some(&src), Some(&dst)) = (
            index.get(input.key(&edge.start)?.as_ref()),
            index.get(input.key(&edge.end)?.as_ref()),
        ) else {
            continue;
        };
        let edge_text = input.label(&edge.label)?;
        let label = layout_label(
            edge_text.as_deref(),
            true,
            resources,
            options,
            small,
            200.,
            0,
        )?;
        let lp = if src == dst {
            let i = loops.len();
            loops.push(Loop {
                label: label.clone(),
                band: 0.,
                at: 0.,
            });
            loops_by_node[src].push(i);
            Some(i)
        } else {
            None
        };
        loop_of.push(lp);
        drawn.push(edge);
        cedges.push(CEdge {
            src,
            dst,
            minlen: 1.,
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
    let horizontal = |i: usize| -> Result<bool, PeleError> {
        let mut at = i;
        for _ in 0..nodes.len() {
            let Some(parent_id) = &nodes[at].parent_id else {
                break;
            };
            let Some(&up) = index.get(input.key(parent_id)?.as_ref()) else {
                break;
            };
            if let Some(dir) = &nodes[up].dir {
                if input.truthy(dir)? {
                    return Ok(matches!(input.scalar_direction(dir)?.as_ref(), "LR" | "RL"));
                }
            }
            at = up;
        }
        Ok(matches!(input.direction()?.as_ref(), "LR" | "RL"))
    };
    for (i, node) in nodes.iter().enumerate() {
        let mut styles = node.css_compiled_styles.clone();
        match input.styles(&node.css_styles)? {
            std::borrow::Cow::Borrowed(values) => styles.extend_from_slice(values),
            std::borrow::Cow::Owned(values) => styles.extend(values),
        }
        let style = resolve_style(&styles);
        let base = u8::from(style.bold) | if style.italic { 2 } else { 0 };
        let name_size = style.font_size.unwrap_or(size);
        let row_size = style.font_size.unwrap_or(small);
        let key_size = style.font_size.unwrap_or(round(size * 0.75));
        let label_value = if node.is_group || !input.truthy(&node.alias)? {
            &node.label
        } else {
            &node.alias
        };
        let label_text = input.label(label_value)?;
        let label = layout_label(
            label_text.as_deref(),
            true,
            resources,
            options,
            name_size,
            if node.is_group { 4000. } else { 240. },
            base,
        )?;
        let own = &loops_by_node[i];
        let below = !own.is_empty() && !node.is_group && horizontal(i)?;
        let mut view = View {
            node: node.clone(),
            style,
            label,
            w: 0.,
            h: 0.,
            head_h: 0.,
            rows: vec![],
            cols: vec![],
            row_size,
            key_size,
            below,
        };
        let mut reach: f64 = 0.;
        let mut bands = 0.;
        for &l in own {
            let lp = &mut loops[l];
            lp.band = LOOP_BAND.js_max(if below {
                lp.label.width + 12.
            } else {
                lp.label.height + 6.
            });
            bands += lp.band;
            reach = reach.js_max(if below {
                lp.label.height
            } else {
                lp.label.width
            });
        }
        let min_w = MIN_WIDTH.js_max(if below { bands + 8. } else { 0. });
        let mut c;
        if node.is_group {
            c = CNode::new(0., 0., -1);
            c.is_group = true;
            c.dir = match &node.dir {
                Some(value) if input.truthy(value)? => {
                    Some(direction(Some(input.scalar_direction(value)?.as_ref())))
                }
                _ => None,
            };
            c.pad_x = GROUP_PAD;
            c.pad_top = if view.label.height > 0. {
                view.label.height + 16.
            } else {
                GROUP_PAD
            };
            c.pad_bottom = GROUP_PAD;
            c.min_w = (view.label.width + 2. * GROUP_PAD - 12.).js_max(MIN_WIDTH);
        } else {
            let name_h = view.label.height.js_max(round(name_size * 1.5));
            let attribute_count = match &node.attributes {
                Some(attributes) => input.attribute_len(attributes)?,
                None => 0,
            };
            if attribute_count == 0 {
                view.w = (view.label.width + 2. * BOX_PAD_X).js_max(min_w);
                view.h = (name_h + 2. * BOX_PAD_Y).js_max(if below { 0. } else { bands + 8. });
            } else {
                let row_line = round(row_size * 1.5);
                let mut widths = [0_f64; 4];
                let attributes = node
                    .attributes
                    .as_ref()
                    .expect("entity attributes captured");
                let mut attribute_index = 0;
                while let Some(attr) = input.attribute_at(attributes, attribute_index)? {
                    attribute_index += 1;
                    // Retain this row handle across callbacks, but fetch each
                    // subsequent field only after the preceding cell is laid out.
                    let cells = [
                        lab(
                            &cell_text(input.cell(&attr, CellField::Type)?.as_ref()),
                            row_size,
                            1e6,
                            base,
                            options,
                            resources,
                        )?,
                        lab(
                            &cell_text(input.cell(&attr, CellField::Name)?.as_ref()),
                            row_size,
                            1e6,
                            base,
                            options,
                            resources,
                        )?,
                        layout_label(
                            Some(&input.keys(&attr)?),
                            false,
                            resources,
                            options,
                            key_size,
                            1e6,
                            base,
                        )?,
                        lab(
                            &cell_text(input.cell(&attr, CellField::Comment)?.as_ref()),
                            row_size,
                            260.,
                            base,
                            options,
                            resources,
                        )?,
                    ];
                    let mut h = row_line;
                    for k in 0..4 {
                        if cells[k].width > widths[k] {
                            widths[k] = cells[k].width;
                        }
                        if cells[k].height > h {
                            h = cells[k].height;
                        }
                    }
                    view.rows.push(Row {
                        cells,
                        h: h + ROW_PAD,
                    });
                    view.h += h + ROW_PAD;
                }
                let mut columns = 0;
                let mut inner = 0.;
                for w in widths {
                    if w == 0. {
                        continue;
                    }
                    inner += w + if columns > 0 { CELL_GAP } else { 0. };
                    columns += 1;
                }
                view.w = (inner + 2. * CELL_PAD_X)
                    .js_max(view.label.width + 2. * BOX_PAD_X)
                    .js_max(min_w);
                let spare = if columns > 0 {
                    (view.w - inner - 2. * CELL_PAD_X) / columns as f64
                } else {
                    0.
                };
                let mut x = CELL_PAD_X;
                for w in widths {
                    view.cols.push(x);
                    if w > 0. {
                        x += w + spare + CELL_GAP
                    }
                }
                view.head_h = name_h + 2. * HEAD_PAD_Y;
                view.h += view.head_h;
            }
            let room = if !own.is_empty() {
                2. * (MARKER_LENGTH + LOOP * 0.75 + 10. + reach)
            } else {
                0.
            };
            c = if below {
                CNode::new(view.w, view.h + room, -1)
            } else {
                CNode::new(view.w + room, view.h, -1)
            };
        }
        c.seq = views.len();
        views.push(view);
        cnodes.push(c);
    }
    for (i, n) in nodes.iter().enumerate() {
        cnodes[i].parent = match &n.parent_id {
            Some(parent) => index
                .get(input.key(parent)?.as_ref())
                .map_or(-1, |i| *i as isize),
            None => -1,
        }
    }
    let layout = compound_layout(
        &mut cnodes,
        &mut cedges,
        if turned {
            Dir::TB
        } else {
            direction(Some(input.direction()?.as_ref()))
        },
        &LayeredOptions {
            node_sep: number_option(config, "nodeSpacing", 48.),
            edge_sep: 20.,
            rank_sep: number_option(config, "rankSpacing", 64.),
            port_sep: 24.,
            tracks: false,
            straight: false,
            apart: false,
            head_room: 0.,
        },
    );
    spread_ends(&mut cedges, &cnodes, &views);
    let mut layout_width = layout.width;
    for (i, own) in loops_by_node.iter().enumerate() {
        let c = &cnodes[i];
        if !c.is_group || own.is_empty() {
            continue;
        }
        let reach = own
            .iter()
            .map(|i| loops[*i].label.width)
            .fold(0., NumberMath::js_max);
        layout_width =
            layout_width.js_max(c.x + c.w / 2. + MARKER_LENGTH + LOOP * 0.75 + 10. + reach);
    }
    let title_text = input.title()?;
    let title = layout_label(
        title_text.as_deref(),
        false,
        resources,
        options,
        size,
        4000.,
        1,
    )?;
    let title_height = if title.height > 0. {
        title.height + 12.
    } else {
        0.
    };
    let inner = layout_width.js_max(title.width);
    let ox = pad + (inner - layout_width) / 2.;
    let oy = pad + title_height;
    shift_layout(&mut cnodes, &mut cedges, ox, oy);
    let width = (inner + 2. * pad).ceil();
    let height = (layout.height + title_height + 2. * pad).ceil();
    // This is the established typed/native presentation phase. The browser
    // host entry rejects native presentation before drawing; its accessibility
    // reads therefore occur only at the late SVG root boundary below.
    let mut presentation = if let Some(budget) = resources.presentation_budget() {
        let acc_title = input.acc_title()?;
        let source_title = input.title()?;
        let acc_descr = input.acc_descr()?;
        Some(crate::compat::presentation::FrameBuilder::new(
            budget,
            width,
            height,
            acc_title.as_deref().or(source_title.as_deref()),
            acc_descr.as_deref(),
        ))
    } else {
        None
    };
    let mut links = vec![];
    let mut crossings = Crossings::new();
    if cnodes.iter().any(|c| c.is_group) {
        for e in &cedges {
            mark_crossings(&mut crossings, &e.route)
        }
    }
    let mut clusters = String::new();
    let mut nodes_out = String::new();
    for (i, view) in views.iter_mut().enumerate() {
        let c = &cnodes[i];
        let node = &view.node;
        if let Some(frame) = &mut presentation {
            frame.label(
                input
                    .scalar_text(if c.is_group { &node.id } else { &node.label }, "capture")?
                    .as_ref(),
                &view.label,
            );
        }
        let css_classes = input.scalar_text(&node.css_classes, "replace")?;
        let stripped = css_classes
            .strip_prefix("default")
            .map(|s| {
                if s.chars().next().is_some_and(crate::compat::source::space) {
                    &s[s.chars().next().unwrap().len_utf8()..]
                } else {
                    s
                }
            })
            .unwrap_or(css_classes.as_ref());
        let classes = class_names(stripped);
        if c.is_group {
            view.w = c.w;
            view.h = c.h;
            let x = c.x - c.w / 2.;
            let y = c.y - c.h / 2.;
            let title_x = cluster_title_x(
                &ClusterBox {
                    x: c.x,
                    y: c.y,
                    w: c.w,
                    h: c.h,
                },
                view.label.width,
                &crossings,
                GROUP_PAD,
            );
            clusters.push_str(&format!("<g class=\"pele-cluster{classes}\" data-id=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{RADIUS}\" fill=\"var(--_a)\" fill-opacity=\"0.5\" stroke=\"var(--_b)\"{}/>{}</g>",esc_text(input.scalar_text(&node.id, "accessibility")?.as_ref())?,num(x),num(y),num(c.w),num(c.h),view.style.shape,label_svg_with_resources(&view.label,title_x,y+8.+view.label.height/2.,&format!(" class=\"pele-cluster-label\" fill=\"var(--_m)\"{}",view.style.text),resources)?));
            continue;
        }
        let w = view.w;
        let h = view.h;
        let x0 = -w / 2.;
        let y0 = -h / 2.;
        let text = &view.style.text;
        let shape = format!(
            "x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{RADIUS}\"",
            num(x0),
            num(y0),
            num(w),
            num(h)
        );
        let mut name = label_svg_with_resources(
            &view.label,
            0.,
            if !view.rows.is_empty() {
                y0 + view.head_h / 2.
            } else {
                0.
            },
            &format!(" class=\"pele-label\"{text}"),
            resources,
        )?;
        let display_value = if input.truthy(&node.alias)? {
            &node.alias
        } else {
            &node.label
        };
        let display = decode_entities(input.scalar_text(display_value, "accessibility")?.as_ref())?;
        if format!("{classes} ").contains(" internal-link ")
            && !name.is_empty()
            && options.links != Some(false)
        {
            let safe = link_url(&sanitize_url(&display), options).is_some();
            let target = esc(&display);
            name = format!(
                "<a class=\"internal-link\"{} data-href=\"{target}\">{name}</a>",
                if safe {
                    format!(" href=\"{target}\"{}", rel_attr(options))
                } else {
                    String::new()
                }
            );
            if safe {
                if let Some(frame) = &mut presentation {
                    frame.link(
                        input.scalar_text(&node.label, "capture")?.as_ref(),
                        &display,
                        true,
                        crate::compat::PresentationRect::centered(
                            c.x,
                            c.y + if !view.rows.is_empty() {
                                y0 + view.head_h / 2.
                            } else {
                                0.
                            },
                            view.label.width,
                            view.label.height,
                        ),
                    );
                }
            }
            links.push(LinkInfo {
                id: input.scalar_text(&node.label, "capture")?.into_owned(),
                href: display,
                internal: true,
            })
        }
        let body = if view.rows.is_empty() {
            // Explicit style follows box defaults, exactly as upstream.
            format!(
                "<rect {shape} fill=\"var(--_s)\" stroke=\"var(--_b)\"{}/>{name}",
                view.style.shape
            )
        } else {
            let row_line = round(view.row_size * 1.5);
            let line_color = view
                .style
                .stroke
                .as_ref()
                .map(|s| esc(s))
                .unwrap_or_else(|| "var(--_b)".into());
            let mut stripes = String::new();
            let mut cells = String::new();
            let mut y = y0 + view.head_h;
            for (r, row) in view.rows.iter().enumerate() {
                if let Some(frame) = &mut presentation {
                    for cell in &row.cells {
                        frame.label(input.scalar_text(&node.label, "capture")?.as_ref(), cell);
                    }
                }
                if r % 2 == 0 {
                    let span = format!("x=\"{}\" width=\"{}\"", num(x0), num(w));
                    if r == view.rows.len() - 1 {
                        stripes.push_str(&format!("<rect {span} y=\"{}\" height=\"{}\" rx=\"{RADIUS}\"/><rect {span} y=\"{}\" height=\"{}\"/>",num(y),num(row.h),num(y),num(row.h/2.)))
                    } else {
                        stripes.push_str(&format!(
                            "<rect {span} y=\"{}\" height=\"{}\"/>",
                            num(y),
                            num(row.h)
                        ))
                    }
                }
                let baseline = y + ROW_PAD / 2. + row_line / 2. + view.row_size * 0.35;
                cells.push_str(&text_left(
                    &row.cells[0],
                    x0 + view.cols[0],
                    baseline,
                    &format!(" class=\"pele-er-type\" fill=\"var(--_m)\"{text}"),
                    resources,
                )?);
                cells.push_str(&text_left(
                    &row.cells[1],
                    x0 + view.cols[1],
                    baseline,
                    &format!(" class=\"pele-er-name\"{text}"),
                    resources,
                )?);
                cells.push_str(&text_left(
                    &row.cells[2],
                    x0 + view.cols[2],
                    baseline,
                    &format!(
                        " class=\"pele-er-keys\" font-size=\"{}\" fill=\"var(--_m)\"{text}",
                        js_num(view.key_size)
                    ),
                    resources,
                )?);
                cells.push_str(&text_left(
                    &row.cells[3],
                    x0 + view.cols[3],
                    baseline,
                    &format!(" class=\"pele-er-comment\" fill=\"var(--_m)\"{text}"),
                    resources,
                )?);
                y += row.h;
            }
            let divider = num(y0 + view.head_h);
            format!("<rect {shape} fill=\"var(--_s)\" stroke=\"var(--_b)\"{}/><g class=\"pele-er-stripes\" fill=\"var(--_bg)\"{}>{stripes}</g><path class=\"pele-er-divider\" d=\"M{},{divider}H{}\" stroke=\"{line_color}\"/><rect {shape} fill=\"none\" stroke=\"var(--_b)\"{}/>{name}<g class=\"pele-er-attributes\" font-size=\"{}\">{cells}</g>",view.style.shape,if view.style.shape.contains("fill:"){" opacity=\"0.6\""}else{""},num(x0),num(x0+w),view.style.line,js_num(view.row_size))
        };
        nodes_out.push_str(&format!("<g class=\"pele-node pele-entity{classes}\" data-id=\"{}\" transform=\"translate({},{})\">{body}</g>",esc_text(input.scalar_text(&node.label, "accessibility")?.as_ref())?,num(c.x),num(c.y)));
    }
    for (i, own) in loops_by_node.iter().enumerate() {
        if own.is_empty() {
            continue;
        }
        let view = &views[i];
        let total: f64 = own.iter().map(|j| loops[*j].band).sum();
        let scale =
            1_f64.js_min(((if view.below { view.w } else { view.h }) - 8.).js_max(0.) / total);
        let mut at = -total * scale / 2.;
        for &j in own {
            let lp = &mut loops[j];
            lp.band *= scale;
            lp.at = at + lp.band / 2.;
            at += lp.band;
        }
    }
    let mut edges_out = String::new();
    let mut labels_out = String::new();
    for (i, ce) in cedges.iter_mut().enumerate() {
        let edge = drawn[i];
        let label = &labels[i];
        let (d, start, end);
        if let Some(lp) = loop_of[i].map(|j| &loops[j]) {
            let c = &cnodes[ce.src];
            let view = &views[ce.src];
            let spread = (lp.band / 2. - 6.).js_min(13.).js_max(1.);
            let far = MARKER_LENGTH + LOOP * 0.75 + 6.;
            if view.below {
                let x = c.x + lp.at;
                let y = c.y + view.h / 2.;
                let turn = num(y + MARKER_LENGTH);
                let out = num(y + MARKER_LENGTH + LOOP);
                d = format!(
                    "M{},{}V{turn}C{},{out} {},{out} {},{turn}V{}",
                    num(x - spread),
                    num(y),
                    num(x - spread),
                    num(x + spread),
                    num(x + spread),
                    num(y)
                );
                start = marker(&edge.arrow_start, x - spread, y, 0., -1.);
                end = marker(&edge.arrow_end, x + spread, y, 0., -1.);
                ce.label_x = x;
                ce.label_y = y + far + label.height / 2.;
            } else {
                let x = c.x + view.w / 2.;
                let y = c.y + lp.at;
                let turn = num(x + MARKER_LENGTH);
                let out = num(x + MARKER_LENGTH + LOOP);
                d = format!(
                    "M{},{}H{turn}C{out},{} {out},{} {turn},{}H{}",
                    num(x),
                    num(y - spread),
                    num(y - spread),
                    num(y + spread),
                    num(y + spread),
                    num(x)
                );
                start = marker(&edge.arrow_start, x, y - spread, -1., 0.);
                end = marker(&edge.arrow_end, x, y + spread, -1., 0.);
                ce.label_x = x + far + label.width / 2.;
                ce.label_y = y;
            }
        } else {
            let mut route = straighten(&ce.route);
            if route.len() < 6 {
                continue;
            }
            inset(&mut route, 0, &views[ce.src], &cnodes[ce.src]);
            let last = route.len() - 3;
            inset(&mut route, last, &views[ce.dst], &cnodes[ce.dst]);
            let dx = (route[last] - route[0]).abs();
            let dy = (route[last + 1] - route[1]).abs();
            let direct =
                route.len() == 6 && (dx < 0.01 || dy < 0.01 || dx.hypot(dy) <= 3. * MARKER_LENGTH);
            let stub = if direct { 0. } else { MARKER_LENGTH };
            let path = route_path(&route, None, stub, stub, None, None);
            d = if stub > 0. {
                format!(
                    "M{},{}L{}L{},{}",
                    num(path.sx),
                    num(path.sy),
                    &path.d[1..],
                    num(path.ex),
                    num(path.ey)
                )
            } else {
                path.d
            };
            start = marker(&edge.arrow_start, path.sx, path.sy, path.sdx, path.sdy);
            end = marker(&edge.arrow_end, path.ex, path.ey, path.edx, path.edy);
        }
        edges_out.push_str(&format!(
            "<g class=\"pele-edge pele-er-{}\" data-id=\"{}\"><path d=\"{d}\"{}/>{start}{end}</g>",
            if edge.dashed {
                "non-identifying"
            } else {
                "identifying"
            },
            esc_text(&edge.id)?,
            if edge.dashed {
                " stroke-dasharray=\"4 3\""
            } else {
                ""
            }
        ));
        if label.width > 0. {
            if let Some(frame) = &mut presentation {
                frame.label(&edge.id, label);
            }
            labels_out.push_str(&edge_label_svg(
                &edge.id, label, ce.label_x, ce.label_y, "", resources,
            )?)
        }
    }
    let body = format!(
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
    // Root metadata follows all body/icon callbacks, as in the pinned rt.
    let acc_title = input.acc_title()?;
    let acc_descr = input.acc_descr()?;
    let svg = svg::document(
        "er",
        width,
        height,
        size,
        options,
        acc_title.as_deref(),
        acc_descr.as_deref(),
        &body,
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
fn straighten(route: &[f64]) -> Vec<f64> {
    let mut out = route[..route.len().min(3)].to_vec();
    let mut k = 3;
    while k < route.len() {
        let prev = out.len() - 3;
        let flat = |axis: usize| {
            (out[prev + axis] - route[k + axis]).abs() < 0.01
                && (route[k + axis] - route[k + 3 + axis]).abs() < 0.01
        };
        if !(k + 3 < route.len() && (flat(0) || flat(1))) {
            out.extend_from_slice(&route[k..k + 3])
        }
        k += 3;
    }
    out
}
struct End {
    edge: usize,
    at: usize,
    toward: f64,
}
fn spread_ends<A: Clone, S: Clone, V: Clone>(
    edges: &mut [CEdge],
    nodes: &[CNode],
    views: &[View<A, S, V>],
) {
    let mut sides: Vec<(usize, Vec<End>)> = vec![];
    for (i, e) in edges.iter().enumerate() {
        if e.src == e.dst || e.route.len() < 6 {
            continue;
        }
        let last = e.route.len() - 3;
        for (node, at, next) in [(e.src, 0, 3), (e.dst, last, last - 3)] {
            let c = &nodes[node];
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
            let end = End {
                edge: i,
                at,
                toward: e.route[next + if side & 1 == 1 { 1 } else { 0 }],
            };
            if let Some((_, list)) = sides.iter_mut().find(|(k, _)| *k == key) {
                list.push(end)
            } else {
                sides.push((key, vec![end]))
            }
        }
    }
    for (key, mut ends) in sides {
        if ends.len() < 2 {
            continue;
        }
        let node = key >> 2;
        let vertical = key & 1 == 1;
        let c = &nodes[node];
        let v = &views[node];
        let length = if c.is_group {
            if vertical {
                c.h
            } else {
                c.w
            }
        } else if vertical {
            v.h
        } else {
            v.w
        };
        let step = 18_f64
            .js_min((length - 28.) / (ends.len() - 1) as f64)
            .js_max(0.);
        let center = if vertical { c.y } else { c.x };
        ends.sort_by(|a, b| {
            a.toward
                .partial_cmp(&b.toward)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let count = ends.len();
        for (i, end) in ends.into_iter().enumerate() {
            edges[end.edge].route[end.at + usize::from(vertical)] =
                center + (i as f64 - (count - 1) as f64 / 2.) * step;
        }
    }
}
fn inset<A: Clone, S: Clone, V: Clone>(route: &mut [f64], at: usize, v: &View<A, S, V>, c: &CNode) {
    if c.is_group {
        return;
    }
    let dx = route[at] - c.x;
    let dy = route[at + 1] - c.y;
    if c.w != v.w && (dx.abs() - c.w / 2.).abs() < 0.5 {
        route[at] -= dx.signum() * (c.w - v.w) / 2.
    }
    if c.h != v.h && (dy.abs() - c.h / 2.).abs() < 0.5 {
        route[at + 1] -= dy.signum() * (c.h - v.h) / 2.
    }
}
