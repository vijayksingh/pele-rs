//! Pele flowchart/render.ts actual compound flowchart renderer, 458322a5 (MIT, kepano).
use super::phase::{PhaseInput, Typed};
use super::{
    graph::{FlowGraph, GraphEdge, GraphNode},
    model::{FlowchartModel, LabelType},
    shapes::canonical,
};
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    layout::{
        compound::{compound_layout, direction, shift_layout, CEdge, CNode, CompoundResult, Dir},
        layered::LayeredOptions,
    },
    source::{
        decode_entities,
        url::{image_url, link_url, linked, rel_attr, safe_url, sanitize_url},
    },
    svg::{
        self,
        edges::{
            cluster::{
                cluster_title_x, mark_crossings, struck_title, title_crossed, ClusterBox, Crossings,
            },
            edge_label_svg, loop_path, marker, marker_trim, route_path,
            theme::{class_names, resolve_style, ResolvedStyle},
            LoopLabel, SideReach,
        },
        esc, esc_text, js_num, label_svg, label_svg_with_resources, num,
        shapes::{draw_shape, end_side, inset_route, shape_has_label, shape_size, shape_span},
    },
    text::{layout_label, Label},
    LinkInfo, Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
use std::collections::HashMap;
const GROUP_PAD: f64 = 20.;
const LOOP: f64 = 26.;
const SHAPE_ATTRS: &str = " fill=\"var(--_s)\" stroke=\"var(--_b)\"";
const LINE_ATTRS: &str = " stroke=\"var(--_b)\"";
fn number_option(flow: Option<&Object>, key: &str, fallback: f64) -> f64 {
    match flow.and_then(|o| o.get(key)) {
        Some(Value::Number(v)) if *v > 0. => v.js_min(2000.),
        _ => fallback,
    }
}
fn asset_size(value: Option<&Value>, fallback: f64) -> f64 {
    match value {
        Some(Value::Number(v)) if *v > 0. && *v <= 4000. => *v,
        _ => fallback,
    }
}
struct NodeView {
    node: GraphNode,
    style: ResolvedStyle,
    label: Label,
    shape: String,
    w: f64,
    h: f64,
    dy: f64,
    loops: usize,
}
fn sideways(cnodes: &[CNode], i: usize, root: Dir) -> bool {
    let mut dir = None;
    let mut p = cnodes[i].parent;
    for _ in 0..10000 {
        if p < 0 || dir.is_some() {
            break;
        }
        dir = cnodes[p as usize].dir;
        p = cnodes[p as usize].parent
    }
    dir.unwrap_or(root).sideways()
}
fn title_room(options: &RenderSettings) -> f64 {
    options.max_width.filter(|w| *w > 0.).map_or(4000., |w| {
        (w - 2. * options.padding.unwrap_or(8.)).js_max(80.)
    })
}
/// A family that consumes the same flow graph but supplies its own group/layout behavior.
/// The first consumer is the flowchart engine; swimlane can supply lane semantics without
/// copying source preparation, node styling, resource loading or edge rendering.
#[allow(dead_code)]
pub(crate) trait FlowVariant {
    fn diagram_type(&self) -> &str;
    fn prepare(&self, graph: &mut FlowGraph);
    fn layout(
        &self,
        nodes: &mut [CNode],
        edges: &mut [CEdge],
        dir: Dir,
        options: &LayeredOptions,
        tight: bool,
    ) -> Result<CompoundResult, PeleError>;
    #[allow(clippy::too_many_arguments)]
    fn draw_group(
        &self,
        _node: &CNode,
        _label: &Label,
        _style: &ResolvedStyle,
        _classes: &str,
        _id: &str,
        _dir: Dir,
        _resources: &mut dyn RenderResources,
    ) -> Result<Option<String>, PeleError> {
        Ok(None)
    }
}
pub(super) fn render(
    db: &FlowchartModel,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_flowchart(db, config, options, resources, None)
}
pub(crate) fn render_flowchart(
    db: &FlowchartModel,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
    variant: Option<&dyn FlowVariant>,
) -> Result<Rendered, PeleError> {
    render_phases(&mut Typed(db), config, options, resources, variant)
}
pub(super) fn render_phases<P: PhaseInput>(
    db: &mut P,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
    variant: Option<&dyn FlowVariant>,
) -> Result<Rendered, P::Error> {
    let runs_across = matches!(db.direction()?.as_deref(), Some("LR" | "RL"));
    let mut fitted = |down, resources: &mut dyn RenderResources| {
        let loose = draw(db, config, options, resources, variant, down, false)?;
        let Some(available) = options.max_width.filter(|w| *w > 0.) else {
            return Ok(loose);
        };
        if loose.0.width <= available {
            return Ok(loose);
        }
        let tight = draw(db, config, options, resources, variant, down, true)?;
        Ok::<_, P::Error>(if tight.0.width < loose.0.width {
            tight
        } else {
            loose
        })
    };
    let across = fitted(false, resources)?;
    let selected = if runs_across
        && options
            .max_width
            .filter(|w| *w > 0.)
            .is_some_and(|available| {
                !(options.auto_direction == Some(false)
                    || across.0.width <= available
                    || available
                        >= options
                            .direction_breakpoint
                            .filter(|b| *b > 0.)
                            .unwrap_or(640.))
            }) {
        let down = fitted(true, resources)?;
        if down.0.width < across.0.width {
            down
        } else {
            across
        }
    } else {
        across
    };
    if let Some(frame) = selected.1 {
        resources.capture_presentation(frame);
    }
    Ok(selected.0)
}
fn draw<P: PhaseInput>(
    db: &mut P,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
    variant: Option<&dyn FlowVariant>,
    turned: bool,
    tight: bool,
) -> Result<(Rendered, Option<crate::compat::PresentationOutcome>), P::Error> {
    let group_pad = if tight { 12. } else { GROUP_PAD };
    let flow = variant
        .and_then(|v| config.get(v.diagram_type()))
        .and_then(Value::as_object)
        .or_else(|| config.get("flowchart").and_then(Value::as_object));
    let curve = flow.and_then(|o| o.get("curve")).and_then(|v| {
        if let Value::String(s) = v {
            Some(s.as_str())
        } else {
            None
        }
    });
    let mut graph = db.begin_draw(curve, resources)?;
    if let Some(variant) = variant {
        variant.prepare(&mut graph)
    }
    let size = options.font_size.unwrap_or(16.);
    let edge_size = round(size * 0.875);
    let wrap_width =
        number_option(flow, "wrappingWidth", 200.).js_min(if variant.is_some() && tight {
            100.
        } else {
            f64::INFINITY
        });
    let pad = options.padding.unwrap_or(8.);
    let index: HashMap<_, _> = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.clone(), i))
        .collect();
    let mut views = Vec::new();
    let mut cnodes = Vec::new();
    for mut node in graph.nodes {
        db.resolve_styles(&mut node)?;
        let mut declarations = node.css_compiled_styles.clone();
        declarations.extend(node.css_styles.iter().cloned());
        let style = resolve_style(&declarations);
        let text_style = u8::from(style.bold) | if style.italic { 2 } else { 0 };
        let markdown = matches!(node.label_type, LabelType::Markdown);
        let font_size = style.font_size.unwrap_or(size);
        let (mut c, view) = if node.is_group {
            let label = layout_label(
                node.label.as_deref(),
                markdown,
                resources,
                options,
                font_size,
                if variant.is_some() && tight {
                    (title_room(options) / 2.).js_max(80.)
                } else {
                    4000.
                },
                text_style,
            )?;
            let mut c = CNode::new(0., 0., -1);
            c.is_group = true;
            c.dir = node
                .dir
                .as_ref()
                .filter(|s| !s.is_empty())
                .map(|d| direction(Some(d)));
            c.pad_x = group_pad;
            c.pad_top = if label.height > 0. {
                label.height + 16.
            } else {
                GROUP_PAD
            };
            c.pad_bottom = group_pad;
            c.min_w = label.width + 2. * group_pad - 12.;
            (
                c,
                NodeView {
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
        } else {
            let shape = canonical(Some(&node.shape)).to_string();
            let has_asset = node.img.as_ref().is_some_and(|s| !s.is_empty())
                || node.icon.as_ref().is_some_and(|s| !s.is_empty());
            let text = if has_asset || shape_has_label(&shape) {
                node.label.as_deref()
            } else {
                None
            };
            let label = layout_label(
                text, markdown, resources, options, font_size, wrap_width, text_style,
            )?;
            let (w, h, dy) = if has_asset {
                let bx = if node.img.as_ref().is_some_and(|s| !s.is_empty()) {
                    80.
                } else {
                    48.
                };
                let aw = asset_size(node.asset_width.as_ref(), bx);
                let ah = asset_size(node.asset_height.as_ref(), bx);
                (
                    aw.js_max(label.width + 8.),
                    ah + if label.height > 0. {
                        label.height + 6.
                    } else {
                        0.
                    },
                    if label.height == 0. {
                        0.
                    } else if node.pos.as_deref() == Some("t") {
                        -ah / 2.
                    } else {
                        ah / 2.
                    },
                )
            } else {
                let s = shape_size(&shape, label.width, label.height);
                (s.w, s.h, s.dy)
            };
            (
                CNode::new(w, h, -1),
                NodeView {
                    node,
                    style,
                    label,
                    shape,
                    w,
                    h,
                    dy,
                    loops: 0,
                },
            )
        };
        c.seq = views.len();
        views.push(view);
        cnodes.push(c)
    }
    for (i, view) in views.iter().enumerate() {
        cnodes[i].parent = view
            .node
            .parent_id
            .as_ref()
            .and_then(|p| index.get(p))
            .map_or(-1, |i| *i as isize)
    }
    let root_dir = if turned {
        Dir::TB
    } else {
        direction(db.direction()?.as_deref())
    };
    for (i, view) in views.iter().enumerate() {
        if view.node.is_group {
            cnodes[i].span = f64::INFINITY
        } else if view.node.img.is_none() && view.node.icon.is_none() {
            cnodes[i].span = shape_span(&view.shape, view.w, view.h, sideways(&cnodes, i, root_dir))
        }
    }
    let mut cedges = Vec::new();
    let mut edge_labels = Vec::new();
    let mut drawn: Vec<GraphEdge> = Vec::new();
    for edge in graph.edges {
        let Some(&src) = index.get(&edge.start) else {
            continue;
        };
        let Some(&dst) = index.get(&edge.end) else {
            continue;
        };
        let label = layout_label(
            Some(&edge.label),
            matches!(edge.label_type, LabelType::Markdown),
            resources,
            options,
            edge_size,
            wrap_width,
            0,
        )?;
        if src == dst {
            views[src].loops += 1;
            if sideways(&cnodes, src, root_dir) {
                cnodes[src].h += 2.
                    * (LOOP
                        + if label.height > 0. {
                            label.height + 8.
                        } else {
                            0.
                        })
            } else {
                cnodes[src].w += 2.
                    * (LOOP
                        + if label.width > 0. {
                            label.width + 12.
                        } else {
                            0.
                        })
            }
        }
        cedges.push(CEdge {
            src,
            dst,
            minlen: edge
                .minlen
                .as_ref()
                .and_then(|v| {
                    if let Value::Number(n) = v {
                        Some(*n)
                    } else {
                        None
                    }
                })
                .unwrap_or(1.),
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
            route: Vec::new(),
            label_x: 0.,
            label_y: 0.,
        });
        drawn.push(edge);
        edge_labels.push(label)
    }
    let layout_options = LayeredOptions {
        node_sep: number_option(flow, "nodeSpacing", 40.) * if tight { 0.5 } else { 1. },
        edge_sep: 16.,
        rank_sep: number_option(flow, "rankSpacing", 48.) * if tight { 0.75 } else { 1. },
        port_sep: 20.,
        tracks: curve.is_some_and(|s| s.starts_with("step")),
        straight: curve == Some("linear"),
        apart: false,
        head_room: marker_trim("arrow_point"),
    };
    let layout = if let Some(variant) = variant {
        variant.layout(&mut cnodes, &mut cedges, root_dir, &layout_options, tight)?
    } else {
        compound_layout(&mut cnodes, &mut cedges, root_dir, &layout_options)
    };
    let title_text = db.metadata("title")?;
    let title = layout_label(
        title_text.as_deref(),
        false,
        resources,
        options,
        size,
        if tight { title_room(options) } else { 4000. },
        1,
    )?;
    let title_height = if title.height > 0. {
        title.height + 12.
    } else {
        0.
    };
    let inner = layout.width.js_max(title.width);
    let ox = pad + (inner - layout.width) / 2.;
    let oy = pad + title_height;
    shift_layout(&mut cnodes, &mut cedges, ox, oy);
    let width = (inner + 2. * pad).ceil();
    let height = (layout.height + title_height + 2. * pad).ceil();
    let frame_access = if resources.presentation_budget().is_some() {
        Some((
            db.metadata("accTitle")?,
            db.metadata("title")?,
            db.metadata("accDescr")?,
        ))
    } else {
        None
    };
    let mut presentation = resources.presentation_budget().map(|budget| {
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
    let mut links = Vec::new();
    let mut crossings = Crossings::new();
    if cnodes.iter().any(|c| c.is_group) {
        for e in &cedges {
            mark_crossings(&mut crossings, &e.route)
        }
    }
    let mut clusters = String::new();
    let mut struck_titles = String::new();
    let mut nodes_out = String::new();
    let parents: Vec<_> = cnodes.iter().map(|c| c.parent as i32).collect();
    for (i, view) in views.iter().enumerate() {
        let c = &cnodes[i];
        let node = &view.node;
        if let Some(frame) = &mut presentation {
            frame.label(&node.id, &view.label);
        }
        let classes = class_names(
            node.css_classes
                .strip_prefix("default")
                .map_or(node.css_classes.as_str(), |s| {
                    s.strip_prefix(' ').unwrap_or(s)
                }),
        );
        let id = esc_text(&node.id)?;
        if c.is_group && c.parent < 0 {
            if let Some(variant) = variant {
                if let Some(markup) = variant.draw_group(
                    c,
                    &view.label,
                    &view.style,
                    &classes,
                    &id,
                    root_dir,
                    resources,
                )? {
                    clusters.push_str(&markup);
                    continue;
                }
            }
        }
        if c.is_group {
            let bx = ClusterBox {
                x: c.x,
                y: c.y,
                w: c.w,
                h: c.h,
            };
            let title_x = cluster_title_x(&bx, view.label.width, &crossings, group_pad);
            let title_y = c.y - c.h / 2. + 8. + view.label.height / 2.;
            let title_attrs = format!(
                " class=\"pele-cluster-label\" fill=\"var(--_m)\"{}",
                view.style.text
            );
            let struck = title_crossed(&bx, view.label.width, title_x, &crossings);
            if struck {
                struck_titles.push_str(&struck_title(
                    &view.label,
                    title_x,
                    title_y,
                    &title_attrs,
                    &id,
                    &parents,
                    i,
                    resources,
                )?)
            }
            clusters.push_str(&format!("<g class=\"pele-cluster{classes}\" data-id=\"{id}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"var(--pele-radius,4px)\" fill=\"var(--_a)\" fill-opacity=\"0.5\" stroke=\"var(--_b)\"{}/>{}</g>",num(c.x-c.w/2.),num(c.y-c.h/2.),num(c.w),num(c.h),view.style.shape,if struck{String::new()}else{label_svg_with_resources(&view.label,title_x,title_y,&title_attrs,resources)?}));
            continue;
        }
        let inner = if let Some(img) = node.img.as_ref().filter(|s| !s.is_empty()) {
            let aw = asset_size(node.asset_width.as_ref(), 80.);
            let ah = asset_size(node.asset_height.as_ref(), 80.);
            let top = if view.label.height > 0. {
                if node.pos.as_deref() == Some("t") {
                    view.h / 2. - ah
                } else {
                    -view.h / 2.
                }
            } else {
                -ah / 2.
            };
            if let Some(src) = image_url(&safe_url(img)?, options) {
                let embedded = resources.image(&src)?;
                let src = match &embedded {
                    crate::compat::ImageResource::Passthrough => src.as_str(),
                    crate::compat::ImageResource::Embedded(image) => image.data_uri(),
                };
                format!("<image href=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"{}\"/>",esc(&src),num(-aw/2.),num(top),num(aw),num(ah),if node.constraint.as_deref()==Some("on"){"xMidYMid meet"}else{"none"})
            } else {
                String::new()
            }
        } else if let Some(icon) = node.icon.as_ref().filter(|s| !s.is_empty()) {
            let bx = asset_size(node.asset_height.as_ref(), 48.);
            let top = if view.label.height > 0. {
                if node.pos.as_deref() == Some("t") {
                    view.h / 2. - bx
                } else {
                    -view.h / 2.
                }
            } else {
                -bx / 2.
            };
            let frame = match node.shape.as_str() {
                "icon" => String::new(),
                "iconCircle" => format!(
                    "<circle cy=\"{}\" r=\"{}\"{SHAPE_ATTRS}{}/>",
                    num(top + bx / 2.),
                    num(bx / 2.),
                    view.style.shape
                ),
                _ => format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\"{SHAPE_ATTRS}{}/>",
                    num(-bx / 2.),
                    num(top),
                    num(bx),
                    num(bx),
                    if node.shape == "iconRounded" { 8 } else { 0 },
                    view.style.shape
                ),
            };
            let glyph = bx * 0.6;
            let markup = resources
                .direct_icon(icon)?
                .map(|v| v.markup().to_owned())
                .unwrap_or_default();
            format!("{frame}<svg class=\"pele-icon\" data-icon=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{markup}</svg>",esc_text(icon)?,num(-glyph/2.),num(top+(bx-glyph)/2.),num(glyph),num(glyph))
        } else {
            draw_shape(
                &view.shape,
                view.w,
                view.h,
                &format!("{SHAPE_ATTRS}{}", view.style.shape),
                &format!("{LINE_ATTRS}{}", view.style.line),
            )
        };
        let mut text = label_svg_with_resources(
            &view.label,
            0.,
            view.dy,
            &format!(" class=\"pele-label\"{}", view.style.text),
            resources,
        )?;
        let internal = classes.split_whitespace().any(|s| s == "internal-link");
        if internal
            && node.label.as_ref().is_some_and(|s| !s.is_empty())
            && options.links != Some(false)
        {
            let name = decode_entities(node.label.as_deref().unwrap())?;
            let safe = link_url(&sanitize_url(&name), options).is_some();
            let target = esc(&name);
            text = format!(
                "<a class=\"internal-link\"{} data-href=\"{target}\">{text}</a>",
                if safe {
                    format!(" href=\"{target}\"{}", rel_attr(options))
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
                            c.y + view.dy,
                            view.label.width,
                            view.label.height,
                        ),
                    );
                }
            }
            links.push(LinkInfo {
                id: node.id.clone(),
                href: name,
                internal: true,
            })
        }
        let mut body = inner + &text;
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
            body = linked(&body, link, &node.id, &mut links, options, &attrs);
            if let Some(frame) = &mut presentation {
                if let Some(approved) = links.get(link_index) {
                    frame.link(
                        &node.id,
                        &approved.href,
                        false,
                        crate::compat::PresentationRect::centered(c.x, c.y, view.w, view.h),
                    );
                }
            }
        }
        nodes_out.push_str(&format!("<g class=\"pele-node pele-shape-{}{classes}\" data-id=\"{id}\" transform=\"translate({},{})\">{body}</g>",view.shape,num(c.x),num(c.y)));
    }
    let mut reach = SideReach::default();
    let end = |route: &[f64], at: usize, c: &CNode| end_side(route, at, c.x, c.y, c.w, c.h);
    for (i, ce) in cedges.iter().enumerate() {
        if ce.src == ce.dst
            || drawn[i].thickness.as_deref() == Some("invisible")
            || ce.route.len() < 6
        {
            continue;
        }
        reach.add(
            ce.src,
            end(&ce.route, 0, &cnodes[ce.src]),
            &drawn[i].arrow_type_start,
        );
        reach.add(
            ce.dst,
            end(&ce.route, ce.route.len() - 3, &cnodes[ce.dst]),
            &drawn[i].arrow_type_end,
        )
    }
    let mut edges_out = String::new();
    let mut labels_out = String::new();
    let mut loop_seen = HashMap::new();
    for i in 0..cedges.len() {
        let ce = &mut cedges[i];
        let edge = &drawn[i];
        let label = &edge_labels[i];
        if let Some(frame) = &mut presentation {
            frame.label(&edge.id, label);
        }
        let style = if !edge.style.is_empty() || !edge.css_compiled_styles.is_empty() {
            let mut declarations = edge.css_compiled_styles.clone();
            declarations.extend(edge.style.iter().cloned());
            Some(resolve_style(&declarations))
        } else {
            None
        };
        if edge.thickness.as_deref() != Some("invisible") {
            let start_type = &edge.arrow_type_start;
            let end_type = &edge.arrow_type_end;
            let path = if ce.src == ce.dst {
                let view = &views[ce.src];
                let k = loop_seen.entry(ce.src).or_insert(0usize);
                let mut lab = LoopLabel {
                    label_w: ce.label_w,
                    label_h: ce.label_h,
                    label_x: ce.label_x,
                    label_y: ce.label_y,
                };
                let path = loop_path(
                    cnodes[ce.src].x,
                    cnodes[ce.src].y,
                    view.w,
                    view.h,
                    *k,
                    sideways(&cnodes, ce.src, root_dir),
                    LOOP,
                    marker_trim(end_type),
                    &mut lab,
                );
                *k += 1;
                ce.label_x = lab.label_x;
                ce.label_y = lab.label_y;
                path
            } else {
                let mut route = ce.route.clone();
                let last = route.len() - 3;
                let leaves = reach.of(ce.src, end(&route, 0, &cnodes[ce.src]));
                let arrives = reach.of(ce.dst, end(&route, last, &cnodes[ce.dst]));
                for (at, v, c) in [
                    (0, &views[ce.src], &cnodes[ce.src]),
                    (last, &views[ce.dst], &cnodes[ce.dst]),
                ] {
                    inset_route(
                        &mut route, at, &v.shape, v.w, v.h, c.x, c.y, c.w, c.h, c.is_group,
                    )
                }
                route_path(
                    &route,
                    edge.curve.as_ref().and_then(|v| {
                        if let Value::String(s) = v {
                            Some(s.as_str())
                        } else {
                            None
                        }
                    }),
                    marker_trim(start_type),
                    marker_trim(end_type),
                    Some(leaves),
                    Some(arrives),
                )
            };
            let color = style
                .as_ref()
                .and_then(|s| s.stroke.as_ref())
                .filter(|s| !s.is_empty())
                .map_or_else(|| "var(--_l)".into(), |s| esc(s));
            let attrs = if edge.thickness.as_deref() == Some("thick") {
                " stroke-width=\"2.5\""
            } else if edge.thickness.as_deref() == Some("dotted") {
                " stroke-dasharray=\"3 4\""
            } else {
                ""
            };
            let classes = class_names(
                &edge
                    .classes
                    .replace("edge-thickness-normal", "")
                    .replace("edge-pattern-solid", "")
                    .replace("flowchart-link", ""),
            );
            edges_out.push_str(&format!(
                "<g class=\"pele-edge{classes}\" data-id=\"{}\"{}><path d=\"{}\"{attrs}/>{}{}</g>",
                esc_text(&edge.id)?,
                style.as_ref().map_or("", |s| s.line.as_str()),
                path.d,
                marker(start_type, path.sx, path.sy, path.sdx, path.sdy, &color),
                marker(end_type, path.ex, path.ey, path.edx, path.edy, &color)
            ))
        }
        if label.width > 0. {
            labels_out.push_str(&edge_label_svg(
                &edge.id,
                label,
                ce.label_x,
                ce.label_y,
                style.as_ref().map_or("", |s| s.text.as_str()),
                resources,
            )?)
        }
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
    if !struck_titles.is_empty() {
        content.push_str(&format!(
            "<g class=\"pele-cluster-titles\">{struck_titles}</g>"
        ))
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
    Ok((
        Rendered {
            svg: svg::document(
                variant.map_or("flowchart", |v| v.diagram_type()),
                width,
                height,
                size,
                options,
                acc_title.as_deref(),
                acc_descr.as_deref(),
                &content,
            )?,
            width,
            height,
            links,
        },
        presentation.map(|frame| frame.finish()),
    ))
}
