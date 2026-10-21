//! Direct Pele class/render.ts geometry and SVG translation, 458322a5 (MIT, kepano).
use super::super::common::generics::parse_generic_types;
use super::{
    graph::{build, GraphEdge, GraphNode},
    markers,
    model::*,
};
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    layout::{
        compound::{compound_layout, direction, CEdge, CNode, Dir},
        layered::LayeredOptions,
    },
    source::{decode_entities, url::linked},
    svg::{
        document,
        edges::{
            cluster::{
                cluster_title_x, mark_crossings, struck_title, title_crossed, ClusterBox, Crossings,
            },
            edge_label_svg, route_path,
            theme::{class_names, resolve_style, ResolvedStyle},
            EdgePath,
        },
        esc, esc_text, label_svg_with_resources, num,
        shapes::end_side,
    },
    text::{layout_label, measure, Label},
    Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
use std::collections::{BTreeMap, HashMap};
const RADIUS: &str = "var(--pele-radius,4px)";
const LINE: &str = "var(--_l)";
#[derive(Clone)]
pub(super) struct Line {
    pub(super) text: String,
    pub(super) width: f64,
    pub(super) classifier: String,
}
/// Mutable public models supply rows at the upstream callsite. Geometry stays here.
pub(super) trait LiveMembers {
    fn begin_draw(&mut self) -> Result<ClassModel, PeleError>;
    fn class(&mut self, index: usize) -> Result<ClassNode, PeleError>;
    fn definition(&mut self, name: &str) -> Result<Vec<String>, PeleError>;
    /// Read after the class label has completed its resource callbacks.
    fn annotations(&mut self, index: usize) -> Result<Vec<String>, PeleError>;
    /// Read after the class label has been painted, before linking/wrapping it.
    fn paint(&mut self, index: usize) -> Result<ClassNode, PeleError>;
    fn metadata(&mut self, key: &str) -> Result<Option<String>, PeleError>;
    fn lines(
        &mut self,
        index: usize,
        field: &str,
        size: f64,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Vec<Line>, PeleError>;
}
struct View {
    node: GraphNode,
    style: ResolvedStyle,
    label: Label,
    w: f64,
    h: f64,
    annotations: Vec<String>,
    attributes: Vec<Line>,
    methods: Vec<Line>,
    title_h: f64,
    attributes_h: f64,
    font_size: f64,
}
impl View {
    fn base(node: GraphNode, label: Label, w: f64, h: f64, font_size: f64) -> Self {
        Self {
            node,
            style: ResolvedStyle::default(),
            label,
            w,
            h,
            annotations: vec![],
            attributes: vec![],
            methods: vec![],
            title_h: 0.,
            attributes_h: 0.,
            font_size,
        }
    }
}
#[derive(Clone)]
struct End {
    edge: usize,
    at: usize,
    toward: f64,
    order: usize,
    room: f64,
    side: i32,
    after: bool,
}
fn number(conf: &Object, key: &str, fallback: f64) -> f64 {
    match conf.get(key) {
        Some(Value::Number(n)) if *n > 0. => n.js_min(2000.),
        _ => fallback,
    }
}
#[derive(Default)]
struct StyleCache {
    effect: HashMap<String, String>,
    from_defs: HashMap<String, Vec<(String, String)>>,
}
impl StyleCache {
    fn keep(&mut self, kept: &mut Vec<(String, String)>, decl: &str) {
        let prop = self.effect.entry(decl.into()).or_insert_with(|| {
            let one = resolve_style(&[decl.into()]);
            if one.shape.is_empty() && one.text.is_empty() {
                String::new()
            } else {
                decl.split(':')
                    .next()
                    .unwrap()
                    .trim_matches(crate::compat::source::space)
                    .to_lowercase()
            }
        });
        if prop.is_empty() {
            return;
        }
        if let Some((_, value)) = kept.iter_mut().find(|(key, _)| key == prop) {
            *value = decl.into()
        } else {
            kept.push((prop.clone(), decl.into()))
        }
    }
    fn resolve(&mut self, db: &ClassModel, cls: &ClassNode) -> ResolvedStyle {
        let mut kept = if let Some(base) = self.from_defs.get(&cls.css_classes) {
            base.clone()
        } else {
            let mut kept = vec![];
            for name in cls.css_classes.split(' ') {
                if let Some((_, def)) = db.style_classes.iter().find(|(k, _)| k == name) {
                    for s in &def.styles {
                        for part in s.split(',') {
                            self.keep(&mut kept, part)
                        }
                    }
                }
            }
            self.from_defs.insert(cls.css_classes.clone(), kept.clone());
            kept
        };
        for s in &cls.styles {
            self.keep(&mut kept, s)
        }
        resolve_style(&kept.into_iter().map(|(_, s)| s).collect::<Vec<_>>())
    }
    fn resolve_live(
        &mut self,
        cls: &ClassNode,
        live: &mut dyn LiveMembers,
    ) -> Result<ResolvedStyle, PeleError> {
        let mut kept = if let Some(base) = self.from_defs.get(&cls.css_classes) {
            base.clone()
        } else {
            let mut kept = vec![];
            for name in cls.css_classes.split(' ') {
                for declaration in live.definition(name)? {
                    for part in declaration.split(',') {
                        self.keep(&mut kept, part);
                    }
                }
            }
            self.from_defs.insert(cls.css_classes.clone(), kept.clone());
            kept
        };
        for declaration in &cls.styles {
            self.keep(&mut kept, declaration);
        }
        Ok(resolve_style(
            &kept.into_iter().map(|(_, s)| s).collect::<Vec<_>>(),
        ))
    }
}

// Budget the same compacted cascade that geometry consumes. Raw definitions can
// contain millions of repeated/ignored declarations without expanding SVG styles.
pub(super) fn projected_style_bytes(db: &ClassModel) -> usize {
    let mut cache = StyleCache::default();
    db.classes.iter().fold(0usize, |total, (_, class)| {
        let style = cache.resolve(db, class);
        total
            .saturating_add(style.shape.len())
            .saturating_add(style.line.len())
            .saturating_add(style.text.len())
            .saturating_add(style.stroke.as_ref().map_or(0, String::len))
    })
}

fn member_line(
    m: &ClassMember,
    size: f64,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Line, PeleError> {
    let text = decode_entities(&m.display())?
        .trim_matches(crate::compat::source::space)
        .to_string();
    let width = measure(r, o, &text, size, if m.classifier == "*" { 2 } else { 0 })?;
    Ok(Line {
        text,
        width,
        classifier: m.classifier.clone(),
    })
}
fn class_view(
    db: &ClassModel,
    cache: &mut StyleCache,
    node: GraphNode,
    cls: &ClassNode,
    index: usize,
    live: &mut Option<&mut dyn LiveMembers>,
    size: f64,
    tiny: f64,
    tiny_h: f64,
    hide_empty: bool,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<View, PeleError> {
    let current;
    let cls = if let Some(live) = live.as_deref_mut() {
        current = live.class(index)?;
        &current
    } else {
        cls
    };
    let style = if let Some(live) = live.as_deref_mut() {
        cache.resolve_live(cls, live)?
    } else {
        cache.resolve(db, cls)
    };
    let font_size = style.font_size.unwrap_or(size);
    let member_size = round(font_size * 0.875);
    let line_height = round(member_size * 1.5);
    let generic = if cls.generic.is_empty() {
        String::new()
    } else {
        format!(
            "&lt;{}&gt;",
            parse_generic_types(&cls.generic)
                .replace('<', "&lt;")
                .replace('>', "&gt;")
        )
    };
    let label = layout_label(
        Some(&format!("{}{generic}", cls.label)),
        false,
        r,
        o,
        font_size,
        4000.,
        1 | if style.italic { 2 } else { 0 },
    )?;
    let current_annotations;
    let annotation_source = if let Some(live) = live.as_deref_mut() {
        current_annotations = live.annotations(index)?;
        &current_annotations
    } else {
        &cls.annotations
    };
    let annotations = annotation_source
        .iter()
        .map(|a| {
            Ok(format!(
                "«{}»",
                decode_entities(a)?.trim_matches(crate::compat::source::space)
            ))
        })
        .collect::<Result<Vec<_>, PeleError>>()?;
    let attributes = if let Some(live) = live.as_deref_mut() {
        live.lines(index, "members", member_size, o, r)?
    } else {
        cls.members
            .iter()
            .map(|m| member_line(m, member_size, o, r))
            .collect::<Result<Vec<_>, _>>()?
    };
    let methods = if let Some(live) = live.as_deref_mut() {
        live.lines(index, "methods", member_size, o, r)?
    } else {
        cls.methods
            .iter()
            .map(|m| member_line(m, member_size, o, r))
            .collect::<Result<Vec<_>, _>>()?
    };
    let mut inner = label.width;
    for a in &annotations {
        inner = inner.js_max(measure(r, o, a, tiny, 0)?)
    }
    for l in attributes.iter().chain(&methods) {
        inner = inner.js_max(l.width)
    }
    let sections = !(hide_empty && attributes.is_empty() && methods.is_empty());
    let section = |count: usize| {
        if !sections {
            0.
        } else if count > 0 {
            12. + count as f64 * line_height
        } else {
            10.
        }
    };
    let title_h =
        16. + annotations.len() as f64 * tiny_h + label.height.js_max(round(font_size * 1.5));
    let attributes_h = section(attributes.len());
    let h = title_h + attributes_h + section(methods.len());
    Ok(View {
        node,
        style,
        label,
        w: (inner + 24.).js_max(64.),
        h,
        annotations,
        attributes,
        methods,
        title_h,
        attributes_h,
        font_size,
    })
}
pub fn render(
    db: &ClassModel,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_impl(db, c, o, r, Preflight::LegacyInteger, None)
}
pub(super) fn render_exact(
    db: &ClassModel,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_impl(db, c, o, r, Preflight::ExactPostRender, None)
}
pub(super) fn render_live(
    db: &ClassModel,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    exact: bool,
    live: &mut dyn LiveMembers,
) -> Result<Rendered, PeleError> {
    render_impl(
        db,
        c,
        o,
        r,
        if exact {
            Preflight::ExactPostRender
        } else {
            Preflight::LegacyInteger
        },
        Some(live),
    )
}
#[derive(Clone, Copy)]
enum Preflight {
    LegacyInteger,
    ExactPostRender,
}
fn render_impl(
    db: &ClassModel,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    preflight: Preflight,
    mut live: Option<&mut dyn LiveMembers>,
) -> Result<Rendered, PeleError> {
    // Every drawn object emits its authored id. Refuse a proven output overflow
    // before allocating the compound geometry or the entire oversized SVG.
    let hierarchical = c
        .get("class")
        .and_then(Value::as_object)
        .and_then(|c| c.get("hierarchicalNamespaces"))
        != Some(&Value::Bool(false));
    let mut minimum = 0usize;
    for id in db
        .namespaces
        .iter()
        .filter(|(_, n)| hierarchical || n.explicit)
        .map(|(key, value)| match preflight {
            Preflight::LegacyInteger => key,
            Preflight::ExactPostRender => &value.id,
        })
        .chain(db.classes.iter().map(|(key, value)| match preflight {
            Preflight::LegacyInteger => key,
            Preflight::ExactPostRender => &value.id,
        }))
        .chain(db.notes.iter().map(|(key, value)| match preflight {
            Preflight::LegacyInteger => key,
            Preflight::ExactPostRender => &value.id,
        }))
    {
        let escaped = match preflight {
            Preflight::LegacyInteger => esc_text(id)?,
            // The bound is optional. Normal rendering must establish the order
            // of authored-ID decoding errors relative to host callbacks.
            Preflight::ExactPostRender => match esc_text(id) {
                Ok(escaped) => escaped,
                Err(_) => continue,
            },
        };
        match preflight {
            Preflight::LegacyInteger => {
                minimum = minimum.saturating_add(escaped.encode_utf16().count());
                if minimum > o.output_limit() {
                    let mut e = PeleError::new(
                        format!(
                            "Diagram output is longer than the limit of {} characters.",
                            o.output_limit()
                        ),
                        crate::compat::ErrorCode::Limit,
                    );
                    e.diagram_type = Some("class".into());
                    return Err(e);
                }
            }
            Preflight::ExactPostRender => {
                // Only the independent native byte ceiling may reject early.
                // A caller's semantic threshold is checked after callbacks.
                minimum = minimum.saturating_add(escaped.len());
                if minimum > crate::compat::MAX_PUBLIC_OUTPUT_BYTES {
                    return Err(crate::compat::OutputLimit::byte_error("class"));
                }
            }
        }
    }
    let mut fitted = |turned| -> Result<Rendered, PeleError> {
        let loose = draw(db, c, o, r, turned, false, &mut live)?;
        if o.max_width.is_none_or(|a| !(a > 0.) || loose.width <= a) {
            return Ok(loose);
        }
        let tight = draw(db, c, o, r, turned, true, &mut live)?;
        Ok(if tight.width < loose.width {
            tight
        } else {
            loose
        })
    };
    let across = fitted(false)?;
    let available = o.max_width.unwrap_or(0.);
    if !matches!(db.direction.as_str(), "LR" | "RL")
        || o.auto_direction == Some(false)
        || !(available > 0.)
        || across.width <= available
        || available >= o.direction_breakpoint.filter(|n| *n > 0.).unwrap_or(640.)
    {
        return Ok(across);
    }
    let down = fitted(true)?;
    Ok(if down.width < across.width {
        down
    } else {
        across
    })
}
fn draw(
    db: &ClassModel,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    turned: bool,
    tight: bool,
    live: &mut Option<&mut dyn LiveMembers>,
) -> Result<Rendered, PeleError> {
    let current;
    let db = if let Some(live) = live.as_deref_mut() {
        current = live.begin_draw()?;
        &current
    } else {
        db
    };
    let empty = Object::default();
    let conf = c.get("class").and_then(Value::as_object).unwrap_or(&empty);
    let graph = build(
        db,
        conf.get("hierarchicalNamespaces") != Some(&Value::Bool(false)),
    );
    let hide_empty = conf.get("hideEmptyMembersBox") == Some(&Value::Bool(true));
    let size = o.font_size.unwrap_or(16.);
    let small = round(size * 0.875);
    let tiny = round(size * 0.8);
    let tiny_h = round(tiny * 1.4);
    let pad = o.padding.unwrap_or(8.);
    let root_dir = if turned {
        Dir::TB
    } else {
        direction(Some(&db.direction))
    };
    let n = graph.nodes.len();
    let mut children = vec![0usize; n];
    for node in &graph.nodes {
        if node.parent >= 0 {
            children[node.parent as usize] += 1
        }
    }
    let mut crossed = vec![false; n];
    for edge in &graph.edges {
        if edge.source < 0 || edge.target < 0 {
            continue;
        }
        let a = graph.nodes[edge.source as usize].parent;
        let b = graph.nodes[edge.target as usize].parent;
        if a == b {
            continue;
        }
        for mut g in [a, b] {
            while g >= 0 && !crossed[g as usize] {
                crossed[g as usize] = true;
                g = graph.nodes[g as usize].parent
            }
        }
    }
    let mut cache = StyleCache::default();
    let mut views = vec![];
    let mut cnodes = vec![];
    for (i, node) in graph.nodes.iter().cloned().enumerate() {
        let mut view;
        let mut cn;
        if node.is_group {
            let label = layout_label(Some(&node.label), false, r, o, size, 4000., 0)?;
            view = View::base(node, label, 0., 0., size);
            if children[i] > 0 {
                cn = CNode::new(0., 0., -1);
                cn.is_group = true;
                cn.pad_x = 20.;
                cn.pad_top = if view.label.height > 0. {
                    view.label.height + 16.
                } else {
                    20.
                };
                cn.pad_bottom = 20.;
                cn.min_w = view.label.width + 40. - 12.
                    + if crossed[i] && !tight {
                        (view.label.width + 24.).js_min(240.)
                    } else {
                        0.
                    };
            } else {
                view.w = (view.label.width + 40.).js_max(80.);
                view.h = view.label.height + 40.;
                cn = CNode::new(view.w, view.h, -1)
            }
        } else if let Some(at) = node.class_node {
            view = class_view(
                db,
                &mut cache,
                node,
                &db.classes[at].1,
                at,
                live,
                size,
                tiny,
                tiny_h,
                hide_empty,
                o,
                r,
            )?;
            cn = CNode::new(view.w, view.h, -1)
        } else if node.shape == "note" {
            let label = layout_label(Some(&node.label), true, r, o, small, 260., 0)?;
            let w = (label.width + 24.).js_max(40.);
            let h = label.height + 16.;
            view = View::base(node, label, w, h, small);
            cn = CNode::new(w, h, -1)
        } else {
            let label = layout_label(Some(&node.label), false, r, o, small, 200., 0)?;
            let w = label.width.js_max(12.) + 4.;
            let h = label.height.js_max(12.);
            view = View::base(node, label, w, h, small);
            cn = CNode::new(w, h, -1)
        }
        cn.parent = view.node.parent;
        cn.seq = i;
        views.push(view);
        cnodes.push(cn)
    }
    let mut cedges: Vec<CEdge> = vec![];
    let mut drawn: Vec<GraphEdge> = vec![];
    let mut edge_labels = vec![];
    let mut start_room = vec![];
    let mut end_room = vec![];
    let mut crowd = vec![0f64; n * 4];
    for edge in graph.edges {
        if edge.source < 0 || edge.target < 0 {
            continue;
        }
        let label = layout_label(edge.label.as_deref(), true, r, o, small, 200., 0)?;
        let sw = if edge.start_label_right.is_empty() {
            0.
        } else {
            measure(r, o, &decode_entities(&edge.start_label_right)?, tiny, 0)?
        };
        let ew = if edge.end_label_left.is_empty() {
            0.
        } else {
            measure(r, o, &decode_entities(&edge.end_label_left)?, tiny, 0)?
        };
        let src = edge.source as usize;
        let dst = edge.target as usize;
        if src == dst {
            cnodes[src].w += 2.
                * (26.
                    + if label.width > 0. {
                        label.width + 12.
                    } else {
                        0.
                    })
        } else {
            for (item, slot, room) in [(src, 0, sw), (dst, 2, ew)] {
                crowd[item * 4 + slot] += 1.;
                if room > crowd[item * 4 + slot + 1] {
                    crowd[item * 4 + slot + 1] = room;
                }
            }
        }
        start_room.push(sw);
        end_room.push(ew);
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
        edge_labels.push(label)
    }
    if matches!(root_dir, Dir::TB | Dir::BT) {
        for i in 0..n {
            if views[i].node.class_node.is_none() {
                continue;
            }
            let mut need: f64 = 0.;
            for slot in [0, 2] {
                let count = crowd[i * 4 + slot];
                let room = crowd[i * 4 + slot + 1];
                if count > 1. {
                    need =
                        need.js_max((count - 1.) * if room > 0. { room + 24. } else { 14. } + 24.)
                }
            }
            need = need.js_min(480.);
            if need > views[i].w {
                cnodes[i].w += need - views[i].w;
                views[i].w = need
            }
        }
    }
    let lettered = start_room.iter().chain(&end_room).any(|w| *w > 0.);
    let layout = compound_layout(
        &mut cnodes,
        &mut cedges,
        root_dir,
        &LayeredOptions {
            node_sep: number(conf, "nodeSpacing", 40.),
            edge_sep: 16.,
            rank_sep: number(conf, "rankSpacing", if lettered { 72. } else { 56. }),
            port_sep: 20.,
            apart: true,
            tracks: false,
            straight: false,
            head_room: 0.,
        },
    );
    let mut sides: BTreeMap<usize, Vec<End>> = BTreeMap::new();
    let mut starts = vec![];
    let mut ends = vec![];
    for (i, ce) in cedges.iter_mut().enumerate() {
        let routed = ce.src != ce.dst && ce.route.len() >= 6;
        let route_end = ce.route.len().saturating_sub(3);
        let mut attach = |at: usize, item: usize, room: f64| {
            let c = &cnodes[item];
            let side = end_side(&ce.route, at, c.x, c.y, c.w, c.h);
            if side < 0 {
                return None;
            }
            let slack = (c.w - views[item].w) / 2.;
            if side == 1 {
                ce.route[at] -= slack
            } else if side == 3 {
                ce.route[at] += slack
            }
            let across = (side & 1) as usize;
            let toward = ce.route[if at == 0 { 3 + across } else { at - 3 + across }];
            let bend = toward - ce.route[at + across];
            let end = End {
                edge: i,
                at,
                toward,
                order: i,
                room,
                side,
                after: if across != 0 { bend > 0. } else { bend >= 0. },
            };
            sides
                .entry(item * 4 + side as usize)
                .or_default()
                .push(end.clone());
            Some(end)
        };
        if routed {
            starts.push(attach(0, ce.src, start_room[i]));
            ends.push(attach(route_end, ce.dst, end_room[i]))
        } else {
            starts.push(None);
            ends.push(None)
        }
    }
    let on_border = |item: usize, x: f64, y: f64| {
        let mut g = cnodes[item].parent;
        let mut hops = 0;
        while g >= 0 && hops < 10000 {
            let c = &cnodes[g as usize];
            let dx = (x - c.x).abs();
            let dy = (y - c.y).abs();
            if ((dy - c.h / 2.).abs() < 0.5 && dx <= c.w / 2. + 0.5)
                || ((dx - c.w / 2.).abs() < 0.5 && dy <= c.h / 2. + 0.5)
            {
                return true;
            }
            g = c.parent;
            hops += 1
        }
        false
    };
    for (key, list) in &mut sides {
        if list.len() < 2 {
            continue;
        }
        let item = key >> 2;
        let across = key & 1;
        let length = if across != 0 {
            views[item].h
        } else {
            views[item].w
        };
        list.sort_by(|a, b| a.toward.total_cmp(&b.toward).then(a.order.cmp(&b.order)));
        let mut gaps = vec![0.; list.len()];
        let mut total = 0.;
        for j in 0..list.len() - 1 {
            let a = &list[j];
            let b = &list[j + 1];
            let mut gap = if across != 0 { 20. } else { 24. };
            if a.room > 0. && a.after {
                gap += if across != 0 { tiny_h + 6. } else { a.room }
            }
            if b.room > 0. && !b.after {
                gap += if across != 0 { tiny_h + 6. } else { b.room }
            }
            gaps[j] = gap;
            total += gap
        }
        let scale = ((length - 24.) / total).js_min(1.);
        if scale <= 0. {
            continue;
        }
        let mut at = if across != 0 {
            cnodes[item].y
        } else {
            cnodes[item].x
        } - total * scale / 2.;
        for (j, e) in list.iter().enumerate() {
            let route = &mut cedges[e.edge].route;
            let end = e.at;
            let step = if end == 0 { 3isize } else { -3 };
            let first = end as isize + step;
            let mut border = None;
            let mut k = first;
            while k >= 0
                && (k as usize) < route.len()
                && (route[k as usize + across] - route[first as usize + across]).abs() < 0.5
            {
                if on_border(item, route[k as usize], route[k as usize + 1]) {
                    border = Some(k);
                    break;
                }
                k += step
            }
            if let Some(border) = border {
                if (route[first as usize + across] - at).abs() <= 24. {
                    let mut k = first;
                    while k != border + step {
                        route[k as usize + across] = at;
                        k += step
                    }
                }
            }
            route[end + across] = at;
            at += gaps[j] * scale
        }
    }
    for (i, ce) in cedges.iter_mut().enumerate() {
        for (end, room, first) in [
            (&starts[i], start_room[i], true),
            (&ends[i], end_room[i], false),
        ] {
            let Some(e) = end else { continue };
            if room <= 0. {
                continue;
            }
            let across = (e.side & 1) as usize;
            let at = if first { 0 } else { ce.route.len() - 3 };
            let along = at + 1 - across;
            let reach = (ce.route[if first {
                3 + 1 - across
            } else {
                at - 3 + 1 - across
            }] - ce.route[along])
                .abs();
            let length = if across != 0 { room + 10. } else { tiny_h + 6. }.js_min(reach * 0.35);
            if length < 4. {
                continue;
            }
            let mut point = [ce.route[at], ce.route[at + 1], ce.route[at + 2]];
            point[1 - across] += if e.side == 0 || e.side == 3 { -1. } else { 1. } * length;
            ce.route.splice(if first { 3..3 } else { at..at }, point);
        }
    }
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (0f64, 0f64, layout.width, layout.height);
    let mut cards = vec![];
    let mut card =
        |raw: &str, width: f64, x: f64, y: f64, side: i32, after: bool| -> Result<(), PeleError> {
            if raw.is_empty() {
                return Ok(());
            }
            let (mut cx, mut cy) = (x, y);
            if side & 1 != 0 {
                cx += if side == 3 { -1. } else { 1. } * (5. + width / 2.);
                cy += if after { 1. } else { -1. } * (7. + tiny_h / 2.)
            } else {
                cx += if after { 1. } else { -1. } * (9. + width / 2.);
                cy += if side == 0 { -1. } else { 1. } * (3. + tiny_h / 2.)
            }
            cards.push((decode_entities(raw)?, cx, cy));
            min_x = min_x.js_min(cx - width / 2.);
            max_x = max_x.js_max(cx + width / 2.);
            min_y = min_y.js_min(cy - tiny_h / 2.);
            max_y = max_y.js_max(cy + tiny_h / 2.);
            Ok(())
        };
    let mut loop_seen: HashMap<usize, usize> = HashMap::new();
    let mut loop_at = vec![0usize; cedges.len()];
    for (i, ce) in cedges.iter().enumerate() {
        let e = &drawn[i];
        if ce.src == ce.dst {
            let k = *loop_seen.get(&ce.src).unwrap_or(&0);
            loop_seen.insert(ce.src, k + 1);
            loop_at[i] = k;
            let c = &cnodes[ce.src];
            let v = &views[ce.src];
            let x = c.x + v.w / 2.;
            let spread = (v.h / 2. - 4.).js_min(8. + k as f64 * 6.);
            card(
                &e.start_label_right,
                start_room[i],
                x,
                c.y - spread,
                1,
                false,
            )?;
            card(&e.end_label_left, end_room[i], x, c.y + spread, 1, true)?
        } else if ce.route.len() >= 6 {
            let last = ce.route.len() - 3;
            card(
                &e.start_label_right,
                start_room[i],
                ce.route[0],
                ce.route[1],
                starts[i].as_ref().map_or(2, |e| e.side),
                starts[i].as_ref().is_none_or(|e| e.after),
            )?;
            card(
                &e.end_label_left,
                end_room[i],
                ce.route[last],
                ce.route[last + 1],
                ends[i].as_ref().map_or(0, |e| e.side),
                ends[i].as_ref().is_none_or(|e| e.after),
            )?
        }
    }
    let ox = pad - min_x;
    let oy = pad - min_y;
    let mut width = (max_x - min_x + 2. * pad).ceil();
    let mut height = (max_y - min_y + 2. * pad).ceil();
    let mut links = vec![];
    let mut crossings = Crossings::new();
    if views.first().is_some_and(|v| v.node.is_group) {
        for e in &cedges {
            mark_crossings(&mut crossings, &e.route)
        }
    }
    let mut clusters = String::new();
    let mut struck_titles = String::new();
    let mut nodes_out = String::new();
    let parents: Vec<i32> = views.iter().map(|v| v.node.parent as i32).collect();
    for (i, v) in views.iter().enumerate() {
        let c = &cnodes[i];
        let node = &v.node;
        let x = c.x + ox;
        let y = c.y + oy;
        let id = esc_text(&node.id)?;
        let w = if c.is_group { c.w } else { v.w };
        let h = if c.is_group { c.h } else { v.h };
        if node.is_group {
            let cb = ClusterBox {
                x: c.x,
                y: c.y,
                w: c.w,
                h: c.h,
            };
            let tx = if c.is_group {
                cluster_title_x(&cb, v.label.width, &crossings, 20.)
            } else {
                c.x
            };
            let ty = y - h / 2. + 8. + v.label.height / 2.;
            let attrs = " class=\"pele-cluster-label\" fill=\"var(--_m)\"";
            let struck = c.is_group && title_crossed(&cb, v.label.width, tx, &crossings);
            if struck {
                struck_titles += &struck_title(&v.label, tx + ox, ty, attrs, &id, &parents, i, r)?
            }
            clusters+=&format!("<g class=\"pele-cluster pele-namespace\" data-id=\"{id}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{RADIUS}\" fill=\"var(--_a)\" fill-opacity=\"0.5\" stroke=\"var(--_b)\"/>{}</g>",num(x-w/2.),num(y-h/2.),num(w),num(h),if struck{String::new()}else{label_svg_with_resources(&v.label,tx+ox,ty,attrs,r)?});
            continue;
        }
        let at = format!(" transform=\"translate({},{})\"", num(x), num(y));
        if let Some(at_class) = node.class_node {
            nodes_out += &class_svg(
                v,
                &db.classes[at_class].1,
                at_class,
                live,
                &id,
                &at,
                size,
                tiny,
                tiny_h,
                o,
                r,
                &mut links,
            )?
        } else if node.shape == "note" {
            let a = w / 2.;
            let t = -h / 2.;
            nodes_out+=&format!("<g class=\"pele-node pele-note\" data-id=\"{id}\"{at}><path d=\"M{},{}H{}L{},{}V{}H{}Z\" fill=\"var(--_a)\" stroke=\"var(--_b)\" stroke-linejoin=\"round\"/><path d=\"M{},{}V{}H{}\" fill=\"none\" stroke=\"var(--_b)\" stroke-linejoin=\"round\"/>{}</g>",num(-a),num(t),num(a-8.),num(a),num(t+8.),num(-t),num(-a),num(a-8.),num(t),num(t+8.),num(a),label_svg_with_resources(&v.label,0.,0.,&format!(" class=\"pele-label\" font-size=\"{}\"",num(small)),r)?)
        } else {
            nodes_out += &format!(
                "<g class=\"pele-node pele-interface\" data-id=\"{id}\"{at}>{}</g>",
                label_svg_with_resources(
                    &v.label,
                    0.,
                    0.,
                    &format!(" class=\"pele-label\" font-size=\"{}\"", num(small)),
                    r
                )?
            )
        }
    }
    let mut edges_out = String::new();
    let mut labels_out = String::new();
    for (i, ce) in cedges.iter_mut().enumerate() {
        let e = &drawn[i];
        let label = &edge_labels[i];
        let st = markers::trim(e.arrow_type_start);
        let et = markers::trim(e.arrow_type_end);
        let path;
        if ce.src == ce.dst {
            let c = &cnodes[ce.src];
            let v = &views[ce.src];
            let k = loop_at[i] as f64;
            let x = c.x + ox + v.w / 2.;
            let y = c.y + oy;
            let spread = (v.h / 2. - 4.).js_min(8. + k * 6.);
            let reach = 26. + k * 8.;
            path = EdgePath {
                d: format!(
                    "M{},{}C{},{} {},{} {},{}",
                    num(x + st),
                    num(y - spread),
                    num(x + reach + st),
                    num(y - spread),
                    num(x + reach + et),
                    num(y + spread),
                    num(x + et),
                    num(y + spread)
                ),
                sx: x,
                sy: y - spread,
                sdx: -1.,
                sdy: 0.,
                ex: x,
                ey: y + spread,
                edx: -1.,
                edy: 0.,
            };
            ce.label_x = c.x + v.w / 2. + reach + 4. + ce.label_w / 2.;
            ce.label_y = c.y;
        } else {
            for p in ce.route.chunks_exact_mut(3) {
                p[0] += ox;
                p[1] += oy
            }
            path = route_path(&ce.route, None, st, et, None, None)
        }
        let attrs = if e.pattern == "dashed" {
            " stroke-dasharray=\"6 4\""
        } else if e.pattern == "dotted" {
            " stroke-dasharray=\"2 4\""
        } else {
            ""
        };
        edges_out += &format!(
            "<g class=\"pele-edge{}\" data-id=\"{}\"><path d=\"{}\"{attrs}/>{}{}</g>",
            if e.pattern == "dotted" {
                " pele-note-edge"
            } else {
                " pele-relation"
            },
            esc_text(&e.id)?,
            path.d,
            markers::draw(
                e.arrow_type_start,
                path.sx,
                path.sy,
                path.sdx,
                path.sdy,
                LINE
            ),
            markers::draw(e.arrow_type_end, path.ex, path.ey, path.edx, path.edy, LINE)
        );
        if label.width > 0. {
            labels_out += &edge_label_svg(&e.id, label, ce.label_x + ox, ce.label_y + oy, "", r)?
        }
    }
    let mut cards_out = String::new();
    for (text, x, y) in cards {
        cards_out += &format!(
            "<text class=\"pele-cardinality\" x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}</text>",
            num(x + ox),
            num(y + oy + tiny * 0.35),
            esc(&text)
        )
    }
    let mut content = String::new();
    if !clusters.is_empty() {
        content += &format!("<g class=\"pele-clusters\">{clusters}</g>")
    }
    if !edges_out.is_empty() {
        content+=&format!("<g class=\"pele-edges\" fill=\"none\" stroke=\"{LINE}\" stroke-linecap=\"round\">{edges_out}</g>")
    }
    if !struck_titles.is_empty() {
        content += &format!("<g class=\"pele-cluster-titles\">{struck_titles}</g>")
    }
    if !labels_out.is_empty() {
        content += &format!(
            "<g class=\"pele-edge-labels\" font-size=\"{}\">{labels_out}</g>",
            num(small)
        )
    }
    if !cards_out.is_empty() {
        content += &format!(
            "<g class=\"pele-cardinalities\" font-size=\"{}\" fill=\"var(--_m)\">{cards_out}</g>",
            num(tiny)
        )
    }
    content += &format!("<g class=\"pele-nodes\">{nodes_out}</g>");
    let current_title;
    let title_text = if let Some(live) = live.as_deref_mut() {
        current_title = live.metadata("title")?;
        current_title.as_deref()
    } else {
        db.title.as_deref()
    };
    let title = layout_label(title_text, false, r, o, size, 4000., 1)?;
    if title.height > 0. {
        let shift = title.height + 12.;
        let full = width.js_max((title.width + 2. * pad).ceil());
        content = format!(
            "{}<g transform=\"translate({},{})\">{content}</g>",
            label_svg_with_resources(
                &title,
                full / 2.,
                pad + title.height / 2.,
                " class=\"pele-title\" font-weight=\"var(--_tw)\"",
                r
            )?,
            num((full - width) / 2.),
            num(shift)
        );
        width = full;
        height = (height + shift).ceil()
    }
    let current_accessibility;
    let (acc_title, acc_descr) = if let Some(live) = live.as_deref_mut() {
        current_accessibility = (live.metadata("accTitle")?, live.metadata("accDescr")?);
        (
            current_accessibility.0.as_deref(),
            current_accessibility.1.as_deref(),
        )
    } else {
        (db.acc_title.as_deref(), db.acc_descr.as_deref())
    };
    let svg = document(
        "class", width, height, size, o, acc_title, acc_descr, &content,
    )?;
    Ok(Rendered {
        svg,
        width,
        height,
        links,
    })
}
fn class_svg(
    v: &View,
    cls: &ClassNode,
    index: usize,
    live: &mut Option<&mut dyn LiveMembers>,
    id: &str,
    at: &str,
    size: f64,
    tiny: f64,
    tiny_h: f64,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    links: &mut Vec<crate::compat::LinkInfo>,
) -> Result<String, PeleError> {
    let w = v.w;
    let h = v.h;
    let style = &v.style;
    let left = -w / 2.;
    let top = -h / 2.;
    let member_size = round(v.font_size * 0.875);
    let line_h = round(member_size * 1.5);
    let mut body=format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{RADIUS}\" fill=\"var(--_s)\" stroke=\"var(--_b)\"{}/>",num(left),num(top),num(w),num(h),style.shape);
    let first = top + v.title_h;
    let second = first + v.attributes_h;
    if h > v.title_h {
        let stroke = style
            .stroke
            .as_ref()
            .map_or(String::new(), |s| format!(" style=\"stroke:{}\"", esc(s)));
        body+=&format!("<path class=\"pele-divider\" d=\"M{},{}H{}M{},{}H{}\" fill=\"none\" stroke=\"var(--_b)\"{stroke}/>",num(left),num(first),num(-left),num(left),num(second),num(-left))
    }
    let mut y = top + 8.;
    for a in &v.annotations {
        body+=&format!("<text class=\"pele-annotation\" x=\"0\" y=\"{}\" text-anchor=\"middle\" font-size=\"{}\" fill=\"var(--_m)\">{}</text>",num(y+tiny_h/2.+tiny*0.35),num(tiny),esc(a));
        y += tiny_h
    }
    let title_size = if v.font_size == size {
        String::new()
    } else {
        format!(" font-size=\"{}\"", num(v.font_size))
    };
    body += &label_svg_with_resources(
        &v.label,
        0.,
        (y + first - 8.) / 2.,
        &format!(
            " class=\"pele-label\"{title_size} font-weight=\"var(--_hw)\"{}",
            style.text
        ),
        r,
    )?;
    let lines = |list: &[Line], kind: &str, from: f64| {
        let mut out = String::new();
        let mut baseline = from + 6. + line_h / 2. + member_size * 0.35;
        for l in list {
            let mark = if l.classifier == "$" {
                format!(
                    " pele-static\"{}",
                    if style.text.is_empty() {
                        " style=\"text-decoration:underline\"".into()
                    } else {
                        format!(
                            "{}text-decoration:underline\"",
                            &style.text[..style.text.len() - 1]
                        )
                    }
                )
            } else if l.classifier == "*" {
                format!(" pele-abstract\" font-style=\"italic\"{}", style.text)
            } else {
                format!("\"{}", style.text)
            };
            out += &format!(
                "<text class=\"pele-member pele-{kind}{mark} x=\"{}\" y=\"{}\">{}</text>",
                num(left + 12.),
                num(baseline),
                esc(&l.text)
            );
            baseline += line_h
        }
        out
    };
    if !v.attributes.is_empty() || !v.methods.is_empty() {
        body += &format!(
            "<g class=\"pele-members\" font-size=\"{}\">{}{}</g>",
            num(member_size),
            lines(&v.attributes, "attribute", first),
            lines(&v.methods, "method", second)
        )
    }
    let current;
    let cls = if let Some(live) = live.as_deref_mut() {
        current = live.paint(index)?;
        &current
    } else {
        cls
    };
    if let Some(t) = cls.tooltip.as_deref().filter(|s| !s.is_empty()) {
        body = format!("<title>{}</title>{body}", esc_text(t)?)
    }
    if let Some(link) = cls.link.as_deref().filter(|s| !s.is_empty()) {
        let target = cls
            .link_target
            .as_deref()
            .filter(|t| matches!(*t, "_self" | "_blank" | "_parent" | "_top"))
            .map_or(String::new(), |t| format!(" target=\"{t}\""));
        body = linked(&body, link, &cls.id, links, o, &target)
    }
    let css = cls
        .css_classes
        .strip_prefix("default")
        .map_or(cls.css_classes.as_str(), |s| {
            s.strip_prefix(' ').unwrap_or(s)
        });
    Ok(format!(
        "<g class=\"pele-node pele-class{}\" data-id=\"{id}\"{at}>{body}</g>",
        class_names(css)
    ))
}
