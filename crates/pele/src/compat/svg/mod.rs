//! Pele svg/{builder,root,theme}.ts rendering primitives, 458322a5 (MIT, kepano).
pub(crate) mod edges;
mod native;
pub(crate) mod shapes;
pub(crate) mod title;
use crate::compat::{
    source::decode_entities,
    text::{Label, SPACE},
    PeleError, RenderSettings,
};
pub(crate) use native::resolved_svg;
pub use native::NativeTheme;
use regex::Regex;
use std::sync::OnceLock;
pub(crate) fn esc(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\0'..='\x08' | '\x0b' | '\x0c' | '\x0e'..='\x1f' | '\x7f' => {}
            _ => out.push(c),
        }
    }
    out
}
pub(crate) fn esc_text(text: &str) -> Result<String, PeleError> {
    Ok(esc(&decode_entities(text)?))
}
pub(crate) fn js_num(n: f64) -> String {
    let mut b = ryu_js::Buffer::new();
    b.format(n).to_string()
}
pub(crate) fn num(value: f64) -> String {
    let n = (value * 100. + 0.5).floor();
    if n == 0. {
        return "0".into();
    }
    if n.abs() < 1e11 {
        let m = n.abs() as u64;
        let q = m / 100;
        let r = m % 100;
        // At most nine integer digits here. Write directly into one stack
        // buffer; format! previously allocated a fraction and a second string
        // for every SVG coordinate. Keep rounding and the non-finite lane above
        // and below identical to Pele.
        let mut bytes = [0u8; 16];
        let mut at = 10;
        let mut integer = q;
        loop {
            at -= 1;
            bytes[at] = b'0' + (integer % 10) as u8;
            integer /= 10;
            if integer == 0 {
                break;
            }
        }
        if n < 0. {
            at -= 1;
            bytes[at] = b'-';
        }
        let mut end = 10;
        if r != 0 {
            bytes[end] = b'.';
            bytes[end + 1] = b'0' + (r / 10) as u8;
            end += 2;
            if r % 10 != 0 {
                bytes[end] = b'0' + (r % 10) as u8;
                end += 1;
            }
        }
        std::str::from_utf8(&bytes[at..end])
            .expect("SVG coordinates contain only ASCII digits")
            .to_owned()
    } else {
        js_num(n / 100.)
    }
}
fn span_style(style: u8) -> String {
    format!(
        "{}{}",
        if style & 1 != 0 {
            " font-weight=\"var(--_w)\""
        } else {
            ""
        },
        if style & 2 != 0 {
            " font-style=\"italic\""
        } else {
            ""
        }
    )
}
pub(crate) fn label_svg(label: &Label, cx: f64, cy: f64, attrs: &str) -> String {
    label_svg_inner(label, cx, cy, attrs, &mut |_| Ok(String::new()))
        .expect("empty icon writer cannot fail")
}
pub(crate) fn label_svg_with_resources(
    label: &Label,
    cx: f64,
    cy: f64,
    attrs: &str,
    resources: &mut dyn crate::compat::RenderResources,
) -> Result<String, PeleError> {
    label_svg_inner(label, cx, cy, attrs, &mut |name| {
        Ok(resources
            .direct_icon(name)?
            .map(|svg| svg.markup().to_string())
            .unwrap_or_default())
    })
}
fn label_svg_inner(
    label: &Label,
    cx: f64,
    cy: f64,
    attrs: &str,
    icons: &mut dyn FnMut(&str) -> Result<String, PeleError>,
) -> Result<String, PeleError> {
    if label.lines.is_empty() {
        return Ok(String::new());
    }
    let shift = label.size * 0.35;
    let mut y = cy - label.height / 2. + label.line_height / 2. + shift;
    if label.lines.len() == 1
        && label.lines[0].len() == 1
        && label.lines[0][0].icon.is_none()
        && label.lines[0][0].style == 0
    {
        return Ok(format!(
            "<text{attrs} x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}</text>",
            num(cx),
            num(y),
            esc(&label.lines[0][0].text)
        ));
    }
    let mut shared = 3;
    let mut any = false;
    for span in label
        .lines
        .iter()
        .flatten()
        .filter(|s| s.icon.is_none() && !s.text.is_empty())
    {
        shared &= span.style;
        any = true
    }
    if !any {
        shared = 0
    }
    let mut attrs = attrs.to_string();
    if shared & 1 != 0 && !attrs.contains(" font-weight=\"") {
        attrs.push_str(" font-weight=\"var(--_w)\"")
    }
    if shared & 2 != 0 && !attrs.contains(" font-style=\"") {
        attrs.push_str(" font-style=\"italic\"")
    }
    let mut body = String::new();
    let mut extra = String::new();
    for (i, line) in label.lines.iter().enumerate() {
        if line.iter().any(|s| s.icon.is_some()) {
            let mut x = cx - label.widths[i] / 2.;
            for span in line {
                if let Some(icon) = &span.icon {
                    let side = label.size * 1.1;
                    let inner = icons(icon)?;
                    extra.push_str(&format!("<svg class=\"pele-icon\" data-icon=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{inner}</svg>",esc(&decode_entities(icon).unwrap_or_else(|_|icon.clone())),num(x),num(y-shift-side/2.),num(side),num(side)));
                } else if !span.text.is_empty() {
                    body.push_str(&format!(
                        "<tspan x=\"{}\" y=\"{}\" text-anchor=\"start\"{}>{}</tspan>",
                        num(x),
                        num(y),
                        span_style(span.style & !shared),
                        esc(&span.text)
                    ))
                }
                x += span.width;
            }
        } else {
            for (j, span) in line.iter().enumerate() {
                let pos = if j == 0 {
                    format!(" x=\"{}\" y=\"{}\"", num(cx), num(y))
                } else {
                    String::new()
                };
                body.push_str(&format!(
                    "<tspan{pos}{}>{}</tspan>",
                    span_style(span.style & !shared),
                    esc(&span.text)
                ))
            }
            if line.is_empty() {
                body.push_str(&format!(
                    "<tspan x=\"{}\" y=\"{}\"> </tspan>",
                    num(cx),
                    num(y)
                ))
            }
        }
        y += label.line_height;
    }
    Ok(format!(
        "<text{attrs} text-anchor=\"middle\" xml:space=\"preserve\">{body}</text>{extra}"
    ))
}
fn family<'a>(list: Option<&'a str>, generic: &'a str) -> &'a str {
    static FAMILY: OnceLock<Regex> = OnceLock::new();
    let re = FAMILY.get_or_init(|| {
        let f =
            r#"(?:"[^"'\\\n<>{}();:@]*"|'[^"'\\\n<>{}();:@]*'|[\p{L}\p{N}_-][\p{L}\p{N}_ .-]*)"#;
        Regex::new(&format!("^{SPACE}*{f}(?:{SPACE}*,{SPACE}*{f})*{SPACE}*$")).unwrap()
    });
    list.filter(|v| !v.is_empty() && re.is_match(v))
        .unwrap_or(generic)
}
// Match Pele root.ts: put the default radius first so an authored radius wins.
fn rounded(content: &str) -> std::borrow::Cow<'_, str> {
    use std::borrow::Cow;
    const ROUNDED: &str = " rx=\"var(--pele-radius,4px)\"";
    if !content.contains(ROUNDED) {
        return Cow::Borrowed(content);
    }
    static RECT: OnceLock<Regex> = OnceLock::new();
    let re = RECT.get_or_init(|| Regex::new(r"<rect\b[^>]*>").unwrap());
    Cow::Owned(
        re.replace_all(content, |captures: &regex::Captures<'_>| {
            let tag = &captures[0];
            let Some(at) = tag.find(ROUNDED) else {
                return tag.to_string();
            };
            let rest = format!("{}{}", &tag[..at], &tag[at + ROUNDED.len()..]);
            if let Some(style) = rest.find(" style=\"") {
                format!("{}rx:var(--_r);{}", &rest[..style + 8], &rest[style + 8..])
            } else {
                let end = rest.len() - if rest.ends_with("/>") { 2 } else { 1 };
                format!("{} style=\"rx:var(--_r)\"{}", &rest[..end], &rest[end..])
            }
        })
        .into_owned(),
    )
}
pub(crate) fn document(
    kind: &str,
    width: f64,
    height: f64,
    size: f64,
    options: &RenderSettings,
    title: Option<&str>,
    description: Option<&str>,
    content: &str,
) -> Result<String, PeleError> {
    let prefix = esc(options.id_prefix.as_deref().unwrap_or("pele"));
    let mut head = String::new();
    let mut aria = String::new();
    if let Some(t) = title.filter(|s| !s.is_empty()) {
        head.push_str(&format!(
            "<title id=\"{prefix}-title\">{}</title>",
            esc_text(t)?
        ));
        aria.push_str(&format!(" aria-labelledby=\"{prefix}-title\""))
    }
    if let Some(t) = description.filter(|s| !s.is_empty()) {
        head.push_str(&format!(
            "<desc id=\"{prefix}-desc\">{}</desc>",
            esc_text(t)?
        ));
        aria.push_str(&format!(" aria-describedby=\"{prefix}-desc\""))
    }
    let fit = if options.responsive == Some(false) {
        ""
    } else {
        "max-width:100%;height:auto;"
    };
    let content = rounded(content);
    let mut weights = if content.contains("var(--_r)") {
        ";--_r:var(--pele-radius,4px)".to_string()
    } else {
        String::new()
    };
    for (alias, name) in [("--_tw", "title"), ("--_hw", "heading"), ("--_w", "bold")] {
        if content.contains(&format!("var({alias})")) {
            weights.push_str(&format!(";{alias}:var(--pele-{name}-weight,bold)"))
        }
    }
    let font=format!(";--_fm:var(--pele-font-mono,{});font:{}px sans-serif;letter-spacing:normal;word-spacing:normal;text-transform:none;direction:ltr;font-family:var(--pele-font,{})",family(options.font_family_mono.as_deref(),"monospace"),js_num(size),family(options.font_family.as_deref(),"sans-serif"));
    let root="--_bg:var(--pele-bg,#fff);--_fg:var(--pele-fg,#1f1f1f);--_m:var(--pele-muted,#6e6e6e);--_l:var(--pele-line,#1f1f1f);--_s:var(--pele-surface,#f3f3f3);--_a:var(--pele-surface-alt,#e6e6e6);--_b:var(--pele-border,#8a8a8a);--_c:var(--pele-accent,#5b7bd5);color:var(--_fg)";
    Ok(format!("<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"pele pele-{kind}\" viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\" style=\"{fit}{root}{weights}{}\" fill=\"var(--_fg)\" role=\"graphics-document document\" aria-roledescription=\"{kind}\"{aria}>{head}{content}</svg>",js_num(width),js_num(height),js_num(width),js_num(height),esc(&font)))
}
pub(crate) fn series_color(index: usize) -> String {
    let colors = [
        "#4c78a8", "#f58518", "#54a24b", "#e45756", "#72b7b2", "#eeca3b", "#b279a2", "#9d755d",
    ];
    format!("var(--pele-series-{},{})", index % 8 + 1, colors[index % 8])
}
