//! Pele svg/{builder,root,theme}.ts rendering primitives, 458322a5 (MIT, kepano).
mod append;
pub(crate) use append::SvgSink;
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
pub(crate) fn esc(text: &str) -> std::borrow::Cow<'_, str> {
    use std::borrow::Cow;
    if !append::needs_escape(text) {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    append::escaped_slow(&mut out, text);
    Cow::Owned(out)
}
pub(crate) fn esc_text(text: &str) -> Result<String, PeleError> {
    let decoded = decode_entities(text)?;
    match esc(&decoded) {
        std::borrow::Cow::Borrowed(_) => Ok(decoded),
        std::borrow::Cow::Owned(escaped) => Ok(escaped),
    }
}
pub(crate) fn js_num(n: f64) -> String {
    append::js_number(n, str::to_owned)
}
pub(crate) fn num(value: f64) -> String {
    append::coordinate(value, str::to_owned)
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
// Pele root.ts uses literal substring edits, not XML parsing. Keep those
// semantics while borrowing the input and writing only the final SVG buffer.
const DEFAULT_RADIUS: &str = " rx=\"var(--pele-radius,4px)\"";
const EXISTING_STYLE_RADIUS: &str = "rx:var(--_r);";
const NEW_STYLE_RADIUS: &str = " style=\"rx:var(--_r)\"";
const STYLE_ATTRIBUTE: &str = " style=\"";

// Only ASCII needles are used. A match crossing a removed substring starts
// earlier when more of the needle is in `before`, hence the reverse order.
fn crossing_ascii(before: &str, after: &str, needle: &str) -> Option<usize> {
    debug_assert!(needle.is_ascii());
    (1..needle.len()).rev().find_map(|split| {
        (before.ends_with(&needle[..split]) && after.starts_with(&needle[split..]))
            .then(|| before.len() - split)
    })
}
fn find_joined_ascii(before: &str, after: &str, needle: &str) -> Option<usize> {
    before
        .find(needle)
        .or_else(|| crossing_ascii(before, after, needle))
        .or_else(|| after.find(needle).map(|at| before.len() + at))
}
fn joined_self_closing(before: &str, after: &str) -> bool {
    after.ends_with("/>") || (after == ">" && before.ends_with('/'))
}

#[derive(Default)]
struct RootStyles {
    radius: bool,
    title: bool,
    heading: bool,
    bold: bool,
}
impl RootStyles {
    fn scan(content: &str) -> Self {
        let mut styles = Self::default();
        // One complete body scan instead of four separate contains searches.
        for (at, _) in content.match_indices("var(--_") {
            let suffix = &content[at + 7..];
            styles.radius |= suffix.starts_with("r)");
            styles.title |= suffix.starts_with("tw)");
            styles.heading |= suffix.starts_with("hw)");
            styles.bold |= suffix.starts_with("w)");
        }
        styles
    }
    fn include_join(&mut self, before: &str, after: &str) {
        // DEFAULT_RADIUS contains none of these aliases. Removing it can join
        // two formerly separated pieces into an alias; inspect that boundary.
        self.title |= crossing_ascii(before, after, "var(--_tw)").is_some();
        self.heading |= crossing_ascii(before, after, "var(--_hw)").is_some();
        self.bold |= crossing_ascii(before, after, "var(--_w)").is_some();
        // Every rewritten rect inserts this alias regardless of its old text.
        self.radius = true;
    }
}
struct RadiusEdit {
    remove: usize,
    insert: usize,
    addition: &'static str,
}
struct RoundedRects<'a> {
    content: &'a str,
    cursor: usize,
}
impl<'a> RoundedRects<'a> {
    fn new(content: &'a str) -> Self {
        Self { content, cursor: 0 }
    }
}
impl Iterator for RoundedRects<'_> {
    type Item = RadiusEdit;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(relative) = self.content[self.cursor..].find("<rect") {
            let start = self.cursor + relative;
            self.cursor = start + 5;
            // JS /<rect\b[^>]*>/g has an ASCII word boundary. Non-ASCII
            // letters after "rect" match; a Unicode-regex boundary would not.
            if self
                .content
                .as_bytes()
                .get(self.cursor)
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            {
                continue;
            }
            let end = self.cursor + self.content[self.cursor..].find('>')? + 1;
            self.cursor = end;
            let tag = &self.content[start..end];
            let Some(remove) = tag.find(DEFAULT_RADIUS) else {
                continue;
            };
            let before = &tag[..remove];
            let after = &tag[remove + DEFAULT_RADIUS.len()..];
            // Search the logical joined tag, including a style marker created
            // by removal. The former implementation allocated `rest` for this.
            let (insert, addition) =
                if let Some(style) = find_joined_ascii(before, after, STYLE_ATTRIBUTE) {
                    (style + STYLE_ATTRIBUTE.len(), EXISTING_STYLE_RADIUS)
                } else {
                    (
                        before.len() + after.len()
                            - if joined_self_closing(before, after) {
                                2
                            } else {
                                1
                            },
                        NEW_STYLE_RADIUS,
                    )
                };
            let insert = start
                + insert
                + if insert > before.len() {
                    DEFAULT_RADIUS.len()
                } else {
                    0
                };
            return Some(RadiusEdit {
                remove: start + remove,
                insert,
                addition,
            });
        }
        None
    }
}
struct RootContent<'a> {
    source: &'a str,
    styles: RootStyles,
    length: usize,
    has_edits: bool,
}
impl<'a> RootContent<'a> {
    fn inspect(source: &'a str) -> Self {
        let mut content = Self {
            source,
            styles: RootStyles::scan(source),
            length: source.len(),
            has_edits: false,
        };
        if source.contains(DEFAULT_RADIUS) {
            for edit in RoundedRects::new(source) {
                content.has_edits = true;
                content.styles.include_join(
                    &source[..edit.remove],
                    &source[edit.remove + DEFAULT_RADIUS.len()..],
                );
                // Both static replacement strings are shorter than the removed
                // attribute, so length remains bounded by source.len().
                content.length -= DEFAULT_RADIUS.len() - edit.addition.len();
            }
        }
        content
    }
    fn append_to(&self, output: &mut String) {
        if !self.has_edits {
            output.push_str(self.source);
            return;
        }
        let mut cursor = 0;
        for edit in RoundedRects::new(self.source) {
            if edit.insert <= edit.remove {
                output.push_str(&self.source[cursor..edit.insert]);
                output.push_str(edit.addition);
                output.push_str(&self.source[edit.insert..edit.remove]);
                cursor = edit.remove + DEFAULT_RADIUS.len();
            } else {
                output.push_str(&self.source[cursor..edit.remove]);
                cursor = edit.remove + DEFAULT_RADIUS.len();
                output.push_str(&self.source[cursor..edit.insert]);
                output.push_str(edit.addition);
                cursor = edit.insert;
            }
        }
        output.push_str(&self.source[cursor..]);
    }
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
    let content = RootContent::inspect(content);
    let mut weights = if content.styles.radius {
        ";--_r:var(--pele-radius,4px)".to_string()
    } else {
        String::new()
    };
    for (used, alias, name) in [
        (content.styles.title, "--_tw", "title"),
        (content.styles.heading, "--_hw", "heading"),
        (content.styles.bold, "--_w", "bold"),
    ] {
        if used {
            weights.push_str(&format!(";{alias}:var(--pele-{name}-weight,bold)"))
        }
    }
    let font=format!(";--_fm:var(--pele-font-mono,{});font:{}px sans-serif;letter-spacing:normal;word-spacing:normal;text-transform:none;direction:ltr;font-family:var(--pele-font,{})",family(options.font_family_mono.as_deref(),"monospace"),js_num(size),family(options.font_family.as_deref(),"sans-serif"));
    let root="--_bg:var(--pele-bg,#fff);--_fg:var(--pele-fg,#1f1f1f);--_m:var(--pele-muted,#6e6e6e);--_l:var(--pele-line,#1f1f1f);--_s:var(--pele-surface,#f3f3f3);--_a:var(--pele-surface-alt,#e6e6e6);--_b:var(--pele-border,#8a8a8a);--_c:var(--pele-accent,#5b7bd5);color:var(--_fg)";
    let mut output = format!("<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"pele pele-{kind}\" viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\" style=\"{fit}{root}{weights}{}\" fill=\"var(--_fg)\" role=\"graphics-document document\" aria-roledescription=\"{kind}\"{aria}>{head}",js_num(width),js_num(height),js_num(width),js_num(height),esc(&font));
    output.reserve(content.length + "</svg>".len());
    content.append_to(&mut output);
    output.push_str("</svg>");
    Ok(output)
}
pub(crate) fn series_color(index: usize) -> String {
    let colors = [
        "#4c78a8", "#f58518", "#54a24b", "#e45756", "#72b7b2", "#eeca3b", "#b279a2", "#9d755d",
    ];
    format!("var(--pele-series-{},{})", index % 8 + 1, colors[index % 8])
}
