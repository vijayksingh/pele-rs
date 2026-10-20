//! Exact ordered lexer, recursive parser and database validation; pinned Pele quadrant/*.ts.
use super::*;
use crate::compat::{source::space, ErrorCode};
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
enum T {
    End,
    Error,
    Newline,
    Title,
    TitleValue,
    AccTitle,
    AccTitleValue,
    AccDescr,
    AccDescrValue,
    AccDescrMulti,
    XAxis,
    YAxis,
    Delimiter,
    Q1,
    Q2,
    Q3,
    Q4,
    Classdef,
    Md,
    Str,
    ClassName,
    PointStart,
    X,
    Y,
    Quadrant,
    Alpha,
    Unicode,
    Colon,
    Plus,
    Comma,
    Equals,
    Mult,
    Brkt,
    Underscore,
    Dot,
    Amp,
    Minus,
    Num,
    Space,
    Semi,
    Punctuation,
    Eof,
}
const NAMES: [&str; 42] = [
    "$end",
    "INVALID",
    "NEWLINE",
    "title",
    "title_value",
    "acc_title",
    "acc_title_value",
    "acc_descr",
    "acc_descr_value",
    "acc_descr_multiline_value",
    "X-AXIS",
    "Y-AXIS",
    "AXIS-TEXT-DELIMITER",
    "QUADRANT_1",
    "QUADRANT_2",
    "QUADRANT_3",
    "QUADRANT_4",
    "CLASSDEF",
    "MD_STR",
    "STR",
    "class_name",
    "point_start",
    "point_x",
    "point_y",
    "QUADRANT",
    "ALPHA",
    "UNICODE_TEXT",
    "COLON",
    "PLUS",
    "COMMA",
    "EQUALS",
    "MULT",
    "BRKT",
    "UNDERSCORE",
    "DOT",
    "AMP",
    "MINUS",
    "NUM",
    "SPACE",
    "SEMI",
    "PUNCTUATION",
    "EOF",
];
#[derive(Clone)]
struct Token {
    kind: T,
    text: String,
    start: usize,
}
fn word(c: Option<char>) -> bool {
    c.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
}
fn at(src: &str, p: usize) -> Option<char> {
    src.get(p..).and_then(|s| s.chars().next())
}
fn keyword(src: &str, p: usize, w: &str) -> bool {
    src.get(p..p + w.len())
        .is_some_and(|s| s.eq_ignore_ascii_case(w))
}
fn run(src: &str, mut p: usize) -> usize {
    while let Some(c) = at(src, p).filter(|c| space(*c)) {
        p += c.len_utf8()
    }
    p
}
fn spaces(src: &str, mut p: usize) -> usize {
    while at(src, p) == Some(' ') {
        p += 1
    }
    p
}
fn coord(src: &str, p: usize) -> Option<usize> {
    match at(src, p)? {
        '1' => Some(p + 1),
        '0' => {
            let mut e = p + 1;
            if let Some(c) = at(src, e).filter(|c| {
                c.len_utf16() == 1 && !matches!(*c, '\n' | '\r' | '\u{2028}' | '\u{2029}')
            }) {
                if at(src, e + c.len_utf8()).is_some_and(|c| c.is_ascii_digit()) {
                    e += c.len_utf8() + 1;
                    while at(src, e).is_some_and(|c| c.is_ascii_digit()) {
                        e += 1
                    }
                }
            }
            Some(e)
        }
        _ => None,
    }
}
fn cached_run(src: &str, p: usize, cache: &mut (usize, usize), kind: u8) -> usize {
    if p < cache.0 || p >= cache.1 {
        cache.0 = p;
        cache.1 = p;
        while let Some(c) = at(src, cache.1) {
            if match kind {
                0 => c == ' ',
                1 => c == '-',
                _ => space(c),
            } {
                cache.1 += c.len_utf8()
            } else {
                break;
            }
        }
    }
    cache.1
}
fn tokenize(src: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut stack = Vec::new();
    let mut state = 0;
    let mut p = 0;
    let n = src.len();
    let (mut space_cache, mut dash_cache, mut ws_cache, mut gap_cache) =
        ((0, 0), (0, 0), (0, 0), (0, 0));
    let emit = |out: &mut Vec<Token>, kind: T, s: usize, e: usize| {
        out.push(Token {
            kind,
            text: src[s..e].into(),
            start: s,
        })
    };
    while p < n {
        let c = at(src, p).unwrap();
        let next = at(src, p + c.len_utf8());
        match state {
            1..=3 => {
                let e = src[p..].find('\n').map_or(n, |i| p + i);
                emit(
                    &mut out,
                    [T::Eof, T::TitleValue, T::AccTitleValue, T::AccDescrValue][state],
                    p,
                    e,
                );
                p = e;
                state = stack.pop().unwrap_or(state);
                continue;
            }
            4 => {
                if c == '}' {
                    p += 1;
                    state = stack.pop().unwrap_or(state)
                } else {
                    let e = src[p..].find('}').map_or(n, |i| p + i);
                    emit(&mut out, T::AccDescrMulti, p, e);
                    p = e
                }
                continue;
            }
            5 => {
                if c == '"' {
                    p += 1;
                    state = stack.pop().unwrap_or(state)
                } else {
                    let e = src[p..].find('"').map_or(n, |i| p + i);
                    emit(&mut out, T::Str, p, e);
                    p = e
                }
                continue;
            }
            6 => {
                if c == '`' && next == Some('"') {
                    p += 2;
                    state = stack.pop().unwrap_or(state);
                    continue;
                }
                if c != '`' && c != '"' {
                    let e = src[p..].find(['`', '"']).map_or(n, |i| p + i);
                    emit(&mut out, T::Md, p, e);
                    p = e;
                    continue;
                }
                break;
            }
            7 => {
                if word(Some(c)) {
                    let mut e = p + c.len_utf8();
                    while word(at(src, e)) {
                        e += 1
                    }
                    emit(&mut out, T::ClassName, p, e);
                    p = e;
                    state = stack.pop().unwrap_or(state);
                    continue;
                }
                break;
            }
            8 => {
                if let Some(e) = coord(src, p) {
                    emit(&mut out, T::X, p, e);
                    p = e;
                    stack.push(state);
                    state = 9;
                    continue;
                }
                let k = run(src, p);
                if at(src, k) == Some(']') {
                    p = spaces(src, k + 1);
                    state = stack.pop().unwrap_or(state);
                    continue;
                }
                break;
            }
            9 => {
                let k = run(src, p);
                if at(src, k) == Some(',') {
                    p = run(src, k + 1);
                    state = 10;
                    continue;
                }
                break;
            }
            10 => {
                if let Some(e) = coord(src, p) {
                    emit(&mut out, T::Y, p, e);
                    p = e;
                    state = stack.pop().unwrap_or(state);
                    continue;
                }
                break;
            }
            _ => {}
        }
        if c == '%' && next == Some('%') {
            p = src[p + 2..].find('\n').map_or(n, |i| p + 2 + i);
            continue;
        }
        if c != '}' && next == Some('%') && at(src, p + c.len_utf8() + 1) == Some('%') {
            p = src[p + c.len_utf8() + 2..]
                .find('\n')
                .map_or(n, |i| p + c.len_utf8() + 2 + i);
            continue;
        }
        if c == '\n' || c == '\r' {
            let mut e = p + 1;
            while at(src, e).is_some_and(|c| c == '\n' || c == '\r') {
                e += 1
            }
            emit(&mut out, T::Newline, p, e);
            p = e;
            continue;
        }
        if keyword(src, p, "title") && !word(at(src, p + 5)) {
            emit(&mut out, T::Title, p, p + 5);
            p += 5;
            stack.push(state);
            state = 1;
            continue;
        }
        if keyword(src, p, "acc") {
            let descr = keyword(src, p + 3, "descr");
            if descr || keyword(src, p + 3, "title") {
                let k = run(src, p + 8);
                if at(src, k) == Some(':') {
                    let e = run(src, k + 1);
                    emit(
                        &mut out,
                        if descr { T::AccDescr } else { T::AccTitle },
                        p,
                        e,
                    );
                    p = e;
                    stack.push(state);
                    state = if descr { 3 } else { 2 };
                    continue;
                }
                if at(src, k) == Some('{') && descr {
                    p = run(src, k + 1);
                    stack.push(state);
                    state = 4;
                    continue;
                }
            }
        }
        let j = if c == ' ' {
            cached_run(src, p, &mut space_cache, 0)
        } else {
            p
        };
        let mut found = None;
        if keyword(src, j, "x-axis") {
            found = Some((T::XAxis, j + 6))
        } else if keyword(src, j, "y-axis") {
            found = Some((T::YAxis, j + 6))
        } else if at(src, j) == Some('-') {
            let e = cached_run(src, j, &mut dash_cache, 1);
            if e - j >= 2 && at(src, e) == Some('>') {
                found = Some((T::Delimiter, e + 1))
            }
        } else if keyword(src, j, "quadrant") {
            if at(src, j + 8) == Some('-') {
                if let Some(q) = at(src, j + 9).filter(|c| ('1'..='4').contains(c)) {
                    found = Some((
                        [T::Q1, T::Q2, T::Q3, T::Q4][q as usize - '1' as usize],
                        j + 10,
                    ))
                }
            } else if keyword(src, j + 8, "chart") {
                found = Some((T::Quadrant, j + 13))
            }
        }
        if let Some((t, e)) = found {
            let e = spaces(src, e);
            emit(&mut out, t, p, e);
            p = e;
            continue;
        }
        if keyword(src, p, "classdef") && !word(at(src, p + 8)) {
            emit(&mut out, T::Classdef, p, p + 8);
            p += 8;
            continue;
        }
        if c == '"' {
            let md = next == Some('`');
            p += if md { 2 } else { 1 };
            stack.push(state);
            state = if md { 6 } else { 5 };
            continue;
        }
        if src[p..].starts_with(":::") {
            p += 3;
            stack.push(state);
            state = 7;
            continue;
        }
        if c == ':' || space(c) {
            let k = if c == ':' {
                p
            } else {
                cached_run(src, p, &mut ws_cache, 2)
            };
            if at(src, k) == Some(':') {
                let gap = cached_run(src, k + 1, &mut gap_cache, 2);
                if at(src, gap) == Some('[') {
                    let e = run(src, gap + 1);
                    emit(&mut out, T::PointStart, p, e);
                    p = e;
                    stack.push(state);
                    state = 8;
                    continue;
                }
            }
        }
        let kind = if c.is_ascii_alphabetic() {
            Some(T::Alpha)
        } else if c as u32 >= 128 {
            Some(T::Unicode)
        } else if c.is_ascii_digit() {
            Some(T::Num)
        } else {
            None
        };
        if let Some(t) = kind {
            let mut e = p + c.len_utf8();
            while let Some(d) = at(src, e) {
                if (t == T::Alpha && d.is_ascii_alphabetic())
                    || (t == T::Unicode && d as u32 >= 128)
                    || (t == T::Num && d.is_ascii_digit())
                {
                    e += d.len_utf8()
                } else {
                    break;
                }
            }
            emit(&mut out, t, p, e);
            p = e;
            continue;
        }
        let t = match c {
            ':' => T::Colon,
            '+' => T::Plus,
            ',' => T::Comma,
            '=' => T::Equals,
            '*' => T::Mult,
            '#' => T::Brkt,
            '_' => T::Underscore,
            '.' => T::Dot,
            '&' => T::Amp,
            '-' => T::Minus,
            ' ' | '\t' | '\x0b' | '\x0c' => T::Space,
            ';' => T::Semi,
            '!' | '$' | '%' | '\'' | '`' | '?' | '\\' | '/' => T::Punctuation,
            _ => break,
        };
        emit(&mut out, t, p, p + 1);
        p += 1;
    }
    if p < n {
        out.push(Token {
            kind: T::Error,
            text: String::new(),
            start: p,
        });
        return out;
    }
    if state < 6 {
        out.push(Token {
            kind: [
                T::Eof,
                T::TitleValue,
                T::AccTitleValue,
                T::AccDescrValue,
                T::AccDescrMulti,
                T::Str,
            ][state],
            text: String::new(),
            start: n,
        })
    }
    out.push(Token {
        kind: T::End,
        text: String::new(),
        start: n,
    });
    out
}
fn is_word(t: T) -> bool {
    matches!(
        t,
        T::Punctuation
            | T::Amp
            | T::Num
            | T::Alpha
            | T::Comma
            | T::Plus
            | T::Equals
            | T::Mult
            | T::Dot
            | T::Brkt
            | T::Underscore
            | T::Unicode
    )
}
fn is_text(t: T) -> bool {
    matches!(t, T::Str | T::Md) || is_word(t)
}
fn is_id(t: T) -> bool {
    matches!(
        t,
        T::Alpha
            | T::Num
            | T::Minus
            | T::Comma
            | T::Colon
            | T::Amp
            | T::Brkt
            | T::Mult
            | T::Unicode
    )
}
fn is_style(t: T) -> bool {
    matches!(
        t,
        T::Alpha | T::Num | T::Colon | T::Space | T::Brkt | T::Minus
    )
}
fn is_eol(t: T) -> bool {
    matches!(t, T::Newline | T::Semi | T::Eof)
}
struct Reader<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    i: usize,
}
impl Reader<'_> {
    fn kind(&self) -> T {
        self.tokens.get(self.i).map_or(T::End, |t| t.kind)
    }
    fn take(&mut self) -> String {
        let t = self.tokens[self.i].text.clone();
        self.i += 1;
        t
    }
    fn fail(&self, expected: &str) -> PeleError {
        let t = &self.tokens[self.i];
        PeleError::syntax(
            "quadrantChart",
            self.source,
            t.start,
            &format!("Expecting {expected}, got '{}'", NAMES[t.kind as usize]),
            t.kind == T::Error,
        )
    }
    fn expect(&mut self, t: T) -> Result<String, PeleError> {
        if self.kind() != t {
            return Err(self.fail(&format!("'{}'", NAMES[t as usize])));
        }
        Ok(self.take())
    }
    fn text(&mut self) -> Result<QuadrantText, PeleError> {
        let t = self.kind();
        if !is_text(t) {
            return Err(self.fail("'STR', 'MD_STR', 'ALPHA', 'NUM', 'UNICODE_TEXT', 'PUNCTUATION'"));
        }
        let mut s = self.take();
        while is_word(self.kind()) || matches!(self.kind(), T::Space | T::Minus) {
            s.push_str(&self.take())
        }
        Ok(QuadrantText {
            text: s,
            label_type: if t == T::Md { "markdown" } else { "text" }.into(),
        })
    }
    fn styles(&mut self) -> Result<PointStyle, PeleError> {
        let mut strings = Vec::new();
        loop {
            if !is_style(self.kind()) {
                return Err(self.fail("'ALPHA', 'NUM', 'COLON', 'SPACE', 'BRKT', 'MINUS'"));
            }
            let mut s = self.take();
            while is_style(self.kind()) {
                s.push_str(&self.take())
            }
            strings.push(s);
            if self.kind() != T::Comma {
                break;
            }
            self.i += 1;
        }
        let mut out = PointStyle::default();
        for s in strings {
            let mut parts = s.trim_matches(space).split(':');
            let key = parts.next().unwrap().trim_end_matches(space);
            let value = parts.next().map_or("undefined", |s| s.trim_matches(space));
            let invalid = |kind: &str| {
                let mut e = PeleError::new(
                    format!("value for {key} {value} is invalid, please use a valid {kind}"),
                    ErrorCode::Semantic,
                );
                e.diagram_type = Some("quadrantChart".into());
                e
            };
            let field = match key {
                "radius" => "radius",
                "color" => "color",
                "stroke-color" => "strokeColor",
                "stroke-width" => "strokeWidth",
                _ => "",
            };
            if !field.is_empty() && !out.order.contains(&field) {
                out.order.push(field)
            }
            match key {
                "radius" => {
                    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
                        return Err(invalid("number"));
                    }
                    out.radius = Some(value.parse::<f64>().unwrap_or(f64::INFINITY))
                }
                "color" | "stroke-color" => {
                    let hex = value.strip_prefix('#').unwrap_or(value);
                    if !matches!(hex.len(), 3 | 6) || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                        return Err(invalid("hex code"));
                    }
                    if key == "color" {
                        out.color = Some(value.into())
                    } else {
                        out.stroke_color = Some(value.into())
                    }
                }
                "stroke-width" => {
                    if !value
                        .strip_suffix("px")
                        .is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
                    {
                        return Err(invalid("number of pixels (eg. 10px)"));
                    }
                    out.stroke_width = Some(value.into())
                }
                _ => {
                    let mut e = PeleError::new(
                        format!("style named {key} is not supported."),
                        ErrorCode::Semantic,
                    );
                    e.diagram_type = Some("quadrantChart".into());
                    return Err(e);
                }
            }
        }
        Ok(out)
    }
}
fn number(s: &str) -> f64 {
    let s = s.trim_matches(space);
    if s.is_empty() {
        return 0.;
    }
    if s.len() > 2 {
        let radix = match s.as_bytes().get(..2) {
            Some(b"0x" | b"0X") => Some(16),
            Some(b"0b" | b"0B") => Some(2),
            Some(b"0o" | b"0O") => Some(8),
            _ => None,
        };
        if let Some(radix) = radix {
            let mut n = 0.;
            for c in s[2..].chars() {
                let Some(d) = c.to_digit(radix) else {
                    return f64::NAN;
                };
                n = n * radix as f64 + d as f64
            }
            return n;
        }
    }
    s.parse().unwrap_or(f64::NAN)
}
pub(super) fn parse(src: &str, title: Option<&str>) -> Result<QuadrantModel, PeleError> {
    let mut r = Reader {
        source: src,
        tokens: tokenize(src),
        i: 0,
    };
    let mut m = QuadrantModel::empty(title);
    while is_eol(r.kind()) || r.kind() == T::Space {
        r.i += 1
    }
    r.expect(T::Quadrant)?;
    while r.kind() != T::End {
        while r.kind() == T::Space {
            r.i += 1
        }
        let t = r.kind();
        match t {
            T::Classdef => {
                r.i += 1;
                r.expect(T::Space)?;
                if !is_id(r.kind()) {
                    return Err(r.fail("'ALPHA', 'NUM', 'UNICODE_TEXT'"));
                }
                let mut id = r.take();
                while is_id(r.kind()) {
                    id.push_str(&r.take())
                }
                r.expect(T::Space)?;
                let styles = r.styles()?;
                if let Some(entry) = m.classes.iter_mut().find(|e| e.0 == id) {
                    entry.1 = styles
                } else {
                    m.classes.push((id, styles))
                }
            }
            T::XAxis | T::YAxis => {
                r.i += 1;
                let mut from = r.text()?;
                let mut to = None;
                if r.kind() == T::Delimiter {
                    r.i += 1;
                    if is_text(r.kind()) {
                        to = Some(r.text()?)
                    } else {
                        from.text.push_str(" ⟶ ");
                        from.text = from.text.trim_matches(space).into()
                    }
                }
                from.text = from.text.trim_matches(space).into();
                if let Some(to) = &mut to {
                    to.text = to.text.trim_matches(space).into()
                }
                if t == T::XAxis {
                    m.x_axis_left = Some(from);
                    if to.is_some() {
                        m.x_axis_right = to
                    }
                } else {
                    m.y_axis_bottom = Some(from);
                    if to.is_some() {
                        m.y_axis_top = to
                    }
                }
            }
            T::Q1 | T::Q2 | T::Q3 | T::Q4 => {
                r.i += 1;
                let mut text = r.text()?;
                text.text = text.text.trim_matches(space).into();
                m.quadrants[t as usize - T::Q1 as usize] = Some(text)
            }
            T::Title => {
                r.i += 1;
                m.title = Some(r.expect(T::TitleValue)?.trim_matches(space).into())
            }
            T::AccTitle => {
                r.i += 1;
                m.acc_title = Some(r.expect(T::AccTitleValue)?.trim_matches(space).into())
            }
            T::AccDescr | T::AccDescrMulti => {
                let text = if t == T::AccDescr {
                    r.i += 1;
                    r.expect(T::AccDescrValue)?
                } else {
                    r.take()
                };
                let text = text.trim_matches(space);
                let mut out = String::new();
                let mut chars = text.chars().peekable();
                while let Some(c) = chars.next() {
                    out.push(c);
                    if c == '\n' {
                        while chars.peek().is_some_and(|c| space(*c)) {
                            chars.next();
                        }
                    }
                }
                m.acc_descr = Some(out)
            }
            _ => {
                if is_text(t) {
                    let mut text = r.text()?;
                    text.text = text.text.trim_matches(space).into();
                    let class_name = if r.kind() == T::ClassName {
                        r.take()
                    } else {
                        String::new()
                    };
                    r.expect(T::PointStart)?;
                    let x = number(&r.expect(T::X)?);
                    let y = number(&r.expect(T::Y)?);
                    let style = if is_style(r.kind()) {
                        r.styles()?
                    } else {
                        PointStyle::default()
                    };
                    m.points.push(QuadrantPoint {
                        x,
                        y,
                        text,
                        class_name,
                        style,
                    })
                }
            }
        }
        if !is_eol(r.kind()) {
            return Err(r.fail("'NEWLINE', 'SEMI', 'EOF'"));
        }
        r.i += 1;
    }
    Ok(m)
}
