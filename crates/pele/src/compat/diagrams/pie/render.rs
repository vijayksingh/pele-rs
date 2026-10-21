//! Pele diagrams/pie/render.ts + common/fit-width.ts, 458322a5 (MIT, kepano).
use super::PieModel;
use crate::compat::number_math::{round, NumberMath};
use crate::compat::{
    svg::{self, esc_text, js_num, label_svg, num, series_color},
    text::{layout_label, measure},
    Object, PeleError, RenderResources, RenderSettings, Rendered, Value,
};
use std::f64::consts::PI;
fn point(angle: f64, radius: f64) -> String {
    format!(
        "{},{}",
        num(angle.sin() * radius),
        num(-angle.cos() * radius)
    )
}
fn slice(start: f64, end: f64, outer: f64, inner: f64) -> String {
    if end - start >= PI * 2. - 1e-9 {
        let ring = |r: f64, sweep: u8| {
            format!(
                "M0,{}A{},{} 0 1 {sweep} 0,{}A{},{} 0 1 {sweep} 0,{}Z",
                num(-r),
                num(r),
                num(r),
                num(r),
                num(r),
                num(r),
                num(-r)
            )
        };
        return ring(outer, 1)
            + &if inner > 0. {
                ring(inner, 0)
            } else {
                String::new()
            };
    }
    let large = u8::from(end - start > PI);
    if inner <= 0. {
        return format!(
            "M0,0L{}A{},{} 0 {large} 1 {}Z",
            point(start, outer),
            num(outer),
            num(outer),
            point(end, outer)
        );
    }
    format!(
        "M{}A{},{} 0 {large} 1 {}L{}A{},{} 0 {large} 0 {}Z",
        point(start, outer),
        num(outer),
        num(outer),
        point(end, outer),
        point(end, inner),
        num(inner),
        num(inner),
        point(start, inner)
    )
}
pub(super) trait PhaseInput {
    type Error: From<PeleError>;
    type Entry;
    type Entries: AsRef<[Self::Entry]>;
    type Text: AsRef<str>;
    fn entries(&mut self) -> Result<Self::Entries, Self::Error>;
    fn value(&mut self, entry: &Self::Entry) -> Result<f64, Self::Error>;
    fn legend_label(&mut self, entry: &Self::Entry) -> Result<Self::Text, Self::Error>;
    fn data_id(&mut self, entry: &Self::Entry) -> Result<Self::Text, Self::Error>;
    fn title(&mut self) -> Result<Option<Self::Text>, Self::Error>;
    fn accessibility(&mut self, description: bool) -> Result<Option<Self::Text>, Self::Error>;
}
struct Typed<'a>(&'a PieModel);
impl<'a> PhaseInput for Typed<'a> {
    type Error = PeleError;
    type Entry = super::PieSection;
    type Entries = &'a [super::PieSection];
    type Text = std::borrow::Cow<'a, str>;
    fn entries(&mut self) -> Result<Self::Entries, PeleError> {
        Ok(&self.0.sections)
    }
    fn value(&mut self, entry: &Self::Entry) -> Result<f64, PeleError> {
        Ok(entry.value)
    }
    fn legend_label(&mut self, entry: &Self::Entry) -> Result<Self::Text, PeleError> {
        Ok(if self.0.show_data {
            format!("{} [{}]", entry.label, js_num(entry.value)).into()
        } else {
            entry.label.clone().into()
        })
    }
    fn data_id(&mut self, entry: &Self::Entry) -> Result<Self::Text, PeleError> {
        Ok(entry.label.clone().into())
    }
    fn title(&mut self) -> Result<Option<Self::Text>, PeleError> {
        Ok(self.0.title.as_deref().map(Into::into))
    }
    fn accessibility(&mut self, description: bool) -> Result<Option<Self::Text>, PeleError> {
        Ok(if description {
            self.0.acc_descr.as_deref()
        } else {
            self.0.acc_title.as_deref()
        }
        .map(Into::into))
    }
}
pub(super) fn render(
    model: &PieModel,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    render_phases(&mut Typed(model), config, options, resources)
}
pub(super) fn render_phases<P: PhaseInput>(
    model: &mut P,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, P::Error> {
    let mut result = draw(model, config, options, resources, 120., false)?;
    let Some(available) = options.max_width.filter(|n| *n > 0.) else {
        return Ok(result);
    };
    if result.width <= available {
        return Ok(result);
    }
    let mut plot = 240.;
    result = draw(model, config, options, resources, plot / 2., true)?;
    for _ in 0..3 {
        if !(result.width > available && plot > 140.) {
            break;
        }
        plot = (plot - (result.width - available)).js_max(140.);
        result = draw(model, config, options, resources, plot / 2., true)?
    }
    Ok(result)
}
fn draw<P: PhaseInput>(
    model: &mut P,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
    radius: f64,
    tight: bool,
) -> Result<Rendered, P::Error> {
    let get = |key: &str| {
        config
            .get("pie")
            .and_then(Value::as_object)
            .and_then(|o| o.get(key))
    };
    let size = options.font_size.unwrap_or(16.);
    let small = round(size * 0.875);
    let pad = options.padding.unwrap_or(8.);
    let text_position = match get("textPosition") {
        Some(Value::Number(n)) if *n >= 0. && *n <= 1. => *n,
        _ => 0.75,
    };
    let inner = match get("donutHole") {
        Some(Value::Number(n)) if *n > 0. && *n <= 0.9 => *n * radius,
        _ => 0.,
    };
    let placed = get("legendPosition")
        .filter(|v| !matches!(v, Value::Null))
        .map_or("right".into(), Value::js_string);
    let position = if tight && matches!(placed.as_str(), "left" | "right") {
        "bottom"
    } else {
        placed.as_str()
    };
    let captured = model.entries()?;
    let entries = captured.as_ref();
    let mut sum = 0.;
    for entry in entries {
        sum += model.value(entry)?;
    }
    let mut legend = Vec::new();
    for entry in entries {
        let label = model.legend_label(entry)?;
        legend.push(layout_label(
            Some(label.as_ref()),
            false,
            resources,
            options,
            small,
            4000.,
            0,
        )?)
    }
    let legend_width = legend.iter().map(|l| l.width).fold(0., NumberMath::js_max)
        + if legend.is_empty() { 0. } else { 20. };
    let legend_height = legend.len() as f64 * 26.;
    let title_room = options
        .max_width
        .filter(|n| *n > 0.)
        .map_or(4000., |w| (w - 2. * pad).js_max(80.));
    let title_text = model.title()?;
    let title = layout_label(
        title_text.as_ref().map(AsRef::as_ref),
        false,
        resources,
        options,
        size,
        title_room,
        1,
    )?;
    let title_height = if title.height > 0. {
        title.height + 16.
    } else {
        0.
    };
    let mut shares = Vec::with_capacity(entries.len());
    for entry in entries {
        let value = model.value(entry)?;
        let share = if sum > 0. { value / sum } else { 0. };
        shares.push(if share * 100. >= 1. && (share * 100.).round() != 0. {
            share
        } else {
            0.
        });
    }
    let mut narrow = Vec::new();
    for share in &shares {
        let v = if *share == 0. || *share >= 0.5 {
            false
        } else {
            let chord = 2. * radius * text_position * (share * PI).sin();
            chord
                < measure(
                    resources,
                    options,
                    &format!("{:.0}%", (share * 100.).round()),
                    small,
                    0,
                )? + 6.
        };
        narrow.push(v)
    }
    let margin = if narrow.iter().any(|v| *v) { 28. } else { 0. };
    let side = if matches!(position, "left" | "right" | "center") {
        position
    } else {
        ""
    };
    let stacked = matches!(position, "top" | "bottom");
    let diameter = (radius + margin) * 2.;
    let mut width = diameter;
    let mut height = diameter;
    if matches!(side, "left" | "right") {
        width += if legend_width > 0. {
            32. + legend_width
        } else {
            0.
        };
        height = height.js_max(legend_height)
    } else if stacked {
        width = width.js_max(legend_width);
        height += if legend_height > 0. {
            20. + legend_height
        } else {
            0.
        }
    }
    width = width.js_max(title.width);
    let cx = pad
        + if side == "left" {
            legend_width + 32. + diameter / 2.
        } else if side == "right" {
            diameter / 2.
        } else {
            width / 2.
        };
    let body_top = pad + title_height;
    let cy = body_top
        + if position == "top" {
            legend_height + 20. + diameter / 2.
        } else if stacked {
            diameter / 2.
        } else {
            height / 2.
        };
    let (mut slices, mut labels, mut outside) = (String::new(), String::new(), String::new());
    let mut angle = 0.;
    let rim = radius + 14.;
    let mut placed_outside: Vec<(f64, f64, f64)> = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        let share = shares[index];
        if share == 0. {
            continue;
        }
        let end = angle + share * PI * 2.;
        slices.push_str(&format!(
            "<path class=\"pele-slice\" data-id=\"{}\" d=\"{}\" fill=\"{}\"/>",
            esc_text(model.data_id(entry)?.as_ref())?,
            slice(angle, end, radius, inner),
            series_color(index)
        ));
        let mid = (angle + end) / 2.;
        let text = format!("{:.0}%", (share * 100.).round());
        if narrow[index] {
            let width = measure(resources, options, &text, small, 0)?;
            let mut at = mid;
            for _ in 0..360 {
                let (x, y) = (at.sin() * rim, -at.cos() * rim);
                if placed_outside.iter().all(|(ox, oy, ow)| {
                    (x - ox).abs() >= (width + ow) / 2. + 2. || (y - oy).abs() >= small + 2.
                }) {
                    break;
                }
                at += 2. / rim;
            }
            let (x, y) = (at.sin() * rim, -at.cos() * rim);
            placed_outside.push((x, y, width));
            outside.push_str(&format!(
                "<text x=\"{}\" y=\"{}\">{text}</text>",
                num(x),
                num(y + small * 0.35)
            ))
        } else {
            let r = radius * text_position;
            labels.push_str(&format!(
                "<text x=\"{}\" y=\"{}\">{text}</text>",
                num(mid.sin() * r),
                num(-mid.cos() * r + small * 0.35)
            ))
        }
        angle = end;
    }
    if slices.is_empty() {
        slices = format!(
            "<circle class=\"pele-slice\" r=\"{}\" fill=\"none\" stroke=\"var(--_b)\"/>",
            js_num(radius)
        )
    }
    let (lx, ly) = if side == "right" {
        (
            cx + diameter / 2. + 32.,
            body_top + (height - legend_height) / 2.,
        )
    } else if side == "left" {
        (pad, body_top + (height - legend_height) / 2.)
    } else if side == "center" {
        (cx - legend_width / 2., cy - legend_height / 2.)
    } else {
        (
            pad + (width - legend_width) / 2.,
            if position == "top" {
                body_top
            } else {
                body_top + diameter + 20.
            },
        )
    };
    let mut legend_out = String::new();
    for (index, label) in legend.iter().enumerate() {
        let y = ly + index as f64 * 26. + 13.;
        legend_out.push_str(&format!("<g class=\"pele-legend-item\" data-id=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"12\" height=\"12\" rx=\"2\" fill=\"{}\"/>{}</g>",esc_text(model.data_id(&entries[index])?.as_ref())?,num(lx),num(y-6.),series_color(index),label_svg(label,lx+20.+label.width/2.,y,"")))
    }
    let total_width = (width + 2. * pad).ceil();
    let total_height = (height + title_height + 2. * pad).ceil();
    let mut content = label_svg(
        &title,
        total_width / 2.,
        pad + title.height / 2.,
        " class=\"pele-title\" font-weight=\"var(--_tw)\"",
    );
    content.push_str(&format!("<g class=\"pele-slices\" transform=\"translate({},{})\" stroke=\"var(--_bg)\" stroke-width=\"1\" stroke-linejoin=\"round\">{slices}</g>",num(cx),num(cy)));
    if !labels.is_empty() {
        content.push_str(&format!("<g class=\"pele-slice-labels\" transform=\"translate({},{})\" font-size=\"{}\" text-anchor=\"middle\" fill=\"var(--_bg)\">{labels}</g>",num(cx),num(cy),js_num(small)))
    }
    if !outside.is_empty() {
        content.push_str(&format!("<g class=\"pele-slice-labels\" transform=\"translate({},{})\" font-size=\"{}\" text-anchor=\"middle\">{outside}</g>",num(cx),num(cy),js_num(small)))
    }
    if !legend_out.is_empty() {
        content.push_str(&format!(
            "<g class=\"pele-legend\" font-size=\"{}\">{legend_out}</g>",
            js_num(small)
        ))
    }
    let acc_title = model.accessibility(false)?;
    let acc_descr = model.accessibility(true)?;
    Ok(Rendered {
        svg: svg::document(
            "pie",
            total_width,
            total_height,
            size,
            options,
            acc_title.as_ref().map(AsRef::as_ref),
            acc_descr.as_ref().map(AsRef::as_ref),
            &content,
        )?,
        width: total_width,
        height: total_height,
        links: Vec::new(),
    })
}
