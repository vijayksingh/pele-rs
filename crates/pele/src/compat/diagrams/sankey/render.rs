//! Pele sankey/render.ts measured labels, bands, configuration and responsive fit.
//! 458322a5 (MIT, kepano).
use super::{
    indexed::Input,
    layout::{self, max, min, Alignment, Options},
};
use crate::compat::number_math::round;
use crate::compat::{
    svg::{
        self, edges::theme::resolve_style, esc_text, js_num, label_svg, num, series_color, SvgSink,
    },
    text::{layout_label, measure},
    Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
fn as_text(v: &Value) -> Option<&str> {
    match v {
        Value::String(s) => Some(s),
        _ => None,
    }
}
fn number(conf: &Object, key: &str, fallback: f64, least: f64, most: f64) -> f64 {
    match conf.get(key) {
        Some(Value::Number(n)) if *n >= least => min(*n, most),
        _ => fallback,
    }
}
fn text(conf: &Object, key: &str) -> String {
    match conf.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) if n.is_finite() => js_num(*n),
        _ => String::new(),
    }
}
fn rounded(n: f64) -> f64 {
    if n.abs() < 1e15 {
        round(n * 100.) / 100.
    } else {
        n
    }
}
fn fill(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => resolve_style(&[format!("fill:{s}")]).shape,
        _ => String::new(),
    }
}
pub(super) trait PhaseInput {
    type Error: From<PeleError>;
    type Label;
    type Text: AsRef<str>;
    fn layout(&mut self, options: Options) -> Result<layout::Layout, Self::Error>;
    fn label(&mut self, index: usize) -> Result<Self::Label, Self::Error>;
    fn text(&mut self, label: &Self::Label) -> Result<Self::Text, Self::Error>;
    fn color_key(&mut self, index: usize) -> Result<Self::Text, Self::Error>;
    fn title(&mut self) -> Result<Option<Self::Text>, Self::Error>;
    fn accessibility(&mut self, description: bool) -> Result<Option<Self::Text>, Self::Error>;
}
struct Typed<'a, M>(&'a M);
impl<'a, M: Input> PhaseInput for Typed<'a, M> {
    type Error = PeleError;
    type Label = &'a str;
    type Text = &'a str;
    fn layout(&mut self, options: Options) -> Result<layout::Layout, PeleError> {
        layout::layout(self.0, options)
    }
    fn label(&mut self, index: usize) -> Result<Self::Label, PeleError> {
        Ok(self.0.node(index).id)
    }
    fn text(&mut self, label: &Self::Label) -> Result<Self::Text, PeleError> {
        Ok(*label)
    }
    fn color_key(&mut self, index: usize) -> Result<Self::Text, PeleError> {
        self.label(index)
    }
    fn title(&mut self) -> Result<Option<Self::Text>, PeleError> {
        Ok(self.0.title())
    }
    fn accessibility(&mut self, description: bool) -> Result<Option<Self::Text>, PeleError> {
        Ok(if description {
            self.0.acc_descr()
        } else {
            self.0.acc_title()
        })
    }
}
pub(super) fn render(
    m: &impl Input,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_phases(&mut Typed(m), c, o, r)
}
pub(super) fn render_phases<P: PhaseInput>(
    m: &mut P,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    let empty = Object::default();
    let conf = match c.get("sankey") {
        Some(Value::Object(c)) => c,
        _ => &empty,
    };
    let natural = number(conf, "width", 600., 1., 10000.);
    let mut result = draw(m, conf, o, r, natural)?;
    let Some(available) = o.max_width.filter(|v| *v > 0.) else {
        return Ok(result);
    };
    if result.width <= available {
        return Ok(result);
    }
    let floor = min(160., natural);
    let mut plot = natural;
    result = draw(m, conf, o, r, plot)?;
    for _ in 0..3 {
        if !(result.width > available && plot > floor) {
            break;
        }
        plot = max(floor, plot - (result.width - available));
        result = draw(m, conf, o, r, plot)?;
    }
    Ok(result)
}
fn draw<P: PhaseInput>(
    m: &mut P,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    chart_width: f64,
) -> Result<Rendered, P::Error> {
    let size = o.font_size.unwrap_or(16.);
    let small = round(size * 0.875);
    let pad = o.padding.unwrap_or(8.);
    let chart_height = number(c, "height", 400., 1., 10000.);
    let node_width = min(number(c, "nodeWidth", 10., 1., 200.), chart_width);
    let show = c.get("showValues") != Some(&Value::Bool(false));
    let prefix = text(c, "prefix");
    let suffix = text(c, "suffix");
    let outlined = c.get("labelStyle").and_then(as_text) == Some("outlined");
    let alignment = match c.get("nodeAlignment").and_then(as_text) {
        Some("left") => Alignment::Left,
        Some("right") => Alignment::Right,
        Some("center") => Alignment::Center,
        _ => Alignment::Justify,
    };
    let color = c.get("linkColor").and_then(as_text).unwrap_or("gradient");
    let placed = m.layout(Options {
        width: chart_width,
        height: chart_height,
        node_width,
        node_padding: number(c, "nodePadding", 12., 0., 200.) + if show { 15. } else { 0. },
        label_height: small * 1.15,
        alignment,
    })?;
    let nodes = placed.nodes;
    let links = placed.links;
    let custom = match c.get("nodeColors") {
        Some(Value::Object(v)) => Some(v),
        _ => None,
    };
    fn node_style<P: PhaseInput>(
        m: &mut P,
        custom: Option<&Object>,
        i: usize,
    ) -> Result<String, P::Error> {
        Ok(match custom {
            Some(custom) => fill(custom.get(m.color_key(i)?.as_ref())),
            None => String::new(),
        })
    }
    let (mut central, mut most) = (0, 0.);
    for n in &nodes {
        if n.value > most {
            most = n.value;
            central = n.layer;
        }
    }
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (0., chart_width, 0., chart_height);
    let (mut labels_out, mut nodes_out) = (String::new(), String::new());
    for (i, n) in nodes.iter().enumerate() {
        let captured_label = m.label(i)?;
        let id = m.text(&captured_label)?;
        let id = id.as_ref();
        let height = max(n.height, 1.);
        let mut node = SvgSink::new(&mut nodes_out);
        node.raw("<rect class=\"pele-sankey-node\" data-id=\"");
        node.decoded_text(id)?;
        node.raw("\" x=\"");
        node.coordinate(n.x);
        node.raw("\" y=\"");
        node.coordinate(n.y);
        node.raw("\" width=\"");
        node.coordinate(node_width);
        node.raw("\" height=\"");
        node.coordinate(height);
        node.raw("\" rx=\"");
        node.coordinate(min(2., height / 2.));
        node.raw("\" fill=\"");
        node.raw(&series_color(i));
        node.raw("\"");
        node.raw(&node_style(m, custom, i)?);
        node.raw("/>");
        let mut value = String::new();
        if show {
            let mut text = SvgSink::new(&mut value);
            text.raw(&prefix);
            text.js_number(rounded(n.value));
            text.raw(&suffix);
        }
        let after = if outlined {
            n.layer >= central
        } else {
            n.x < chart_width / 2.
        };
        let text_width = measure(
            r,
            o,
            &if value.is_empty() {
                id.to_owned()
            } else {
                format!("{id} {value}")
            },
            small,
            0,
        )?;
        let x = if after {
            n.x + node_width + 6.
        } else {
            n.x - 6.
        };
        let y = n.y + n.height / 2.;
        min_x = min(min_x, if after { x } else { x - text_width });
        max_x = max(max_x, if after { x + text_width } else { x });
        min_y = min(min_y, y - small * 0.75);
        max_y = max(max_y, y + small * 0.75);
        let id_after_width = m.text(&captured_label)?;
        let id = id_after_width.as_ref();
        let mut label = SvgSink::new(&mut labels_out);
        label.raw("<g class=\"pele-label\" data-id=\"");
        label.decoded_text(id)?;
        label.raw("\">");
        if outlined {
            label.raw("<text");
            position(&mut label, x, y + small * 0.35, after);
            label.raw(" fill=\"none\" stroke=\"var(--_bg)\" stroke-width=\"3\" stroke-linejoin=\"round\">");
            label.escaped(id);
            if !value.is_empty() {
                label.raw(" ");
                label.escaped(&value);
            }
            label.raw("</text>");
        }
        label.raw("<text");
        position(&mut label, x, y + small * 0.35, after);
        label.raw(">");
        label.escaped(id);
        if !value.is_empty() {
            label.raw(" <tspan class=\"pele-sankey-value\" fill=\"var(--_m)\">");
            label.escaped(&value);
            label.raw("</tspan>");
        }
        label.raw("</text></g>");
    }
    let fixed = if matches!(color, "gradient" | "source" | "target") {
        String::new()
    } else {
        fill(Some(&Value::String(color.into())))
    };
    let mut links_out = String::new();
    for link in links {
        let end = if color == "target" {
            link.target
        } else {
            link.source
        };
        let half = max(link.thickness, 1.) / 2.;
        let x0 = nodes[link.source].x + node_width;
        let x1 = nodes[link.target].x;
        // Reuse this four-times-emitted coordinate rather than reformatting it.
        let mid = num((x0 + x1) / 2.);
        // These escaped fragments occur in both the attribute and title.
        // Decode in the original order before appending the element.
        let source_label = m.label(link.source)?;
        let from = esc_text(m.text(&source_label)?.as_ref())?;
        let target_label = m.label(link.target)?;
        let to = esc_text(m.text(&target_label)?.as_ref())?;
        let mut path = SvgSink::new(&mut links_out);
        path.raw("<path class=\"pele-sankey-link\" data-id=\"");
        path.raw(&from);
        path.raw("-&gt;");
        path.raw(&to);
        path.raw("\" d=\"M");
        path.coordinate(x0);
        path.raw(",");
        path.coordinate(link.source_y - half);
        path.raw("C");
        path.raw(&mid);
        path.raw(",");
        path.coordinate(link.source_y - half);
        path.raw(" ");
        path.raw(&mid);
        path.raw(",");
        path.coordinate(link.target_y - half);
        path.raw(" ");
        path.coordinate(x1);
        path.raw(",");
        path.coordinate(link.target_y - half);
        path.raw("L");
        path.coordinate(x1);
        path.raw(",");
        path.coordinate(link.target_y + half);
        path.raw("C");
        path.raw(&mid);
        path.raw(",");
        path.coordinate(link.target_y + half);
        path.raw(" ");
        path.raw(&mid);
        path.raw(",");
        path.coordinate(link.source_y + half);
        path.raw(" ");
        path.coordinate(x0);
        path.raw(",");
        path.coordinate(link.source_y + half);
        path.raw("Z\" fill=\"");
        path.raw(&series_color(end));
        path.raw("\"");
        if fixed.is_empty() {
            path.raw(&node_style(m, custom, end)?);
        } else {
            path.raw(&fixed);
        }
        path.raw("><title>");
        path.raw(&from);
        path.raw(" → ");
        path.raw(&to);
        path.raw(": ");
        path.decoded_text(&prefix)?;
        path.js_number(rounded(link.value));
        path.decoded_text(&suffix)?;
        path.raw("</title></path>");
    }
    let room = o
        .max_width
        .filter(|v| *v > 0.)
        .map_or(4000., |v| max(80., v - 2. * pad));
    let title_value = m.title()?;
    let title = layout_label(
        title_value.as_ref().map(AsRef::as_ref),
        false,
        r,
        o,
        size,
        room,
        1,
    )?;
    let title_height = if title.height > 0. {
        title.height + 12.
    } else {
        0.
    };
    let inner = max(max_x - min_x, title.width);
    let ox = pad - min_x + (inner - (max_x - min_x)) / 2.;
    let oy = pad + title_height - min_y;
    let width = (inner + 2. * pad).ceil();
    let height = (max_y - min_y + title_height + 2. * pad).ceil();
    // Paint order differs from measurement order; retain all three layers.
    let mut body = label_svg(
        &title,
        width / 2.,
        pad + title.height / 2.,
        " class=\"pele-title\" font-weight=\"var(--_tw)\"",
    );
    body.reserve(
        links_out
            .len()
            .saturating_add(nodes_out.len())
            .saturating_add(labels_out.len()),
    );
    let mut output = SvgSink::new(&mut body);
    output.raw("<g transform=\"translate(");
    output.coordinate(ox);
    output.raw(",");
    output.coordinate(oy);
    output.raw(")\"><g class=\"pele-sankey-links\" fill-opacity=\"0.4\">");
    output.raw(&links_out);
    output.raw("</g><g class=\"pele-sankey-nodes\">");
    output.raw(&nodes_out);
    output.raw("</g><g class=\"pele-labels\" font-size=\"");
    output.js_number(small);
    output.raw("\">");
    output.raw(&labels_out);
    output.raw("</g></g>");
    let acc_title = m.accessibility(false)?;
    let acc_descr = m.accessibility(true)?;
    let svg = svg::document(
        "sankey",
        width,
        height,
        size,
        o,
        acc_title.as_ref().map(AsRef::as_ref),
        acc_descr.as_ref().map(AsRef::as_ref),
        &body,
    )?;
    Ok(Rendered {
        svg,
        width,
        height,
        links: vec![],
    })
}

fn position(output: &mut SvgSink<'_>, x: f64, y: f64, after: bool) {
    output.raw(" x=\"");
    output.coordinate(x);
    output.raw("\" y=\"");
    output.coordinate(y);
    output.raw("\" text-anchor=\"");
    output.raw(if after { "start" } else { "end" });
    output.raw("\"");
}
