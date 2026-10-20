//! Pele sankey/render.ts measured labels, bands, configuration and responsive fit.
//! 458322a5 (MIT, kepano).
use super::{
    indexed::Input,
    layout::{self, max, min, Alignment, Options},
};
use crate::compat::number_math::round;
use crate::compat::{
    svg::{self, edges::theme::resolve_style, esc, esc_text, js_num, label_svg, num, series_color},
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
pub(super) fn render(
    m: &impl Input,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
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
fn draw(
    m: &impl Input,
    c: &Object,
    o: &RenderSettings,
    r: &mut dyn RenderResources,
    chart_width: f64,
) -> Result<Rendered, PeleError> {
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
    let placed = layout::layout(
        m,
        Options {
            width: chart_width,
            height: chart_height,
            node_width,
            node_padding: number(c, "nodePadding", 12., 0., 200.) + if show { 15. } else { 0. },
            label_height: small * 1.15,
            alignment,
        },
    )?;
    let nodes = placed.nodes;
    let links = placed.links;
    let custom = match c.get("nodeColors") {
        Some(Value::Object(v)) => Some(v),
        _ => None,
    };
    let node_style = |i: usize| fill(custom.and_then(|v| v.get(&m.nodes()[i].id)));
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
        let id = &m.nodes()[i].id;
        let height = max(n.height, 1.);
        nodes_out.push_str(&format!("<rect class=\"pele-sankey-node\" data-id=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"{}\"{}/>",esc_text(id)?,num(n.x),num(n.y),num(node_width),num(height),num(min(2.,height/2.)),series_color(i),node_style(i)));
        let value = if show {
            format!("{prefix}{}{suffix}", js_num(rounded(n.value)))
        } else {
            String::new()
        };
        let after = if outlined {
            n.layer >= central
        } else {
            n.x < chart_width / 2.
        };
        let text_width = measure(
            r,
            o,
            &if value.is_empty() {
                id.clone()
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
        let position = format!(
            " x=\"{}\" y=\"{}\" text-anchor=\"{}\"",
            num(x),
            num(y + small * 0.35),
            if after { "start" } else { "end" }
        );
        let text = format!(
            "{}{}",
            esc(id),
            if value.is_empty() {
                String::new()
            } else {
                format!(
                    " <tspan class=\"pele-sankey-value\" fill=\"var(--_m)\">{}</tspan>",
                    esc(&value)
                )
            }
        );
        labels_out.push_str(&format!(
            "<g class=\"pele-label\" data-id=\"{}\">",
            esc_text(id)?
        ));
        if outlined {
            labels_out.push_str(&format!("<text{position} fill=\"none\" stroke=\"var(--_bg)\" stroke-width=\"3\" stroke-linejoin=\"round\">{}{}</text>",esc(id),if value.is_empty(){String::new()}else{format!(" {}",esc(&value))}));
        }
        labels_out.push_str(&format!("<text{position}>{text}</text></g>"));
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
        let mid = num((x0 + x1) / 2.);
        let from = esc_text(&m.nodes()[link.source].id)?;
        let to = esc_text(&m.nodes()[link.target].id)?;
        links_out.push_str(&format!("<path class=\"pele-sankey-link\" data-id=\"{from}-&gt;{to}\" d=\"M{},{}C{mid},{} {mid},{} {},{}L{},{}C{mid},{} {mid},{} {},{}Z\" fill=\"{}\"{}><title>{from} → {to}: {}{}{}</title></path>",num(x0),num(link.source_y-half),num(link.source_y-half),num(link.target_y-half),num(x1),num(link.target_y-half),num(x1),num(link.target_y+half),num(link.target_y+half),num(link.source_y+half),num(x0),num(link.source_y+half),series_color(end),if fixed.is_empty(){node_style(end)}else{fixed.clone()},esc_text(&prefix)?,js_num(rounded(link.value)),esc_text(&suffix)?));
    }
    let room = o
        .max_width
        .filter(|v| *v > 0.)
        .map_or(4000., |v| max(80., v - 2. * pad));
    let title = layout_label(m.title(), false, r, o, size, room, 1)?;
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
    let body=format!("{}<g transform=\"translate({},{})\"><g class=\"pele-sankey-links\" fill-opacity=\"0.4\">{links_out}</g><g class=\"pele-sankey-nodes\">{nodes_out}</g><g class=\"pele-labels\" font-size=\"{}\">{labels_out}</g></g>",label_svg(&title,width/2.,pad+title.height/2.," class=\"pele-title\" font-weight=\"var(--_tw)\""),num(ox),num(oy),js_num(small));
    let svg = svg::document(
        "sankey",
        width,
        height,
        size,
        o,
        m.acc_title(),
        m.acc_descr(),
        &body,
    )?;
    Ok(Rendered {
        svg,
        width,
        height,
        links: vec![],
    })
}
