//! Shared rich labels translated from Pele text/label.ts (458322a5, MIT, kepano).
mod icon_cursor;
mod metrics;
use crate::compat::{
    source::{decode_entities, space, word},
    PeleError, RenderResources, RenderSettings, TextRequest,
};
pub use metrics::{width as metrics_width, MetricsResources};
use regex::Regex;
use std::sync::OnceLock;
pub(crate) const SPACE: &str =
    r"[\t-\r \u{a0}\u{1680}\u{2000}-\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}\u{feff}]";
#[derive(Clone, Debug)]
pub(crate) struct Span {
    pub text: String,
    pub style: u8,
    pub icon: Option<String>,
    pub width: f64,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct Label {
    pub lines: Vec<Vec<Span>>,
    pub widths: Vec<f64>,
    pub width: f64,
    pub height: f64,
    pub size: f64,
    pub line_height: f64,
}
pub(crate) fn measure(
    resources: &mut dyn RenderResources,
    settings: &RenderSettings,
    text: &str,
    size: f64,
    style: u8,
) -> Result<f64, PeleError> {
    resources.text_width(TextRequest {
        text,
        size,
        style,
        font_family: settings.font_family.as_deref(),
        font_family_mono: settings.font_family_mono.as_deref(),
    })
}
fn push_text(line: &mut Vec<Span>, text: &str, style: u8) -> Result<(), PeleError> {
    if text.is_empty() {
        return Ok(());
    }
    let text = decode_entities(text)?;
    if let Some(last) = line
        .last_mut()
        .filter(|last| last.style == style && last.icon.is_none())
    {
        last.text.push_str(&text)
    } else {
        line.push(Span {
            text,
            style,
            icon: None,
            width: 0.,
        })
    }
    Ok(())
}
fn push_icons(
    line: &mut Vec<Span>,
    text: &str,
    style: u8,
    resources: &mut dyn RenderResources,
) -> Result<(), PeleError> {
    // The pinned JS fast path does not touch the shared expression cursor.
    if !text.contains(":fa-") {
        return push_text(line, text, style);
    }
    let mut cursor = icon_cursor::IconCursor::new();
    let mut last = 0;
    while let Some(found) = cursor.next(text) {
        let name = &text[found.clone()];
        // JS slice(start, end) is empty when a nested scan rewinds the next
        // match before our previous end; it must not become a Rust range panic.
        push_text(line, &text[last.min(found.start)..found.start], style)?;
        last = found.end;
        if resources
            .icon(name)?
            .is_some_and(|svg| !svg.markup().is_empty())
        {
            line.push(Span {
                text: String::new(),
                style,
                icon: Some(name.into()),
                width: 0.,
            })
        } else if text[last..].starts_with(' ') {
            last += 1
        } else if last == text.len() {
            if let Some(before) = line.last_mut().filter(|s| s.icon.is_none()) {
                before.text = before.text.trim_end_matches(space).into()
            }
        }
    }
    push_text(line, &text[last..], style)
}
fn inline(
    line: &mut Vec<Span>,
    source: &str,
    markdown: bool,
    base: u8,
    resources: &mut dyn RenderResources,
) -> Result<(), PeleError> {
    static TAG: OnceLock<Regex> = OnceLock::new();
    let tag_re = TAG.get_or_init(|| {
        Regex::new(&format!(
            r"^</?([a-zA-Z][A-Za-z0-9_-]*)(?:{SPACE}[^<>]*)?/?>"
        ))
        .unwrap()
    });
    let mut style = base;
    let mut marked = 0u8;
    let mut text = String::new();
    let mut i = 0;
    while i < source.len() {
        let ch = source[i..].chars().next().unwrap();
        if ch == '<' {
            if let Some(m) = tag_re.captures(&source[i..]) {
                let tag = m[1].to_ascii_lowercase();
                let bit = match tag.as_str() {
                    "b" | "strong" => 1,
                    "i" | "em" => 2,
                    _ => 0,
                };
                if bit != 0 {
                    push_icons(line, &text, style, resources)?;
                    text.clear();
                    style = if source[i..].starts_with("</") {
                        (style & !bit) | (base & bit)
                    } else {
                        style | bit
                    }
                }
                i += m.get(0).unwrap().end();
                continue;
            }
        } else if markdown && matches!(ch, '*' | '_') {
            let double = source[i + 1..].starts_with(ch);
            let bit = if double { 1 } else { 2 };
            let len = if double { 2 } else { 1 };
            let open = marked & bit == 0;
            let prev = source[..i].chars().next_back().unwrap_or(' ');
            let next = source[i + len..].chars().next().unwrap_or(' ');
            let marker = ch.to_string().repeat(len);
            let can_open = open && !space(next) && source[i + len..].contains(&marker);
            let can_close = !open && !space(prev);
            let wordy = ch == '_' && word(prev) && word(next);
            if (can_open || can_close) && !wordy {
                push_icons(line, &text, style, resources)?;
                text.clear();
                marked = if can_open {
                    marked | bit
                } else {
                    marked & !bit
                };
                style = if can_open {
                    style | bit
                } else {
                    (style & !bit) | (base & bit)
                };
                i += len;
                continue;
            }
        }
        text.push(ch);
        i += ch.len_utf8();
    }
    push_icons(line, &text, style, resources)
}
fn commit(
    current: &mut Vec<Span>,
    width: &mut f64,
    out: &mut Vec<Vec<Span>>,
    widths: &mut Vec<f64>,
    resources: &mut dyn RenderResources,
    settings: &RenderSettings,
    size: f64,
) -> Result<(), PeleError> {
    if let Some(last) = current.last_mut().filter(|s| s.icon.is_none()) {
        let trimmed = last.text.trim_end_matches(space);
        if trimmed != last.text {
            let w = measure(resources, settings, trimmed, size, last.style)?;
            last.text = trimmed.into();
            *width += w - last.width;
            last.width = w
        }
    }
    out.push(std::mem::take(current));
    widths.push(*width);
    *width = 0.;
    Ok(())
}
fn wrap(
    line: Vec<Span>,
    resources: &mut dyn RenderResources,
    settings: &RenderSettings,
    size: f64,
    max: f64,
    out: &mut Vec<Vec<Span>>,
    widths: &mut Vec<f64>,
) -> Result<(), PeleError> {
    let mut current: Vec<Span> = Vec::new();
    let mut width = 0.;
    for span in line {
        if span.icon.is_some() {
            let w = size * 1.25;
            if width > 0. && width + w > max {
                commit(
                    &mut current,
                    &mut width,
                    out,
                    widths,
                    resources,
                    settings,
                    size,
                )?
            }
            current.push(Span { width: w, ..span });
            width += w;
            continue;
        }
        // JS /\s*\S+\s*|\s+/g: the following word owns both surrounding whitespace runs.
        let chars: Vec<_> = span.text.char_indices().collect();
        let mut p = 0;
        while p < chars.len() {
            let start = p;
            while p < chars.len() && space(chars[p].1) {
                p += 1
            }
            while p < chars.len() && !space(chars[p].1) {
                p += 1
            }
            while p < chars.len() && space(chars[p].1) {
                p += 1
            }
            let begin = chars[start].0;
            let end = chars.get(p).map_or(span.text.len(), |v| v.0);
            let mut part = &span.text[begin..end];
            if width > 0.
                && width
                    + measure(
                        resources,
                        settings,
                        part.trim_end_matches(space),
                        size,
                        span.style,
                    )?
                    > max
            {
                commit(
                    &mut current,
                    &mut width,
                    out,
                    widths,
                    resources,
                    settings,
                    size,
                )?;
                part = part.trim_start_matches(space)
            }
            if let Some(last) = current
                .last_mut()
                .filter(|s| s.style == span.style && s.icon.is_none())
            {
                last.text.push_str(part);
                last.width = measure(resources, settings, &last.text, size, last.style)?;
                width = current.iter().map(|s| s.width).sum()
            } else {
                let w = measure(resources, settings, part, size, span.style)?;
                current.push(Span {
                    text: part.into(),
                    style: span.style,
                    icon: None,
                    width: w,
                });
                width += w
            }
        }
    }
    if !current.is_empty() || out.is_empty() {
        commit(
            &mut current,
            &mut width,
            out,
            widths,
            resources,
            settings,
            size,
        )?
    }
    Ok(())
}
pub(crate) fn layout_label(
    raw: Option<&str>,
    markdown: bool,
    resources: &mut dyn RenderResources,
    settings: &RenderSettings,
    size: f64,
    max_width: f64,
    base: u8,
) -> Result<Label, PeleError> {
    let Some(raw) = raw.filter(|s| !s.is_empty()) else {
        return Ok(Label::default());
    };
    let line_height = crate::compat::number_math::round(size * 1.5);
    static MARKUP: OnceLock<Regex> = OnceLock::new();
    let markup = MARKUP.get_or_init(|| Regex::new(r"[<\\\n*_]|fa[bklrs]?:fa-").unwrap());
    if !markup.is_match(raw) && !raw.contains('&') && !raw.contains('ﬂ') {
        let text = raw.trim_matches(space);
        let width = measure(resources, settings, text, size, base)?;
        let line = vec![Span {
            text: text.into(),
            style: base,
            icon: None,
            width,
        }];
        if width <= max_width || !text.contains(' ') {
            return Ok(Label {
                lines: vec![line],
                widths: vec![width],
                width,
                height: line_height,
                size,
                line_height,
            });
        }
        let (mut lines, mut widths) = (Vec::new(), Vec::new());
        wrap(
            line,
            resources,
            settings,
            size,
            max_width,
            &mut lines,
            &mut widths,
        )?;
        let widest = widths.iter().copied().fold(0., f64::max);
        let height = lines.len() as f64 * line_height;
        return Ok(Label {
            lines,
            widths,
            width: widest,
            height,
            size,
            line_height,
        });
    }
    let mut source = if markdown {
        raw.replace("\r\n", "\n").replace('\r', "\n")
    } else {
        raw.into()
    };
    if markdown {
        let min = source
            .split('\n')
            .filter(|l| !l.trim_matches(space).is_empty())
            .map(|l| l.chars().take_while(|c| space(*c)).count())
            .min()
            .unwrap_or(0);
        if min > 0 {
            source = source
                .split('\n')
                .map(|l| {
                    let n = min.min(l.chars().take_while(|c| space(*c)).count());
                    &l[l.char_indices().nth(n).map_or(l.len(), |v| v.0)..]
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
    let source = source.trim_matches(space);
    static BREAK: OnceLock<Regex> = OnceLock::new();
    let breaks = BREAK.get_or_init(|| Regex::new(&format!(r"(?i:<br{SPACE}*/?>)|\\n|\n")).unwrap());
    let (mut lines, mut widths) = (Vec::new(), Vec::new());
    for part in breaks.split(source) {
        let mut line = Vec::new();
        inline(
            &mut line,
            part.trim_matches(space),
            markdown,
            base,
            resources,
        )?;
        let mut width = 0.;
        for span in &mut line {
            span.width = if span.icon.is_some() {
                size * 1.25
            } else {
                measure(resources, settings, &span.text, size, span.style)?
            };
            width += span.width
        }
        if width > max_width {
            wrap(
                line,
                resources,
                settings,
                size,
                max_width,
                &mut lines,
                &mut widths,
            )?
        } else {
            lines.push(line);
            widths.push(width)
        }
    }
    let width = widths.iter().copied().fold(0., f64::max);
    let height = lines.len() as f64 * line_height;
    Ok(Label {
        lines,
        widths,
        width,
        height,
        size,
        line_height,
    })
}
